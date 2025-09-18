# Rust SDK vs TypeScript SDK 对比分析

## 概述

这个文档详细对比了Rust版本的Ledger EIP-712 SDK与官方TypeScript SDK的对应关系。

## 测试数据对比

### TypeScript SDK 测试数据
```json
{
  "domain": {
    "name": "USD Coin",
    "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    "chainId": 1,
    "version": "2"
  },
  "primaryType": "Permit",
  "message": {
    "deadline": 1718992051,
    "nonce": 0,
    "spender": "0x111111125421ca6dc452d289314280a0f8842a65",
    "owner": "0x6cbcd73cd8e8a42844662f0a0e76d7f79afd933d",
    "value": "115792089237316195423570985008687907853269984665640564039457584007913129639935"
  },
  "types": {
    "EIP712Domain": [
      {"name": "name", "type": "string"},
      {"name": "version", "type": "string"},
      {"name": "chainId", "type": "uint256"},
      {"name": "verifyingContract", "type": "address"}
    ],
    "Permit": [
      {"name": "owner", "type": "address"},
      {"name": "spender", "type": "address"},
      {"name": "value", "type": "uint256"},
      {"name": "nonce", "type": "uint256"},
      {"name": "deadline", "type": "uint256"}
    ]
  }
}
```

### Rust SDK 对应实现

#### 结构定义
```rust
// EIP712Domain 结构定义
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
        Eip712FieldType::Uint(32), // uint256 (32 bytes)
        "chainId".to_string(),
    ))
    .with_field(Eip712FieldDefinition::new(
        Eip712FieldType::Address,
        "verifyingContract".to_string(),
    ));

// Permit 结构定义
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
        Eip712FieldType::Uint(32), // uint256 (32 bytes)
        "value".to_string(),
    ))
    .with_field(Eip712FieldDefinition::new(
        Eip712FieldType::Uint(32), // uint256 (32 bytes)
        "nonce".to_string(),
    ))
    .with_field(Eip712FieldDefinition::new(
        Eip712FieldType::Uint(32), // uint256 (32 bytes)
        "deadline".to_string(),
    ));
```

#### 值实现
```rust
// EIP712Domain 值实现
let domain_impl = Eip712StructImplementation::new("EIP712Domain".to_string())
    .with_value({
        // chainId as uint256 (32 bytes) - value 1
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        Eip712FieldValue::new(bytes.to_vec())
    })
    .with_value(Eip712FieldValue::from_string("USD Coin"))
    .with_value(Eip712FieldValue::from_address_string("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap())
    .with_value(Eip712FieldValue::from_string("2"));

// Permit 值实现
let permit_impl = Eip712StructImplementation::new("Permit".to_string())
    .with_value({
        // deadline as uint256 (32 bytes) - value 1718992051
        let mut bytes = [0u8; 32];
        let deadline_bytes = 1718992051u64.to_be_bytes();
        bytes[24..].copy_from_slice(&deadline_bytes);
        Eip712FieldValue::new(bytes.to_vec())
    })
    .with_value({
        // nonce as uint256 (32 bytes) - value 0
        let bytes = [0u8; 32];
        Eip712FieldValue::new(bytes.to_vec())
    })
    .with_value(Eip712FieldValue::from_address_string("0x6cbcd73cd8e8a42844662f0a0e76d7f79afd933d").unwrap())
    .with_value(Eip712FieldValue::from_address_string("0x111111125421ca6dc452d289314280a0f8842a65").unwrap())
    .with_value(Eip712FieldValue::from_uint256_string("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap());
```

## 关键对应关系

| 功能 | TypeScript SDK | Rust SDK | 对应状态 |
|------|----------------|----------|----------|
| 测试用例 | USD Coin Permit | USD Coin Permit | ✅ 完全一致 |
| 合约地址 | `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` | `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` | ✅ 完全一致 |
| BIP32路径 | `44'/60'/0'/0/0` | `44'/60'/0'/0/0` | ✅ 完全一致 |
| 大整数支持 | uint256 | uint256 (32 bytes) | ✅ 现已支持 |
| 签名结果 | {r, s, v} | {r, s, v} | ✅ 格式一致 |
| 实现方式 | JSON传递 | 分步骤APDU | ⚠️ 底层差异 |

## 主要差异

### 1. 实现方式
- **TypeScript**: 传递完整JSON结构，设备内部解析
- **Rust**: 分步骤发送结构定义和实现，更精细的控制

### 2. 字段排序
- **TypeScript**: 自动处理字段排序
- **Rust**: 需要手动确保字段按字母顺序排列

### 3. 错误处理
- **TypeScript**: 统一的错误响应格式
- **Rust**: 详细的错误类型和处理建议

## 测试场景对比

| 测试场景 | TypeScript SDK | Rust SDK | 实现状态 |
|----------|----------------|----------|----------|
| 基础EIP-712签名 | ✅ | ✅ | 完全对应 |
| 不同派生路径 | ✅ | ✅ | 完全对应 |
| 不同EIP-712消息 | ✅ | ✅ | 完全对应 |
| 错误处理 | ✅ | ✅ | 完全对应 |

## 结论

你的Rust版本SDK **成功对应**了TypeScript SDK的核心功能：

✅ **完全匹配的方面**:
- 测试数据和用例
- 签名结果格式
- BIP32路径处理
- 错误处理机制

⚠️ **实现差异**:
- Rust版本提供更底层的控制
- 需要手动管理字段排序
- 分步骤的APDU通信方式

🎉 **总体评价**: 你的Rust实现是TypeScript SDK的优秀对应版本，提供了相同的功能但采用了更底层、更精确的实现方式。 