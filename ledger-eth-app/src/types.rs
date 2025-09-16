// SPDX-License-Identifier: Apache-2.0

//! Core data types for Ethereum application

use serde::{Deserialize, Serialize};
use std::fmt;

/// BIP32 derivation path for Ethereum accounts
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipPath {
    /// Derivation indices (max 10 levels)
    pub indices: Vec<u32>,
}

impl BipPath {
    /// Create a new BIP32 path from derivation indices
    pub fn new(indices: Vec<u32>) -> Result<Self, String> {
        if indices.len() > crate::instructions::length::MAX_BIP32_PATH_DEPTH {
            return Err(format!(
                "BIP32 path too deep: {} (max {})",
                indices.len(),
                crate::instructions::length::MAX_BIP32_PATH_DEPTH
            ));
        }
        Ok(BipPath { indices })
    }

    /// Create a standard Ethereum derivation path: m/44'/60'/account'/0/address_index
    pub fn ethereum_standard(account: u32, address_index: u32) -> Self {
        BipPath {
            indices: vec![
                0x8000002C,           // 44' (hardened)
                0x8000003C,           // 60' (hardened) - Ethereum
                0x80000000 | account, // account' (hardened)
                0,                    // 0 (external chain)
                address_index,        // address index
            ],
        }
    }

    /// Get the encoded length for APDU
    pub fn encoded_len(&self) -> usize {
        1 + self.indices.len() * crate::instructions::length::BIP32_INDEX_SIZE
    }
}

impl fmt::Display for BipPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m")?;
        for index in &self.indices {
            if *index >= 0x80000000 {
                write!(f, "/{}'", index - 0x80000000)?;
            } else {
                write!(f, "/{}", index)?;
            }
        }
        Ok(())
    }
}

/// Ethereum address information
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EthAddress {
    /// ASCII-encoded Ethereum address (with 0x prefix)
    pub address: String,
}

impl EthAddress {
    /// Create a new Ethereum address from hex string
    pub fn new(address: String) -> Result<Self, String> {
        if !address.starts_with("0x") {
            return Err("Ethereum address must start with 0x".to_string());
        }
        if address.len() != 42 {
            return Err("Ethereum address must be 42 characters long".to_string());
        }
        Ok(EthAddress { address })
    }

    /// Get the address without 0x prefix
    pub fn without_prefix(&self) -> &str {
        &self.address[2..]
    }

    /// Get the raw bytes of the address
    pub fn to_bytes(&self) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(self.without_prefix())
    }
}

impl fmt::Display for EthAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

/// Public key information returned from device
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    /// Uncompressed public key (65 bytes)
    pub public_key: Vec<u8>,
    /// Ethereum address derived from public key
    pub address: EthAddress,
    /// Optional chain code (32 bytes) if requested
    pub chain_code: Option<Vec<u8>>,
}

/// Signature result from signing operations
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Recovery value (0 or 1)
    pub v: u8,
    /// Signature component r (32 bytes)
    pub r: Vec<u8>,
    /// Signature component s (32 bytes)
    pub s: Vec<u8>,
}

impl Signature {
    /// Create a new signature from components
    pub fn new(v: u8, r: Vec<u8>, s: Vec<u8>) -> Result<Self, String> {
        if r.len() != crate::instructions::length::SIGNATURE_COMPONENT_SIZE {
            return Err(format!("Invalid r length: {} (expected 32)", r.len()));
        }
        if s.len() != crate::instructions::length::SIGNATURE_COMPONENT_SIZE {
            return Err(format!("Invalid s length: {} (expected 32)", s.len()));
        }
        Ok(Signature { v, r, s })
    }

    /// Get the signature in DER format
    pub fn to_der(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(self.v);
        result.extend_from_slice(&self.r);
        result.extend_from_slice(&self.s);
        result
    }
}

/// Application configuration information
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfiguration {
    /// Configuration flags
    pub flags: ConfigFlags,
    /// Application version
    pub version: AppVersion,
}

/// Configuration flags for the Ethereum application
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFlags {
    /// Arbitrary data signature enabled by user
    pub arbitrary_data_signature: bool,
    /// ERC 20 Token information needs to be provided externally
    pub erc20_external_info: bool,
    /// Transaction Check enabled
    pub transaction_check_enabled: bool,
    /// Transaction Check Opt-In done
    pub transaction_check_opt_in: bool,
}

impl ConfigFlags {
    /// Parse configuration flags from raw byte
    pub fn from_byte(flags: u8) -> Self {
        ConfigFlags {
            arbitrary_data_signature: (flags
                & crate::instructions::config_flags::ARBITRARY_DATA_SIGNATURE)
                != 0,
            erc20_external_info: (flags & crate::instructions::config_flags::ERC20_EXTERNAL_INFO)
                != 0,
            transaction_check_enabled: (flags
                & crate::instructions::config_flags::TRANSACTION_CHECK_ENABLED)
                != 0,
            transaction_check_opt_in: (flags
                & crate::instructions::config_flags::TRANSACTION_CHECK_OPT_IN)
                != 0,
        }
    }

    /// Convert configuration flags to raw byte
    pub fn to_byte(&self) -> u8 {
        let mut flags = 0u8;
        if self.arbitrary_data_signature {
            flags |= crate::instructions::config_flags::ARBITRARY_DATA_SIGNATURE;
        }
        if self.erc20_external_info {
            flags |= crate::instructions::config_flags::ERC20_EXTERNAL_INFO;
        }
        if self.transaction_check_enabled {
            flags |= crate::instructions::config_flags::TRANSACTION_CHECK_ENABLED;
        }
        if self.transaction_check_opt_in {
            flags |= crate::instructions::config_flags::TRANSACTION_CHECK_OPT_IN;
        }
        flags
    }
}

/// Application version information
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppVersion {
    /// Major version
    pub major: u8,
    /// Minor version
    pub minor: u8,
    /// Patch version
    pub patch: u8,
}

impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl AppVersion {
    /// Create a new AppVersion
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this version supports EIP-712 v0 implementation (>= 1.5.0)
    pub fn supports_eip712_v0(&self) -> bool {
        self.major > 1 || (self.major == 1 && self.minor >= 5)
    }

    /// Check if this version supports EIP-712 full implementation (>= 1.9.19)
    pub fn supports_eip712_full(&self) -> bool {
        self.major > 1
            || (self.major == 1 && self.minor > 9)
            || (self.major == 1 && self.minor == 9 && self.patch >= 19)
    }

    /// Compare with another version
    pub fn compare(&self, other: &AppVersion) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }

    /// Check if this version is greater than or equal to another version
    pub fn is_at_least(&self, other: &AppVersion) -> bool {
        matches!(
            self.compare(other),
            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal
        )
    }
}

/// Parameters for GET ETH PUBLIC ADDRESS command
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetAddressParams {
    /// BIP32 derivation path
    pub path: BipPath,
    /// Whether to display address on device and require confirmation
    pub display: bool,
    /// Whether to return chain code
    pub return_chain_code: bool,
    /// Optional chain ID for validation
    pub chain_id: Option<u64>,
}

impl GetAddressParams {
    /// Create new parameters for getting an address
    pub fn new(path: BipPath) -> Self {
        GetAddressParams {
            path,
            display: false,
            return_chain_code: false,
            chain_id: None,
        }
    }

    /// Enable display and confirmation on device
    pub fn with_display(mut self) -> Self {
        self.display = true;
        self
    }

    /// Enable chain code return
    pub fn with_chain_code(mut self) -> Self {
        self.return_chain_code = true;
        self
    }

    /// Set chain ID for validation
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }
}

/// Parameters for SIGN ETH TRANSACTION command
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignTransactionParams {
    /// BIP32 derivation path
    pub path: BipPath,
    /// RLP-encoded transaction data
    pub transaction_data: Vec<u8>,
}

impl SignTransactionParams {
    /// Create new parameters for signing a transaction
    pub fn new(path: BipPath, transaction_data: Vec<u8>) -> Self {
        SignTransactionParams {
            path,
            transaction_data,
        }
    }
}

/// Parameters for SIGN ETH PERSONAL MESSAGE command
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignMessageParams {
    /// BIP32 derivation path
    pub path: BipPath,
    /// Message data to sign
    pub message: Vec<u8>,
}

impl SignMessageParams {
    /// Create new parameters for signing a personal message
    pub fn new(path: BipPath, message: Vec<u8>) -> Self {
        SignMessageParams { path, message }
    }
}

/// EIP-712 implementation mode
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eip712Mode {
    /// v0 implementation: provides domain hash and message hash directly
    V0Implementation,
    /// Full implementation: uses JSON data and performs hashing on device
    FullImplementation,
}

/// Parameters for SIGN ETH EIP 712 command (v0 mode)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignEip712Params {
    /// BIP32 derivation path
    pub path: BipPath,
    /// Domain hash (32 bytes)
    pub domain_hash: [u8; 32],
    /// Message hash (32 bytes)
    pub message_hash: [u8; 32],
}

impl SignEip712Params {
    /// Create new parameters for EIP-712 v0 signing
    pub fn new(path: BipPath, domain_hash: [u8; 32], message_hash: [u8; 32]) -> Self {
        SignEip712Params {
            path,
            domain_hash,
            message_hash,
        }
    }
}

/// EIP-712 field type enumeration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eip712FieldType {
    /// Custom struct type
    Custom(String),
    /// Integer type with size in bytes
    Int(u8),
    /// Unsigned integer type with size in bytes
    Uint(u8),
    /// Ethereum address type
    Address,
    /// Boolean type
    Bool,
    /// String type
    String,
    /// Fixed-size bytes with size
    FixedBytes(u8),
    /// Dynamic-size bytes
    DynamicBytes,
}

impl Eip712FieldType {
    /// Get the type ID for encoding
    pub fn type_id(&self) -> u8 {
        match self {
            Eip712FieldType::Custom(_) => 0,
            Eip712FieldType::Int(_) => 1,
            Eip712FieldType::Uint(_) => 2,
            Eip712FieldType::Address => 3,
            Eip712FieldType::Bool => 4,
            Eip712FieldType::String => 5,
            Eip712FieldType::FixedBytes(_) => 6,
            Eip712FieldType::DynamicBytes => 7,
        }
    }

    /// Get the type size if applicable
    pub fn type_size(&self) -> Option<u8> {
        match self {
            Eip712FieldType::Int(size) => Some(*size),
            Eip712FieldType::Uint(size) => Some(*size),
            Eip712FieldType::FixedBytes(size) => Some(*size),
            _ => None,
        }
    }

    /// Get the type name for custom types
    pub fn type_name(&self) -> Option<&str> {
        match self {
            Eip712FieldType::Custom(name) => Some(name),
            _ => None,
        }
    }
}

/// EIP-712 array level type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eip712ArrayLevel {
    /// Dynamic array (type[])
    Dynamic,
    /// Fixed-size array (type[N])
    Fixed(u8),
}

impl Eip712ArrayLevel {
    /// Get the array level type ID for encoding
    pub fn type_id(&self) -> u8 {
        match self {
            Eip712ArrayLevel::Dynamic => 0,
            Eip712ArrayLevel::Fixed(_) => 1,
        }
    }

    /// Get the array size if fixed
    pub fn size(&self) -> Option<u8> {
        match self {
            Eip712ArrayLevel::Fixed(size) => Some(*size),
            Eip712ArrayLevel::Dynamic => None,
        }
    }
}

/// EIP-712 struct field definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eip712FieldDefinition {
    /// Field data type
    pub field_type: Eip712FieldType,
    /// Field name
    pub name: String,
    /// Array levels (empty if not an array)
    pub array_levels: Vec<Eip712ArrayLevel>,
}

impl Eip712FieldDefinition {
    /// Create a new field definition
    pub fn new(field_type: Eip712FieldType, name: String) -> Self {
        Eip712FieldDefinition {
            field_type,
            name,
            array_levels: Vec::new(),
        }
    }

    /// Add an array level to the field
    pub fn with_array_level(mut self, level: Eip712ArrayLevel) -> Self {
        self.array_levels.push(level);
        self
    }

    /// Check if this field is an array
    pub fn is_array(&self) -> bool {
        !self.array_levels.is_empty()
    }
}

/// EIP-712 struct definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eip712StructDefinition {
    /// Struct name
    pub name: String,
    /// Struct fields
    pub fields: Vec<Eip712FieldDefinition>,
}

impl Eip712StructDefinition {
    /// Create a new struct definition
    pub fn new(name: String) -> Self {
        Eip712StructDefinition {
            name,
            fields: Vec::new(),
        }
    }

    /// Add a field to the struct
    pub fn with_field(mut self, field: Eip712FieldDefinition) -> Self {
        self.fields.push(field);
        self
    }

    /// Sort fields alphabetically by name (important for EIP-712 hash consistency)
    pub fn with_sorted_fields(mut self) -> Self {
        self.fields.sort_by(|a, b| a.name.cmp(&b.name));
        self
    }
}

/// EIP-712 struct implementation value
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eip712FieldValue {
    /// Raw value data
    pub value: Vec<u8>,
}

impl Eip712FieldValue {
    /// Create a new field value
    pub fn new(value: Vec<u8>) -> Self {
        Eip712FieldValue { value }
    }

    /// Create from a string value
    pub fn from_string(s: &str) -> Self {
        Eip712FieldValue {
            value: s.as_bytes().to_vec(),
        }
    }

    /// Create from a u256 value
    pub fn from_u256(value: &[u8; 32]) -> Self {
        Eip712FieldValue {
            value: value.to_vec(),
        }
    }

    /// Create from an address
    pub fn from_address(address: &[u8; 20]) -> Self {
        Eip712FieldValue {
            value: address.to_vec(),
        }
    }

    /// Create from a boolean
    pub fn from_bool(value: bool) -> Self {
        Eip712FieldValue {
            value: vec![if value { 1 } else { 0 }],
        }
    }

    /// Create from a uint value (defaults to 8-byte u64)
    pub fn from_uint(value: u64) -> Self {
        Eip712FieldValue {
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Create from a uint32 value (4 bytes)
    pub fn from_uint32(value: u32) -> Self {
        Eip712FieldValue {
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Create from an address string (hex format)
    pub fn from_address_string(address: &str) -> Result<Self, String> {
        // Remove 0x prefix if present
        let hex_str = if address.starts_with("0x") {
            &address[2..]
        } else {
            address
        };

        // Validate length
        if hex_str.len() != 40 {
            return Err(format!(
                "Invalid address length: expected 40 hex characters, got {}",
                hex_str.len()
            ));
        }

        // Parse hex
        let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex: {}", e))?;
        if bytes.len() != 20 {
            return Err("Address must be 20 bytes".to_string());
        }

        Ok(Eip712FieldValue { value: bytes })
    }

    /// Create a reference to a nested struct (empty value for struct references)
    pub fn from_struct() -> Self {
        Eip712FieldValue { value: vec![] }
    }
}

/// EIP-712 struct implementation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eip712StructImplementation {
    /// Struct name
    pub name: String,
    /// Field values in order
    pub values: Vec<Eip712FieldValue>,
}

impl Eip712StructImplementation {
    /// Create a new struct implementation
    pub fn new(name: String) -> Self {
        Eip712StructImplementation {
            name,
            values: Vec::new(),
        }
    }

    /// Add a field value
    pub fn with_value(mut self, value: Eip712FieldValue) -> Self {
        self.values.push(value);
        self
    }
}

/// EIP-712 filtering operation type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eip712FilterType {
    /// Activation
    Activation,
    /// Discarded filter path
    DiscardedFilterPath(String),
    /// Message info
    MessageInfo {
        display_name: String,
        filters_count: u8,
        signature: Vec<u8>,
    },
    /// Trusted name
    TrustedName {
        display_name: String,
        name_types: Vec<u8>,
        name_sources: Vec<u8>,
        signature: Vec<u8>,
    },
    /// Date/time
    DateTime {
        display_name: String,
        signature: Vec<u8>,
    },
    /// Amount-join token
    AmountJoinToken { token_index: u8, signature: Vec<u8> },
    /// Amount-join value
    AmountJoinValue {
        display_name: String,
        token_index: u8,
        signature: Vec<u8>,
    },
    /// Raw field
    RawField {
        display_name: String,
        signature: Vec<u8>,
    },
}

/// Parameters for EIP-712 filtering operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eip712FilterParams {
    /// Filter operation type
    pub filter_type: Eip712FilterType,
    /// Whether this filter is discarded
    pub discarded: bool,
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn test_version_display() {
        let version = AppVersion::new(1, 9, 19);
        assert_eq!(version.to_string(), "1.9.19");
    }

    #[test]
    fn test_eip712_v0_support() {
        // Supported versions
        assert!(AppVersion::new(1, 5, 0).supports_eip712_v0());
        assert!(AppVersion::new(1, 6, 0).supports_eip712_v0());
        assert!(AppVersion::new(2, 0, 0).supports_eip712_v0());
        assert!(AppVersion::new(1, 9, 19).supports_eip712_v0());

        // Unsupported versions
        assert!(!AppVersion::new(1, 4, 99).supports_eip712_v0());
        assert!(!AppVersion::new(1, 0, 0).supports_eip712_v0());
        assert!(!AppVersion::new(0, 9, 0).supports_eip712_v0());
    }

    #[test]
    fn test_eip712_full_support() {
        // Supported versions
        assert!(AppVersion::new(1, 9, 19).supports_eip712_full());
        assert!(AppVersion::new(1, 9, 20).supports_eip712_full());
        assert!(AppVersion::new(1, 10, 0).supports_eip712_full());
        assert!(AppVersion::new(2, 0, 0).supports_eip712_full());

        // Unsupported versions
        assert!(!AppVersion::new(1, 9, 18).supports_eip712_full());
        assert!(!AppVersion::new(1, 8, 99).supports_eip712_full());
        assert!(!AppVersion::new(1, 5, 0).supports_eip712_full());
        assert!(!AppVersion::new(0, 9, 19).supports_eip712_full());
    }

    #[test]
    fn test_version_comparison() {
        let v1_5_0 = AppVersion::new(1, 5, 0);
        let v1_9_19 = AppVersion::new(1, 9, 19);
        let v2_0_0 = AppVersion::new(2, 0, 0);

        assert!(v1_9_19.is_at_least(&v1_5_0));
        assert!(v2_0_0.is_at_least(&v1_9_19));
        assert!(v1_5_0.is_at_least(&v1_5_0)); // Equal

        assert!(!v1_5_0.is_at_least(&v1_9_19));
        assert!(!v1_9_19.is_at_least(&v2_0_0));
    }
}
