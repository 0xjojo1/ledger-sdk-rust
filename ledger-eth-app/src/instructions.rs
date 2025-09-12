// SPDX-License-Identifier: Apache-2.0

//! APDU instruction constants for Ethereum application

/// APDU instruction codes for Ethereum application
pub mod ins {
    /// GET ETH PUBLIC ADDRESS
    pub const GET_ETH_PUBLIC_ADDRESS: u8 = 0x02;
    /// SIGN ETH TRANSACTION
    pub const SIGN_ETH_TRANSACTION: u8 = 0x04;
    /// GET APP CONFIGURATION
    pub const GET_APP_CONFIGURATION: u8 = 0x06;
    /// SIGN ETH PERSONAL MESSAGE
    pub const SIGN_ETH_PERSONAL_MESSAGE: u8 = 0x08;
    /// PROVIDE ERC 20 TOKEN INFORMATION
    pub const PROVIDE_ERC20_TOKEN_INFO: u8 = 0x0A;
    /// SIGN ETH EIP 712
    pub const SIGN_ETH_EIP712: u8 = 0x0C;
    /// GET ETH2 PUBLIC KEY
    pub const GET_ETH2_PUBLIC_KEY: u8 = 0x0E;
    /// SET ETH2 WITHDRAWAL INDEX
    pub const SET_ETH2_WITHDRAWAL_INDEX: u8 = 0x10;
    /// SET EXTERNAL PLUGIN
    pub const SET_EXTERNAL_PLUGIN: u8 = 0x12;
    /// PROVIDE NFT INFORMATION
    pub const PROVIDE_NFT_INFORMATION: u8 = 0x14;
    /// SET PLUGIN
    pub const SET_PLUGIN: u8 = 0x16;
    /// PERFORM PRIVACY OPERATION
    pub const PERFORM_PRIVACY_OPERATION: u8 = 0x18;
    /// EIP712 SEND STRUCT DEFINITION
    pub const EIP712_SEND_STRUCT_DEFINITION: u8 = 0x1A;
    /// EIP712 SEND STRUCT IMPLEMENTATION
    pub const EIP712_SEND_STRUCT_IMPLEMENTATION: u8 = 0x1C;
    /// EIP712 FILTERING
    pub const EIP712_FILTERING: u8 = 0x1E;
    /// GET CHALLENGE
    pub const GET_CHALLENGE: u8 = 0x20;
    /// PROVIDE DOMAIN NAME
    pub const PROVIDE_DOMAIN_NAME: u8 = 0x22;
    /// PROVIDE NETWORK INFORMATION
    pub const PROVIDE_NETWORK_INFORMATION: u8 = 0x30;
    /// PROVIDE TX SIMULATION
    pub const PROVIDE_TX_SIMULATION: u8 = 0x32;
    /// SIGN EIP 7702 AUTHORIZATION
    pub const SIGN_EIP7702_AUTHORIZATION: u8 = 0x34;
    /// PROVIDE SAFE ACCOUNT
    pub const PROVIDE_SAFE_ACCOUNT: u8 = 0x36;
}

/// P1 parameter constants for GET ETH PUBLIC ADDRESS
pub mod p1_get_address {
    /// Return address without confirmation
    pub const RETURN_ADDRESS: u8 = 0x00;
    /// Display address and confirm before returning
    pub const DISPLAY_AND_CONFIRM: u8 = 0x01;
}

/// P2 parameter constants for GET ETH PUBLIC ADDRESS
pub mod p2_get_address {
    /// Do not return the chain code
    pub const NO_CHAIN_CODE: u8 = 0x00;
    /// Return the chain code
    pub const RETURN_CHAIN_CODE: u8 = 0x01;
}

/// P1 parameter constants for SIGN ETH TRANSACTION
pub mod p1_sign_transaction {
    /// First transaction data block
    pub const FIRST_DATA_BLOCK: u8 = 0x00;
    /// Subsequent transaction data block
    pub const SUBSEQUENT_DATA_BLOCK: u8 = 0x80;
}

/// P2 parameter constants for SIGN ETH TRANSACTION
pub mod p2_sign_transaction {
    /// Process and start flow
    pub const PROCESS_AND_START: u8 = 0x00;
    /// Store only
    pub const STORE_ONLY: u8 = 0x01;
    /// Start flow
    pub const START_FLOW: u8 = 0x02;
}

/// P1 parameter constants for SIGN ETH PERSONAL MESSAGE
pub mod p1_sign_message {
    /// First message data block
    pub const FIRST_DATA_BLOCK: u8 = 0x00;
    /// Subsequent message data block
    pub const SUBSEQUENT_DATA_BLOCK: u8 = 0x80;
}

/// P1 parameter constants for PERFORM PRIVACY OPERATION
pub mod p1_privacy_operation {
    /// Return data without confirmation
    pub const RETURN_DATA: u8 = 0x00;
    /// Display data and confirm before returning
    pub const DISPLAY_AND_CONFIRM: u8 = 0x01;
}

/// P2 parameter constants for PERFORM PRIVACY OPERATION
pub mod p2_privacy_operation {
    /// Return the public encryption key
    pub const RETURN_PUBLIC_KEY: u8 = 0x00;
    /// Return the shared secret
    pub const RETURN_SHARED_SECRET: u8 = 0x01;
}

/// P1 parameter constants for GET ETH2 PUBLIC KEY
pub mod p1_get_eth2_key {
    /// Return public key without confirmation
    pub const RETURN_KEY: u8 = 0x00;
    /// Display public key and confirm before returning
    pub const DISPLAY_AND_CONFIRM: u8 = 0x01;
}

/// Data length constants
pub mod length {
    /// Maximum BIP 32 derivation path depth
    pub const MAX_BIP32_PATH_DEPTH: usize = 10;
    /// Size of each BIP 32 derivation index
    pub const BIP32_INDEX_SIZE: usize = 4;
    /// Size of chain ID
    pub const CHAIN_ID_SIZE: usize = 8;
    /// Size of Ethereum address
    pub const ETH_ADDRESS_SIZE: usize = 20;
    /// Size of chain code
    pub const CHAIN_CODE_SIZE: usize = 32;
    /// Size of signature component (r or s)
    pub const SIGNATURE_COMPONENT_SIZE: usize = 32;
    /// Size of signature recovery value (v)
    pub const SIGNATURE_V_SIZE: usize = 1;
    /// Maximum message chunk size for chunked operations
    pub const MAX_MESSAGE_CHUNK_SIZE: usize = 255;
}

/// App configuration flags
pub mod config_flags {
    /// Arbitrary data signature enabled by user
    pub const ARBITRARY_DATA_SIGNATURE: u8 = 0x01;
    /// ERC 20 Token information needs to be provided externally
    pub const ERC20_EXTERNAL_INFO: u8 = 0x02;
    /// Transaction Check enabled
    pub const TRANSACTION_CHECK_ENABLED: u8 = 0x10;
    /// Transaction Check Opt-In done
    pub const TRANSACTION_CHECK_OPT_IN: u8 = 0x20;
}
