// SPDX-License-Identifier: Apache-2.0

//! EIP-712 struct implementation APDU implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use crate::commands::eip712::encoding::APDU_MAX_PAYLOAD;
use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, p1_eip712_struct_impl, p2_eip712_struct_impl};
use crate::types::Eip712StructImplementation;
use crate::EthApp;

/// EIP-712 struct implementation trait
#[async_trait]
pub trait Eip712StructImpl<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Send EIP-712 struct implementation
    async fn send_struct_implementation(
        transport: &E,
        struct_impl: &Eip712StructImplementation,
    ) -> EthAppResult<(), E::Error>;

    /// Set array size for upcoming array fields
    async fn set_array_size(transport: &E, size: u8) -> EthAppResult<(), E::Error>;
}

#[async_trait]
impl<E> Eip712StructImpl<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn send_struct_implementation(
        transport: &E,
        struct_impl: &Eip712StructImplementation,
    ) -> EthAppResult<(), E::Error> {
        let struct_name_command = APDUCommand {
            cla: Self::CLA,
            ins: ins::EIP712_SEND_STRUCT_IMPLEMENTATION,
            p1: p1_eip712_struct_impl::COMPLETE_SEND,
            p2: p2_eip712_struct_impl::ROOT_STRUCT,
            data: struct_impl.name.as_bytes(),
        };

        let response = transport
            .exchange(&struct_name_command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| crate::errors::map_ledger_error(e))?;

        // Send each field value as FIELD type
        for (_, value) in struct_impl.values.iter().enumerate() {
            // Encode field value with a 2-byte big-endian length prefix
            let mut buffer = Vec::with_capacity(2 + value.value.len());
            buffer.extend_from_slice(&(value.value.len() as u16).to_be_bytes());
            buffer.extend_from_slice(&value.value);

            // Chunk the buffer into APDU_MAX_PAYLOAD-sized frames
            let mut offset = 0usize;
            while offset < buffer.len() {
                let end = core::cmp::min(offset + APDU_MAX_PAYLOAD, buffer.len());
                let chunk = &buffer[offset..end];
                let is_last_chunk = end == buffer.len();

                let p1 = if is_last_chunk {
                    p1_eip712_struct_impl::COMPLETE_SEND
                } else {
                    p1_eip712_struct_impl::PARTIAL_SEND
                };

                let field_command = APDUCommand {
                    cla: Self::CLA,
                    ins: ins::EIP712_SEND_STRUCT_IMPLEMENTATION,
                    p1,
                    p2: p2_eip712_struct_impl::STRUCT_FIELD,
                    data: chunk,
                };

                let response = transport
                    .exchange(&field_command)
                    .await
                    .map_err(|e| EthAppError::Transport(e.into()))?;

                <EthApp as AppExt<E>>::handle_response_error(&response)
                    .map_err(|e| EthAppError::Transport(e))?;

                offset = end;
            }
        }

        Ok(())
    }

    async fn set_array_size(transport: &E, size: u8) -> EthAppResult<(), E::Error> {
        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::EIP712_SEND_STRUCT_IMPLEMENTATION,
            p1: p1_eip712_struct_impl::PARTIAL_SEND,
            p2: p2_eip712_struct_impl::ARRAY,
            data: vec![size],
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
