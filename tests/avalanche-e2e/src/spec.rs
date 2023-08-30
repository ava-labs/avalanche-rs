use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
    string::String,
};

use crate::{c, x};
use avalanche_types::key;
use serde::{Deserialize, Serialize};

pub const RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER: &str = "network-runner-rpc-server";
pub const RPC_ENDPOINT_KIND_AVALANCHEGO_RPC_ENDPOINT: &str = "avalanchego-rpc-endpoint";

/// Represents the e2e test specification.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Spec {
    /// RPC endpoint type:
    /// - "network-runner-rpc-server"
    /// - "avalanchego-rpc"
    pub rpc_endpoint_kind: String,
    /// May initially be set network runner server endpoints
    /// but later to be updated with avalanche RPC endpoints.
    pub rpc_endpoints: Vec<String>,

    /// If empty, it downloads the latest from the github release page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avalanchego_path: Option<String>,
    /// If empty, it downloads the latest from the github release page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avalanchego_plugin_dir: Option<String>,

    /// Only use it for testing.
    pub key_infos: Vec<key::secp256k1::Info>,

    /// "true" to randomize test run order.
    pub randomize: bool,
    /// "true" to run tests in parallel.
    pub parallelize: bool,
    /// "true" to ignore errors in the tests.
    pub ignore_errors: bool,

    /// Represents test scenario Ids.
    pub scenarios: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_simple_transfers: Option<x::simple_transfers::Config>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_exports: Option<x::exports::Config>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_simple_transfers: Option<c::simple_transfers::Config>,

    /// Read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

impl Default for Spec {
    fn default() -> Self {
        Self::default()
    }
}

impl Spec {
    pub fn default() -> Self {
        Self {
            rpc_endpoint_kind: String::from(RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER),
            rpc_endpoints: Vec::new(),

            avalanchego_path: None,
            avalanchego_plugin_dir: None,

            key_infos: Vec::new(),

            randomize: false,
            parallelize: false,
            ignore_errors: false,

            scenarios: vec![
                x::simple_transfers::NAME.to_string(),
                c::simple_transfers::NAME.to_string(),
                //
                // TODO: not working
                // x::exports::NAME.to_string(),
            ],

            x_simple_transfers: Some(x::simple_transfers::Config::default()),
            x_exports: Some(x::exports::Config::default()),
            c_simple_transfers: Some(c::simple_transfers::Config::default()),

            status: None,
        }
    }

    /// Converts to string in YAML format.
    pub fn encode_yaml(&self) -> io::Result<String> {
        match serde_yaml::to_string(&self) {
            Ok(s) => Ok(s),
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to serialize Spec to YAML {}", e),
                ));
            }
        }
    }

    /// Saves the current spec to disk and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing Spec to '{}'", file_path);
        let path = Path::new(file_path);
        if let Some(parent_dir) = path.parent() {
            log::info!("creating parent dir '{}'", parent_dir.display());
            fs::create_dir_all(parent_dir)?;
        }

        let d = serde_yaml::to_string(self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize YAML {}", e)))?;

        let mut f = File::create(file_path)?;
        f.write_all(d.as_bytes())?;

        Ok(())
    }

    /// Loads the spec from the file.
    pub fn load(file_path: &str) -> io::Result<Self> {
        log::info!("loading Spec from {}", file_path);

        if !Path::new(file_path).exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("file {} does not exists", file_path),
            ));
        }

        let f = File::open(&file_path).map_err(|e| {
            return Error::new(
                ErrorKind::Other,
                format!("failed to open {} ({})", file_path, e),
            );
        })?;
        serde_yaml::from_reader(f).map_err(|e| {
            return Error::new(ErrorKind::InvalidInput, format!("invalid YAML: {}", e));
        })
    }

    /// Validates the spec.
    /// TODO: check byzantine test cases
    pub fn validate(&self) -> io::Result<()> {
        log::info!("validating Spec");

        match self.rpc_endpoint_kind.as_str() {
            RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER => {}
            RPC_ENDPOINT_KIND_AVALANCHEGO_RPC_ENDPOINT => {}
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("unknown rpc_endpoint_kind '{}'", self.rpc_endpoint_kind),
                ));
            }
        }

        if self.key_infos.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "empty key_infos"));
        }

        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-e2e -- spec::test_spec --exact --show-output
#[test]
fn test_spec() {
    let d = r#"
rpc_endpoint_kind: network-runner-rpc-server
rpc_endpoints:
- a
- b
- c

key_infos:
- key_type: hot
  private_key_cb58: PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN
  private_key_hex: 0x56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027
  addresses:
    1:
      x: X-avax18jma8ppw3nhx5r4ap8clazz0dps7rv5ukulre5
      p: P-avax18jma8ppw3nhx5r4ap8clazz0dps7rv5ukulre5
    9999:
      x: X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p
      p: P-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p
  short_address: 6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV
  eth_address: 0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC
  h160_address: 0x8db97c7cece249c2b98bdc0226cc4c2a57bf52fc

randomize: true
parallelize: false
ignore_errors: false

scenarios:
- X_SIMPLE_TRANSFERS

"#;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    assert!(f.write_all(d.as_bytes()).is_ok());

    let spec_path = f.path().to_str().unwrap();

    let loaded_spec = Spec::load(spec_path).unwrap();
    loaded_spec.validate().unwrap();
    assert!(loaded_spec.validate().is_ok());
    assert!(loaded_spec.sync(spec_path).is_ok());

    let expected = Spec {
        rpc_endpoint_kind: String::from(RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER),
        rpc_endpoints: vec!["a".to_string(), "b".to_string(), "c".to_string()],

        avalanchego_path: None,
        avalanchego_plugin_dir: None,

        key_infos: vec![key::secp256k1::TEST_INFOS[0].clone()],

        randomize: true,
        parallelize: false,
        ignore_errors: false,

        scenarios: vec![x::simple_transfers::NAME.to_string()],

        x_simple_transfers: None,
        x_exports: None,
        c_simple_transfers: None,

        status: None,
    };
    assert_eq!(expected, loaded_spec);
}

/// Represents read-only test status.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_runner_endpoint: Option<String>,
    pub network_id: u32,
    pub randomized_scenarios: Vec<String>,
}
