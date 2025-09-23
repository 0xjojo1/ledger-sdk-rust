// SPDX-License-Identifier: Apache-2.0

//! Ledger Ethereum Application SDK
//!
//! This crate provides a comprehensive interface for interacting with the Ethereum application
//! on Ledger hardware wallets. It implements the full APDU command set as specified in the
//! Ethereum application technical documentation.
//!
//! ## Features
//!
//! - **Core Operations**: Get addresses, sign transactions, sign personal messages
//! - **Configuration**: Query application configuration and capabilities
//! - **BIP32 Support**: Full BIP32 derivation path support with validation
//! - **Chunked Operations**: Support for large data transmission via chunked APDU commands
//! - **Type Safety**: Strongly typed parameters and responses
//! - **Async/Await**: Fully async API using async-trait
//!
//!

use async_trait::async_trait;
use ledger_sdk_device_base::App;
use ledger_sdk_transport::Exchange;

// Re-export all public types and traits
pub mod commands;
pub mod errors;
pub mod instructions;
pub mod types;
pub mod utils;

pub use commands::*;
pub use errors::*;
pub use types::*;

/// Ethereum app marker implementing `App` trait CLA.
#[derive(Debug, Clone)]
pub struct EthApp;

impl App for EthApp {
    /// CLA for Ethereum app on Ledger (0xE0)
    const CLA: u8 = 0xE0;
}

/// High-level Ethereum application client
///
/// This struct provides a convenient interface for all Ethereum application operations.
/// It wraps the transport layer and provides type-safe methods for interacting with
/// the Ledger device.
#[derive(Debug)]
pub struct EthereumApp<E: Exchange> {
    transport: E,
}

impl<E: Exchange> EthereumApp<E> {
    /// Create a new Ethereum application client
    pub fn new(transport: E) -> Self {
        Self { transport }
    }

    /// Get a reference to the underlying transport
    pub fn transport(&self) -> &E {
        &self.transport
    }
}

#[async_trait]
impl<E> GetAddress<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn get_address(
        transport: &E,
        params: GetAddressParams,
    ) -> EthAppResult<PublicKeyInfo, E::Error> {
        EthApp::get_address(transport, params).await
    }
}

#[async_trait]
impl<E> GetConfiguration<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn get_configuration(transport: &E) -> EthAppResult<AppConfiguration, E::Error> {
        EthApp::get_configuration(transport).await
    }
}

#[async_trait]
impl<E> SignPersonalMessage<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_personal_message(
        transport: &E,
        params: SignMessageParams,
    ) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_personal_message(transport, params).await
    }
}

#[async_trait]
impl<E> SignTransaction<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_transaction(
        transport: &E,
        params: SignTransactionParams,
    ) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_transaction(transport, params).await
    }

    async fn sign_transaction_with_mode(
        transport: &E,
        params: SignTransactionParams,
        mode: commands::sign_transaction::TransactionMode,
    ) -> EthAppResult<Option<Signature>, E::Error> {
        EthApp::sign_transaction_with_mode(transport, params, mode).await
    }
}

#[async_trait]
impl<E> SignEip712V0<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_eip712_v0(
        transport: &E,
        params: SignEip712Params,
    ) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_eip712_v0(transport, params).await
    }
}

#[async_trait]
impl<E> SignEip712Full<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn sign_eip712_full(transport: &E, path: &BipPath) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_eip712_full(transport, path).await
    }
}

#[async_trait]
impl<E> Eip712StructDef<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn send_struct_definition(
        transport: &E,
        struct_def: &Eip712StructDefinition,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_struct_definition(transport, struct_def).await
    }
}

#[async_trait]
impl<E> Eip712StructImpl<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn send_struct_implementation(
        transport: &E,
        struct_impl: &Eip712StructImplementation,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_struct_implementation(transport, struct_impl).await
    }

    async fn set_array_size(transport: &E, size: u8) -> EthAppResult<(), E::Error> {
        EthApp::set_array_size(transport, size).await
    }
}

#[async_trait]
impl<E> Eip712Filtering<E> for EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn send_filter_config(
        transport: &E,
        filter_params: &Eip712FilterParams,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_filter_config(transport, filter_params).await
    }

    async fn activate_filtering(transport: &E) -> EthAppResult<(), E::Error> {
        EthApp::activate_filtering(transport).await
    }
}

impl<E> EthereumApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Get Ethereum public address for the given BIP 32 path
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters for address retrieval including path, display options, etc.
    ///
    /// # Returns
    ///
    /// Returns `PublicKeyInfo` containing the public key, address, and optionally chain code.
    ///
    ///
    pub async fn get_address(
        &self,
        params: GetAddressParams,
    ) -> EthAppResult<PublicKeyInfo, E::Error> {
        EthApp::get_address(&self.transport, params).await
    }

    /// Get Ethereum application configuration
    ///
    /// Returns information about the application's capabilities and version.
    ///
    ///
    pub async fn get_configuration(&self) -> EthAppResult<AppConfiguration, E::Error> {
        EthApp::get_configuration(&self.transport).await
    }

    /// Sign an Ethereum personal message
    ///
    /// Signs a message using the personal_sign specification. The message will be
    /// displayed on the device for user confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including BIP32 path and message data
    ///
    ///
    pub async fn sign_personal_message(
        &self,
        params: SignMessageParams,
    ) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_personal_message(&self.transport, params).await
    }

    /// Sign an Ethereum transaction
    ///
    /// Signs a transaction using the provided RLP-encoded transaction data.
    /// The transaction details will be displayed on the device for user confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including BIP32 path and RLP-encoded transaction data
    ///
    ///
    pub async fn sign_transaction(
        &self,
        params: SignTransactionParams,
    ) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_transaction(&self.transport, params).await
    }

    /// Sign an Ethereum transaction with specific processing mode
    ///
    /// Provides fine-grained control over transaction processing, allowing for
    /// operations like storing transaction data without immediate signing.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including BIP32 path and RLP-encoded transaction data
    /// * `mode` - Processing mode (ProcessAndStart, StoreOnly, or StartFlow)
    ///
    /// # Returns
    ///
    /// Returns `Some(Signature)` for modes that produce a signature, or `None` for store-only mode.
    pub async fn sign_transaction_with_mode(
        &self,
        params: SignTransactionParams,
        mode: commands::sign_transaction::TransactionMode,
    ) -> EthAppResult<Option<Signature>, E::Error> {
        EthApp::sign_transaction_with_mode(&self.transport, params, mode).await
    }

    /// Sign an EIP-712 message using v0 implementation (domain hash + message hash)
    ///
    /// This is the simpler EIP-712 signing mode where domain and message hashes
    /// are computed externally and provided directly to the device.
    ///
    /// **Version Requirements**: Requires app version >= 1.5.0
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including BIP32 path, domain hash, and message hash
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.5.0
    ///
    pub async fn sign_eip712_v0(
        &self,
        params: SignEip712Params,
    ) -> EthAppResult<Signature, E::Error> {
        // Check version requirement for EIP-712 v0 (>= 1.5.0)
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_v0() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 v0 requires app version >= 1.5.0, found {}",
                config.version
            )));
        }

        EthApp::sign_eip712_v0(&self.transport, params).await
    }

    /// Sign an EIP-712 message using full implementation
    ///
    /// This mode requires sending struct definitions and implementations before
    /// calling this final signing method. Use the struct definition and
    /// implementation methods first to set up the EIP-712 data.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `path` - BIP32 derivation path for the signing key
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn sign_eip712_full(&self, path: &BipPath) -> EthAppResult<Signature, E::Error> {
        // Check version requirement for EIP-712 full (>= 1.9.19)
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 full implementation requires app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::sign_eip712_full(&self.transport, path).await
    }

    /// Send EIP-712 struct definition to the device
    ///
    /// This method sends type definitions for EIP-712 structures. Must be called
    /// before sending struct implementations in full EIP-712 mode.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `struct_def` - The struct definition including name and field types
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn send_struct_definition(
        &self,
        struct_def: &Eip712StructDefinition,
    ) -> EthAppResult<(), E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 struct definitions require app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::send_struct_definition(&self.transport, struct_def).await
    }

    /// Send EIP-712 struct implementation to the device
    ///
    /// This method sends the actual data values for EIP-712 structures.
    /// Must be called after sending struct definitions.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `struct_impl` - The struct implementation with field values
    /// * `complete` - Whether this is a complete send or partial
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn send_struct_implementation(
        &self,
        struct_impl: &Eip712StructImplementation,
    ) -> EthAppResult<(), E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 struct implementations require app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::send_struct_implementation(&self.transport, struct_impl).await
    }

    /// Set array size for upcoming array fields in EIP-712 implementation
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the array
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn set_array_size(&self, size: u8) -> EthAppResult<(), E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 array operations require app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::set_array_size(&self.transport, size).await
    }

    /// Send EIP-712 filtering configuration
    ///
    /// Configure how EIP-712 data should be filtered and displayed on the device.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `filter_params` - Filtering parameters and configuration
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn send_filter_config(
        &self,
        filter_params: &Eip712FilterParams,
    ) -> EthAppResult<(), E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 filtering requires app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::send_filter_config(&self.transport, filter_params).await
    }

    /// Activate EIP-712 filtering on the device
    ///
    /// Must be called to enable filtering before sending struct definitions.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn activate_filtering(&self) -> EthAppResult<(), E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 filtering requires app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::activate_filtering(&self.transport).await
    }

    /// Sign EIP-712 typed data using the high-level API (matching viem interface)
    ///
    /// This method provides a simple interface for EIP-712 signing that matches the viem
    /// interface. It automatically handles the conversion from high-level typed data to
    /// the low-level struct definitions and implementations required by the Ledger device.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `path` - BIP32 derivation path for the signing key
    /// * `typed_data` - EIP-712 typed data structure matching viem interface
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ledger_eth_app::{Eip712Domain, Eip712Field, Eip712Struct, Eip712Types, Eip712TypedData};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let domain = Eip712Domain::new()
    ///     .with_name("Ether Mail".to_string())
    ///     .with_version("1".to_string())
    ///     .with_chain_id(1);
    ///
    /// let mut types = Eip712Types::new();
    /// types.insert(
    ///     "Person".to_string(),
    ///     Eip712Struct::new()
    ///         .with_field(Eip712Field::new("name".to_string(), "string".to_string()))
    ///         .with_field(Eip712Field::new("wallet".to_string(), "address".to_string())),
    /// );
    ///
    /// let message = json!({
    ///     "from": {
    ///         "name": "Cow",
    ///         "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
    ///     },
    ///     "to": {
    ///         "name": "Bob",
    ///         "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
    ///     },
    ///     "contents": "Hello, Bob!"
    /// });
    ///
    /// let typed_data = Eip712TypedData::new(domain, types, "Mail".to_string(), message);
    /// // let signature = app.sign_eip712_typed_data(&path, &typed_data).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    ///
    pub async fn sign_eip712_typed_data(
        &self,
        path: &BipPath,
        typed_data: &Eip712TypedData,
    ) -> EthAppResult<crate::types::Signature, E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 typed data signing requires app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::sign_eip712_typed_data(&self.transport, path, typed_data).await
    }

    /// Sign EIP-712 typed data from JSON string
    ///
    /// This method accepts a JSON string containing EIP-712 typed data and automatically
    /// parses, validates, and signs it. The JSON format should match the standard EIP-712
    /// structure with domain, types, primaryType, and message fields.
    ///
    /// **Version Requirements**: Requires app version >= 1.9.19
    ///
    /// # Arguments
    ///
    /// * `path` - BIP32 derivation path for the signing key
    /// * `json_str` - JSON string containing EIP-712 typed data
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let json_str = r#"{
    ///   "domain": {
    ///     "name": "USD Coin",
    ///     "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    ///     "chainId": 1,
    ///     "version": "2"
    ///   },
    ///   "primaryType": "Permit",
    ///   "message": {
    ///     "deadline": 1718992051,
    ///     "nonce": 0,
    ///     "spender": "0x111111125421ca6dc452d289314280a0f8842a65",
    ///     "owner": "0x6cbcd73cd8e8a42844662f0a0e76d7f79afd933d",
    ///     "value": "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    ///   },
    ///   "types": {
    ///     "EIP712Domain": [
    ///       {"name": "name", "type": "string"},
    ///       {"name": "version", "type": "string"},
    ///       {"name": "chainId", "type": "uint256"},
    ///       {"name": "verifyingContract", "type": "address"}
    ///     ],
    ///     "Permit": [
    ///       {"name": "owner", "type": "address"},
    ///       {"name": "spender", "type": "address"},
    ///       {"name": "value", "type": "uint256"},
    ///       {"name": "nonce", "type": "uint256"},
    ///       {"name": "deadline", "type": "uint256"}
    ///     ]
    ///   }
    /// }"#;
    ///
    /// // let signature = app.sign_eip712_from_json(&path, json_str).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `EthAppError::UnsupportedVersion` if app version is below 1.9.19
    /// Returns `EthAppError::InvalidEip712Data` if JSON format is invalid
    ///
    pub async fn sign_eip712_from_json(
        &self,
        path: &BipPath,
        json_str: &str,
    ) -> EthAppResult<crate::types::Signature, E::Error> {
        // Check version requirement for EIP-712 full implementation
        let config = self.get_configuration().await?;
        if !config.version.supports_eip712_full() {
            return Err(EthAppError::UnsupportedVersion(format!(
                "EIP-712 JSON signing requires app version >= 1.9.19, found {}",
                config.version
            )));
        }

        EthApp::sign_eip712_from_json(&self.transport, path, json_str).await
    }
}
