# Ledger SDK Rust

A modular Rust SDK for communicating with Ledger hardware wallets, featuring comprehensive Ethereum app support including EIP-712 typed data signing.

## Features

- **Modular Architecture**: Clean separation of concerns across multiple crates
- **Ethereum Support**: Full Ethereum app integration with EIP-712 typed data signing
- **Transport Abstraction**: Support for multiple transport layers (HID, etc.)
- **Type Safety**: Strongly typed APIs with comprehensive error handling
- **Async Support**: Built on async/await for modern Rust applications

## Crates

| Crate                                                                           | Description                        | Version                                                                                                                         |
| ------------------------------------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| [`ledger-sdk-apdu`](https://crates.io/crates/ledger-sdk-apdu)                   | APDU types and helpers             | [![crates.io](https://img.shields.io/crates/v/ledger-sdk-apdu.svg)](https://crates.io/crates/ledger-sdk-apdu)                   |
| [`ledger-sdk-transport`](https://crates.io/crates/ledger-sdk-transport)         | Transport abstraction layer        | [![crates.io](https://img.shields.io/crates/v/ledger-sdk-transport.svg)](https://crates.io/crates/ledger-sdk-transport)         |
| [`ledger-sdk-transport-hid`](https://crates.io/crates/ledger-sdk-transport-hid) | HID transport implementation       | [![crates.io](https://img.shields.io/crates/v/ledger-sdk-transport-hid.svg)](https://crates.io/crates/ledger-sdk-transport-hid) |
| [`ledger-sdk-device-base`](https://crates.io/crates/ledger-sdk-device-base)     | Device and app information helpers | [![crates.io](https://img.shields.io/crates/v/ledger-sdk-device-base.svg)](https://crates.io/crates/ledger-sdk-device-base)     |
| [`ledger-sdk-eth-app`](https://crates.io/crates/ledger-sdk-eth-app)             | Ethereum app with EIP-712 support  | [![crates.io](https://img.shields.io/crates/v/ledger-sdk-eth-app.svg)](https://crates.io/crates/ledger-sdk-eth-app)             |

## Installation

Add the desired crates to your `Cargo.toml`:

```toml
[dependencies]
ledger-sdk-eth-app = "0.0.1"
ledger-sdk-transport-hid = "0.0.1"
```

## Quick Start

### Basic Ethereum Operations

```rust
use ledger_sdk_eth_app::EthApp;
use ledger_sdk_transport_hid::TransportNativeHID;
use ledger_sdk_eth_app::types::BipPath;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HID transport
    let transport = TransportNativeHID::new()?;

    // Create Ethereum app instance
    let eth_app = EthApp::new();

    // Get address for derivation path
    let path = BipPath::from_string("44'/60'/0'/0/0")?;
    let address = eth_app.get_address(&transport, &path).await?;

    println!("Address: {}", address);
    Ok(())
}
```

### EIP-712 Typed Data Signing

```rust
use ledger_sdk_eth_app::EthApp;
use ledger_sdk_eth_app::commands::eip712::SignEip712TypedData;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = TransportNativeHID::new()?;
    let eth_app = EthApp::new();

    // EIP-712 typed data
    let typed_data = json!({
        "domain": {
            "name": "MyApp",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0x1234567890123456789012345678901234567890"
        },
        "types": {
            "Person": [
                {"name": "name", "type": "string"},
                {"name": "wallet", "type": "address"}
            ]
        },
        "primaryType": "Person",
        "message": {
            "name": "Alice",
            "wallet": "0x1234567890123456789012345678901234567890"
        }
    });

    let path = BipPath::from_string("44'/60'/0'/0/0")?;
    let signature = eth_app.sign_eip712_from_json(&transport, &path, &typed_data.to_string()).await?;

    println!("Signature: {:?}", signature);
    Ok(())
}
```

## Building

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_test
```

## Examples

Check the `examples/` directory for more comprehensive usage examples:

- `basic_test.rs` - Basic Ethereum operations
- `usdc_permit_example.rs` - USDC permit signing with EIP-712

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Attribution

Portions of this code are derived from the Zondax `ledger-rs` project (Apache-2.0). See [NOTICE](NOTICE) for details.
