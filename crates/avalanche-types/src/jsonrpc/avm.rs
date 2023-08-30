//! AVM JSON-RPC API.
use std::io::{self, Error, ErrorKind};

use crate::{choices, codec::serde::hex_0x_utxo::Hex0xUtxo, ids, jsonrpc, txs};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain#avmissuetx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IssueTxRequest {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<IssueTxParams>,
}

impl Default for IssueTxRequest {
    fn default() -> Self {
        Self::default()
    }
}

impl IssueTxRequest {
    pub fn default() -> Self {
        Self {
            jsonrpc: String::from(super::DEFAULT_VERSION),
            id: super::DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueTxParams {
    pub tx: String,
    pub encoding: String,
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain#avmissuetx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IssueTxResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<IssueTxResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<super::ResponseError>,
}

impl Default for IssueTxResponse {
    fn default() -> Self {
        Self::default()
    }
}

impl IssueTxResponse {
    pub fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: None,
            error: None,
        }
    }
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain#avmissuetx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IssueTxResult {
    #[serde(rename = "txID")]
    pub tx_id: ids::Id,
}

impl Default for IssueTxResult {
    fn default() -> Self {
        Self::default()
    }
}

impl IssueTxResult {
    pub fn default() -> Self {
        Self {
            tx_id: ids::Id::empty(),
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::avm::test_issue_tx --exact --show-output
#[test]
fn test_issue_tx() {
    use std::str::FromStr;

    let resp: IssueTxResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"txID\": \"G3BuH6ytQ2averrLxJJugjWZHTRubzCrUZEXoheG5JMqL5ccY\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = IssueTxResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(IssueTxResult {
            tx_id: ids::Id::from_str("G3BuH6ytQ2averrLxJJugjWZHTRubzCrUZEXoheG5JMqL5ccY").unwrap(),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgettxstatus>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTxStatusResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetTxStatusResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

impl Default for GetTxStatusResponse {
    fn default() -> Self {
        Self::default()
    }
}

impl GetTxStatusResponse {
    pub fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(GetTxStatusResult::default()),
            error: None,
        }
    }
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgettxstatus>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTxStatusResult {
    #[serde_as(as = "DisplayFromStr")]
    pub status: choices::status::Status,
}

impl Default for GetTxStatusResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetTxStatusResult {
    pub fn default() -> Self {
        Self {
            status: choices::status::Status::Unknown(String::new()),
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::avm::test_get_tx_status --exact --show-output
#[test]
fn test_get_tx_status() {
    // ref. https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgettxstatus
    let resp: GetTxStatusResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"status\": \"Accepted\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetTxStatusResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetTxStatusResult {
            status: choices::status::Status::Accepted,
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/issuing-api-calls>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetUtxosRequest {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<GetUtxosParams>,
}

impl Default for GetUtxosRequest {
    fn default() -> Self {
        Self::default()
    }
}

impl GetUtxosRequest {
    pub fn default() -> Self {
        Self {
            jsonrpc: String::from(super::DEFAULT_VERSION),
            id: super::DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgetutxos>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUtxosParams {
    pub addresses: Vec<String>,
    pub limit: u32,
    pub encoding: String,
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgetutxos>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetUtxosResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetUtxosResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<super::ResponseError>,
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgetutxos>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUtxosResult {
    #[serde_as(as = "DisplayFromStr")]
    pub num_fetched: u32,

    #[serde_as(as = "Option<Vec<Hex0xUtxo>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utxos: Option<Vec<txs::utxo::Utxo>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<super::EndIndex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

impl Default for GetUtxosResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetUtxosResult {
    pub fn default() -> Self {
        Self {
            num_fetched: 0,
            utxos: None,
            end_index: None,
            encoding: None,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::avm::test_get_utxos_empty --exact --show-output
#[test]
fn test_get_utxos_empty() {
    // ref. https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgetutxos
    let resp: GetUtxosResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"numFetched\": \"0\",
        \"utxos\": [],
        \"endIndex\": {
            \"address\": \"P-custom152qlr6zunz7nw2kc4lfej3cn3wk46u3002k4w5\",
            \"utxo\": \"11111111111111111111111111111111LpoYY\"
        },
        \"encoding\":\"hex\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetUtxosResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetUtxosResult {
            num_fetched: 0,
            utxos: Some(Vec::new()),
            end_index: Some(super::EndIndex {
                address: String::from("P-custom152qlr6zunz7nw2kc4lfej3cn3wk46u3002k4w5"),
                utxo: String::from("11111111111111111111111111111111LpoYY"),
            }),
            encoding: Some(String::from("hex")),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::avm::test_get_utxos_non_empty --exact --show-output
#[test]
fn test_get_utxos_non_empty() {
    // ref. https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetbalance
    let resp: GetUtxosResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"numFetched\": \"1\",
        \"utxos\": [
            \"0x000000000000000000000000000000000000000000000000000000000000000000000000000088eec2e099c6a528e689618e8721e04ae85ea574c7a15a7968644d14d54780140000000702c68af0bb1400000000000000000000000000010000000165844a05405f3662c1928142c6c2a783ef871de939b564db\"
        ],
        \"endIndex\": {
            \"address\": \"X-avax1x459sj0ssujguq723cljfty4jlae28evjzt7xz\",
            \"utxo\": \"LUC1cmcxnfNR9LdkACS2ccGKLEK7SYqB4gLLTycQfg1koyfSq\"
        },
        \"encoding\": \"hex\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let raw_utxo =  String::from("0x000000000000000000000000000000000000000000000000000000000000000000000000000088eec2e099c6a528e689618e8721e04ae85ea574c7a15a7968644d14d54780140000000702c68af0bb1400000000000000000000000000010000000165844a05405f3662c1928142c6c2a783ef871de939b564db");
    let utxo = txs::utxo::Utxo::from_hex(&raw_utxo).unwrap();

    let expected = GetUtxosResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetUtxosResult {
            num_fetched: 1,
            utxos: Some(vec![utxo]),
            end_index: Some(super::EndIndex {
                address: String::from("X-avax1x459sj0ssujguq723cljfty4jlae28evjzt7xz"),
                utxo: String::from("LUC1cmcxnfNR9LdkACS2ccGKLEK7SYqB4gLLTycQfg1koyfSq"),
            }),
            encoding: Some(String::from("hex")),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/x-chain#avmgetbalance>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetBalanceResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetBalanceResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/x-chain#avmgetbalance>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetBalanceResult {
    #[serde_as(as = "DisplayFromStr")]
    pub balance: u64,

    #[serde(rename = "utxoIDs")]
    pub utxo_ids: Option<Vec<txs::utxo::Id>>,
}

impl Default for GetBalanceResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetBalanceResult {
    pub fn default() -> Self {
        Self {
            balance: 0,
            utxo_ids: None,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::avm::test_get_balance --exact --show-output
#[test]
fn test_get_balance() {
    use std::str::FromStr;

    // ref. https://docs.avax.network/build/avalanchego-apis/x-chain#avmgetbalance
    let resp: GetBalanceResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"balance\": \"299999999999900\",
        \"utxoIDs\": [
            {
                \"txID\": \"WPQdyLNqHfiEKp4zcCpayRHYDVYuh1hqs9c1RqgZXS4VPgdvo\",
                \"outputIndex\": 1
            }
        ]
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetBalanceResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetBalanceResult {
            balance: 299999999999900,
            utxo_ids: Some(vec![txs::utxo::Id {
                tx_id: ids::Id::from_str("WPQdyLNqHfiEKp4zcCpayRHYDVYuh1hqs9c1RqgZXS4VPgdvo")
                    .unwrap(),
                output_index: 1,
                ..txs::utxo::Id::default()
            }]),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/x-chain/#avmgetassetdescription>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetAssetDescriptionResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetAssetDescriptionResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/x-chain/#avmgetassetdescription>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetAssetDescriptionResult {
    #[serde(rename = "assetID")]
    pub asset_id: ids::Id,

    pub name: String,
    pub symbol: String,

    #[serde_as(as = "DisplayFromStr")]
    pub denomination: usize,
}

impl Default for GetAssetDescriptionResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetAssetDescriptionResult {
    pub fn default() -> Self {
        Self {
            asset_id: ids::Id::default(),
            name: String::new(),
            symbol: String::new(),
            denomination: 0,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::avm::test_get_asset_description --exact --show-output
#[test]
fn test_get_asset_description() {
    use std::str::FromStr;

    // ref. https://docs.avax.network/build/avalanchego-apis/x-chain/#avmgetassetdescription
    let resp: GetAssetDescriptionResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"assetID\": \"2fombhL7aGPwj3KH4bfrmJwW6PVnMobf9Y2fn9GwxiAAJyFDbe\",
        \"name\": \"Avalanche\",
        \"symbol\": \"AVAX\",
        \"denomination\": \"9\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetAssetDescriptionResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetAssetDescriptionResult {
            asset_id: ids::Id::from_str("2fombhL7aGPwj3KH4bfrmJwW6PVnMobf9Y2fn9GwxiAAJyFDbe")
                .unwrap(),
            name: String::from("Avalanche"),
            symbol: String::from("AVAX"),
            denomination: 9,
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/issuing-api-calls>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IssueStopVertexRequest {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<IssueStopVertexParams>,
}

impl Default for IssueStopVertexRequest {
    fn default() -> Self {
        Self::default()
    }
}

impl IssueStopVertexRequest {
    pub fn default() -> Self {
        Self {
            jsonrpc: String::from(super::DEFAULT_VERSION),
            id: super::DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueStopVertexParams {}
