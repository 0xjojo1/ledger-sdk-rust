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

    /// Invalid EIP-712 data
    #[error("Invalid EIP-712 data: {0}")]
    InvalidEip712Data(String),

    /// EIP-712 struct definition error
    #[error("EIP-712 struct error: {0}")]
    Eip712StructError(String),

    /// EIP-712 filtering error
    #[error("EIP-712 filter error: {0}")]
    Eip712FilterError(String),

    /// Unsupported app version
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    /// Device returned a specific status word
    #[error("Device status 0x{sw:04X}: {description}")]
    DeviceStatus { sw: u16, description: String },
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

/// Map LedgerAppError to Ethereum app specific error with SW decoding when possible
pub fn map_ledger_error<E: std::error::Error>(err: LedgerAppError<E>) -> EthAppError<E> {
    match err {
        // User cancel / security status not satisfied
        LedgerAppError::AppSpecific(sw, _) if sw == 0x6982 => EthAppError::UserRejected,
        LedgerAppError::Unknown(sw) if sw == 0x6982 => EthAppError::UserRejected,

        // Map known ETH app status words to descriptions
        LedgerAppError::AppSpecific(sw, _) | LedgerAppError::Unknown(sw) => {
            EthAppError::DeviceStatus {
                sw,
                description: describe_eth_status(sw).to_string(),
            }
        }

        // Fallback: treat as transport-layer app error
        other => EthAppError::Transport(other),
    }
}

/// ETH app specific status word descriptions (subset per spec)
fn describe_eth_status(sw: u16) -> &'static str {
    match sw {
        0x6001 => "Mode check fail",
        0x6501 => "TransactionType not supported",
        0x6502 => "Output buffer too small for chainId conversion",
        0x6982 => "Security status not satisfied (Canceled by user)",
        0x6983 => "Wrong Data length",
        0x6984 => "Plugin not installed",
        0x6985 => "Condition not satisfied",
        0x6A00 => "Error without info",
        0x6A80 => "Invalid data",
        0x6A84 => "Insufficient memory",
        0x6A88 => "Data not found",
        0x6B00 => "Incorrect parameter P1 or P2",
        0x6D00 => "Incorrect parameter INS",
        0x6E00 => "Incorrect parameter CLA",
        0x9000 => "Normal ending of the command",
        0x911C => "Command code not supported (Ledger-PKI not yet available)",
        _ if (sw & 0xFF00) == 0x6800 => "Internal error (Please report)",
        _ if (sw & 0xFF00) == 0x6F00 => "Technical problem (Internal error, please report)",
        _ => "Unknown status",
    }
}
