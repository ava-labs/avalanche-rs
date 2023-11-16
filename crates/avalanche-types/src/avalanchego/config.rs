//! AvalancheGo configuration.
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
};

use crate::{avalanchego::genesis, constants, units};
use serde::{Deserialize, Serialize};

/// Represents AvalancheGo configuration.
/// All file paths must be valid on the remote machines.
/// For example, you may configure cert paths on your local laptop
/// but the actual Avalanche nodes run on the remote machines
/// so the paths will be invalid.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/config>
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.8/config/flags.go>
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.8/config/keys.go>
/// ref. <https://serde.rs/container-attrs.html>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// File path to persist all fields below.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<String>,

    /// Genesis file path.
    /// MUST BE NON-EMPTY for custom network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_file: Option<String>,

    /// Network ID. Default to custom network ID.
    /// Set it to 1 for mainnet.
    /// e.g., "mainnet" is 1, "fuji" is 5, "local" is 12345.
    /// "utils/constants/NetworkID" only accepts string for known networks.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants#pkg-constants>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants#NetworkName>
    #[serde(default)]
    pub network_id: u32,

    #[serde(default)]
    pub db_type: String,
    /// Database directory, must be a valid path in remote host machine.
    #[serde(default)]
    pub db_dir: String,
    /// Chain data directory, must be a valid path in remote host machine.
    #[serde(default)]
    pub chain_data_dir: String,

    /// Logging directory, must be a valid path in remote host machine.
    #[serde(default)]
    pub log_dir: String,

    /// "avalanchego" logging level.
    /// See "utils/logging/level.go".
    /// e.g., "INFO", "FATAL", "DEBUG", "VERBO", etc..
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    /// "avalanchego" logging format.
    /// e.g., "json", etc..
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_display_level: Option<String>,

    /// HTTP port.
    #[serde(default)]
    pub http_port: u32,
    /// HTTP host, which avalanchego defaults to 127.0.0.1.
    /// Set it to 0.0.0.0 to expose the HTTP API to all incoming traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_tls_enabled: Option<bool>,
    /// MUST BE a valid path in remote host machine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_tls_key_file: Option<String>,
    /// MUST BE a valid path in remote host machine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_tls_cert_file: Option<String>,
    /// Public IP of this node for P2P communication.
    /// If empty, try to discover with NAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sybil_protection_enabled: Option<bool>,
    /// Staking port.
    #[serde(default)]
    pub staking_port: u32,
    /// MUST BE a valid path in remote host machine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staking_tls_key_file: Option<String>,
    /// MUST BE a valid path in remote host machine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staking_tls_cert_file: Option<String>,
    /// MUST BE a valid path in remote host machine.
    /// Path to the BLS key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staking_signer_key_file: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_ips: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_ids: Option<String>,

    /// The sample size k, snowball.Parameters.K.
    /// If zero, use the default value set via avalanche node code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_sample_size: Option<u32>,
    /// The quorum size α, snowball.Parameters.Alpha.
    /// If zero, use the default value set via avalanche node code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_quorum_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_concurrent_repolls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_max_time_processing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_rogue_commit_threshold: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_virtuous_commit_threshold: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_peer_list_gossip_frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_max_reconnect_delay: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_allow_incomplete: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_admin_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_info_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_keystore_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_metrics_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_health_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_ipcs_enabled: Option<bool>,

    /// A list of whitelisted/tracked subnet IDs (comma-separated).
    /// From avalanchego v1.9.7, it's renamed to "track-subnets".
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.8/config/keys.go>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_subnets: Option<String>,

    /// Plugin directory.
    /// Default to "/data/avalanche-plugins".
    #[serde(default)]
    pub plugin_dir: String,
    /// Subnet configuration directory (e.g., /data/avalanche-configs/subnets/C.json).
    /// If a subnet id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
    /// the config file for this subnet is located at {subnet-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt.json.
    #[serde(default)]
    pub subnet_config_dir: String,
    /// Chain configuration directory (e.g., /data/avalanche-configs/chains/C/config.json).
    /// If a Subnet's chain id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
    /// the config file for this chain is located at {chain-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt/config.json.
    #[serde(default)]
    pub chain_config_dir: String,

    /// A comma separated string of explicit nodeID and IPs
    /// to contact for starting state sync. Useful for testing.
    /// NOTE: Actual state data will be downloaded from nodes
    /// specified in the C-Chain config, or the entire network
    /// if no list specified there.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_ids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_sync_ips: Option<String>,

    /// Continuous profile flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_continuous_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_continuous_freq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_continuous_max_files: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposervm_use_current_height: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_node_max_processing_msgs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_bandwidth_refill_rate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_bandwidth_max_burst_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_cpu_validator_alloc: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_disk_validator_alloc: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_at_large_alloc_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_validator_alloc_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_inbound_node_max_at_large_bytes: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_mixed_query_num_push_vdr: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_accepted_frontier_gossip_frequency: Option<i64>,
    /// ref. <https://github.com/ava-labs/avalanchego/pull/1322>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_app_concurrency: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_on_accept_gossip_validator_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_on_accept_gossip_non_validator_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_on_accept_gossip_peer_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_accepted_frontier_gossip_peer_size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_outbound_at_large_alloc_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_outbound_validator_alloc_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttler_outbound_node_max_at_large_bytes: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_minimum_timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_require_validator_to_connect: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_compression_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_context_file: Option<String>,
}

/// Default "config-file" path on the remote linux machines.
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_CONFIG_FILE_PATH: &str = "/data/avalanche-configs/config.json";
/// Default "genesis" path on the remote linux machines.
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_GENESIS_PATH: &str = "/data/avalanche-configs/genesis.json";
/// Default "chain aliases" path on the remote linux machine.
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_CHAIN_ALIASES_PATH: &str = "/data/avalanche-configs/chains/aliases.json";

pub const DEFAULT_DB_TYPE: &str = "leveldb";
/// Default "db-dir" directory path for remote linux machines.
/// MUST BE matched with the attached physical storage volume path.
/// MUST BE a valid path in remote host machine.
/// ref. See "cfn-templates/avalanche-node/asg_amd64_ubuntu.yaml" "ASGLaunchTemplate"
pub const DEFAULT_DB_DIR: &str = "/data/db";
/// Default "chain-data-dir" directory path for remote linux machines.
/// MUST BE matched with the attached physical storage volume path.
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_CHAIN_DATA_DIR: &str = "/data/chainData";

/// Default "log-dir" directory path for remote linux machines.
/// MUST BE a valid path in remote host machine.
/// ref. See "cfn-templates/avalanche-node/asg_amd64_ubuntu.yaml" "ASGLaunchTemplate"
pub const DEFAULT_LOG_DIR: &str = "/var/log/avalanchego";
pub const DEFAULT_LOG_LEVEL: &str = "INFO";
pub const DEFAULT_LOG_FORMAT: &str = "json";

/// Default HTTP port.
/// NOTE: keep default value in sync with "avalanchego/config/flags.go".
pub const DEFAULT_HTTP_PORT: u32 = 9650;
/// Default HTTP host.
/// Open listener to "0.0.0.0" to allow all incoming traffic.
/// e.g., If set to default "127.0.0.1", the external client
/// cannot access "/ext/metrics". Set different values to
/// make this more restrictive.
pub const DEFAULT_HTTP_HOST: &str = "0.0.0.0";
pub const DEFAULT_HTTP_TLS_ENABLED: bool = false;

pub const DEFAULT_SYBIL_PROTECTION_ENABLED: bool = true;
/// Default staking port.
/// NOTE: keep default value in sync with "avalanchego/config/flags.go".
pub const DEFAULT_STAKING_PORT: u32 = 9651;
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_STAKING_TLS_KEY_FILE: &str = "/data/staking.key";
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_STAKING_TLS_CERT_FILE: &str = "/data/staking.crt";
/// MUST BE a valid path in remote host machine.
pub const DEFAULT_STAKING_SIGNER_KEY_FILE: &str = "/data/staking-signer.bls.key";

/// Default snow sample size.
/// NOTE: keep this in sync with "avalanchego/config/flags.go".
pub const DEFAULT_SNOW_SAMPLE_SIZE: u32 = 20;
/// Default snow quorum size.
/// NOTE: keep this in sync with "avalanchego/config/flags.go".
pub const DEFAULT_SNOW_QUORUM_SIZE: u32 = 15;

pub const DEFAULT_INDEX_ENABLED: bool = false;
pub const DEFAULT_INDEX_ALLOW_INCOMPLETE: bool = false;

pub const DEFAULT_API_ADMIN_ENABLED: bool = false;
pub const DEFAULT_API_INFO_ENABLED: bool = true;
pub const DEFAULT_API_KEYSTORE_ENABLED: bool = false;
pub const DEFAULT_API_METRICS_ENABLED: bool = true;
pub const DEFAULT_API_HEALTH_ENABLED: bool = true;
pub const DEFAULT_API_IPCS_ENABLED: bool = false;

/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go>
pub const DEFAULT_PLUGIN_DIR: &str = "/data/avalanche-plugins";

/// If a subnet id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
/// the config file for this subnet is located at {subnet-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt.json.
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go>
/// ref. <https://docs.avax.network/subnets/customize-a-subnet#chain-configs>
pub const DEFAULT_SUBNET_CONFIG_DIR: &str = "/data/avalanche-configs/subnets";

/// If a Subnet's chain id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
/// the config file for this chain is located at {chain-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt/config.json.
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go>
/// ref. <https://docs.avax.network/subnets/customize-a-subnet#chain-configs>
pub const DEFAULT_CHAIN_CONFIG_DIR: &str = "/data/avalanche-configs/chains";

/// MUST BE a valid path in remote host machine.
pub const DEFAULT_PROFILE_DIR: &str = "/var/log/avalanchego-profile/avalanche";

/// ref. [DefaultInboundThrottlerAtLargeAllocSize](https://github.com/ava-labs/avalanchego/blob/master/utils/constants/networking.go#L88)
pub const DEFAULT_THROTTLER_INBOUND_AT_LARGE_ALLOC_SIZE: u64 = 6 * units::MIB;
/// ref. [DefaultInboundThrottlerVdrAllocSize](https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go)
pub const DEFAULT_THROTTLER_INBOUND_VALIDATOR_ALLOC_SIZE: u64 = 32 * units::MIB;
/// ref. [DefaultInboundThrottlerNodeMaxAtLargeBytes](https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go)
pub const DEFAULT_THROTTLER_INBOUND_NODE_MAX_AT_LARGE_BYTES: u64 = 2 * units::MIB;

/// ref. [DefaultOutboundThrottlerAtLargeAllocSize](https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go)
pub const DEFAULT_THROTTLER_OUTBOUND_AT_LARGE_ALLOC_SIZE: u64 = 32 * units::MIB;
/// ref. [DefaultOutboundThrottlerVdrAllocSize](https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go)
pub const DEFAULT_THROTTLER_OUTBOUND_VALIDATOR_ALLOC_SIZE: u64 = 32 * units::MIB;
/// ref. [DefaultOutboundThrottlerNodeMaxAtLargeBytes](https://github.com/ava-labs/avalanchego/blob/v1.9.11/config/flags.go)
pub const DEFAULT_THROTTLER_OUTBOUND_NODE_MAX_AT_LARGE_BYTES: u64 = 2 * units::MIB;

pub const DEFAULT_NETWORK_COMPRESSION_TYPE: &str = "zstd";

pub const DEFAULT_PROCESS_CONTEXT_FILE: &str = "/data/process.json";

impl Default for Config {
    fn default() -> Self {
        Self::default_main()
    }
}

impl Config {
    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default_main() -> Self {
        Self {
            config_file: Some(String::from(DEFAULT_CONFIG_FILE_PATH)),
            genesis_file: None,

            network_id: 1,

            db_type: String::from(DEFAULT_DB_TYPE),
            db_dir: String::from(DEFAULT_DB_DIR),
            chain_data_dir: String::from(DEFAULT_CHAIN_DATA_DIR),
            log_dir: String::from(DEFAULT_LOG_DIR),
            log_level: Some(String::from(DEFAULT_LOG_LEVEL)),
            log_format: Some(String::from(DEFAULT_LOG_FORMAT)),
            log_display_level: None,

            http_port: DEFAULT_HTTP_PORT,
            http_host: Some(String::from(DEFAULT_HTTP_HOST)),
            http_tls_enabled: Some(DEFAULT_HTTP_TLS_ENABLED),
            http_tls_key_file: None,
            http_tls_cert_file: None,
            public_ip: None,

            sybil_protection_enabled: Some(DEFAULT_SYBIL_PROTECTION_ENABLED),
            staking_port: DEFAULT_STAKING_PORT,
            staking_tls_key_file: Some(String::from(DEFAULT_STAKING_TLS_KEY_FILE)),
            staking_tls_cert_file: Some(String::from(DEFAULT_STAKING_TLS_CERT_FILE)),
            staking_signer_key_file: Some(String::from(DEFAULT_STAKING_SIGNER_KEY_FILE)),

            bootstrap_ips: None,
            bootstrap_ids: None,

            snow_sample_size: Some(DEFAULT_SNOW_SAMPLE_SIZE),
            snow_quorum_size: Some(DEFAULT_SNOW_QUORUM_SIZE),
            snow_concurrent_repolls: None,
            snow_max_time_processing: None,
            snow_rogue_commit_threshold: None,
            snow_virtuous_commit_threshold: None,

            network_peer_list_gossip_frequency: None,
            network_max_reconnect_delay: None,

            index_enabled: Some(DEFAULT_INDEX_ENABLED),
            index_allow_incomplete: Some(DEFAULT_INDEX_ALLOW_INCOMPLETE),

            api_admin_enabled: Some(DEFAULT_API_ADMIN_ENABLED),
            api_info_enabled: Some(DEFAULT_API_INFO_ENABLED),
            api_keystore_enabled: Some(DEFAULT_API_KEYSTORE_ENABLED),
            api_metrics_enabled: Some(DEFAULT_API_METRICS_ENABLED),
            api_health_enabled: Some(DEFAULT_API_HEALTH_ENABLED),
            api_ipcs_enabled: Some(DEFAULT_API_IPCS_ENABLED),

            track_subnets: None,

            plugin_dir: String::from(DEFAULT_PLUGIN_DIR),
            subnet_config_dir: String::from(DEFAULT_SUBNET_CONFIG_DIR),
            chain_config_dir: String::from(DEFAULT_CHAIN_CONFIG_DIR),

            state_sync_ids: None,
            state_sync_ips: None,

            profile_dir: Some(String::from(DEFAULT_PROFILE_DIR)),
            profile_continuous_enabled: None,
            profile_continuous_freq: None,
            profile_continuous_max_files: None,

            proposervm_use_current_height: Some(true),
            throttler_inbound_node_max_processing_msgs: Some(100000),
            throttler_inbound_bandwidth_refill_rate: Some(1073741824),
            throttler_inbound_bandwidth_max_burst_size: Some(1073741824),
            throttler_inbound_cpu_validator_alloc: Some(100000),
            throttler_inbound_disk_validator_alloc: Some(10737418240000),

            throttler_inbound_at_large_alloc_size: Some(
                DEFAULT_THROTTLER_INBOUND_AT_LARGE_ALLOC_SIZE,
            ),
            throttler_inbound_validator_alloc_size: Some(
                DEFAULT_THROTTLER_INBOUND_VALIDATOR_ALLOC_SIZE,
            ),
            throttler_inbound_node_max_at_large_bytes: Some(
                DEFAULT_THROTTLER_INBOUND_NODE_MAX_AT_LARGE_BYTES,
            ),

            snow_mixed_query_num_push_vdr: Some(10),

            consensus_accepted_frontier_gossip_frequency: Some(10000000000), // 10-second
            consensus_app_concurrency: Some(2),

            consensus_on_accept_gossip_validator_size: Some(0),
            consensus_on_accept_gossip_non_validator_size: Some(0),
            consensus_on_accept_gossip_peer_size: Some(10),
            consensus_accepted_frontier_gossip_peer_size: Some(10),

            throttler_outbound_at_large_alloc_size: Some(
                DEFAULT_THROTTLER_OUTBOUND_AT_LARGE_ALLOC_SIZE,
            ),
            throttler_outbound_validator_alloc_size: Some(
                DEFAULT_THROTTLER_OUTBOUND_VALIDATOR_ALLOC_SIZE,
            ),
            throttler_outbound_node_max_at_large_bytes: Some(
                DEFAULT_THROTTLER_OUTBOUND_NODE_MAX_AT_LARGE_BYTES,
            ),

            network_minimum_timeout: None,
            network_require_validator_to_connect: None,

            network_compression_type: Some(DEFAULT_NETWORK_COMPRESSION_TYPE.to_string()),

            tracing_enabled: None,

            process_context_file: Some(DEFAULT_PROCESS_CONTEXT_FILE.to_string()),
        }
    }

    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default_fuji() -> Self {
        let mut cfg = Self::default_main();
        cfg.network_id = 5;
        cfg
    }

    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default_custom() -> Self {
        let mut cfg = Self::default_main();
        cfg.network_id = constants::DEFAULT_CUSTOM_NETWORK_ID;
        cfg.genesis_file = Some(String::from(DEFAULT_GENESIS_PATH));
        cfg
    }

    /// Returns true if the configuration is mainnet.
    pub fn is_mainnet(&self) -> bool {
        self.network_id == 1
    }

    /// Returns true if the configuration is a custom network
    /// thus requires a custom genesis file.
    pub fn is_custom_network(&self) -> bool {
        !self.is_mainnet() && (self.network_id == 0 || self.network_id > 5)
    }

    pub fn add_track_subnets(&mut self, ids: Option<String>) {
        let mut all_ids = BTreeSet::new();
        if let Some(existing) = &self.track_subnets {
            let ss: Vec<&str> = existing.split(',').collect();
            for id in ss {
                all_ids.insert(id.trim().to_string());
            }
        }

        if let Some(new_ids) = &ids {
            let ss: Vec<&str> = new_ids.split(',').collect();
            for id in ss {
                all_ids.insert(id.trim().to_string());
            }
        }

        let mut ids = Vec::new();
        for id in all_ids.iter() {
            ids.push(id.trim().to_string());
        }

        if !ids.is_empty() {
            self.track_subnets = Some(ids.join(","))
        }
    }

    /// Converts to string with JSON encoder.
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }

    /// Saves the current configuration to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: Option<String>) -> io::Result<()> {
        if file_path.is_none() && self.config_file.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "empty config_file path",
            ));
        }
        let p = file_path.unwrap_or_else(|| {
            self.config_file
                .clone()
                .expect("unexpected None config_file")
        });

        log::info!("mkdir avalanchego configuration dir for '{}'", p);
        let path = Path::new(&p);
        if let Some(parent_dir) = path.parent() {
            log::info!("creating parent dir '{}'", parent_dir.display());
            fs::create_dir_all(parent_dir)?;
        }

        let d = serde_json::to_vec(self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))?;

        log::info!("syncing avalanchego Config to '{}'", p);
        let mut f = File::create(p)?;
        f.write_all(&d)?;

        Ok(())
    }

    pub fn load(file_path: &str) -> io::Result<Self> {
        log::info!("loading avalanchego config from {}", file_path);

        if !Path::new(file_path).exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("file {} does not exists", file_path),
            ));
        }

        let f = File::open(file_path).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to open {} ({})", file_path, e),
            )
        })?;
        serde_json::from_reader(f)
            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid JSON: {}", e)))
    }

    /// Validates the configuration.
    pub fn validate(&self) -> io::Result<()> {
        log::info!("validating the avalanchego configuration");

        // mainnet does not need genesis file
        if !self.is_custom_network() && self.genesis_file.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "non-empty '--genesis={}' for network_id {}",
                    self.genesis_file.clone().expect("unexpected None genesis"),
                    self.network_id,
                ),
            ));
        }

        // custom network requires genesis file
        if self.is_custom_network() && self.genesis_file.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "non-empty '--network-id={}' but empty '--genesis'",
                    self.network_id
                ),
            ));
        }

        // custom network requires genesis file
        if self.genesis_file.is_some()
            && !Path::new(&self.genesis_file.clone().expect("unexpected None genesis")).exists()
        {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "non-empty '--genesis={}' but genesis file does not exist",
                    self.genesis_file.clone().expect("unexpected None genesis")
                ),
            ));
        }

        // network ID must match with the one in genesis file
        if self.genesis_file.is_some() {
            let genesis_file_path = self.genesis_file.clone().expect("unexpected None genesis");
            let genesis_config =
                genesis::Genesis::load(&genesis_file_path).expect("unexpected None genesis config");
            if genesis_config.network_id.ne(&self.network_id) {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "'genesis' network ID {} != avalanchego::Config.network_id {}",
                        genesis_config.network_id, self.network_id
                    ),
                ));
            }
        }

        // staking
        if self.sybil_protection_enabled.is_some() && !self.sybil_protection_enabled.unwrap() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "'sybil_protection_enabled' must be true",
            ));
        }
        if self.staking_tls_cert_file.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "'staking-tls-cert-file' not defined",
            ));
        }
        if self.staking_tls_key_file.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "'staking-tls-key-file' not defined",
            ));
        }
        if self.staking_signer_key_file.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "'staking-signer-key-file' not defined",
            ));
        }

        // state sync
        if self.state_sync_ids.is_some() && self.state_sync_ips.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "non-empty 'state-sync-ids' but empty 'state-sync-ips'",
            ));
        }
        if self.state_sync_ids.is_none() && self.state_sync_ips.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "non-empty 'state-sync-ips' but empty 'state-sync-ids'",
            ));
        }

        // continuous profiles
        if self.profile_continuous_enabled.is_some() && !self.profile_continuous_enabled.unwrap() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "'profile-continuous-enabled' must be true",
            ));
        }
        if self.profile_continuous_freq.is_some() && self.profile_continuous_enabled.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "non-empty 'profile-continuous-freq' but empty 'profile-continuous-enabled'",
            ));
        }
        if self.profile_continuous_max_files.is_some() && self.profile_continuous_enabled.is_none()
        {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "non-empty 'profile-continuous-max-files' but empty 'profile-continuous-enabled'",
            ));
        }

        Ok(())
    }
}

#[test]
fn test_config() {
    use std::fs;
    let _ = env_logger::builder().is_test(true).try_init();

    let mut config = Config::default_custom();
    config.network_id = 1337;

    let ret = config.encode_json();
    assert!(ret.is_ok());
    let s = ret.unwrap();
    log::info!("config: {}", s);

    let p = random_manager::tmp_path(10, Some(".yaml")).unwrap();
    let ret = config.sync(Some(p.clone()));
    assert!(ret.is_ok());

    let config_loaded = Config::load(&p).unwrap();
    assert_eq!(config, config_loaded);

    config.add_track_subnets(Some("x,y,a,b,d,f".to_string()));
    println!("{}", config.track_subnets.clone().unwrap());
    assert_eq!(config.track_subnets.unwrap(), "a,b,d,f,x,y");

    fs::remove_file(p).unwrap();
}
