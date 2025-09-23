// SPDX-License-Identifier: Apache-2.0

//! EIP-712 signing functionality
//!
//! This module contains the EIP-712 signing implementations.

use async_trait::async_trait;
use ledger_sdk_device_base::{App, AppExt};
use ledger_sdk_transport::{APDUCommand, Exchange};

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, length, p1_sign_eip712, p2_sign_eip712};
use crate::types::{BipPath, SignEip712Params, Signature};
use crate::utils::{encode_bip32_path, validate_bip32_path};
use crate::EthApp;

/// Parse signature response data
pub fn parse_signature_response<E: std::error::Error>(data: &[u8]) -> EthAppResult<Signature, E> {
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

/// EIP-712 full implementation trait
#[async_trait]
pub trait SignEip712Full<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign an EIP-712 message using full implementation
    async fn sign_eip712_full(transport: &E, path: &BipPath) -> EthAppResult<Signature, E::Error>;
}

#[async_trait]
impl<E> SignEip712Full<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_eip712_full(transport: &E, path: &BipPath) -> EthAppResult<Signature, E::Error> {
        // Validate BIP32 path
        validate_bip32_path(path)?;

        let path_data = encode_bip32_path(path);

        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::SIGN_ETH_EIP712,
            p1: p1_sign_eip712::FIRST_CHUNK,
            p2: p2_sign_eip712::FULL_IMPLEMENTATION,
            data: path_data,
        };

        let response = transport
            .exchange(&command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response).map_err(EthAppError::Transport)?;

        // Parse signature from response
        parse_signature_response::<E::Error>(response.data())
    }
}

/// EIP-712 v0 signing trait (simple domain + message hash mode)
#[async_trait]
pub trait SignEip712V0<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign an EIP-712 message using v0 implementation (domain hash + message hash)
    async fn sign_eip712_v0(
        transport: &E,
        params: SignEip712Params,
    ) -> EthAppResult<Signature, E::Error>;
}

#[async_trait]
impl<E> SignEip712V0<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_eip712_v0(
        transport: &E,
        params: SignEip712Params,
    ) -> EthAppResult<Signature, E::Error> {
        // Validate BIP32 path
        validate_bip32_path(&params.path)?;

        // Validate hash sizes
        if params.domain_hash.len() != length::EIP712_DOMAIN_HASH_SIZE {
            return Err(EthAppError::InvalidEip712Data(format!(
                "Invalid domain hash size: {} (expected {})",
                params.domain_hash.len(),
                length::EIP712_DOMAIN_HASH_SIZE
            )));
        }

        if params.message_hash.len() != length::EIP712_MESSAGE_HASH_SIZE {
            return Err(EthAppError::InvalidEip712Data(format!(
                "Invalid message hash size: {} (expected {})",
                params.message_hash.len(),
                length::EIP712_MESSAGE_HASH_SIZE
            )));
        }

        // Prepare command data
        let path_data = encode_bip32_path(&params.path);
        let mut command_data = Vec::new();
        command_data.extend_from_slice(&path_data);
        command_data.extend_from_slice(&params.domain_hash);
        command_data.extend_from_slice(&params.message_hash);

        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::SIGN_ETH_EIP712,
            p1: p1_sign_eip712::FIRST_CHUNK,
            p2: p2_sign_eip712::V0_IMPLEMENTATION,
            data: command_data,
        };

        let response = transport
            .exchange(&command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response).map_err(EthAppError::Transport)?;

        // Parse signature from response
        parse_signature_response::<E::Error>(response.data())
    }
}
