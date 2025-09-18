// SPDX-License-Identifier: Apache-2.0

//! SIGN ETH EIP 712 command implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{
    ins, length, p1_eip712_filtering, p1_eip712_struct_impl, p1_sign_eip712, p2_eip712_filtering,
    p2_eip712_struct_def, p2_eip712_struct_impl, p2_sign_eip712,
};
use crate::types::{
    Eip712FieldDefinition, Eip712FilterParams, Eip712StructDefinition, Eip712StructImplementation,
    SignEip712Params, Signature,
};
use crate::utils::{encode_bip32_path, validate_bip32_path};
use crate::EthApp;

// Maximum APDU payload size for a single frame (data field only)
const APDU_MAX_PAYLOAD: usize = 255;

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

/// EIP-712 full implementation trait
#[async_trait]
pub trait SignEip712Full<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign an EIP-712 message using full implementation
    async fn sign_eip712_full(
        transport: &E,
        path: &crate::types::BipPath,
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

        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| EthAppError::Transport(e))?;

        // Parse signature from response
        parse_signature_response::<E::Error>(response.data())
    }
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
        for (index, value) in struct_impl.values.iter().enumerate() {
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

#[async_trait]
impl<E> SignEip712Full<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_eip712_full(
        transport: &E,
        path: &crate::types::BipPath,
    ) -> EthAppResult<Signature, E::Error> {
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

/// Encode EIP-712 field definition for APDU
fn encode_field_definition<E: std::error::Error>(
    field: &Eip712FieldDefinition,
) -> EthAppResult<Vec<u8>, E> {
    let mut data = Vec::new();

    // TypeDesc byte (according to Ledger documentation format)
    let mut type_desc = field.field_type.type_id();
    if field.is_array() {
        type_desc |= 0x80; // Set TypeArray bit (MSB)
    }
    if field.field_type.type_size().is_some() {
        type_desc |= 0x40; // Set TypeSize bit
    }
    data.push(type_desc);

    // TypeNameLength and TypeName (only for custom types, when Type=0)
    if let Some(type_name) = field.field_type.type_name() {
        data.push(type_name.len() as u8);
        data.extend_from_slice(type_name.as_bytes());
    }

    // TypeSize (if applicable)
    if let Some(type_size) = field.field_type.type_size() {
        data.push(type_size);
    }

    // ArrayLevelCount and ArrayLevels (if array)
    if field.is_array() {
        data.push(field.array_levels.len() as u8);
        for level in &field.array_levels {
            data.push(level.type_id());
            if let Some(size) = level.size() {
                data.push(size);
            }
        }
    }

    // KeyNameLength and KeyName (always present)
    data.push(field.name.len() as u8);
    data.extend_from_slice(field.name.as_bytes());

    Ok(data)
}

/// Encode filter parameters for APDU
fn encode_filter_params<E: std::error::Error>(
    filter_params: &Eip712FilterParams,
) -> EthAppResult<(u8, u8, Vec<u8>), E> {
    use crate::types::Eip712FilterType;

    let p1 = if filter_params.discarded {
        p1_eip712_filtering::DISCARDED
    } else {
        p1_eip712_filtering::STANDARD
    };

    let (p2, data) = match &filter_params.filter_type {
        Eip712FilterType::Activation => (p2_eip712_filtering::ACTIVATION, vec![]),

        Eip712FilterType::DiscardedFilterPath(path) => {
            let mut data = Vec::new();
            data.push(path.len() as u8);
            data.extend_from_slice(path.as_bytes());
            (p2_eip712_filtering::DISCARDED_FILTER_PATH, data)
        }

        Eip712FilterType::MessageInfo {
            display_name,
            filters_count,
            signature,
        } => {
            let mut data = Vec::new();
            data.push(display_name.len() as u8);
            data.extend_from_slice(display_name.as_bytes());
            data.push(*filters_count);
            data.push(signature.len() as u8);
            data.extend_from_slice(signature);
            (p2_eip712_filtering::MESSAGE_INFO, data)
        }

        Eip712FilterType::TrustedName {
            display_name,
            name_types,
            name_sources,
            signature,
        } => {
            let mut data = Vec::new();
            data.push(display_name.len() as u8);
            data.extend_from_slice(display_name.as_bytes());
            data.push(name_types.len() as u8);
            data.extend_from_slice(name_types);
            data.push(name_sources.len() as u8);
            data.extend_from_slice(name_sources);
            data.push(signature.len() as u8);
            data.extend_from_slice(signature);
            (p2_eip712_filtering::TRUSTED_NAME, data)
        }

        Eip712FilterType::DateTime {
            display_name,
            signature,
        } => {
            let mut data = Vec::new();
            data.push(display_name.len() as u8);
            data.extend_from_slice(display_name.as_bytes());
            data.push(signature.len() as u8);
            data.extend_from_slice(signature);
            (p2_eip712_filtering::DATE_TIME, data)
        }

        Eip712FilterType::AmountJoinToken {
            token_index,
            signature,
        } => {
            let mut data = Vec::new();
            data.push(*token_index);
            data.push(signature.len() as u8);
            data.extend_from_slice(signature);
            (p2_eip712_filtering::AMOUNT_JOIN_TOKEN, data)
        }

        Eip712FilterType::AmountJoinValue {
            display_name,
            token_index,
            signature,
        } => {
            let mut data = Vec::new();
            data.push(display_name.len() as u8);
            data.extend_from_slice(display_name.as_bytes());
            data.push(*token_index);
            data.push(signature.len() as u8);
            data.extend_from_slice(signature);
            (p2_eip712_filtering::AMOUNT_JOIN_VALUE, data)
        }

        Eip712FilterType::RawField {
            display_name,
            signature,
        } => {
            let mut data = Vec::new();
            data.push(display_name.len() as u8);
            data.extend_from_slice(display_name.as_bytes());
            data.push(signature.len() as u8);
            data.extend_from_slice(signature);
            (p2_eip712_filtering::RAW_FIELD, data)
        }
    };

    Ok((p1, p2, data))
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
    use crate::types::{BipPath, Eip712ArrayLevel, Eip712FieldType};

    #[test]
    fn test_sign_eip712_params() {
        let path = BipPath::ethereum_standard(0, 0);
        let domain_hash = [0xAA; 32];
        let message_hash = [0xBB; 32];
        let params = SignEip712Params::new(path.clone(), domain_hash, message_hash);

        assert_eq!(params.path, path);
        assert_eq!(params.domain_hash, domain_hash);
        assert_eq!(params.message_hash, message_hash);
    }

    #[test]
    fn test_encode_field_definition_simple() {
        let field = Eip712FieldDefinition::new(Eip712FieldType::Uint(32), "amount".to_string());

        let encoded = encode_field_definition::<std::io::Error>(&field).unwrap();

        // Should contain: type_desc(with type_size bit) + type_size + key_name_len + key_name
        assert_eq!(encoded[0], 0x42); // type_id=2(uint) | type_size_bit=0x40
        assert_eq!(encoded[1], 32); // type size
        assert_eq!(encoded[2], 6); // key name length
        assert_eq!(&encoded[3..9], b"amount");
    }

    #[test]
    fn test_encode_field_definition_array() {
        let field = Eip712FieldDefinition::new(Eip712FieldType::Address, "addresses".to_string())
            .with_array_level(Eip712ArrayLevel::Fixed(3));

        let encoded = encode_field_definition::<std::io::Error>(&field).unwrap();

        // Should have array bit set
        assert_eq!(encoded[0] & 0x80, 0x80); // array bit set
        assert_eq!(encoded[0] & 0x0F, 3); // type_id=3(address)
    }

    #[test]
    fn test_parse_signature_response() {
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
}
