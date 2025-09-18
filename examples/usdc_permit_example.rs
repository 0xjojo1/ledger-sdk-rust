// SPDX-License-Identifier: Apache-2.0

//! USDC Permit EIP-712 signing example
//!
//! This example demonstrates how to use the new JSON-based EIP-712 signing API
//! with a real USDC permit transaction.

use std::error::Error;

use ledger_eth_app::{BipPath, EthereumApp};
use ledger_transport_hid::{hidapi::HidApi, TransportNativeHID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔌 Connecting to Ledger device...");

    // Initialize HID API
    let api = HidApi::new()?;

    // List available Ledger devices
    let ledgers: Vec<_> = TransportNativeHID::list_ledgers(&api).collect();

    if ledgers.is_empty() {
        eprintln!("❌ No Ledger device found");
        eprintln!("Please ensure:");
        eprintln!("  1. Device is connected via USB");
        eprintln!("  2. Device is unlocked");
        eprintln!("  3. Ethereum app is open");
        return Ok(());
    }

    println!("✅ Found {} Ledger device(s)", ledgers.len());

    // Connect to the first available device
    let transport = TransportNativeHID::new(&api)?;
    let eth_app = EthereumApp::new(transport);

    println!("🔗 Connected to device");

    // Test the JSON-based EIP-712 API with USDC permit
    test_usdc_permit(&eth_app).await?;

    Ok(())
}

async fn test_usdc_permit(eth_app: &EthereumApp<TransportNativeHID>) -> Result<(), Box<dyn Error>> {
    println!("\n📋 Testing USDC Permit EIP-712 signing...");

    // Create BIP32 path
    let path = BipPath::from_string("m/44'/60'/0'/0/0")?;

    // USDC Permit JSON (exactly as provided by the user)
    let json_str = r#"{"domain":{"name":"USD Coin","verifyingContract":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","chainId":1,"version":"2"},"primaryType":"Permit","message":{"deadline":1718992051,"nonce":0,"spender":"0x111111125421ca6dc452d289314280a0f8842a65","owner":"0x6cbcd73cd8e8a42844662f0a0e76d7f79afd933d","value":"115792089237316195423570985008687907853269984665640564039457584007913129639935"},"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"version","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"Permit":[{"name":"owner","type":"address"},{"name":"spender","type":"address"},{"name":"value","type":"uint256"},{"name":"nonce","type":"uint256"},{"name":"deadline","type":"uint256"}]}}"#;

    // Sign using the JSON-based API
    println!("\n🔐 Signing with JSON-based API...");
    let signature = eth_app.sign_eip712_from_json(&path, json_str).await?;

    println!("✅ Signature received:");
    println!("   v: 0x{:02x}", signature.v);
    println!("   r: 0x{}", hex::encode(&signature.r));
    println!("   s: 0x{}", hex::encode(&signature.s));

    // Display the full signature in a format that can be used in transactions
    println!("\n📋 Full signature (for transaction use):");
    println!("   r: 0x{}", hex::encode(&signature.r));
    println!("   s: 0x{}", hex::encode(&signature.s));
    println!("   v: {}", signature.v);

    Ok(())
}
