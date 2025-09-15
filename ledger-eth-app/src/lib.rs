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
use ledger_device_base::App;
use ledger_transport::Exchange;

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
        complete: bool,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_struct_implementation(transport, struct_impl, complete).await
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
    /// # Arguments
    ///
    /// * `params` - Parameters including BIP32 path, domain hash, and message hash
    ///
    ///
    pub async fn sign_eip712_v0(
        &self,
        params: SignEip712Params,
    ) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_eip712_v0(&self.transport, params).await
    }

    /// Sign an EIP-712 message using full implementation
    ///
    /// This mode requires sending struct definitions and implementations before
    /// calling this final signing method. Use the struct definition and
    /// implementation methods first to set up the EIP-712 data.
    ///
    /// # Arguments
    ///
    /// * `path` - BIP32 derivation path for the signing key
    ///
    ///
    pub async fn sign_eip712_full(&self, path: &BipPath) -> EthAppResult<Signature, E::Error> {
        EthApp::sign_eip712_full(&self.transport, path).await
    }

    /// Send EIP-712 struct definition to the device
    ///
    /// This method sends type definitions for EIP-712 structures. Must be called
    /// before sending struct implementations in full EIP-712 mode.
    ///
    /// # Arguments
    ///
    /// * `struct_def` - The struct definition including name and field types
    ///
    ///
    pub async fn send_struct_definition(
        &self,
        struct_def: &Eip712StructDefinition,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_struct_definition(&self.transport, struct_def).await
    }

    /// Send EIP-712 struct implementation to the device
    ///
    /// This method sends the actual data values for EIP-712 structures.
    /// Must be called after sending struct definitions.
    ///
    /// # Arguments
    ///
    /// * `struct_impl` - The struct implementation with field values
    /// * `complete` - Whether this is a complete send or partial
    ///
    ///
    pub async fn send_struct_implementation(
        &self,
        struct_impl: &Eip712StructImplementation,
        complete: bool,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_struct_implementation(&self.transport, struct_impl, complete).await
    }

    /// Set array size for upcoming array fields in EIP-712 implementation
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the array
    ///
    ///
    pub async fn set_array_size(&self, size: u8) -> EthAppResult<(), E::Error> {
        EthApp::set_array_size(&self.transport, size).await
    }

    /// Send EIP-712 filtering configuration
    ///
    /// Configure how EIP-712 data should be filtered and displayed on the device.
    ///
    /// # Arguments
    ///
    /// * `filter_params` - Filtering parameters and configuration
    ///
    ///
    pub async fn send_filter_config(
        &self,
        filter_params: &Eip712FilterParams,
    ) -> EthAppResult<(), E::Error> {
        EthApp::send_filter_config(&self.transport, filter_params).await
    }

    /// Activate EIP-712 filtering on the device
    ///
    /// Must be called to enable filtering before sending struct definitions.
    ///
    ///
    pub async fn activate_filtering(&self) -> EthAppResult<(), E::Error> {
        EthApp::activate_filtering(&self.transport).await
    }
}
