// SPDX-License-Identifier: Apache-2.0

//! EIP-712 signing example
//!
//! This example demonstrates how to use the EIP-712 signing functionality
//! with the Ledger Ethereum SDK, including both v0 and full implementations.

use std::error::Error;

use ledger_eth_app::{
    BipPath, Eip712FieldDefinition, Eip712FieldType, Eip712FieldValue, Eip712StructDefinition,
    Eip712StructImplementation, EthereumApp,
};
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

    println!("\n📋 Testing raw APDU sequence...");
    test_raw_apdu_sequence(&eth_app).await?;

    println!("\n📋 Testing EIP-712 full implementation...");
    test_eip712_full(&eth_app).await?;

    println!("\n🎉 EIP-712 test completed!");
    Ok(())
}

/// Test raw APDU sequence
async fn test_raw_apdu_sequence<E>(eth_app: &EthereumApp<E>) -> Result<(), Box<dyn Error>>
where
    E: ledger_transport::Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    let _path = BipPath::ethereum_standard(0, 0);
    use ledger_apdu::APDUCommand;
    let apdu_sequence = vec![
        "e01a00000c454950373132446f6d61696e",
        "e01a00ff0605046e616d65", 
        "e01a00ff09050776657273696f6e",
        "e01a00ff0a422007636861696e4964",
        "e01a00ff130311766572696679696e67436f6e7472616374",
        "e01a0000065065726d6974",
        "e01a00ff0703056f776e6572",
        "e01a00ff0903077370656e646572",
        "e01a00ff0842200576616c7565",
        "e01a00ff084220056e6f6e6365",
        "e01a00ff0b422008646561646c696e65",
        "e01c00000c454950373132446f6d61696e",
        "e01c00ff0a000855534420436f696e",
        "e01c00ff03000132",
        "e01c00ff03000101",
        "e01c00ff160014a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "e01e000f651c417574686f72697a65207370656e64696e67206f6620746f6b656e730346304402203b3bf0032c9576c665b38049d5c49e9a5be98eb1a2720ab19d2689a81d47452c02202d6324f8349b6e2e0fd0b4e19b7ff90447f30ede78de33a2b8427263b6d81710",
        "e01c0000065065726d6974",
        "e01c00ff1600146cbcd73cd8e8a42844662f0a0e76d7f79afd933d",
        "e01e00ff4f075370656e646572463044022038dce9f341066fafea655d30a6d61dd229912e2ad84823847907a3187ec1331602205cd875606cfedd1b0a430d82796dd896f00a60b61af7fca732123dc73b92907c",
        "e01c00ff160014111111125421ca6dc452d289314280a0f8842a65",
        "e00c000115058000002c8000003c800000000000000000000000",
    ];

    println!("📋 Starting raw APDU sequence...");

    for (index, apdu_hex) in apdu_sequence.iter().enumerate() {
        println!("Sending APDU #{}: {}", index + 1, apdu_hex);

        // Convert hex string to byte array
        let apdu_bytes = hex::decode(apdu_hex).map_err(|e| format!("Decode failed: {}", e))?;

        if apdu_bytes.len() < 4 {
            return Err(format!("APDU too short: {}", apdu_hex).into());
        }

        let cla = apdu_bytes[0];
        let ins = apdu_bytes[1];
        let p1 = apdu_bytes[2];
        let p2 = apdu_bytes[3];
        let data = if apdu_bytes.len() > 5 {
            apdu_bytes[5..].to_vec()
        } else {
            vec![]
        };

        let command = APDUCommand {
            cla,
            ins,
            p1,
            p2,
            data,
        };

        // Send command - ensure each APDU succeeds before continuing
        match eth_app.transport().exchange(&command).await {
            Ok(response) => {
                println!(
                    "✅ APDU #{} sent successfully, response: {:?}",
                    index + 1,
                    &response.data()
                );
            }
            Err(e) => {
                eprintln!("❌ APDU #{} failed: {}", index + 1, e);
                return Err(format!("APDU #{} failed: {}", index + 1, e).into());
            }
        }

        // Add delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("✅ APDU sequence completed");

    Ok(())
}

/// Test EIP-712 full implementation using USD Coin Permit example
async fn test_eip712_full<E>(eth_app: &EthereumApp<E>) -> Result<(), Box<dyn Error>>
where
    E: ledger_transport::Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    let path = BipPath::ethereum_standard(0, 0);

    // Step 1: Define structs in alphabetical order (important for hash consistency)
    // Send struct definitions in alphabetical order: EIP712Domain -> Permit

    println!("  📤 Sending EIP712Domain struct definition...");
    let domain_struct = Eip712StructDefinition::new("EIP712Domain".to_string())
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::String,
            "name".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::String,
            "version".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Uint(32),
            "chainId".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Address,
            "verifyingContract".to_string(),
        ));

    println!("domain_struct: {:?}", domain_struct);

    match eth_app.send_struct_definition(&domain_struct).await {
        Ok(_) => println!("    ✅ EIP712Domain struct definition sent successfully"),
        Err(e) => {
            eprintln!("    ❌ EIP712Domain struct definition failed: {}", e);
            let error_str = format!("{}", e);
            if error_str.contains("27906") || error_str.contains("6D02") {
                eprintln!("    💡 Please enable Blind Signing and try again");
            }
            return Ok(());
        }
    }

    // Step 2: Define Permit struct
    println!("  📤 Sending Permit struct definition...");
    let permit_struct = Eip712StructDefinition::new("Permit".to_string())
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Address,
            "owner".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Address,
            "spender".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Uint(32),
            "value".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Uint(32),
            "nonce".to_string(),
        ))
        .with_field(Eip712FieldDefinition::new(
            Eip712FieldType::Uint(32),
            "deadline".to_string(),
        ));

    match eth_app.send_struct_definition(&permit_struct).await {
        Ok(_) => println!("    ✅ Permit struct definition sent successfully"),
        Err(e) => {
            eprintln!("    ❌ Permit struct definition failed: {}", e);
            return Ok(());
        }
    }

      // Step 2.5: Activate EIP-712 filtering
      println!("  🔍 Activating EIP-712 filtering...");
      match eth_app.activate_filtering().await {
          Ok(_) => println!("    ✅ Filtering activated successfully"),
          Err(e) => {
              eprintln!("    ❌ Filtering activation failed: {}", e);
              eprintln!("    ⚠️  Continuing without filtering...");
          }
      }
    

    // Step 3: Send EIP712Domain implementation (value order must match raw APDU sequence)
    // Raw APDU order: name, version, chainId, verifyingContract
    println!("  📥 Sending EIP712Domain implementation...");
    let domain_impl = Eip712StructImplementation::new("EIP712Domain".to_string())
        .with_value(Eip712FieldValue::from_string("USD Coin")) // name 
        .with_value(Eip712FieldValue::from_string("2")) // version
        .with_value(Eip712FieldValue::from_uint32(1)) // chainId 
        .with_value(
            Eip712FieldValue::from_address_string("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
                .unwrap(),
        ); // verifyingContract 


    println!("domain_impl: {:?}", domain_impl);
    match eth_app
        .send_struct_implementation(&domain_impl, false)
        .await
    {
        Ok(_) => println!("    ✅ EIP712Domain implementation sent successfully"),
        Err(e) => {
            eprintln!("    ❌ EIP712Domain implementation failed: {}", e);
            return Ok(());
        }
    }

  

    // Step 4: Send Permit implementation (value order must match raw APDU sequence)  
    // Raw APDU order: owner, spender, value, nonce, deadline
    println!("  📥 Sending Permit implementation...");
    let permit_impl = Eip712StructImplementation::new("Permit".to_string())
        .with_value(
            Eip712FieldValue::from_address_string("0x6cbcd73cd8e8a42844662f0a0e76d7f79afd933d")
                .unwrap(),
        ) // owner - 第1个
        .with_value(
            Eip712FieldValue::from_address_string("0x111111125421ca6dc452d289314280a0f8842a65")
                .unwrap(),
        ) // spender - 第2个
        .with_value(Eip712FieldValue::from_uint32(u32::MAX)) // value - 第3个 (uint32 max instead of u64)
        .with_value(Eip712FieldValue::from_uint32(0)) // nonce - 第4个
        .with_value(Eip712FieldValue::from_uint32(1718992051)); // deadline - 第5个

    match eth_app.send_struct_implementation(&permit_impl, true).await {
        Ok(_) => println!("    ✅ Permit implementation sent successfully"),
        Err(e) => {
            eprintln!("    ❌ Permit implementation failed: {}", e);
            return Ok(());
        }
    }

    // Step 5: Final signing
    println!("  ✍️  Executing final signing...");
    match eth_app.sign_eip712_full(&path).await {
        Ok(signature) => {
            println!("✅ USD Coin Permit EIP-712 signature successful:");
            println!("  V: 0x{:02x}", signature.v);
            println!("  R: {}", hex::encode(&signature.r));
            println!("  S: {}", hex::encode(&signature.s));
        }
        Err(e) => {
            eprintln!("❌ EIP-712 signing failed: {}", e);

            let error_str = format!("{}", e);
            if error_str.contains("27013") || error_str.contains("6975") {
                eprintln!("    💡 Please enable Blind Signing on your Ledger device:");
                eprintln!("       1. Enter Ethereum app");
                eprintln!("       2. Navigate to Settings");
                eprintln!("       3. Enable Blind signing");
            } else if error_str.contains("27012") || error_str.contains("6974") {
                eprintln!("    ℹ️  User cancelled the signing operation");
            } else {
                eprintln!("    🔍 Unknown error: {}", e);
            }
        }
    }

    Ok(())
}

/// Example of using EIP-712 filtering (advanced feature)
#[allow(dead_code)]
async fn example_eip712_filtering<E>(eth_app: &EthereumApp<E>) -> Result<(), Box<dyn Error>>
where
    E: ledger_transport::Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    use ledger_eth_app::{Eip712FilterParams, Eip712FilterType};

    println!("  🔍 Activating EIP-712 filtering...");

    // Activate filtering
    match eth_app.activate_filtering().await {
        Ok(_) => println!("    ✅ Filtering activated successfully"),
        Err(e) => {
            eprintln!("    ❌ Filtering activation failed: {}", e);
            return Ok(());
        }
    }

    // Example: Configure a raw field filter
    let filter_params = Eip712FilterParams {
        filter_type: Eip712FilterType::RawField {
            display_name: "Amount".to_string(),
            signature: vec![0x12, 0x34, 0x56, 0x78], // Example signature
        },
        discarded: false,
    };

    match eth_app.send_filter_config(&filter_params).await {
        Ok(_) => println!("    ✅ Filter configuration sent successfully"),
        Err(e) => {
            eprintln!("    ❌ Filter configuration failed: {}", e);
        }
    }

    Ok(())
}
