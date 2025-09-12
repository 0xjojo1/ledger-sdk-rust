// SPDX-License-Identifier: Apache-2.0

//! Error types for Ethereum application

use ledger_device_base::LedgerAppError;
use thiserror::Error;

/// Ethereum application specific errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EthAppError<E: std::error::Error> {
    /// Error from the underlying transport/device
    #[error("Transport error: {0}")]
    Transport(#[from] LedgerAppError<E>),

    /// Invalid BIP32 derivation path
    #[error("Invalid BIP32 path: {0}")]
    InvalidBip32Path(String),

    /// Invalid Ethereum address format
    #[error("Invalid Ethereum address: {0}")]
    InvalidAddress(String),

    /// Invalid signature format
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Transaction data too large
    #[error("Transaction data too large: {size} bytes (max {max})")]
    TransactionTooLarge { size: usize, max: usize },

    /// Message data too large
    #[error("Message data too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },

    /// Invalid transaction format
    #[error("Invalid transaction format: {0}")]
    InvalidTransaction(String),

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Hex encoding/decoding error
    #[error("Hex error: {0}")]
    HexError(String),

    /// Invalid chain ID
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(u64),

    /// Device rejected the operation
    #[error("Operation rejected by device")]
    UserRejected,

    /// Application configuration error
    #[error("App configuration error: {0}")]
    ConfigurationError(String),

    /// Feature not supported by current app version
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    /// Data chunk error during multi-chunk operations
    #[error("Chunk error: {0}")]
    ChunkError(String),

    /// Invalid response data from device
    #[error("Invalid response data: {0}")]
    InvalidResponseData(String),
}

impl<E: std::error::Error> EthAppError<E> {
    /// Check if error is due to user rejection
    pub fn is_user_rejected(&self) -> bool {
        matches!(self, EthAppError::UserRejected)
    }

    /// Check if error is due to transport/communication issues
    pub fn is_transport_error(&self) -> bool {
        matches!(self, EthAppError::Transport(_))
    }

    /// Check if error is due to invalid input parameters
    pub fn is_invalid_input(&self) -> bool {
        matches!(
            self,
            EthAppError::InvalidBip32Path(_)
                | EthAppError::InvalidAddress(_)
                | EthAppError::InvalidSignature(_)
                | EthAppError::InvalidTransaction(_)
                | EthAppError::InvalidMessage(_)
                | EthAppError::InvalidChainId(_)
        )
    }
}

/// Result type alias for Ethereum application operations
pub type EthAppResult<T, E> = Result<T, EthAppError<E>>;
