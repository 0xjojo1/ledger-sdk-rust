// SPDX-License-Identifier: Apache-2.0

//! EIP-712 full signing implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use super::utils::parse_signature_response;
use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, p1_sign_eip712, p2_sign_eip712};
use crate::types::{BipPath, Signature};
use crate::utils::{encode_bip32_path, validate_bip32_path};
use crate::EthApp;

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

        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| EthAppError::Transport(e))?;

        // Parse signature from response
        parse_signature_response::<E::Error>(response.data())
    }
}
