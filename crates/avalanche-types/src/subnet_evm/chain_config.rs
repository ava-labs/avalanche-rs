use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

/// To be persisted in "chain_config_dir".
/// ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/plugin/evm#Config>
/// ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/plugin/evm#Config.SetDefaults>
/// ref. <https://github.com/ava-labs/subnet-evm/blob/v0.4.8/plugin/evm/config.go>
/// ref. <https://serde.rs/container-attrs.html>
///
/// If a Subnet's chain id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
/// the config file for this chain is located at {chain-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt/config.json
/// ref. <https://docs.avax.network/subnets/customize-a-subnet#chain-configs>
///
/// For instance, "2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt" chain config can be found at:
/// $ vi /data/avalanche-configs/chains/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt/config.json
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snowman_api_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_api_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_api_dir: Option<String>,

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

    /// Size of the clean cache of the EVM merkle trie,
    /// so that everything can happen in memory instead of hitting the disk.
    /// Generally increasing these to be large enough to fit the full state size in them
    /// will make a huge difference in performance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trie_clean_cache: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trie_clean_journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trie_clean_rejournal: Option<i64>,
    /// Size of the dirty cache of the EVM merkle trie,
    /// so that everything can happen in memory instead of hitting the disk.
    /// Generally increasing these to be large enough to fit the full state size in them
    /// will make a huge difference in performance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trie_dirty_cache: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trie_dirty_commit_target: Option<i64>,
    /// Provides a flat lookup of the EVM storage.
    /// Generally increasing these to be large enough to fit the full state size in them
    /// will make a huge difference in performance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_cache: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimages_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_verification_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pruning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_queue_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_missing_tries: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub populate_missing_tries: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub populate_missing_tries_parallelism: Option<u64>,

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
    pub remote_gossip_only_enabled: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regossip_frequency: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regossip_max_txs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regossip_txs_per_address: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_regossip_frequency: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_regossip_max_txs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_regossip_txs_per_address: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_regossip_addresses: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_json_format: Option<bool>,

    #[serde(rename = "feeRecipient", skip_serializing_if = "Option::is_none")]
    pub fee_recipient: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_pruning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_pruning_bloom_filter_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_pruning_data_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_outbound_active_requests: Option<i64>,

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
    pub skip_upgrade_check: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_cache_size: Option<i64>,
}

pub const DEFAULT_ADMIN_API_ENABLED: bool = true;

/// MUST BE a valid path in remote host machine.
pub const DEFAULT_PROFILE_DIR: &str = "/var/log/avalanchego-profile/coreth";
pub const DEFAULT_PROFILE_FREQUENCY: i64 = 15 * 60 * 1000 * 1000 * 1000; // 15-min
pub const DEFAULT_PROFILE_MAX_FILES: i64 = 5;

pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_LOG_JSON_FORMAT: bool = true;

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
            admin_api_enabled: Some(DEFAULT_ADMIN_API_ENABLED),
            admin_api_dir: None,

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
            continuous_profiler_frequency: Some(900000000000), // 15-minute
            continuous_profiler_max_files: Some(5),

            rpc_gas_cap: Some(50_000_000), // default to 50M gas limit
            rpc_tx_fee_cap: Some(100f64),  // 100 AVAX

            trie_clean_cache: Some(4096),
            trie_clean_journal: None,
            trie_clean_rejournal: None,
            trie_dirty_cache: Some(4096),
            trie_dirty_commit_target: Some(20),
            snapshot_cache: Some(2048),

            preimages_enabled: None,
            snapshot_async: Some(true),
            snapshot_verification_enabled: None,

            pruning_enabled: Some(true),
            accepted_queue_limit: Some(64),
            commit_interval: Some(4096),
            allow_missing_tries: None,
            // cannot enable populate missing tries while offline pruning
            populate_missing_tries: None,
            populate_missing_tries_parallelism: Some(1024),

            metrics_expensive_enabled: Some(true),

            local_txs_enabled: Some(false),

            // ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/core#DefaultTxPoolConfig>
            tx_pool_journal: Some(String::from("transactions.rlp")),
            tx_pool_rejournal: Some(3600000000000), // 1-hour
            tx_pool_price_limit: Some(1),
            tx_pool_price_bump: Some(10),
            tx_pool_account_slots: Some(64),
            tx_pool_global_slots: Some(100_000),
            tx_pool_account_queue: Some(128),
            tx_pool_global_queue: Some(100_000),

            api_max_duration: Some(0),
            ws_cpu_refill_rate: Some(0),
            ws_cpu_max_stored: Some(0),
            api_max_blocks_per_request: Some(0),
            allow_unfinalized_queries: None,
            allow_unprotected_txs: None,

            keystore_directory: None,
            keystore_external_signer: None,
            keystore_insecure_unlock_allowed: None,

            remote_gossip_only_enabled: None,
            regossip_frequency: Some(60000000000), // 1-minute
            regossip_max_txs: Some(16),
            regossip_txs_per_address: Some(1),
            priority_regossip_frequency: Some(60000000000), // 1-minute
            priority_regossip_max_txs: Some(16),
            priority_regossip_txs_per_address: Some(1),
            priority_regossip_addresses: Some(vec![
                "0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC".to_string(), // ewoq key address
            ]),

            log_level: Some(String::from(DEFAULT_LOG_LEVEL)),
            log_json_format: Some(DEFAULT_LOG_JSON_FORMAT),

            fee_recipient: None,

            // cannot run offline pruning while pruning is disabled
            offline_pruning_enabled: None,
            offline_pruning_bloom_filter_size: Some(512),
            offline_pruning_data_directory: None,

            max_outbound_active_requests: Some(16),

            state_sync_enabled: None,
            state_sync_skip_resume: None,
            state_sync_server_trie_cache: Some(64),
            state_sync_ids: None,
            state_sync_commit_interval: Some(4096 * 4), // defaultCommitInterval * 4
            state_sync_min_blocks: Some(300_000),

            skip_upgrade_check: None,
            accepted_cache_size: Some(32),
        }
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }

    /// Saves the current chain config to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing subnet-evm chain config to '{}'", file_path);
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="subnet_evm" -- subnet_evm::chain_config::test_config --exact --show-output
#[test]
fn test_config() {
    let _ = env_logger::builder().is_test(true).try_init();

    let tmp_path = random_manager::tmp_path(10, Some(".json")).unwrap();
    let cfg = Config::default();
    log::info!("{}", cfg.encode_json().unwrap());

    cfg.sync(&tmp_path).unwrap();

    fs::remove_file(tmp_path).unwrap();
}
