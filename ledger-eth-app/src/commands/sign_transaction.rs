// SPDX-License-Identifier: Apache-2.0

//! SIGN ETH TRANSACTION command implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, length, p1_sign_transaction, p2_sign_transaction};
use crate::types::{SignTransactionParams, Signature};
use crate::utils::{chunk_data, encode_bip32_path, validate_bip32_path};
use crate::EthApp;

/// Transaction processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    /// Process transaction data and start signing flow immediately
    ProcessAndStart,
    /// Store transaction data only, don't start signing flow
    StoreOnly,
    /// Start signing flow using previously stored data
    StartFlow,
}

impl TransactionMode {
    fn to_p2(self) -> u8 {
        match self {
            TransactionMode::ProcessAndStart => p2_sign_transaction::PROCESS_AND_START,
            TransactionMode::StoreOnly => p2_sign_transaction::STORE_ONLY,
            TransactionMode::StartFlow => p2_sign_transaction::START_FLOW,
        }
    }
}

#[async_trait]
pub trait SignTransaction<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign an Ethereum transaction using the given BIP 32 path
    async fn sign_transaction(
        transport: &E,
        params: SignTransactionParams,
    ) -> EthAppResult<Signature, E::Error>;

    /// Sign an Ethereum transaction with specific processing mode
    async fn sign_transaction_with_mode(
        transport: &E,
        params: SignTransactionParams,
        mode: TransactionMode,
    ) -> EthAppResult<Option<Signature>, E::Error>;
}

#[async_trait]
impl<E> SignTransaction<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_transaction(
        transport: &E,
        params: SignTransactionParams,
    ) -> EthAppResult<Signature, E::Error> {
        match Self::sign_transaction_with_mode(transport, params, TransactionMode::ProcessAndStart)
            .await?
        {
            Some(signature) => Ok(signature),
            None => Err(EthAppError::InvalidResponseData(
                "Expected signature but got none".to_string(),
            )),
        }
    }

    async fn sign_transaction_with_mode(
        transport: &E,
        params: SignTransactionParams,
        mode: TransactionMode,
    ) -> EthAppResult<Option<Signature>, E::Error> {
        // Validate BIP32 path
        validate_bip32_path(&params.path)?;

        // Check transaction data size
        if params.transaction_data.is_empty() {
            return Err(EthAppError::InvalidTransaction(
                "Transaction data cannot be empty".to_string(),
            ));
        }

        match mode {
            TransactionMode::StartFlow => {
                // For start flow mode, send empty command
                let command = APDUCommand {
                    cla: Self::CLA,
                    ins: ins::SIGN_ETH_TRANSACTION,
                    p1: p1_sign_transaction::FIRST_DATA_BLOCK,
                    p2: mode.to_p2(),
                    data: Vec::new(),
                };

                let response = transport
                    .exchange(&command)
                    .await
                    .map_err(|e| EthAppError::Transport(e.into()))?;

                <EthApp as AppExt<E>>::handle_response_error_signature(&response)
                    .map_err(EthAppError::Transport)?;

                let signature = parse_signature_response::<E::Error>(response.data())?;
                return Ok(Some(signature));
            }
            _ => {
                // For other modes, process transaction data
                return Self::process_transaction_data(transport, params, mode).await;
            }
        }
    }
}

impl EthApp {
    async fn process_transaction_data<E>(
        transport: &E,
        params: SignTransactionParams,
        mode: TransactionMode,
    ) -> EthAppResult<Option<Signature>, E::Error>
    where
        E: Exchange + Send + Sync,
        E::Error: std::error::Error,
    {
        let path_data = encode_bip32_path(&params.path);

        // Calculate maximum chunk size for transaction data
        // First chunk includes: path_len(1) + path_indices(path.len()*4)
        let first_chunk_overhead = path_data.len();

        if first_chunk_overhead >= length::MAX_MESSAGE_CHUNK_SIZE {
            return Err(EthAppError::InvalidBip32Path(
                "BIP32 path too long for transaction signing".to_string(),
            ));
        }

        let first_chunk_tx_size = length::MAX_MESSAGE_CHUNK_SIZE - first_chunk_overhead;
        let subsequent_chunk_size = length::MAX_MESSAGE_CHUNK_SIZE;

        // Split transaction into chunks
        let (first_tx_chunk, remaining_tx) = if params.transaction_data.len() <= first_chunk_tx_size
        {
            (params.transaction_data.as_slice(), &[][..])
        } else {
            (
                &params.transaction_data[..first_chunk_tx_size],
                &params.transaction_data[first_chunk_tx_size..],
            )
        };

        let remaining_chunks = chunk_data(remaining_tx, subsequent_chunk_size);

        // Send first chunk with path
        let mut first_chunk_data = Vec::new();
        first_chunk_data.extend_from_slice(&path_data);
        first_chunk_data.extend_from_slice(first_tx_chunk);

        let first_command = APDUCommand {
            cla: Self::CLA,
            ins: ins::SIGN_ETH_TRANSACTION,
            p1: p1_sign_transaction::FIRST_DATA_BLOCK,
            p2: mode.to_p2(),
            data: first_chunk_data,
        };

        let mut response = transport
            .exchange(&first_command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        // Handle response (no signature expected yet at this stage)
        <EthApp as AppExt<E>>::handle_response_error(&response).map_err(EthAppError::Transport)?;

        // Send remaining chunks
        for (i, chunk) in remaining_chunks.iter().enumerate() {
            let command = APDUCommand {
                cla: Self::CLA,
                ins: ins::SIGN_ETH_TRANSACTION,
                p1: p1_sign_transaction::SUBSEQUENT_DATA_BLOCK,
                p2: mode.to_p2(),
                data: chunk.clone(),
            };

            response = transport
                .exchange(&command)
                .await
                .map_err(|e| EthAppError::Transport(e.into()))?;

            // Only check for signature on the last chunk if not store-only mode
            if mode == TransactionMode::StoreOnly {
                <EthApp as AppExt<E>>::handle_response_error(&response)
                    .map_err(EthAppError::Transport)?;
            } else if i == remaining_chunks.len() - 1 {
                // Last chunk - expect signature
                <EthApp as AppExt<E>>::handle_response_error_signature(&response)
                    .map_err(EthAppError::Transport)?;
            } else {
                <EthApp as AppExt<E>>::handle_response_error(&response)
                    .map_err(EthAppError::Transport)?;
            }
        }

        // Parse signature from final response if not store-only mode
        if mode == TransactionMode::StoreOnly {
            Ok(None)
        } else {
            let signature = parse_signature_response::<E::Error>(response.data())?;
            Ok(Some(signature))
        }
    }
}

/// Parse signature response data
fn parse_signature_response<E: std::error::Error>(data: &[u8]) -> EthAppResult<Signature, E> {
    if data.len() != 65 {
        return Err(EthAppError::InvalidResponseData(format!(
            "Invalid signature response length: {} bytes (expected 65)",
            data.len()
        )));
    }

    let v = data[0];
    let r = data[1..33].to_vec();
    let s = data[33..65].to_vec();

    Signature::new(v, r, s).map_err(|e| EthAppError::InvalidSignature(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BipPath;

    #[test]
    fn test_transaction_mode_to_p2() {
        assert_eq!(
            TransactionMode::ProcessAndStart.to_p2(),
            p2_sign_transaction::PROCESS_AND_START
        );
        assert_eq!(
            TransactionMode::StoreOnly.to_p2(),
            p2_sign_transaction::STORE_ONLY
        );
        assert_eq!(
            TransactionMode::StartFlow.to_p2(),
            p2_sign_transaction::START_FLOW
        );
    }

    #[test]
    fn test_parse_signature_response() {
        // Mock signature response: v(1) + r(32) + s(32)
        let mut response_data = Vec::new();
        response_data.push(0x1c); // v value
        response_data.extend(vec![0xAA; 32]); // r component
        response_data.extend(vec![0xBB; 32]); // s component

        let result = parse_signature_response::<std::io::Error>(&response_data);
        assert!(result.is_ok());

        let signature = result.unwrap();
        assert_eq!(signature.v, 0x1c);
        assert_eq!(signature.r.len(), 32);
        assert_eq!(signature.s.len(), 32);
        assert!(signature.r.iter().all(|&x| x == 0xAA));
        assert!(signature.s.iter().all(|&x| x == 0xBB));
    }

    #[test]
    fn test_parse_signature_response_invalid_length() {
        let response_data = vec![0x1c; 64]; // Too short

        let result = parse_signature_response::<std::io::Error>(&response_data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EthAppError::InvalidResponseData(_)
        ));
    }

    #[test]
    fn test_sign_transaction_params() {
        let path = BipPath::ethereum_standard(0, 0);
        let tx_data = vec![0xf8, 0x6c]; // Start of RLP encoded transaction
        let params = SignTransactionParams::new(path.clone(), tx_data.clone());

        assert_eq!(params.path, path);
        assert_eq!(params.transaction_data, tx_data);
    }

    #[test]
    fn test_transaction_chunking_calculation() {
        let path = BipPath::new(vec![0x8000002C, 0x8000003C, 0x80000000, 0, 0]).unwrap();
        let path_data = encode_bip32_path(&path);
        let first_chunk_overhead = path_data.len();

        // Should be: 1 (path_len) + 5*4 (indices) = 21 bytes overhead
        assert_eq!(first_chunk_overhead, 21);

        let first_chunk_tx_size = length::MAX_MESSAGE_CHUNK_SIZE - first_chunk_overhead;
        assert_eq!(first_chunk_tx_size, 255 - 21); // 234 bytes for tx data in first chunk
    }
}
