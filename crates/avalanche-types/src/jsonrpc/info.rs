use std::{
    collections::HashMap,
    io::{self, Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crate::{
    ids::{self, node},
    jsonrpc,
    key::bls,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnetworkname>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNetworkNameResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetNetworkNameResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnetworkname>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetNetworkNameResult {
    pub network_name: String,
}

impl Default for GetNetworkNameResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetNetworkNameResult {
    pub fn default() -> Self {
        Self {
            network_name: String::new(),
        }
    }
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnetworkid>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNetworkIdResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetNetworkIdResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnetworkid>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNetworkIdResult {
    #[serde(rename = "networkID")]
    #[serde_as(as = "DisplayFromStr")]
    pub network_id: u32,
}

impl Default for GetNetworkIdResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetNetworkIdResult {
    pub fn default() -> Self {
        Self { network_id: 1 }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_get_network_id --exact --show-output
#[test]
fn test_get_network_id() {
    // ref. https://docs.avax.network/build/avalanchego-apis/info/#infogetnetworkid
    let resp: GetNetworkIdResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"networkID\": \"9999999\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetNetworkIdResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetNetworkIdResult {
            network_id: 9999999_u32,
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetblockchainid>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetBlockchainIdResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetBlockchainIdResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetblockchainid>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetBlockchainIdResult {
    #[serde(rename = "blockchainID")]
    pub blockchain_id: ids::Id,
}

impl Default for GetBlockchainIdResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetBlockchainIdResult {
    pub fn default() -> Self {
        Self {
            blockchain_id: ids::Id::default(),
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_get_blockchain_id --exact --show-output
#[test]
fn test_get_blockchain_id() {
    use std::str::FromStr;

    // ref. https://docs.avax.network/build/avalanchego-apis/info/#infogetblockchainid
    let resp: GetBlockchainIdResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"blockchainID\": \"sV6o671RtkGBcno1FiaDbVcFv2sG5aVXMZYzKdP4VQAWmJQnM\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetBlockchainIdResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetBlockchainIdResult {
            blockchain_id: ids::Id::from_str("sV6o671RtkGBcno1FiaDbVcFv2sG5aVXMZYzKdP4VQAWmJQnM")
                .unwrap(),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeid>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNodeIdResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetNodeIdResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeid>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNodeIdResult {
    #[serde(rename = "nodeID")]
    pub node_id: node::Id,
    #[serde(rename = "nodePOP")]
    pub node_pop: Option<bls::ProofOfPossession>,
}

impl Default for GetNodeIdResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetNodeIdResult {
    pub fn default() -> Self {
        Self {
            node_id: node::Id::default(),
            node_pop: None,
        }
    }
}
/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_get_node_id --exact --show-output
#[test]
fn test_get_node_id() {
    use std::str::FromStr;

    // ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeid>
    let resp: GetNodeIdResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"nodeID\": \"NodeID-5mb46qkSBj81k9g9e4VFjGGSbaaSLFRzD\"
    },
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GetNodeIdResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetNodeIdResult {
            node_id: node::Id::from_str("NodeID-5mb46qkSBj81k9g9e4VFjGGSbaaSLFRzD").unwrap(),
            ..Default::default()
        }),
        error: None,
    };
    assert_eq!(resp, expected);

    let resp: GetNodeIdResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"nodeID\": \"NodeID-5mb46qkSBj81k9g9e4VFjGGSbaaSLFRzD\",
        \"nodePOP\": {
            \"publicKey\": \"0x8f95423f7142d00a48e1014a3de8d28907d420dc33b3052a6dee03a3f2941a393c2351e354704ca66a3fc29870282e15\",
            \"proofOfPossession\": \"0x86a3ab4c45cfe31cae34c1d06f212434ac71b1be6cfe046c80c162e057614a94a5bc9f1ded1a7029deb0ba4ca7c9b71411e293438691be79c2dbf19d1ca7c3eadb9c756246fc5de5b7b89511c7d7302ae051d9e03d7991138299b5ed6a570a98\"
        }
    },
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GetNodeIdResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetNodeIdResult {
            node_id: node::Id::from_str("NodeID-5mb46qkSBj81k9g9e4VFjGGSbaaSLFRzD").unwrap(),
            node_pop: Some(bls::ProofOfPossession {
                public_key: hex::decode("0x8f95423f7142d00a48e1014a3de8d28907d420dc33b3052a6dee03a3f2941a393c2351e354704ca66a3fc29870282e15".trim_start_matches("0x")).unwrap(),
                proof_of_possession: hex::decode("0x86a3ab4c45cfe31cae34c1d06f212434ac71b1be6cfe046c80c162e057614a94a5bc9f1ded1a7029deb0ba4ca7c9b71411e293438691be79c2dbf19d1ca7c3eadb9c756246fc5de5b7b89511c7d7302ae051d9e03d7991138299b5ed6a570a98".trim_start_matches("0x")).unwrap(),
                ..Default::default()
            }),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeip>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNodeIpResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetNodeIpResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeip>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNodeIpResult {
    #[serde_as(as = "crate::codec::serde::ip_port::IpPort")]
    pub ip: SocketAddr,
}

impl Default for GetNodeIpResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetNodeIpResult {
    pub fn default() -> Self {
        Self {
            ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9651),
        }
    }
}
/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_get_node_id --exact --show-output
#[test]
fn test_get_node_ip() {
    // ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeid>
    let resp: GetNodeIpResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"ip\": \"192.168.1.1:9651\"
    },
    \"id\": 1
}

",
    )
    .unwrap();
    let expected = GetNodeIpResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetNodeIpResult {
            ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9651),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeversion>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetNodeVersionResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetNodeVersionResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeversion>
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeVersionResult {
    pub version: String,
    pub database_version: String,
    pub git_commit: String,
    pub vm_versions: VmVersions,
    pub rpc_protocol_version: String,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeversion>
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VmVersions {
    pub avm: String,
    pub evm: String,
    pub platform: String,
    #[serde(flatten)]
    pub subnets: HashMap<String, String>,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_get_node_version --exact --show-output
#[test]
fn test_get_node_version() {
    let resp: GetNodeVersionResponse = serde_json::from_str(
        r#"
{
    "jsonrpc": "2.0",
    "result": {
        "version": "avalanche/1.10.1",
        "databaseVersion": "v1.4.5",
        "rpcProtocolVersion": "26",
        "gitCommit": "ef6a2a2f7facd8fbefd5fb2ac9c4908c2bcae3e2",
        "vmVersions": {
          "avm": "v1.10.1",
          "evm": "v0.12.1",
          "platform": "v1.10.1",
          "subnet-evm": "v0.5.1"
        }
    },
    "id": 1
}
"#,
    )
    .unwrap();
    let expected = GetNodeVersionResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetNodeVersionResult {
            version: String::from("avalanche/1.10.1"),
            database_version: String::from("v1.4.5"),
            git_commit: String::from("ef6a2a2f7facd8fbefd5fb2ac9c4908c2bcae3e2"),
            vm_versions: VmVersions {
                avm: String::from("v1.10.1"),
                evm: String::from("v0.12.1"),
                platform: String::from("v1.10.1"),
                subnets: HashMap::from([(String::from("subnet-evm"), String::from("v0.5.1"))]),
            },
            rpc_protocol_version: String::from("26"),
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetvms>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetVmsResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetVmsResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetvms>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetVmsResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vms: Option<HashMap<String, Vec<String>>>,
}

impl Default for GetVmsResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetVmsResult {
    pub fn default() -> Self {
        Self { vms: None }
    }
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infoisbootstrapped>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IsBootstrappedResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<IsBootstrappedResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infoisbootstrapped>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsBootstrappedResult {
    pub is_bootstrapped: bool,
}

impl Default for IsBootstrappedResult {
    fn default() -> Self {
        Self::default()
    }
}

impl IsBootstrappedResult {
    pub fn default() -> Self {
        Self {
            is_bootstrapped: false,
        }
    }
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogettxfee>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTxFeeResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GetTxFeeResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogettxfee>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetTxFeeResult {
    #[serde_as(as = "DisplayFromStr")]
    pub tx_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub create_asset_tx_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub create_subnet_tx_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub transform_subnet_tx_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub create_blockchain_tx_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub add_primary_network_validator_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub add_primary_network_delegator_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub add_subnet_validator_fee: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub add_subnet_delegator_fee: u64,
}

impl Default for GetTxFeeResult {
    fn default() -> Self {
        Self::default()
    }
}

impl GetTxFeeResult {
    pub fn default() -> Self {
        Self {
            tx_fee: 0,
            create_asset_tx_fee: 0,
            create_subnet_tx_fee: 0,
            transform_subnet_tx_fee: 0,
            create_blockchain_tx_fee: 0,
            add_primary_network_validator_fee: 0,
            add_primary_network_delegator_fee: 0,
            add_subnet_validator_fee: 0,
            add_subnet_delegator_fee: 0,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_get_tx_fee --exact --show-output
#[test]
fn test_get_tx_fee() {
    // ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogettxfee>
    // default local network fees
    let resp: GetTxFeeResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"txFee\": \"1000000\",
        \"createAssetTxFee\": \"1000000\",
        \"createSubnetTxFee\": \"100000000\",
        \"transformSubnetTxFee\": \"100000000\",
        \"createBlockchainTxFee\": \"100000000\",
        \"addPrimaryNetworkValidatorFee\": \"0\",
        \"addPrimaryNetworkDelegatorFee\": \"1000000\",
        \"addSubnetValidatorFee\": \"1000000\",
        \"addSubnetDelegatorFee\": \"1000000\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = GetTxFeeResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(GetTxFeeResult {
            tx_fee: 1000000,
            create_asset_tx_fee: 1000000,
            create_subnet_tx_fee: 100000000,
            transform_subnet_tx_fee: 100000000,
            create_blockchain_tx_fee: 100000000,
            add_primary_network_validator_fee: 0,
            add_primary_network_delegator_fee: 1000000,
            add_subnet_validator_fee: 1000000,
            add_subnet_delegator_fee: 1000000,
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/info#infouptime>
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct UptimeResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<UptimeResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/info#infouptime>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UptimeResult {
    #[serde_as(as = "DisplayFromStr")]
    pub rewarding_stake_percentage: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub weighted_average_percentage: f64,
}

impl Default for UptimeResult {
    fn default() -> Self {
        Self::default()
    }
}

impl UptimeResult {
    pub fn default() -> Self {
        Self {
            rewarding_stake_percentage: 0_f64,
            weighted_average_percentage: 0_f64,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_uptime --exact --show-output
#[test]
fn test_uptime() {
    // ref. https://docs.avax.network/apis/avalanchego/apis/info#infouptime
    let resp: UptimeResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"rewardingStakePercentage\": \"100.0000\",
        \"weightedAveragePercentage\": \"99.0000\"
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let expected = UptimeResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(UptimeResult {
            rewarding_stake_percentage: 100.0000_f64,
            weighted_average_percentage: 99.0000_f64,
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/info#infopeers>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct PeersRequest {
    pub jsonrpc: String,
    pub id: u32,

    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<PeersParams>,
}

impl Default for PeersRequest {
    fn default() -> Self {
        Self::default()
    }
}

impl PeersRequest {
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
pub struct PeersParams {
    #[serde(rename = "nodeIDs")]
    pub node_ids: Option<Vec<ids::node::Id>>,
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/info#infopeers>
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PeersResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<PeersResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<jsonrpc::ResponseError>,
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/info#infopeers>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PeersResult {
    #[serde(rename = "numPeers")]
    #[serde_as(as = "DisplayFromStr")]
    pub num_peers: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<Peer>>,
}

impl Default for PeersResult {
    fn default() -> Self {
        Self::default()
    }
}

impl PeersResult {
    pub fn default() -> Self {
        Self {
            num_peers: 0,
            peers: None,
        }
    }
}

/// TODO: add "benched"
/// ref. <https://docs.avax.network/apis/avalanchego/apis/info#infopeers>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    #[serde_as(as = "crate::codec::serde::ip_port::IpPort")]
    pub ip: SocketAddr,
    #[serde(rename = "publicIP")]
    #[serde_as(as = "crate::codec::serde::ip_port::IpPort")]
    pub public_ip: SocketAddr,
    #[serde(rename = "nodeID")]
    pub node_id: node::Id,
    pub version: String,
    #[serde_as(as = "crate::codec::serde::rfc_3339::DateTimeUtc")]
    pub last_sent: DateTime<Utc>,
    #[serde_as(as = "crate::codec::serde::rfc_3339::DateTimeUtc")]
    pub last_received: DateTime<Utc>,
    #[serde_as(as = "DisplayFromStr")]
    pub observed_uptime: u32,
    #[serde_as(as = "HashMap<_, DisplayFromStr>")]
    pub observed_subnet_uptimes: HashMap<ids::Id, u32>,
    pub tracked_subnets: Vec<ids::Id>,
}

impl Default for Peer {
    fn default() -> Self {
        Self::default()
    }
}

impl Peer {
    pub fn default() -> Self {
        Self {
            ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            public_ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            node_id: node::Id::empty(),
            version: String::new(),
            last_sent: DateTime::<Utc>::MIN_UTC,
            last_received: DateTime::<Utc>::MIN_UTC,
            observed_uptime: 0,
            observed_subnet_uptimes: HashMap::new(),
            tracked_subnets: Vec::new(),
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- jsonrpc::info::test_peers --exact --show-output
#[test]
fn test_peers() {
    use std::str::FromStr;

    use chrono::TimeZone;

    // ref. <https://docs.avax.network/apis/avalanchego/apis/info#infopeers>
    let resp: PeersResponse = serde_json::from_str(
        "

{
    \"jsonrpc\": \"2.0\",
    \"result\": {
        \"numPeers\": \"3\",
        \"peers\": [
            {
                \"ip\": \"206.189.137.87:9651\",
                \"publicIP\": \"206.189.137.87:9651\",
                \"nodeID\": \"NodeID-8PYXX47kqLDe2wD4oPbvRRchcnSzMA4J4\",
                \"version\": \"avalanche/1.9.4\",
                \"lastSent\": \"2020-06-01T15:23:02Z\",
                \"lastReceived\": \"2020-06-01T15:22:57Z\",
                \"benched\": [],
                \"observedUptime\": \"99\",
                \"observedSubnetUptimes\": {},
                \"trackedSubnets\": [],
                \"benched\": []
            },
            {
                \"ip\": \"158.255.67.151:9651\",
                \"publicIP\": \"158.255.67.151:9651\",
                \"nodeID\": \"NodeID-C14fr1n8EYNKyDfYixJ3rxSAVqTY3a8BP\",
                \"version\": \"avalanche/1.9.4\",
                \"lastSent\": \"2020-06-01T15:23:02Z\",
                \"lastReceived\": \"2020-06-01T15:22:34Z\",
                \"benched\": [],
                \"observedUptime\": \"75\",
                \"observedSubnetUptimes\": {
                    \"29uVeLPJB1eQJkzRemU8g8wZDw5uJRqpab5U2mX9euieVwiEbL\": \"100\"
                },
                \"trackedSubnets\": [
                    \"29uVeLPJB1eQJkzRemU8g8wZDw5uJRqpab5U2mX9euieVwiEbL\"
                ],
                \"benched\": []
            }
        ]
    },
    \"id\": 1
}

",
    )
    .unwrap();

    let uptimes: HashMap<ids::Id, u32> = [(
        ids::Id::from_str("29uVeLPJB1eQJkzRemU8g8wZDw5uJRqpab5U2mX9euieVwiEbL").unwrap(),
        100,
    )]
    .iter()
    .cloned()
    .collect();
    let expected = PeersResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(PeersResult {
            num_peers: 3,
            peers: Some(vec![
                Peer {
                    ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(206, 189, 137, 87)), 9651),
                    public_ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(206, 189, 137, 87)), 9651),
                    node_id: node::Id::from_str("NodeID-8PYXX47kqLDe2wD4oPbvRRchcnSzMA4J4")
                        .unwrap(),
                    version: String::from("avalanche/1.9.4"),
                    last_sent: Utc.from_utc_datetime(
                        &DateTime::parse_from_rfc3339("2020-06-01T15:23:02Z")
                            .unwrap()
                            .naive_utc(),
                    ),
                    last_received: Utc.from_utc_datetime(
                        &DateTime::parse_from_rfc3339("2020-06-01T15:22:57Z")
                            .unwrap()
                            .naive_utc(),
                    ),
                    observed_uptime: 99,
                    ..Peer::default()
                },
                Peer {
                    ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(158, 255, 67, 151)), 9651),
                    public_ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(158, 255, 67, 151)), 9651),
                    node_id: node::Id::from_str("NodeID-C14fr1n8EYNKyDfYixJ3rxSAVqTY3a8BP")
                        .unwrap(),
                    version: String::from("avalanche/1.9.4"),
                    last_sent: Utc.from_utc_datetime(
                        &DateTime::parse_from_rfc3339("2020-06-01T15:23:02Z")
                            .unwrap()
                            .naive_utc(),
                    ),
                    last_received: Utc.from_utc_datetime(
                        &DateTime::parse_from_rfc3339("2020-06-01T15:22:34Z")
                            .unwrap()
                            .naive_utc(),
                    ),
                    observed_uptime: 75,
                    observed_subnet_uptimes: uptimes,
                    tracked_subnets: vec![ids::Id::from_str(
                        "29uVeLPJB1eQJkzRemU8g8wZDw5uJRqpab5U2mX9euieVwiEbL",
                    )
                    .unwrap()],
                    ..Peer::default()
                },
            ]),
            ..PeersResult::default()
        }),
        error: None,
    };
    assert_eq!(resp, expected);
}
