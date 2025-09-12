// SPDX-License-Identifier: Apache-2.0

//! GET ETH PUBLIC ADDRESS command implementation

use async_trait::async_trait;
use ledger_device_base::{App, AppExt};
use ledger_transport::{APDUCommand, Exchange};

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::{ins, p1_get_address, p2_get_address};
use crate::types::{GetAddressParams, PublicKeyInfo};
use crate::utils::{
    encode_bip32_path, encode_chain_id, parse_device_address, parse_device_chain_code,
    parse_device_public_key, validate_bip32_path,
};
use crate::EthApp;

#[async_trait]
pub trait GetAddress<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Get Ethereum public address for the given BIP 32 path
    async fn get_address(
        transport: &E,
        params: GetAddressParams,
    ) -> EthAppResult<PublicKeyInfo, E::Error>;
}

#[async_trait]
impl<E> GetAddress<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn get_address(
        transport: &E,
        params: GetAddressParams,
    ) -> EthAppResult<PublicKeyInfo, E::Error> {
        // Validate BIP32 path
        validate_bip32_path(&params.path)?;

        // Prepare command data
        let mut data = Vec::new();

        // Add BIP32 path
        data.extend_from_slice(&encode_bip32_path(&params.path));

        // Add optional chain ID
        if let Some(chain_id) = params.chain_id {
            data.extend_from_slice(&encode_chain_id(chain_id));
        }

        // Set P1 parameter based on display requirement
        let p1 = if params.display {
            p1_get_address::DISPLAY_AND_CONFIRM
        } else {
            p1_get_address::RETURN_ADDRESS
        };

        // Set P2 parameter based on chain code requirement
        let p2 = if params.return_chain_code {
            p2_get_address::RETURN_CHAIN_CODE
        } else {
            p2_get_address::NO_CHAIN_CODE
        };

        // Build APDU command
        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::GET_ETH_PUBLIC_ADDRESS,
            p1,
            p2,
            data,
        };

        // Send command and get response
        let response = transport
            .exchange(&command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        // Handle APDU response
        <EthApp as AppExt<E>>::handle_response_error(&response)
            .map_err(|e| EthAppError::Transport(e))?;

        // Parse response data
        parse_get_address_response::<E::Error>(response.data(), params.return_chain_code)
    }
}

/// Parse GET ETH PUBLIC ADDRESS response data
fn parse_get_address_response<E: std::error::Error>(
    data: &[u8],
    return_chain_code: bool,
) -> EthAppResult<PublicKeyInfo, E> {
    let mut offset = 0;

    // Parse public key
    let (public_key, new_offset) = parse_device_public_key(data, offset)?;
    offset = new_offset;

    // Parse address
    let (address, new_offset) = parse_device_address(data, offset)?;
    offset = new_offset;

    // Parse optional chain code
    let (chain_code, _) = if return_chain_code {
        parse_device_chain_code(data, offset)?
    } else {
        (None, offset)
    };

    Ok(PublicKeyInfo {
        public_key,
        address,
        chain_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BipPath;

    #[test]
    fn test_parse_get_address_response_without_chain_code() {
        // Mock response data: pubkey_len(1) + pubkey(65) + addr_len(1) + addr(42)
        let mut response_data = Vec::new();

        // Public key (65 bytes uncompressed)
        response_data.push(65); // pubkey length
        response_data.extend(vec![0x04; 65]); // mock public key starting with 0x04

        // Address (42 characters)
        response_data.push(42); // address length
        response_data.extend(b"0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90");

        let result = parse_get_address_response::<std::io::Error>(&response_data, false);
        assert!(result.is_ok());

        let public_key_info = result.unwrap();
        assert_eq!(public_key_info.public_key.len(), 65);
        assert_eq!(
            public_key_info.address.address,
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90"
        );
        assert!(public_key_info.chain_code.is_none());
    }

    #[test]
    fn test_parse_get_address_response_with_chain_code() {
        // Mock response data: pubkey_len(1) + pubkey(65) + addr_len(1) + addr(42) + chain_code(32)
        let mut response_data = Vec::new();

        // Public key (65 bytes uncompressed)
        response_data.push(65); // pubkey length
        response_data.extend(vec![0x04; 65]); // mock public key starting with 0x04

        // Address (42 characters)
        response_data.push(42); // address length
        response_data.extend(b"0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90");

        // Chain code (32 bytes)
        response_data.extend(vec![0xAB; 32]);

        let result = parse_get_address_response::<std::io::Error>(&response_data, true);
        assert!(result.is_ok());

        let public_key_info = result.unwrap();
        assert_eq!(public_key_info.public_key.len(), 65);
        assert_eq!(
            public_key_info.address.address,
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90"
        );
        assert!(public_key_info.chain_code.is_some());
        assert_eq!(public_key_info.chain_code.unwrap().len(), 32);
    }

    #[test]
    fn test_get_address_params() {
        let path = BipPath::ethereum_standard(0, 0);
        let params = GetAddressParams::new(path.clone())
            .with_display()
            .with_chain_code()
            .with_chain_id(1);

        assert_eq!(params.path, path);
        assert!(params.display);
        assert!(params.return_chain_code);
        assert_eq!(params.chain_id, Some(1));
    }
}
