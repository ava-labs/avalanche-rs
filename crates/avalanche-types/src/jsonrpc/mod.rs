//! Avalanche JSON-RPC API requests and responses.
pub mod admin;
pub mod avm;
pub mod common;
pub mod evm;
pub mod health;
pub mod info;
pub mod platformvm;

#[cfg(feature = "jsonrpc_client")]
#[cfg_attr(docsrs, doc(cfg(feature = "jsonrpc_client")))]
pub mod client;

use std::{
    collections::HashMap,
    io::{self, Error, ErrorKind},
};

use serde::{Deserialize, Serialize};

pub const DEFAULT_VERSION: &str = "2.0";
pub const DEFAULT_ID: u32 = 1;

/// ref. <https://www.jsonrpc.org/specification>
/// ref. <https://docs.avax.network/build/avalanchego-apis/issuing-api-calls>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Request {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            jsonrpc: String::from(DEFAULT_VERSION),
            id: DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }
}

impl Request {
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/c-chain#eth_getassetbalance>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RequestWithParamsArray {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>,
}

impl Default for RequestWithParamsArray {
    fn default() -> Self {
        Self {
            jsonrpc: String::from(DEFAULT_VERSION),
            id: DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }
}

impl RequestWithParamsArray {
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/c-chain#eth_getassetbalance>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RequestWithParamsHashMapArray {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<HashMap<String, String>>>,
}

impl Default for RequestWithParamsHashMapArray {
    fn default() -> Self {
        Self {
            jsonrpc: String::from(DEFAULT_VERSION),
            id: DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }
}

impl RequestWithParamsHashMapArray {
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetbalance>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RequestWithParamsHashMapToArray {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, Vec<String>>>,
}

impl Default for RequestWithParamsHashMapToArray {
    fn default() -> Self {
        Self {
            jsonrpc: String::from(DEFAULT_VERSION),
            id: DEFAULT_ID,
            method: String::new(),
            params: None,
        }
    }
}

impl RequestWithParamsHashMapToArray {
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgetutxos>
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetutxos>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndIndex {
    pub address: String,
    pub utxo: String,
}

/// e.g., {"jsonrpc":"2.0","error":{"code":-32000,"message":"problem decoding transaction: invalid input checksum","data":null},"id":1}
/// e.g., {"jsonrpc":"2.0","error":{"code":-32000,"message":"problem decoding transaction: missing 0x prefix to hex encoding","data":null},"id":1}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Default)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}
