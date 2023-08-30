//! EVM JSON-RPC requests and responses.
use crate::codec::serde::{hex_0x_bytes::Hex0xBytes, hex_0x_primitive_types_h256::Hex0xH256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Response for "eth_blockNumber".
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_blocknumber>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BlockNumberResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub result: primitive_types::U256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_block_number --exact --show-output
#[test]
fn test_block_number() {
    let resp: BlockNumberResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0x4b7\",
    \"id\": 83
}

",
    )
    .unwrap();
    let expected = BlockNumberResponse {
        jsonrpc: "2.0".to_string(),
        id: 83,
        result: primitive_types::U256::from_str_radix("0x4b7", 16).unwrap(),
    };
    assert_eq!(resp, expected);
}

/// Response for "eth_chainId".
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_blocknumber>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ChainIdResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub result: primitive_types::U256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_chain_id --exact --show-output
#[test]
fn test_chain_id() {
    let resp: ChainIdResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0x4b7\",
    \"id\": 83
}

",
    )
    .unwrap();
    let expected = ChainIdResponse {
        jsonrpc: "2.0".to_string(),
        id: 83,
        result: primitive_types::U256::from_str_radix("0x4b7", 16).unwrap(),
    };
    assert_eq!(resp, expected);
}

/// Response for "eth_gasPrice".
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gasprice>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GasPriceResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub result: primitive_types::U256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_gas_price --exact --show-output
#[test]
fn test_gas_price() {
    let resp: GasPriceResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0x1dfd14000\",
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GasPriceResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: primitive_types::U256::from_str_radix("0x1dfd14000", 16).unwrap(),
    };
    assert_eq!(resp, expected);
}

/// Response for "eth_getBalance".
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_getbalance>
/// ref. <https://docs.avax.network/build/avalanchego-apis/c-chain#eth_getassetbalance>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetBalanceResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub result: primitive_types::U256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_get_balance --exact --show-output
#[test]
fn test_get_balance() {
    // ref. https://docs.avax.network/build/avalanchego-apis/c-chain#eth_getassetbalance
    let resp: GetBalanceResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0x1388\",
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GetBalanceResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: primitive_types::U256::from_str_radix("0x1388", 16).unwrap(),
    };
    assert_eq!(resp, expected);

    let resp: GetBalanceResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0x0234c8a3397aab58\",
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GetBalanceResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: primitive_types::U256::from_str_radix("0x0234c8a3397aab58", 16).unwrap(),
    };
    assert_eq!(resp, expected);
}

/// Response for "eth_getTransactionCount".
/// Returns the number of transactions send from this address.
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactioncount>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTransactionCountResponse {
    pub jsonrpc: String,
    pub id: u32,

    /// The number of transactions send from this address.
    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub result: primitive_types::U256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_get_transaction_count --exact --show-output
#[test]
fn test_get_transaction_count() {
    let resp: GetTransactionCountResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0x1\",
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GetTransactionCountResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: primitive_types::U256::from_str_radix("0x1", 16).unwrap(),
    };
    assert_eq!(resp, expected);
}

/// Response for "eth_getTransactionReceipt".
/// Returns the receipt of a transaction by transaction hash.
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTransactionReceiptResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetTransactionReceiptResult>,
}

/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionReceiptResult {
    pub from: String,
    pub to: String,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub block_number: primitive_types::U256,
    #[serde_as(as = "Hex0xBytes")]
    pub block_hash: Vec<u8>,

    /// Null, if none was created.
    #[serde_as(as = "Option<Hex0xBytes>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_address: Option<Vec<u8>>,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub cumulative_gas_used: primitive_types::U256,
    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub gas_used: primitive_types::U256,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub transaction_index: primitive_types::U256,
    #[serde_as(as = "Hex0xBytes")]
    pub transaction_hash: Vec<u8>,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub status: primitive_types::U256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_get_transaction_receipt --exact --show-output
#[test]
fn test_get_transaction_receipt() {
    let resp: GetTransactionReceiptResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"from\": \"0x7eb4c9d6b763324eea4852f5d40985bbf0f29832\",
        \"to\": \"0x3c42649799074b438889b80312ea9f62bc798aa8\",
        \"blockHash\": \"0xc6ef2fc5426d6ad6fd9e2a26abeab0aa2411b7ab17f30a99d3cb96aed1d1055b\",
        \"blockNumber\": \"0xb\",
        \"cumulativeGasUsed\": \"0x33bc\",
        \"gasUsed\": \"0x4dc\",
        \"transactionIndex\": \"0x1\",
        \"transactionHash\": \"0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238\",
        \"status\": \"0x1\"
    },
    \"id\": 1
}

",
    )
    .unwrap();
    // println!("{:?}", resp);

    let expected = GetTransactionReceiptResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetTransactionReceiptResult {
            from: String::from("0x7eb4c9d6b763324eea4852f5d40985bbf0f29832"),
            to: String::from("0x3c42649799074b438889b80312ea9f62bc798aa8"),

            block_number: primitive_types::U256::from_str_radix("0xb", 16).unwrap(),
            block_hash: vec![
                198, 239, 47, 197, 66, 109, 106, 214, 253, 158, 42, 38, 171, 234, 176, 170, 36, 17,
                183, 171, 23, 243, 10, 153, 211, 203, 150, 174, 209, 209, 5, 91,
            ],

            contract_address: None,

            cumulative_gas_used: primitive_types::U256::from_str_radix("0x33bc", 16).unwrap(),
            gas_used: primitive_types::U256::from_str_radix("0x4dc", 16).unwrap(),

            transaction_index: primitive_types::U256::from_str_radix("0x1", 16).unwrap(),
            transaction_hash: vec![
                185, 3, 35, 159, 133, 67, 208, 75, 93, 193, 186, 101, 121, 19, 43, 20, 48, 135,
                198, 141, 177, 178, 22, 135, 134, 64, 143, 203, 206, 86, 130, 56,
            ],

            status: primitive_types::U256::from_str_radix("0x1", 16).unwrap(),
        }),
    };
    assert_eq!(resp, expected);
}

/// Response for "eth_sendRawTransaction".
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_signtransaction>
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendtransaction>
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SendRawTransactionResponse {
    pub jsonrpc: String,
    pub id: u32,

    /// Transaction hash.
    #[serde_as(as = "Option<Hex0xH256>")]
    pub result: Option<primitive_types::H256>,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::evm::test_send_raw_transaction --exact --show-output
#[test]
fn test_send_raw_transaction() {
    use std::str::FromStr;

    let resp: SendRawTransactionResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": \"0xe16906ec1c7049438bd642023ab15f8633e032940994e6940fff4ec0a2819eb6\",
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = SendRawTransactionResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(
            primitive_types::H256::from_str(
                "e16906ec1c7049438bd642023ab15f8633e032940994e6940fff4ec0a2819eb6",
            )
            .unwrap(),
        ),
    };
    assert_eq!(resp, expected);
}
