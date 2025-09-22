// SPDX-License-Identifier: Apache-2.0

//! EIP-712 signing utilities

use crate::errors::{EthAppError, EthAppResult};
use crate::types::Signature;

/// Parse signature response data
pub fn parse_signature_response<E: std::error::Error>(data: &[u8]) -> EthAppResult<Signature, E> {
    if data.len() != 65 {
        return Err(EthAppError::InvalidResponseData(format!(
            "Invalid signature response length: {} bytes (expected 65)",
            data.len()
        )));
    }

    let v = data[0];
    let r = data[1..33].to_vec();
    let s = data[33..65].to_vec();

    Signature::new(v, r, s).map_err(|e| EthAppError::InvalidSignature(e))
}
