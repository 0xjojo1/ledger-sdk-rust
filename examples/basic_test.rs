// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating how to interact with a real Ledger device
//!
//! This example shows how to:
//! 1. Connect to a Ledger device via HID
//! 2. Get device configuration
//! 3. Get Ethereum address
//! 4. Sign a message (optional)

use std::error::Error;

use ledger_eth_app::{BipPath, EthereumApp, GetAddressParams, SignMessageParams};
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

    // Test 1: Get application configuration
    println!("\n📋 Getting application configuration...");
    match eth_app.get_configuration().await {
        Ok(config) => {
            println!("✅ Application configuration:");
            println!(
                "  Version: {}.{}.{}",
                config.version.major, config.version.minor, config.version.patch
            );
            println!(
                "  Arbitrary data signature: {}",
                config.flags.arbitrary_data_signature
            );
            println!(
                "  ERC20 external info required: {}",
                config.flags.erc20_external_info
            );
            println!(
                "  Transaction check enabled: {}",
                config.flags.transaction_check_enabled
            );
            println!(
                "  Transaction check opt-in: {}",
                config.flags.transaction_check_opt_in
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to get configuration: {}", e);
            return Ok(());
        }
    }

    // Test 2: Get Ethereum address (account 0, address 0)
    println!("\n🏠 Getting Ethereum address...");
    let path = BipPath::ethereum_standard(0, 0);
    println!("BIP32 path: {}", path);

    let address_params = GetAddressParams::new(path.clone()).with_chain_id(1); // Ethereum mainnet

    match eth_app.get_address(address_params).await {
        Ok(key_info) => {
            println!("✅ Address information:");
            println!("  Address: {}", key_info.address);
            println!("  Public key length: {} bytes", key_info.public_key.len());
            if let Some(chain_code) = &key_info.chain_code {
                println!("  Chain code length: {} bytes", chain_code.len());
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get address: {}", e);
            return Ok(());
        }
    }

    // Test 3: Get address with display (requires user confirmation on device)
    println!("\n👀 Getting address with display (requires user confirmation)...");
    let display_params = GetAddressParams::new(path.clone())
        .with_display()
        .with_chain_code()
        .with_chain_id(1);

    match eth_app.get_address(display_params).await {
        Ok(key_info) => {
            println!("✅ Address display successful:");
            println!("  Address: {}", key_info.address);
            println!("  Includes chain code: {}", key_info.chain_code.is_some());
        }
        Err(e) => {
            eprintln!("❌ Address display failed: {}", e);
            eprintln!("User may have rejected the confirmation");
        }
    }

    // Test 4: Sign a simple message (optional - requires user confirmation)
    println!("\n✍️  Signing test message (requires user confirmation)...");
    let message = b"Hello from Rust Ledger SDK!".to_vec();
    let sign_params = SignMessageParams::new(path, message);

    match eth_app.sign_personal_message(sign_params).await {
        Ok(signature) => {
            println!("✅ Signature successful:");
            println!("  V: 0x{:02x}", signature.v);
            println!("  R: {}", hex::encode(&signature.r));
            println!("  S: {}", hex::encode(&signature.s));
        }
        Err(e) => {
            eprintln!("❌ Signature failed: {}", e);
            eprintln!("User may have rejected the signature");
        }
    }

    println!("\n🎉 Test completed!");
    Ok(())
}
