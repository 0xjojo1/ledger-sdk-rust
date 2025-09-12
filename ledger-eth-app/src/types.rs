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
