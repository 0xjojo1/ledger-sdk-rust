// SPDX-License-Identifier: Apache-2.0

//! EIP-712 field definition encoding

use crate::errors::EthAppResult;
use crate::types::Eip712FieldDefinition;

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
