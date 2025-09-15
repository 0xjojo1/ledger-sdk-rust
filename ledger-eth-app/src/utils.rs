// SPDX-License-Identifier: Apache-2.0

//! Utility functions for Ethereum application

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::length;
use crate::types::{BipPath, EthAddress};

/// Encode BIP32 path for APDU command
pub fn encode_bip32_path(path: &BipPath) -> Vec<u8> {
    let mut encoded = Vec::new();

    // Add path length
    encoded.push(path.indices.len() as u8);

    // Add each index in big-endian format
    for &index in &path.indices {
        encoded.extend_from_slice(&index.to_be_bytes());
    }

    encoded
}

/// Decode BIP32 path from APDU response data
pub fn decode_bip32_path<E: std::error::Error>(data: &[u8]) -> EthAppResult<(BipPath, usize), E> {
    if data.is_empty() {
        return Err(EthAppError::InvalidBip32Path("Empty path data".to_string()));
    }

    let path_len = data[0] as usize;
    if path_len > length::MAX_BIP32_PATH_DEPTH {
        return Err(EthAppError::InvalidBip32Path(format!(
            "Path too deep: {} (max {})",
            path_len,
            length::MAX_BIP32_PATH_DEPTH
        )));
    }

    let expected_size = 1 + path_len * length::BIP32_INDEX_SIZE;
    if data.len() < expected_size {
        return Err(EthAppError::InvalidBip32Path(format!(
            "Insufficient data: {} bytes (expected {})",
            data.len(),
            expected_size
        )));
    }

    let mut indices = Vec::with_capacity(path_len);
    let mut offset = 1;

    for _ in 0..path_len {
        let index_bytes = &data[offset..offset + length::BIP32_INDEX_SIZE];
        let index = u32::from_be_bytes([
            index_bytes[0],
            index_bytes[1],
            index_bytes[2],
            index_bytes[3],
        ]);
        indices.push(index);
        offset += length::BIP32_INDEX_SIZE;
    }

    let path = BipPath::new(indices).map_err(|e| EthAppError::InvalidBip32Path(e))?;

    Ok((path, offset))
}

/// Validate BIP32 path for Ethereum usage
pub fn validate_bip32_path<E: std::error::Error>(path: &BipPath) -> EthAppResult<(), E> {
    if path.indices.is_empty() {
        return Err(EthAppError::InvalidBip32Path("Empty path".to_string()));
    }

    if path.indices.len() > length::MAX_BIP32_PATH_DEPTH {
        return Err(EthAppError::InvalidBip32Path(format!(
            "Path too deep: {} (max {})",
            path.indices.len(),
            length::MAX_BIP32_PATH_DEPTH
        )));
    }

    // Validate Ethereum standard path format (optional)
    if path.indices.len() >= 2 {
        // Check for standard Ethereum derivation (m/44'/60'/...)
        if path.indices.len() >= 3 && path.indices[0] == 0x8000002C && path.indices[1] == 0x8000003C
        {
            // Standard Ethereum path: ensure account is hardened
            if (path.indices[2] & 0x80000000) == 0 {
                return Err(EthAppError::InvalidBip32Path(
                    "Account index should be hardened for Ethereum".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Encode chain ID for APDU command (8 bytes big-endian)
pub fn encode_chain_id(chain_id: u64) -> Vec<u8> {
    chain_id.to_be_bytes().to_vec()
}

/// Decode chain ID from APDU response data
pub fn decode_chain_id<E: std::error::Error>(data: &[u8]) -> EthAppResult<u64, E> {
    if data.len() < length::CHAIN_ID_SIZE {
        return Err(EthAppError::InvalidResponseData(format!(
            "Insufficient data for chain ID: {} bytes (expected {})",
            data.len(),
            length::CHAIN_ID_SIZE
        )));
    }

    let chain_id_bytes = &data[0..length::CHAIN_ID_SIZE];
    let chain_id = u64::from_be_bytes([
        chain_id_bytes[0],
        chain_id_bytes[1],
        chain_id_bytes[2],
        chain_id_bytes[3],
        chain_id_bytes[4],
        chain_id_bytes[5],
        chain_id_bytes[6],
        chain_id_bytes[7],
    ]);

    Ok(chain_id)
}

/// Split data into chunks for multi-chunk APDU operations
pub fn chunk_data(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    if chunk_size == 0 {
        return vec![data.to_vec()];
    }

    data.chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

/// Validate Ethereum address format
pub fn validate_ethereum_address<E: std::error::Error>(address: &str) -> EthAppResult<(), E> {
    println!("validate_ethereum_address: {}", address);

    if !address.starts_with("0x") {
        return Err(EthAppError::InvalidAddress(
            "Address must start with 0x".to_string(),
        ));
    }

    if address.len() != 42 {
        return Err(EthAppError::InvalidAddress(format!(
            "Address must be 42 characters long, got {}",
            address.len()
        )));
    }

    // Check if all characters after 0x are valid hexadecimal
    let hex_part = &address[2..];
    for (i, c) in hex_part.chars().enumerate() {
        if !c.is_ascii_hexdigit() {
            return Err(EthAppError::InvalidAddress(format!(
                "Invalid character '{}' at position {}",
                c,
                i + 2
            )));
        }
    }

    Ok(())
}

/// Convert raw address bytes to EthAddress
pub fn bytes_to_eth_address<E: std::error::Error>(bytes: &[u8]) -> EthAppResult<EthAddress, E> {
    if bytes.len() != length::ETH_ADDRESS_SIZE {
        return Err(EthAppError::InvalidAddress(format!(
            "Invalid address length: {} bytes (expected {})",
            bytes.len(),
            length::ETH_ADDRESS_SIZE
        )));
    }

    let hex_string = format!("0x{}", hex::encode(bytes));
    EthAddress::new(hex_string).map_err(|e| EthAppError::InvalidAddress(e))
}

/// Parse ASCII-encoded address from device response
pub fn parse_device_address<E: std::error::Error>(
    data: &[u8],
    offset: usize,
) -> EthAppResult<(EthAddress, usize), E> {
    if offset >= data.len() {
        return Err(EthAppError::InvalidResponseData(
            "Insufficient data for address length".to_string(),
        ));
    }

    let address_len = data[offset] as usize;
    let address_start = offset + 1;
    let address_end = address_start + address_len;

    if address_end > data.len() {
        return Err(EthAppError::InvalidResponseData(format!(
            "Insufficient data for address: available {}, needed {}",
            data.len() - address_start,
            address_len
        )));
    }

    let address_bytes = &data[address_start..address_end];
    let mut address_str = std::str::from_utf8(address_bytes)
        .map_err(|_| EthAppError::InvalidResponseData("Invalid UTF-8 in address".to_string()))?
        .to_string();

    // If address is 40 characters and doesn't start with 0x, add 0x prefix
    if address_str.len() == 40 && !address_str.starts_with("0x") {
        address_str = format!("0x{}", address_str);
    }

    validate_ethereum_address(&address_str)?;
    let address = EthAddress::new(address_str).map_err(|e| EthAppError::InvalidAddress(e))?;

    Ok((address, address_end))
}

/// Parse public key from device response
pub fn parse_device_public_key<E: std::error::Error>(
    data: &[u8],
    offset: usize,
) -> EthAppResult<(Vec<u8>, usize), E> {
    if offset >= data.len() {
        return Err(EthAppError::InvalidResponseData(
            "Insufficient data for public key length".to_string(),
        ));
    }

    let key_len = data[offset] as usize;
    let key_start = offset + 1;
    let key_end = key_start + key_len;

    if key_end > data.len() {
        return Err(EthAppError::InvalidResponseData(format!(
            "Insufficient data for public key: available {}, needed {}",
            data.len() - key_start,
            key_len
        )));
    }

    // Ethereum public keys should be 65 bytes (uncompressed)
    if key_len != 65 {
        return Err(EthAppError::InvalidResponseData(format!(
            "Invalid public key length: {} (expected 65)",
            key_len
        )));
    }

    let public_key = data[key_start..key_end].to_vec();
    Ok((public_key, key_end))
}

/// Parse optional chain code from device response
pub fn parse_device_chain_code<E: std::error::Error>(
    data: &[u8],
    offset: usize,
) -> EthAppResult<(Option<Vec<u8>>, usize), E> {
    if offset >= data.len() {
        // No chain code present
        return Ok((None, offset));
    }

    if data.len() - offset < length::CHAIN_CODE_SIZE {
        return Err(EthAppError::InvalidResponseData(format!(
            "Insufficient data for chain code: available {}, needed {}",
            data.len() - offset,
            length::CHAIN_CODE_SIZE
        )));
    }

    let chain_code = data[offset..offset + length::CHAIN_CODE_SIZE].to_vec();
    Ok((Some(chain_code), offset + length::CHAIN_CODE_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_bip32_path() {
        let path = BipPath::new(vec![0x8000002C, 0x8000003C, 0x80000000, 0, 0]).unwrap();
        let encoded = encode_bip32_path(&path);

        assert_eq!(encoded[0], 5); // path length
        assert_eq!(&encoded[1..5], &0x8000002Cu32.to_be_bytes());
        assert_eq!(&encoded[5..9], &0x8000003Cu32.to_be_bytes());
    }

    #[test]
    fn test_validate_ethereum_address() {
        assert!(validate_ethereum_address::<std::io::Error>(
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90"
        )
        .is_ok());
        assert!(validate_ethereum_address::<std::io::Error>(
            "0x742d35cc6535c244b8c80a79d5d22efEAdBA5B90"
        )
        .is_ok());
        assert!(validate_ethereum_address::<std::io::Error>(
            "742d35Cc6535C244B8c80A79d5d22efeAdBA5B90"
        )
        .is_err());
        assert!(validate_ethereum_address::<std::io::Error>(
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B9"
        )
        .is_err());
        assert!(validate_ethereum_address::<std::io::Error>(
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B9X"
        )
        .is_err());
    }

    #[test]
    fn test_parse_device_address_with_40_char_address() {
        // Test with 40-character address (without 0x prefix)
        let mut response_data = Vec::new();

        // Address length (40 characters)
        response_data.push(40);
        response_data.extend(b"742d35Cc6535C244B8c80A79d5d22efeAdBA5B90");

        let result = parse_device_address::<std::io::Error>(&response_data, 0);
        assert!(result.is_ok());

        let (address, offset) = result.unwrap();
        assert_eq!(
            address.address,
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90"
        );
        assert_eq!(offset, 41); // 1 (length) + 40 (address) = 41
    }

    #[test]
    fn test_parse_device_address_with_42_char_address() {
        // Test with 42-character address (with 0x prefix) - existing behavior
        let mut response_data = Vec::new();

        // Address length (42 characters)
        response_data.push(42);
        response_data.extend(b"0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90");

        let result = parse_device_address::<std::io::Error>(&response_data, 0);
        assert!(result.is_ok());

        let (address, offset) = result.unwrap();
        assert_eq!(
            address.address,
            "0x742d35Cc6535C244B8c80A79d5d22efeAdBA5B90"
        );
        assert_eq!(offset, 43); // 1 (length) + 42 (address) = 43
    }

    #[test]
    fn test_chunk_data() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let chunks = chunk_data(&data, 3);

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], vec![1, 2, 3]);
        assert_eq!(chunks[1], vec![4, 5, 6]);
        assert_eq!(chunks[2], vec![7, 8, 9]);
        assert_eq!(chunks[3], vec![10]);
    }
}
