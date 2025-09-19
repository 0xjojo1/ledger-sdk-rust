// SPDX-License-Identifier: Apache-2.0

//! High-level EIP-712 API matching viem interface
//!
//! This module provides a high-level API for EIP-712 signing that matches the viem interface,
//! making it easy to work with standard typed data structures.

use crate::commands::{Eip712StructDef, Eip712StructImpl, SignEip712Full};
use crate::errors::{EthAppError, EthAppResult};
use crate::types::{
    Eip712ArrayLevel, Eip712Domain, Eip712Field, Eip712FieldDefinition, Eip712FieldType,
    Eip712FieldValue, Eip712Struct, Eip712StructDefinition, Eip712StructImplementation,
    Eip712TypedData, Eip712Types,
};
use crate::utils::validate_bip32_path;
use crate::{BipPath, EthApp};
use async_trait::async_trait;
use ledger_transport::Exchange;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, Zero};
use serde_json::{from_str, Value};

/// High-level EIP-712 signing trait
#[async_trait]
pub trait SignEip712TypedData<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign EIP-712 typed data using the high-level API
    async fn sign_eip712_typed_data(
        transport: &E,
        path: &BipPath,
        typed_data: &Eip712TypedData,
    ) -> EthAppResult<crate::types::Signature, E::Error>;

    /// Sign EIP-712 typed data from JSON string
    async fn sign_eip712_from_json(
        transport: &E,
        path: &BipPath,
        json_str: &str,
    ) -> EthAppResult<crate::types::Signature, E::Error>;
}

/// Convert high-level EIP-712 types to low-level struct definitions
pub struct Eip712Converter;

impl Eip712Converter {
    /// Convert a high-level field type string to low-level Eip712FieldType
    pub fn parse_field_type(type_str: &str) -> Result<Eip712FieldType, String> {
        let type_str = type_str.trim();

        // Handle array types (e.g., "Person[]", "uint256[2]")
        if type_str.ends_with(']') {
            let (base_type, array_spec) = type_str
                .rsplit_once('[')
                .ok_or_else(|| format!("Invalid array type format: {}", type_str))?;

            let array_spec = array_spec.trim_end_matches(']');
            let _array_level = if array_spec.is_empty() {
                Eip712ArrayLevel::Dynamic
            } else {
                let size: u8 = array_spec
                    .parse()
                    .map_err(|_| format!("Invalid array size: {}", array_spec))?;
                Eip712ArrayLevel::Fixed(size)
            };

            let base_field_type = Self::parse_base_field_type(base_type)?;
            return Ok(base_field_type);
        }

        Self::parse_base_field_type(type_str)
    }

    /// Parse base field type (non-array)
    fn parse_base_field_type(type_str: &str) -> Result<Eip712FieldType, String> {
        match type_str {
            "bool" => Ok(Eip712FieldType::Bool),
            "address" => Ok(Eip712FieldType::Address),
            "string" => Ok(Eip712FieldType::String),
            "bytes" => Ok(Eip712FieldType::DynamicBytes),
            _ => {
                // Handle fixed-size bytes (e.g., "bytes32")
                if type_str.starts_with("bytes") {
                    let size_str = &type_str[5..];
                    if let Ok(size) = size_str.parse::<u8>() {
                        if size > 0 && size <= 32 {
                            return Ok(Eip712FieldType::FixedBytes(size));
                        }
                    }
                    return Err(format!("Invalid bytes size: {}", size_str));
                }

                // Handle integer types (e.g., "uint256", "int128")
                if type_str.starts_with("uint") {
                    let size_str = &type_str[4..];
                    if let Ok(size) = size_str.parse::<u16>() {
                        if size > 0 && size <= 256 && size % 8 == 0 {
                            return Ok(Eip712FieldType::Uint((size / 8) as u8));
                        }
                    }
                    return Err(format!("Invalid uint size: {}", size_str));
                }

                if type_str.starts_with("int") {
                    let size_str = &type_str[3..];
                    if let Ok(size) = size_str.parse::<u16>() {
                        if size > 0 && size <= 256 && size % 8 == 0 {
                            return Ok(Eip712FieldType::Int((size / 8) as u8));
                        }
                    }
                    return Err(format!("Invalid int size: {}", size_str));
                }

                // Custom struct type
                Ok(Eip712FieldType::Custom(type_str.to_string()))
            }
        }
    }

    /// Convert high-level EIP-712 types to low-level struct definitions
    pub fn convert_types_to_definitions(
        types: &Eip712Types,
    ) -> Result<Vec<Eip712StructDefinition>, String> {
        let mut definitions = Vec::new();

        for (struct_name, struct_def) in types {
            let mut fields = Vec::new();

            for field in &struct_def.fields {
                let field_type = Self::parse_field_type(&field.r#type)?;
                let field_def = Eip712FieldDefinition::new(field_type, field.name.clone());
                fields.push(field_def);
            }

            let definition = Eip712StructDefinition {
                name: struct_name.clone(),
                fields,
            };

            definitions.push(definition);
        }

        Ok(definitions)
    }

    /// Convert message value to field value
    pub fn convert_value_to_field_value(
        value: &Value,
        field_type: &Eip712FieldType,
    ) -> Result<Eip712FieldValue, String> {
        match field_type {
            Eip712FieldType::Bool => {
                let bool_val = value
                    .as_bool()
                    .ok_or_else(|| "Expected boolean value".to_string())?;
                Ok(Eip712FieldValue::from_bool(bool_val))
            }
            Eip712FieldType::Address => {
                let addr_str = value
                    .as_str()
                    .ok_or_else(|| "Expected string value for address".to_string())?;
                Eip712FieldValue::from_address_string(addr_str)
            }
            Eip712FieldType::String => {
                let str_val = value
                    .as_str()
                    .ok_or_else(|| "Expected string value".to_string())?;
                Ok(Eip712FieldValue::from_string(str_val))
            }
            Eip712FieldType::Uint(size) => {
                let bytes = Self::parse_uint_to_min_be(value, *size)?;
                Ok(Eip712FieldValue::from_bytes(bytes))
            }
            Eip712FieldType::Int(size) => {
                let bytes = Self::parse_int_to_min_be(value, *size)?;
                Ok(Eip712FieldValue::from_bytes(bytes))
            }
            Eip712FieldType::FixedBytes(size) => {
                let hex_str = value
                    .as_str()
                    .ok_or_else(|| "Expected hex string for bytes".to_string())?;
                let bytes = hex::decode(hex_str.trim_start_matches("0x"))
                    .map_err(|e| format!("Invalid hex string: {}", e))?;
                if bytes.len() != *size as usize {
                    return Err(format!("Expected {} bytes, got {}", size, bytes.len()));
                }
                Ok(Eip712FieldValue::from_bytes(bytes))
            }
            Eip712FieldType::DynamicBytes => {
                let hex_str = value
                    .as_str()
                    .ok_or_else(|| "Expected hex string for bytes".to_string())?;
                let bytes = hex::decode(hex_str.trim_start_matches("0x"))
                    .map_err(|e| format!("Invalid hex string: {}", e))?;
                Ok(Eip712FieldValue::from_bytes(bytes))
            }
            Eip712FieldType::Custom(_) => {
                // For custom structs, we return an empty value as the struct reference
                Ok(Eip712FieldValue::from_struct())
            }
        }
    }

    /// Parse unsigned integer (uintN) from JSON number or string into minimal big-endian bytes (with range check)
    fn parse_uint_to_min_be(value: &Value, size_bytes: u8) -> Result<Vec<u8>, String> {
        let bits: u32 = (size_bytes as u32) * 8;
        // Parse into BigUint
        let big: BigUint = if let Some(u) = value.as_u64() {
            BigUint::from(u)
        } else if let Some(s) = value.as_str() {
            let s = s.trim();
            if s.starts_with("0x") || s.starts_with("0X") {
                let hex_str = &s[2..];
                let bytes = hex::decode(hex_str)
                    .map_err(|e| format!("Invalid hex for uint{}: {}", bits, e))?;
                BigUint::from_bytes_be(&bytes)
            } else {
                BigUint::parse_bytes(s.as_bytes(), 10)
                    .ok_or_else(|| format!("Invalid decimal string for uint{}", bits))?
            }
        } else {
            return Err(format!(
                "Expected number or numeric string for uint{}",
                bits
            ));
        };

        // Range check: 0 <= big < 2^(bits)
        let max = BigUint::one() << bits;
        if big >= max {
            return Err(format!("uint{} value out of range", bits));
        }

        // Minimal big-endian: 0 => [0x00], otherwise trim leading zeros
        if big.is_zero() {
            return Ok(vec![0u8]);
        }
        let mut out = big.to_bytes_be();
        while out.len() > 1 && out[0] == 0 {
            out.remove(0);
        }
        // Still ensure it could fit in size_bytes if needed by device constraints
        if out.len() > size_bytes as usize {
            return Err(format!(
                "uint{} minimal encoding exceeds {} bytes",
                bits, size_bytes
            ));
        }
        Ok(out)
    }

    /// Parse signed integer (intN) from JSON number or string into minimal two's-complement big-endian bytes (with range check)
    fn parse_int_to_min_be(value: &Value, size_bytes: u8) -> Result<Vec<u8>, String> {
        let bits: u32 = (size_bytes as u32) * 8;
        // Parse into BigInt
        let big: BigInt = if let Some(i) = value.as_i64() {
            BigInt::from(i)
        } else if let Some(s) = value.as_str() {
            let s = s.trim();
            // Support optional leading '-'
            if s.starts_with("-0x") || s.starts_with("-0X") {
                let hex_str = &s[3..];
                let bytes = hex::decode(hex_str)
                    .map_err(|e| format!("Invalid hex for int{}: {}", bits, e))?;
                -BigInt::from(BigUint::from_bytes_be(&bytes))
            } else if s.starts_with("0x") || s.starts_with("0X") {
                let hex_str = &s[2..];
                let bytes = hex::decode(hex_str)
                    .map_err(|e| format!("Invalid hex for int{}: {}", bits, e))?;
                BigInt::from(BigUint::from_bytes_be(&bytes))
            } else {
                BigInt::parse_bytes(s.as_bytes(), 10)
                    .ok_or_else(|| format!("Invalid decimal string for int{}", bits))?
            }
        } else {
            return Err(format!("Expected number or numeric string for int{}", bits));
        };

        // Range: -(2^(bits-1)) ..= 2^(bits-1)-1
        let one = BigUint::one();
        let max_pos = (one.clone() << (bits - 1)) - one.clone();
        let min_neg = -BigInt::from(one.clone() << (bits - 1));
        if big < min_neg || big > BigInt::from(max_pos.clone()) {
            return Err(format!("int{} value out of range", bits));
        }

        // Two's complement representation modulo 2^bits
        let modulus = one << bits;
        let as_uint = if big.sign() == Sign::Minus {
            let abs = (-&big).to_biguint().unwrap();
            let val = (&modulus - abs) % &modulus;
            val
        } else {
            big.to_biguint().unwrap()
        };
        let mut full = as_uint.to_bytes_be();
        // Left pad to at least 1 byte
        if full.is_empty() {
            full.push(0);
        }
        // Ensure we have at most size_bytes to start with (range already checked)
        if full.len() > size_bytes as usize {
            return Err(format!(
                "int{} minimal encoding exceeds {} bytes",
                bits, size_bytes
            ));
        }
        // Trim redundant sign extension:
        // For negative numbers, while first byte == 0xFF and next byte has MSB 1, drop first byte
        // For non-negative, while first byte == 0x00 and next byte MSB 0, drop first byte
        if big.sign() == Sign::Minus {
            while full.len() > 1 && full[0] == 0xFF && (full[1] & 0x80) == 0x80 {
                full.remove(0);
            }
        } else {
            while full.len() > 1 && full[0] == 0x00 && (full[1] & 0x80) == 0x00 {
                full.remove(0);
            }
        }
        Ok(full)
    }

    /// Convert message data to struct implementation
    pub fn convert_message_to_implementation(
        message: &Value,
        primary_type: &str,
        types: &Eip712Types,
    ) -> Result<Eip712StructImplementation, String> {
        let struct_def = types
            .get(primary_type)
            .ok_or_else(|| format!("Primary type '{}' not found in types", primary_type))?;

        let mut values = Vec::new();

        for field in &struct_def.fields {
            let field_value = message
                .get(&field.name)
                .ok_or_else(|| format!("Field '{}' not found in message", field.name))?;

            let field_type = Self::parse_field_type(&field.r#type)?;
            let field_val = Self::convert_value_to_field_value(field_value, &field_type)?;
            values.push(field_val);
        }

        Ok(Eip712StructImplementation {
            name: primary_type.to_string(),
            values,
        })
    }

    /// Build a JSON Value object for EIP712Domain from the typed domain struct
    fn build_domain_json(domain: &Eip712Domain) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(name) = &domain.name {
            map.insert("name".to_string(), Value::String(name.clone()));
        }
        if let Some(version) = &domain.version {
            map.insert("version".to_string(), Value::String(version.clone()));
        }
        if let Some(chain_id) = domain.chain_id {
            map.insert("chainId".to_string(), Value::Number(chain_id.into()));
        }
        if let Some(addr) = &domain.verifying_contract {
            map.insert("verifyingContract".to_string(), Value::String(addr.clone()));
        }
        if let Some(salt_bytes) = &domain.salt {
            let mut s = String::from("0x");
            s.push_str(&hex::encode(salt_bytes));
            map.insert("salt".to_string(), Value::String(s));
        }
        Value::Object(map)
    }

    /// Parse and validate JSON string to EIP-712 typed data
    pub fn parse_json_to_typed_data(json_str: &str) -> Result<Eip712TypedData, String> {
        // Parse JSON
        let json_value: Value =
            from_str(json_str).map_err(|e| format!("Invalid JSON format: {}", e))?;

        // Validate required fields
        if !json_value.is_object() {
            return Err("JSON must be an object".to_string());
        }

        let obj = json_value.as_object().unwrap();

        // Parse domain
        let domain_value = obj
            .get("domain")
            .ok_or_else(|| "Missing 'domain' field".to_string())?;
        let domain: Eip712Domain = Self::parse_domain(domain_value)?;

        // Parse types
        let types_value = obj
            .get("types")
            .ok_or_else(|| "Missing 'types' field".to_string())?;
        let types = Self::parse_types(types_value)?;

        // Parse primary type
        let primary_type: String = obj
            .get("primaryType")
            .ok_or_else(|| "Missing 'primaryType' field".to_string())?
            .as_str()
            .ok_or_else(|| "primaryType must be a string".to_string())?
            .to_string();

        // Parse message
        let message = obj
            .get("message")
            .ok_or_else(|| "Missing 'message' field".to_string())?
            .clone();

        // Validate that primary type exists in types
        if !types.contains_key(&primary_type) {
            return Err(format!(
                "Primary type '{}' not found in types",
                primary_type
            ));
        }

        Ok(Eip712TypedData::new(domain, types, primary_type, message))
    }

    /// Parse domain from JSON value
    fn parse_domain(domain_value: &Value) -> Result<Eip712Domain, String> {
        if !domain_value.is_object() {
            return Err("Domain must be an object".to_string());
        }

        let domain_obj = domain_value.as_object().unwrap();
        let mut domain = Eip712Domain::new();

        if let Some(name) = domain_obj.get("name") {
            if let Some(name_str) = name.as_str() {
                domain = domain.with_name(name_str.to_string());
            }
        }

        if let Some(version) = domain_obj.get("version") {
            if let Some(version_str) = version.as_str() {
                domain = domain.with_version(version_str.to_string());
            }
        }

        if let Some(chain_id) = domain_obj.get("chainId") {
            if let Some(chain_id_num) = chain_id.as_u64() {
                domain = domain.with_chain_id(chain_id_num);
            }
        }

        if let Some(verifying_contract) = domain_obj.get("verifyingContract") {
            if let Some(contract_str) = verifying_contract.as_str() {
                domain = domain.with_verifying_contract(contract_str.to_string());
            }
        }

        if let Some(salt) = domain_obj.get("salt") {
            if let Some(salt_str) = salt.as_str() {
                let salt_bytes = hex::decode(salt_str.trim_start_matches("0x"))
                    .map_err(|e| format!("Invalid salt hex: {}", e))?;
                domain = domain.with_salt(salt_bytes);
            }
        }

        Ok(domain)
    }

    /// Parse types from JSON value
    fn parse_types(types_value: &Value) -> Result<Eip712Types, String> {
        if !types_value.is_object() {
            return Err("Types must be an object".to_string());
        }

        let types_obj = types_value.as_object().unwrap();
        let mut types = Eip712Types::new();

        for (type_name, type_def) in types_obj {
            if !type_def.is_array() {
                return Err(format!("Type '{}' definition must be an array", type_name));
            }

            let fields_array = type_def.as_array().unwrap();
            let mut fields = Vec::new();

            for field_value in fields_array {
                if !field_value.is_object() {
                    return Err(format!("Field in type '{}' must be an object", type_name));
                }

                let field_obj = field_value.as_object().unwrap();

                let name = field_obj
                    .get("name")
                    .ok_or_else(|| format!("Field in type '{}' missing 'name'", type_name))?
                    .as_str()
                    .ok_or_else(|| format!("Field name in type '{}' must be a string", type_name))?
                    .to_string();

                let field_type = field_obj
                    .get("type")
                    .ok_or_else(|| {
                        format!("Field '{}' in type '{}' missing 'type'", name, type_name)
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        format!(
                            "Field type for '{}' in type '{}' must be a string",
                            name, type_name
                        )
                    })?
                    .to_string();

                fields.push(Eip712Field::new(name, field_type));
            }

            types.insert(type_name.clone(), Eip712Struct { fields });
        }

        Ok(types)
    }
}

#[async_trait]
impl<E> SignEip712TypedData<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_eip712_typed_data(
        transport: &E,
        path: &BipPath,
        typed_data: &Eip712TypedData,
    ) -> EthAppResult<crate::types::Signature, E::Error> {
        // Validate BIP32 path
        validate_bip32_path(path)?;

        // Convert high-level types to low-level struct definitions
        let struct_definitions = Eip712Converter::convert_types_to_definitions(&typed_data.types)
            .map_err(|e| EthAppError::InvalidEip712Data(e))?;

        // Send all struct definitions in deterministic order: alphabetical by name
        let mut defs_sorted = struct_definitions.clone();
        defs_sorted.sort_by(|a, b| a.name.cmp(&b.name));
        for struct_def in &defs_sorted {
            EthApp::send_struct_definition(transport, struct_def).await?;
        }

        // Convert message to struct implementation
        // First, send EIP712Domain implementation if defined in types
        if typed_data.types.contains_key("EIP712Domain") {
            // Some Ledger firmware expect a canonical EIP712Domain value order.
            // Build the domain implementation explicitly in the order:
            // name, version, chainId, verifyingContract (when present)
            let mut domain_values: Vec<Eip712FieldValue> = Vec::new();

            if let Some(name) = &typed_data.domain.name {
                domain_values.push(Eip712FieldValue::from_string(name));
            }
            if let Some(version) = &typed_data.domain.version {
                domain_values.push(Eip712FieldValue::from_string(version));
            }
            if let Some(chain_id) = typed_data.domain.chain_id {
                // Encode as minimal big-endian for uint256
                let chain_id_val = serde_json::Value::Number(chain_id.into());
                let bytes = Eip712Converter::parse_uint_to_min_be(&chain_id_val, 32)
                    .map_err(|e| EthAppError::InvalidEip712Data(e))?;
                domain_values.push(Eip712FieldValue::from_bytes(bytes));
            }
            if let Some(addr) = &typed_data.domain.verifying_contract {
                let addr_val = Eip712FieldValue::from_address_string(addr)
                    .map_err(|e| EthAppError::InvalidEip712Data(e))?;
                domain_values.push(addr_val);
            }

            let domain_impl = Eip712StructImplementation {
                name: "EIP712Domain".to_string(),
                values: domain_values,
            };

            EthApp::send_struct_implementation(transport, &domain_impl).await?;
        }

        let struct_implementation = Eip712Converter::convert_message_to_implementation(
            &typed_data.message,
            &typed_data.primary_type,
            &typed_data.types,
        )
        .map_err(|e| EthAppError::InvalidEip712Data(e))?;

        // Send message struct implementation
        EthApp::send_struct_implementation(transport, &struct_implementation).await?;

        // Perform the final signing
        EthApp::sign_eip712_full(transport, path).await
    }

    async fn sign_eip712_from_json(
        transport: &E,
        path: &BipPath,
        json_str: &str,
    ) -> EthAppResult<crate::types::Signature, E::Error> {
        // Parse and validate JSON string
        let typed_data = Eip712Converter::parse_json_to_typed_data(json_str)
            .map_err(|e| EthAppError::InvalidEip712Data(e))?;

        println!("typed_data: {:?}", &typed_data);

        // Use the existing typed data signing method
        Self::sign_eip712_typed_data(transport, path, &typed_data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_field_type() {
        assert_eq!(
            Eip712Converter::parse_field_type("bool").unwrap(),
            Eip712FieldType::Bool
        );
        assert_eq!(
            Eip712Converter::parse_field_type("address").unwrap(),
            Eip712FieldType::Address
        );
        assert_eq!(
            Eip712Converter::parse_field_type("string").unwrap(),
            Eip712FieldType::String
        );
        assert_eq!(
            Eip712Converter::parse_field_type("bytes").unwrap(),
            Eip712FieldType::DynamicBytes
        );
        assert_eq!(
            Eip712Converter::parse_field_type("bytes32").unwrap(),
            Eip712FieldType::FixedBytes(32)
        );
        assert_eq!(
            Eip712Converter::parse_field_type("uint256").unwrap(),
            Eip712FieldType::Uint(32)
        );
        assert_eq!(
            Eip712Converter::parse_field_type("int128").unwrap(),
            Eip712FieldType::Int(16)
        );
        assert_eq!(
            Eip712Converter::parse_field_type("Person").unwrap(),
            Eip712FieldType::Custom("Person".to_string())
        );
    }

    #[test]
    fn test_parse_array_field_type() {
        let field_type = Eip712Converter::parse_field_type("Person[]").unwrap();
        assert_eq!(field_type, Eip712FieldType::Custom("Person".to_string()));

        let field_type = Eip712Converter::parse_field_type("uint256[3]").unwrap();
        assert_eq!(field_type, Eip712FieldType::Uint(32));
    }

    #[test]
    fn test_convert_value_to_field_value() {
        // Test bool
        let value = json!(true);
        let field_value =
            Eip712Converter::convert_value_to_field_value(&value, &Eip712FieldType::Bool).unwrap();
        assert_eq!(field_value.value, vec![1]);

        // Test address
        let value = json!("0x1234567890123456789012345678901234567890");
        let field_value =
            Eip712Converter::convert_value_to_field_value(&value, &Eip712FieldType::Address)
                .unwrap();
        assert_eq!(field_value.value.len(), 20);

        // Test string
        let value = json!("Hello, World!");
        let field_value =
            Eip712Converter::convert_value_to_field_value(&value, &Eip712FieldType::String)
                .unwrap();
        assert_eq!(field_value.value, b"Hello, World!");
    }

    #[test]
    fn test_convert_message_to_implementation() {
        let mut types = Eip712Types::new();
        types.insert(
            "Person".to_string(),
            Eip712Struct::new()
                .with_field(Eip712Field::new("name".to_string(), "string".to_string()))
                .with_field(Eip712Field::new(
                    "wallet".to_string(),
                    "address".to_string(),
                )),
        );

        let message = json!({
            "name": "Alice",
            "wallet": "0x1234567890123456789012345678901234567890"
        });

        let implementation =
            Eip712Converter::convert_message_to_implementation(&message, "Person", &types).unwrap();
        assert_eq!(implementation.name, "Person");
        assert_eq!(implementation.values.len(), 2);
    }
}
