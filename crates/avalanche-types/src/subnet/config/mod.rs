pub mod consensus;
pub mod gossip;

use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

/// To be persisted in "subnet_config_dir".
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/chains#SubnetConfig>
///
/// If a Subnet's chain id is 2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt,
/// the config file for this chain is located at {subnet-config-dir}/2ebCneCbwthjQ1rYT41nhd7M76Hc6YmosMAQrTFhBq8qeqh6tt.json
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Embeds "gossip_config" at the same level as other fields.
    #[serde(flatten)]
    pub gossip_sender_config: gossip::SenderConfig,

    #[serde(default)]
    pub validator_only: bool,

    /// Embeds "gossip_config" at the same level as other fields.
    pub consensus_parameters: consensus::Parameters,

    #[serde(default)]
    pub proposer_min_block_delay: u64,
}

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
            gossip_sender_config: gossip::SenderConfig::default(),
            validator_only: false,
            consensus_parameters: consensus::Parameters::default(),
            proposer_min_block_delay: 1000 * 1000 * 1000, // 1-second
        }
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }

    /// Saves the current subnet config to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing subnet config to '{}'", file_path);
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- subnet::config::test_config --exact --show-output
#[test]
fn test_config() {
    let _ = env_logger::builder().is_test(true).try_init();

    let tmp_path = random_manager::tmp_path(10, Some(".json")).unwrap();
    let cfg = Config::default();
    cfg.sync(&tmp_path).unwrap();
}
