use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
    str::FromStr,
};

use crate::{errors, ids::short, key, packer};
use serde::{Deserialize, Serialize};

/// ref. <https://github.com/ava-labs/xsvm/blob/master/genesis/genesis.go>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    #[serde(default)]
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocations: Option<Vec<Allocation>>,
}

impl Default for Genesis {
    fn default() -> Self {
        Self::default()
    }
}

pub const CODEC_VERSION: u16 = 0;
pub const DEFAULT_INITIAL_AMOUNT: u64 = 1000000000;

impl Genesis {
    pub fn default() -> Self {
        Self {
            timestamp: 0,
            allocations: Some(vec![Allocation {
                // ref. https://github.com/ava-labs/subnet-evm/blob/master/networks/11111/genesis.json
                address: short::Id::from_str("6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV").unwrap(),
                balance: DEFAULT_INITIAL_AMOUNT,
            }]),
        }
    }

    /// Creates a new Genesis object with "keys" number of generated pre-funded keys.
    pub fn new<T: key::secp256k1::ReadOnly>(seed_keys: &[T]) -> io::Result<Self> {
        // maximize total supply
        let max_total_alloc = u64::MAX;
        let total_keys = seed_keys.len();
        let alloc_per_key = if total_keys > 0 {
            max_total_alloc / total_keys as u64
        } else {
            0u64
        };
        // divide by 2, allow more room for transfers
        let alloc_per_key = alloc_per_key / 2;

        let mut allocs = Vec::new();
        for k in seed_keys.iter() {
            allocs.push(Allocation {
                address: k.short_address().unwrap(),
                balance: alloc_per_key,
            });
        }

        let mut genesis = Self::default();
        genesis.allocations = Some(allocs);

        Ok(genesis)
    }

    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(&self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))
    }

    /// Encodes the genesis to JSON bytes.
    pub fn to_json_bytes(&self) -> io::Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed encode JSON {}", e)))
    }

    /// Saves the current genesis to disk
    /// and overwrites the file in JSON format.
    pub fn sync_json(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing '{}' in JSON", file_path);
        let path = Path::new(file_path);
        if let Some(parent_dir) = path.parent() {
            log::info!("creating parent dir '{}'", parent_dir.display());
            fs::create_dir_all(parent_dir)?;
        }

        let d = self.to_json_bytes()?;

        let mut f = File::create(file_path)?;
        f.write_all(&d)?;

        Ok(())
    }

    /// Encodes the genesis to packer bytes.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/codec#Manager>
    pub fn to_packer_bytes(&self) -> errors::Result<Vec<u8>> {
        let packer = packer::Packer::new((1 << 31) - 1, 128);

        // codec version
        // ref. "avalanchego/codec.manager.Marshal"
        packer.pack_u16(CODEC_VERSION)?;
        packer.pack_u64(self.timestamp as u64)?;

        if let Some(allocs) = &self.allocations {
            packer.pack_u32(allocs.len() as u32)?;
            for alloc in allocs.iter() {
                packer.pack_bytes(alloc.address.as_ref())?;
                packer.pack_u64(alloc.balance)?;
            }
        }

        Ok(packer.take_bytes().into())
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/core#GenesisAlloc>
/// ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/core#GenesisAccount>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Allocation {
    pub address: short::Id,
    #[serde(default)]
    pub balance: u64,
}

impl Default for Allocation {
    fn default() -> Self {
        Self::default()
    }
}

impl Allocation {
    pub fn default() -> Self {
        Self {
            address: short::Id::empty(),
            balance: 0,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- xsvm::genesis::test_encode_packer_bytes --exact --show-output
#[test]
fn test_encode_packer_bytes() {
    let _ = env_logger::builder().is_test(true).try_init();

    let genesis = Genesis {
        timestamp: 123,
        allocations: Some(vec![
            Allocation {
                // ref. https://github.com/ava-labs/subnet-evm/blob/master/networks/11111/genesis.json
                address: short::Id::from_str("6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV").unwrap(),
                balance: 1000000000,
            },
            Allocation {
                // ref. https://github.com/ava-labs/subnet-evm/blob/master/networks/11111/genesis.json
                address: short::Id::from_str("LeKrndtsMxcLMzHz3w4uo1XtLDpfi66c").unwrap(),
                balance: 3000000000,
            },
        ]),
    };
    let genesis_packer_bytes = genesis.to_packer_bytes().unwrap();

    let expected = vec![
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7b, 0x0, 0x0, 0x0, 0x2, 0x3c, 0xb7, 0xd3,
        0x84, 0x2e, 0x8c, 0xee, 0x6a, 0xe, 0xbd, 0x9, 0xf1, 0xfe, 0x88, 0x4f, 0x68, 0x61, 0xe1,
        0xb2, 0x9c, 0x0, 0x0, 0x0, 0x0, 0x3b, 0x9a, 0xca, 0x0, 0x3, 0xb7, 0xf, 0x8c, 0x60, 0x6b,
        0x2b, 0xf, 0x99, 0x84, 0x9d, 0xc8, 0x5c, 0x40, 0xf, 0xd1, 0x7e, 0xfe, 0x1f, 0x60, 0x0, 0x0,
        0x0, 0x0, 0xb2, 0xd0, 0x5e, 0x0,
    ];
    assert_eq!(genesis_packer_bytes, expected);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- xsvm::genesis::test_parse --exact --show-output
#[test]
fn test_parse() {
    let _ = env_logger::builder().is_test(true).try_init();

    // ref. https://github.com/ava-labs/subnet-evm/blob/master/networks/11111/genesis.json
    let resp: Genesis = serde_json::from_str(
        r#"
{
    "unknown1": "field1",
    "unknown2": "field2",

    "timestamp": 123,

        "allocations": [
            {"address": "6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV", "balance": 2000000000},
            {"address": "LeKrndtsMxcLMzHz3w4uo1XtLDpfi66c", "balance": 3000000000}
        ]
}
"#,
    )
    .unwrap();

    let expected = Genesis {
        timestamp: 123,
        allocations: Some(vec![
            Allocation {
                // ref. https://github.com/ava-labs/subnet-evm/blob/master/networks/11111/genesis.json
                address: short::Id::from_str("6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV").unwrap(),
                balance: 2000000000,
            },
            Allocation {
                // ref. https://github.com/ava-labs/subnet-evm/blob/master/networks/11111/genesis.json
                address: short::Id::from_str("LeKrndtsMxcLMzHz3w4uo1XtLDpfi66c").unwrap(),
                balance: 3000000000,
            },
        ]),
    };
    assert_eq!(resp, expected);

    let d = expected.encode_json().unwrap();
    log::info!("{}", d);
}
