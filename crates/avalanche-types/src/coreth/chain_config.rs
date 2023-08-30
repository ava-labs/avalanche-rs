//! Coreth chain config.
use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

/// To be persisted in "chain_config_dir".
/// ref. <https://pkg.go.dev/github.com/ava-labs/coreth/plugin/evm#Config>
/// ref. <https://github.com/ava-labs/coreth/blob/v0.11.5/plugin/evm/config.go>
/// ref. <https://serde.rs/container-attrs.html>
///
/// If a Subnet's chain id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
/// the config file for this chain is located at {chain-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt/config.json
/// ref. <https://docs.avax.network/subnets/customize-a-subnet#chain-configs>
///
/// For instance, "C" chain config can be found at:
/// $ vi /data/avalanche-configs/chains/C/config.json
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snowman_api_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreth_admin_api_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreth_admin_api_dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_apis: Option<Vec<String>>,

    /// If not empty, it enables the profiler.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuous_profiler_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuous_profiler_frequency: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuous_profiler_max_files: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_gas_cap: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_tx_fee_cap: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimages_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pruning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_verification_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics_expensive_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_txs_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_rejournal: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_price_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_price_bump: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_account_slots: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_global_slots: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_account_queue: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_pool_global_queue: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_max_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_cpu_refill_rate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_cpu_max_stored: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_max_blocks_per_request: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unfinalized_queries: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unprotected_txs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub keystore_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keystore_external_signer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keystore_insecure_unlock_allowed: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_tx_gossip_only_enabled: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_regossip_frequency: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_regossip_max_size: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_json_format: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_pruning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_pruning_bloom_filter_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_pruning_data_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_outbound_active_requests: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_outbound_active_cross_chain_requests: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_skip_resume: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_server_trie_cache: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_ids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_commit_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_min_blocks: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inspect_database: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_upgrade_check: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_cache_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_lookup_limit: Option<u64>,
}

pub const DEFAULT_CORETH_ADMIN_API_ENABLED: bool = true;

/// MUST BE a valid path in remote host machine.
pub const DEFAULT_PROFILE_DIR: &str = "/var/log/avalanchego-profile/coreth";
pub const DEFAULT_PROFILE_FREQUENCY: i64 = 15 * 60 * 1000 * 1000 * 1000; // 15-min
pub const DEFAULT_PROFILE_MAX_FILES: i64 = 5;

pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_LOG_JSON_FORMAT: bool = true;

/// ref. <https://docs.avax.network/nodes/maintain/run-offline-pruning>
pub const DEFAULT_OFFLINE_PRUNING_DATA_DIR: &str = "/data/c-chain-offline-pruning";

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

impl Config {
    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default() -> Self {
        Self {
            snowman_api_enabled: None,
            coreth_admin_api_enabled: Some(DEFAULT_CORETH_ADMIN_API_ENABLED),
            coreth_admin_api_dir: None,

            // ref. plugin/evm/vm.go "legacyApiNames"
            eth_apis: Some(vec![
                "eth".to_string(),
                "eth-filter".to_string(),
                "net".to_string(),
                "web3".to_string(),
                "internal-eth".to_string(),
                "internal-blockchain".to_string(),
                "internal-transaction".to_string(),
                "internal-tx-pool".to_string(),
                "debug-tracer".to_string(),
                // "internal-debug".to_string(),
                // "internal-account".to_string(),
                // "internal-personal".to_string(),
                // "admin".to_string(),
                // "debug".to_string(),
            ]),

            continuous_profiler_dir: None,
            continuous_profiler_frequency: None,
            continuous_profiler_max_files: None,

            rpc_gas_cap: None,
            rpc_tx_fee_cap: None,

            preimages_enabled: None,
            pruning_enabled: Some(true),
            snapshot_async: None,
            snapshot_verification_enabled: None,

            metrics_expensive_enabled: Some(true),

            local_txs_enabled: Some(false),

            // ref. <https://pkg.go.dev/github.com/ava-labs/coreth/core#DefaultTxPoolConfig>
            tx_pool_journal: Some(String::from("transactions.rlp")),
            tx_pool_rejournal: Some(3600000000000), // 1-hour
            tx_pool_price_limit: Some(1),
            tx_pool_price_bump: Some(10),
            tx_pool_account_slots: Some(16),
            tx_pool_global_slots: Some(4096 + 1024),
            tx_pool_account_queue: Some(64),
            tx_pool_global_queue: Some(1024),

            api_max_duration: Some(0),
            ws_cpu_refill_rate: Some(0),
            ws_cpu_max_stored: Some(0),
            api_max_blocks_per_request: Some(0),
            allow_unfinalized_queries: None,
            allow_unprotected_txs: None,

            keystore_directory: None,
            keystore_external_signer: None,
            keystore_insecure_unlock_allowed: None,

            remote_tx_gossip_only_enabled: None,
            tx_regossip_frequency: None,
            tx_regossip_max_size: None,

            log_level: Some(String::from(DEFAULT_LOG_LEVEL)),
            log_json_format: Some(DEFAULT_LOG_JSON_FORMAT),

            offline_pruning_enabled: Some(false),
            offline_pruning_bloom_filter_size: None,
            offline_pruning_data_directory: Some(String::from(DEFAULT_OFFLINE_PRUNING_DATA_DIR)),

            max_outbound_active_requests: None,
            max_outbound_active_cross_chain_requests: None,

            state_sync_enabled: Some(true), // faster mainnet sync!
            state_sync_skip_resume: None,
            state_sync_server_trie_cache: Some(64),
            state_sync_ids: None,
            state_sync_commit_interval: Some(4096 * 4), // defaultCommitInterval * 4
            state_sync_min_blocks: Some(300_000),

            inspect_database: None,
            skip_upgrade_check: None,
            accepted_cache_size: Some(32),
            tx_lookup_limit: None,
        }
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }

    /// Saves the current chain config to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing Config to '{}'", file_path);
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

    pub fn load(file_path: &str) -> io::Result<Self> {
        log::info!("loading coreth chain config from {}", file_path);

        if !Path::new(file_path).exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("file {} does not exists", file_path),
            ));
        }

        let f = File::open(&file_path).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to open {} ({})", file_path, e),
            )
        })?;
        serde_json::from_reader(f)
            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid JSON: {}", e)))
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- coreth::chain_config::test_config --exact --show-output
#[test]
fn test_config() {
    let _ = env_logger::builder().is_test(true).try_init();

    let tmp_path = random_manager::tmp_path(10, Some(".json")).unwrap();
    let cfg = Config::default();
    log::info!("{}", cfg.encode_json().unwrap());

    cfg.sync(&tmp_path).unwrap();

    fs::remove_file(tmp_path).unwrap();
}
