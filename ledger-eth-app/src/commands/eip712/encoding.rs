// SPDX-License-Identifier: Apache-2.0

//! EIP-712 encoding utilities
//!
//! This module contains utilities for encoding EIP-712 data structures into APDU format.

use crate::errors::EthAppResult;
use crate::instructions::{p1_eip712_filtering, p2_eip712_filtering};
use crate::types::{Eip712FieldDefinition, Eip712FilterParams, Eip712FilterType};

// Maximum APDU payload size for a single frame (data field only)
pub const APDU_MAX_PAYLOAD: usize = 255;

/// Encode EIP-712 field definition for APDU
pub fn encode_field_definition<E: std::error::Error>(
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
pub fn encode_filter_params<E: std::error::Error>(
    filter_params: &Eip712FilterParams,
) -> EthAppResult<(u8, u8, Vec<u8>), E> {
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
