//! Admin API requests and responses.
use std::io::{self, Error as ioError, ErrorKind};

use serde::{Deserialize, Serialize};

/// The chain alias method name
const ALIAS_METHOD: &str = "admin.aliasChain";

/// The request to alias a chain via the admin API.
/// Ref: <https://docs.avax.network/apis/avalanchego/apis/admin#adminaliaschain>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ChainAliasRequest {
    /// Jsonrpc version
    pub jsonrpc: String,
    /// Id of request
    pub id: u32,
    /// Method (admin.aliasChain)
    pub method: String,
    /// Alias parameters
    pub params: Option<ChainAliasParams>,
}

impl Default for ChainAliasRequest {
    fn default() -> Self {
        Self {
            jsonrpc: String::from(super::DEFAULT_VERSION),
            id: super::DEFAULT_ID,
            method: ALIAS_METHOD.to_string(),
            params: None,
        }
    }
}

impl ChainAliasRequest {
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| ioError::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }
}

/// Parameters for the alias request.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ChainAliasParams {
    /// The long-form chain ID
    pub chain: String,
    /// The newly issues alias
    pub alias: String,
}

/// Response for the alias request.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ChainAliasResponse {
    /// Jsonrpc version
    pub jsonrpc: String,
    /// Id of request
    pub id: u32,
}

#[cfg(test)]
mod tests {
    use crate::jsonrpc::admin::{ChainAliasParams, ChainAliasRequest, ChainAliasResponse};
    use crate::jsonrpc::{DEFAULT_ID, DEFAULT_VERSION};

    #[test]
    fn test_serialization() {
        let chain = String::from("sV6o671RtkGBcno1FiaDbVcFv2sG5aVXMZYzKdP4VQAWmJQnM");
        let alias = String::from("devnet");

        let req = ChainAliasRequest {
            params: Some(ChainAliasParams { chain, alias }),
            ..Default::default()
        };

        let serialized = req.encode_json().expect("failed serialization");

        let expected: &str = r#"{"jsonrpc":"2.0","id":1,"method":"admin.aliasChain","params":{"chain":"sV6o671RtkGBcno1FiaDbVcFv2sG5aVXMZYzKdP4VQAWmJQnM","alias":"devnet"}}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_deserialization() {
        let expected = ChainAliasResponse {
            jsonrpc: String::from(DEFAULT_VERSION),
            id: DEFAULT_ID,
        };

        let response = r#"{"jsonrpc": "2.0","id": 1,"result": {}}"#.as_bytes();
        let deserialized: ChainAliasResponse =
            serde_json::from_slice(response).expect("failed deserialization");

        assert_eq!(expected, deserialized);
    }
}
