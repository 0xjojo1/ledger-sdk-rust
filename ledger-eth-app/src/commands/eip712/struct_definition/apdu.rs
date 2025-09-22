// SPDX-License-Identifier: Apache-2.0

//! EIP-712 struct definition APDU implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use crate::commands::eip712::encoding::encode_field_definition;
use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, p2_eip712_struct_def};
use crate::types::Eip712StructDefinition;
use crate::EthApp;

/// EIP-712 struct definition trait
#[async_trait]
pub trait Eip712StructDef<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Send EIP-712 struct definition
    async fn send_struct_definition(
        transport: &E,
        struct_def: &Eip712StructDefinition,
    ) -> EthAppResult<(), E::Error>;
}

#[async_trait]
impl<E> Eip712StructDef<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn send_struct_definition(
        transport: &E,
        struct_def: &Eip712StructDefinition,
    ) -> EthAppResult<(), E::Error> {
        let struct_name_command = APDUCommand {
            cla: Self::CLA,
            ins: ins::EIP712_SEND_STRUCT_DEFINITION,
            p1: 0x00,
            p2: p2_eip712_struct_def::STRUCT_NAME,
            data: struct_def.name.as_bytes(),
        };

        let response = transport
            .exchange(&struct_name_command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| crate::errors::map_ledger_error(e))?;

        // Send each field definition
        for field in &struct_def.fields {
            let encoded_field = encode_field_definition::<E::Error>(field)?;

            let field_command = APDUCommand {
                cla: Self::CLA,
                ins: ins::EIP712_SEND_STRUCT_DEFINITION,
                p1: 0x00,
                p2: p2_eip712_struct_def::STRUCT_FIELD,
                data: encoded_field,
            };

            let response = transport
                .exchange(&field_command)
                .await
                .map_err(|e| EthAppError::Transport(e.into()))?;

            <EthApp as AppExt<E>>::handle_response_error(&response)
                .map_err(|e| crate::errors::map_ledger_error(e))?;
        }

        Ok(())
    }
}
