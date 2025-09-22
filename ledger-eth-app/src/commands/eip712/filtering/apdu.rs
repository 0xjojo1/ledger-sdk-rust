// SPDX-License-Identifier: Apache-2.0

//! EIP-712 filtering APDU implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use crate::commands::eip712::encoding::encode_filter_params;
use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, p1_eip712_filtering, p2_eip712_filtering};
use crate::types::Eip712FilterParams;
use crate::EthApp;

/// EIP-712 filtering trait
#[async_trait]
pub trait Eip712Filtering<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Send EIP-712 filtering configuration
    async fn send_filter_config(
        transport: &E,
        filter_params: &Eip712FilterParams,
    ) -> EthAppResult<(), E::Error>;

    /// Activate EIP-712 filtering
    async fn activate_filtering(transport: &E) -> EthAppResult<(), E::Error>;
}

#[async_trait]
impl<E> Eip712Filtering<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn send_filter_config(
        transport: &E,
        filter_params: &Eip712FilterParams,
    ) -> EthAppResult<(), E::Error> {
        let (p1, p2, data) = encode_filter_params::<E::Error>(filter_params)?;

        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::EIP712_FILTERING,
            p1,
            p2,
            data,
        };

        let response = transport
            .exchange(&command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| EthAppError::Transport(e))?;

        Ok(())
    }

    async fn activate_filtering(transport: &E) -> EthAppResult<(), E::Error> {
        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::EIP712_FILTERING,
            p1: p1_eip712_filtering::STANDARD,
            p2: p2_eip712_filtering::ACTIVATION,
            data: vec![],
        };

        let response = transport
            .exchange(&command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| EthAppError::Transport(e))?;

        Ok(())
    }
}
