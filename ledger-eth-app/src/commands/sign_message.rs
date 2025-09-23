// SPDX-License-Identifier: Apache-2.0

//! SIGN ETH PERSONAL MESSAGE command implementation

use async_trait::async_trait;
use ledger_sdk_device_base::{App, AppExt};
use ledger_sdk_transport::{APDUCommand, Exchange};

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, length, p1_sign_message};
use crate::types::{SignMessageParams, Signature};
use crate::utils::{chunk_data, encode_bip32_path, validate_bip32_path};
use crate::EthApp;

#[async_trait]
pub trait SignPersonalMessage<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign an Ethereum personal message using the given BIP 32 path
    async fn sign_personal_message(
        transport: &E,
        params: SignMessageParams,
    ) -> EthAppResult<Signature, E::Error>;
}

#[async_trait]
impl<E> SignPersonalMessage<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_personal_message(
        transport: &E,
        params: SignMessageParams,
    ) -> EthAppResult<Signature, E::Error> {
        // Validate BIP32 path
        validate_bip32_path(&params.path)?;

        // Check message size
        if params.message.is_empty() {
            return Err(EthAppError::InvalidMessage(
                "Message cannot be empty".to_string(),
            ));
        }

        // Calculate maximum chunk size for message data
        // First chunk includes: path_len(1) + path_indices(path.len()*4) + message_len(4)
        let path_data = encode_bip32_path(&params.path);
        let first_chunk_overhead = path_data.len() + 4; // +4 for message length

        if first_chunk_overhead >= length::MAX_MESSAGE_CHUNK_SIZE {
            return Err(EthAppError::InvalidBip32Path(
                "BIP32 path too long for message signing".to_string(),
            ));
        }

        let first_chunk_message_size = length::MAX_MESSAGE_CHUNK_SIZE - first_chunk_overhead;
        let subsequent_chunk_size = length::MAX_MESSAGE_CHUNK_SIZE;

        // Split message into chunks
        let (first_message_chunk, remaining_message) =
            if params.message.len() <= first_chunk_message_size {
                (params.message.as_slice(), &[][..])
            } else {
                (
                    &params.message[..first_chunk_message_size],
                    &params.message[first_chunk_message_size..],
                )
            };

        let remaining_chunks = chunk_data(remaining_message, subsequent_chunk_size);

        // Send first chunk with path and message length
        let mut first_chunk_data = Vec::new();
        first_chunk_data.extend_from_slice(&path_data);
        first_chunk_data.extend_from_slice(&(params.message.len() as u32).to_be_bytes());
        first_chunk_data.extend_from_slice(first_message_chunk);

        let first_command = APDUCommand {
            cla: Self::CLA,
            ins: ins::SIGN_ETH_PERSONAL_MESSAGE,
            p1: p1_sign_message::FIRST_DATA_BLOCK,
            p2: 0x00,
            data: first_chunk_data,
        };

        let mut response = transport
            .exchange(&first_command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response).map_err(EthAppError::Transport)?;

        // Send remaining chunks
        for chunk in remaining_chunks {
            let command = APDUCommand {
                cla: Self::CLA,
                ins: ins::SIGN_ETH_PERSONAL_MESSAGE,
                p1: p1_sign_message::SUBSEQUENT_DATA_BLOCK,
                p2: 0x00,
                data: chunk,
            };

            response = transport
                .exchange(&command)
                .await
                .map_err(|e| EthAppError::Transport(e.into()))?;

            <EthApp as AppExt<E>>::handle_response_error_signature(&response)
                .map_err(EthAppError::Transport)?;
        }

        // Parse signature from final response
        parse_signature_response::<E::Error>(response.data())
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
    fn test_sign_message_params() {
        let path = BipPath::ethereum_standard(0, 0);
        let message = b"Hello, Ethereum!".to_vec();
        let params = SignMessageParams::new(path.clone(), message.clone());

        assert_eq!(params.path, path);
        assert_eq!(params.message, message);
    }

    #[test]
    fn test_message_chunking_calculation() {
        let path = BipPath::new(vec![0x8000002C, 0x8000003C, 0x80000000]).unwrap();
        let path_data = encode_bip32_path(&path);
        let first_chunk_overhead = path_data.len() + 4; // +4 for message length

        // Should be: 1 (path_len) + 3*4 (indices) + 4 (message_len) = 17 bytes overhead
        assert_eq!(first_chunk_overhead, 17);

        let first_chunk_message_size = length::MAX_MESSAGE_CHUNK_SIZE - first_chunk_overhead;
        assert_eq!(first_chunk_message_size, 255 - 17); // 238 bytes for message in first chunk
    }
}
