//! Coreth genesis type.
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/core#Genesis>
/// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/params#ChainConfig>
/// ref. <https://github.com/ava-labs/avalanchego/tree/dev/genesis>
/// ref. <https://github.com/ava-labs/avalanche-network-runner/blob/main/local/default/genesis.json>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ChainConfig>,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub nonce: primitive_types::U256,
    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub timestamp: primitive_types::U256,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<String>,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub gas_limit: primitive_types::U256,
    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub difficulty: primitive_types::U256,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coinbase: Option<String>,

    /// MUST BE ordered by its key in order for all nodes to have the same JSON outputs.
    /// ref. <https://doc.rust-lang.org/std/collections/index.html#use-a-btreemap-when>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alloc: Option<BTreeMap<String, AllocAccount>>,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub number: primitive_types::U256,
    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub gas_used: primitive_types::U256,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_hash: Option<String>,
    #[serde(rename = "baseFeePerGas", skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<String>,
}

/// On the X-Chain, one AVAX is 10^9  units.
/// On the P-Chain, one AVAX is 10^9  units.
/// On the C-Chain, one AVAX is 10^18 units.
/// "0x204FCE5E3E25026110000000" is "10000000000000000000000000000" (10,000,000,000 AVAX).
/// ref. <https://www.rapidtables.com/convert/number/decimal-to-hex.html>
/// ref. <https://www.rapidtables.com/convert/number/hex-to-decimal.html>
/// ref. <https://snowtrace.io/unitconverter>
pub const DEFAULT_INITIAL_AMOUNT: &str = "0x204FCE5E3E25026110000000";

impl Default for Genesis {
    fn default() -> Self {
        let mut alloc = BTreeMap::new();
        alloc.insert(
            // ref. <https://github.com/ava-labs/coreth/blob/v0.11.5/params/config.go>
            // ref. <https://github.com/ava-labs/avalanchego/blob/master/genesis/genesis_local.json#L74>
            String::from("8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC"),
            AllocAccount::default(),
        );
        Self {
            config: Some(ChainConfig::default()),
            nonce: primitive_types::U256::zero(),
            timestamp: primitive_types::U256::zero(),
            extra_data: Some(String::from("0x00")),

            gas_limit: primitive_types::U256::from_str_radix("0x5f5e100", 16).unwrap(),

            difficulty: primitive_types::U256::zero(),
            mix_hash: Some(String::from(
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )),
            coinbase: Some(String::from("0x0000000000000000000000000000000000000000")),
            alloc: Some(alloc),
            number: primitive_types::U256::zero(),
            gas_used: primitive_types::U256::zero(),
            parent_hash: Some(String::from(
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )),
            base_fee: None,
        }
    }
}

impl Genesis {
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }

    /// Saves the current coreth genesis to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing Genesis to '{}'", file_path);
        let path = Path::new(file_path);
        if let Some(parent_dir) = path.parent() {
            log::info!("creating parent dir '{}'", parent_dir.display());
            fs::create_dir_all(parent_dir)?;
        }

        let d = serde_json::to_vec(self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))?;

        let mut f = File::create(file_path)?;
        f.write_all(&d)?;

        Ok(())
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/params#ChainConfig>
/// ref. <https://github.com/ava-labs/coreth/blob/v0.11.5/params/config.go>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homestead_block: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dao_fork_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dao_fork_support: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip150_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip150_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip155_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip158_block: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub byzantium_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constantinople_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub petersburg_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub istanbul_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muir_glacier_block: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase1_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase2_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase3_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase4_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase5_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase_pre6_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase6_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apricot_phase_post6_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banff_block_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cortina_block_timestamp: Option<u64>,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            // don't use local ID "43112" to avoid config override
            // ref. <https://github.com/ava-labs/coreth/blob/v0.8.6/plugin/evm/vm.go#L326-L328>
            // ref. <https://github.com/ava-labs/avalanche-ops/issues/8>
            chain_id: Some(1000777),
            homestead_block: Some(0),

            dao_fork_block: Some(0),
            dao_fork_support: Some(true),

            eip150_block: Some(0),
            eip150_hash: Some(String::from(
                "0x2086799aeebeae135c246c65021c82b4e15a2c451340993aacfd2751886514f0",
            )),

            eip155_block: Some(0),
            eip158_block: Some(0),

            byzantium_block: Some(0),
            constantinople_block: Some(0),
            petersburg_block: Some(0),
            istanbul_block: Some(0),
            muir_glacier_block: Some(0),

            // ref. <https://github.com/ava-labs/coreth/blob/v0.11.5/params/config.go>
            apricot_phase1_block_timestamp: Some(0),
            apricot_phase2_block_timestamp: Some(0),
            apricot_phase3_block_timestamp: Some(0),
            apricot_phase4_block_timestamp: Some(0),
            apricot_phase5_block_timestamp: Some(0),
            apricot_phase_pre6_block_timestamp: Some(0),
            apricot_phase6_block_timestamp: Some(0),
            apricot_phase_post6_block_timestamp: Some(0),
            banff_block_timestamp: Some(0),
            cortina_block_timestamp: Some(0),
        }
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/core#GenesisAlloc>
/// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/core#GenesisAccount>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllocAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<BTreeMap<String, String>>,

    #[serde(with = "crate::codec::serde::hex_0x_primitive_types_u256")]
    pub balance: primitive_types::U256,

    /// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/core#GenesisMultiCoinBalance>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcbalance: Option<BTreeMap<String, u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
}

impl Default for AllocAccount {
    fn default() -> Self {
        Self {
            code: None,
            storage: None,
            balance: primitive_types::U256::from_str_radix(DEFAULT_INITIAL_AMOUNT, 16).unwrap(),
            mcbalance: None,
            nonce: None,
        }
    }
}

#[test]
fn test_parse() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    // ref. https://github.com/ava-labs/avalanche-network-runner/blob/main/local/default/genesis.json
    let d: Genesis = serde_json::from_str(
        r#"
{
    "unknown1": "field1",
    "unknown2": "field2",

        "config": {
            "chainId": 1000777,
            "homesteadBlock": 0,
            "daoForkBlock": 0,
            "daoForkSupport": true,
            "eip150Block": 0,
            "eip150Hash": "0x2086799aeebeae135c246c65021c82b4e15a2c451340993aacfd2751886514f0",
            "eip155Block": 0,
            "eip158Block": 0,
            "byzantiumBlock": 0,
            "constantinopleBlock": 0,
            "petersburgBlock": 0,
            "istanbulBlock": 0,
            "muirGlacierBlock": 0,
            "apricotPhase1BlockTimestamp": 0,
            "apricotPhase2BlockTimestamp": 0
        },
        "nonce": "0x0",
        "timestamp": "0x0",
        "extraData": "0x00",
        "gasLimit": "0x5f5e100",
        "difficulty": "0x0",
        "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "coinbase": "0x0000000000000000000000000000000000000000",
        "alloc": {
            "8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC": {
                "balance": "0x204FCE5E3E25026110000000"
            }
        },
        "number": "0x0",
        "gasUsed": "0x0",
        "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
"#,
    )
    .unwrap();
    let d = d.encode_json().unwrap();
    log::info!("{}", d);

    let d = Genesis::default();
    let d = d.encode_json().unwrap();
    log::info!("{}", d);
}
