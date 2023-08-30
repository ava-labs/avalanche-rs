//! AvalancheGo network configuration.
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Error, ErrorKind, Write},
    path::Path,
    time::SystemTime,
    u64,
};

use crate::{constants, coreth::genesis as coreth_genesis, key};
use serde::{Deserialize, Serialize};

/// Represents Avalanche network genesis configuration.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/genesis#Config>
/// ref. <https://serde.rs/container-attrs.html>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Genesis {
    #[serde(rename = "networkID")]
    pub network_id: u32,

    #[serde(rename = "allocations", skip_serializing_if = "Option::is_none")]
    pub allocations: Option<Vec<Allocation>>,

    /// Unix time for start time.
    #[serde(rename = "startTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<u64>,
    /// Number of seconds to stake for the initial stakers.
    #[serde(
        rename = "initialStakeDuration",
        skip_serializing_if = "Option::is_none"
    )]
    pub initial_stake_duration: Option<u64>,
    #[serde(
        rename = "initialStakeDurationOffset",
        skip_serializing_if = "Option::is_none"
    )]
    pub initial_stake_duration_offset: Option<u64>,
    /// MUST BE come from "initial_stakers".
    /// MUST BE the list of X-chain addresses.
    /// Initial staked funds cannot be empty.
    #[serde(rename = "initialStakedFunds", skip_serializing_if = "Option::is_none")]
    pub initial_staked_funds: Option<Vec<String>>,
    /// MUST BE non-empty for an existing network.
    /// Non-anchor nodes request "GetAcceptedFrontier" from initial stakers
    /// (not from specified anchor nodes).
    #[serde(rename = "initialStakers", skip_serializing_if = "Option::is_none")]
    pub initial_stakers: Option<Vec<Staker>>,

    #[serde(rename = "cChainGenesis")]
    pub c_chain_genesis: coreth_genesis::Genesis,

    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// All of the P-chain assets owned by "initialStakedFunds" are evenly
/// distributed over the "initialStakers".
/// The P-chain assets are determined by the "unlockSchedule".
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
struct GenesisFile {
    #[serde(rename = "networkID")]
    network_id: u32,
    #[serde(rename = "allocations", skip_serializing_if = "Option::is_none")]
    allocations: Option<Vec<Allocation>>,
    #[serde(rename = "startTime", skip_serializing_if = "Option::is_none")]
    start_time: Option<u64>,
    #[serde(
        rename = "initialStakeDuration",
        skip_serializing_if = "Option::is_none"
    )]
    initial_stake_duration: Option<u64>,
    #[serde(
        rename = "initialStakeDurationOffset",
        skip_serializing_if = "Option::is_none"
    )]
    initial_stake_duration_offset: Option<u64>,
    /// Initially staked funds are immediately locked for "initial_stake_duration".
    #[serde(rename = "initialStakedFunds", skip_serializing_if = "Option::is_none")]
    initial_staked_funds: Option<Vec<String>>,
    #[serde(rename = "initialStakers", skip_serializing_if = "Option::is_none")]
    initial_stakers: Option<Vec<Staker>>,

    #[serde(rename = "cChainGenesis")]
    c_chain_genesis: String,

    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

pub const DEFAULT_INITIAL_STAKE_DURATION: u64 = 31536000; // 1 year
pub const DEFAULT_INITIAL_STAKE_DURATION_OFFSET: u64 = 5400; // 1.5 hour

impl Default for Genesis {
    fn default() -> Self {
        Self::default()
    }
}

impl Genesis {
    pub fn default() -> Self {
        let now_unix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unexpected None duration_since")
            .as_secs();

        // TODO: 5-hr in the past to unlock "Staking" immediately?
        let start_time = now_unix;

        Self {
            network_id: constants::DEFAULT_CUSTOM_NETWORK_ID,
            allocations: Some(Vec::new()),
            start_time: Some(start_time),
            initial_stake_duration: Some(DEFAULT_INITIAL_STAKE_DURATION),
            initial_stake_duration_offset: Some(DEFAULT_INITIAL_STAKE_DURATION_OFFSET),
            initial_staked_funds: Some(Vec::new()),
            initial_stakers: None,
            c_chain_genesis: coreth_genesis::Genesis::default(),
            message: Some(String::new()),
        }
    }

    /// Creates a new Genesis object with "keys" number of generated pre-funded keys.
    pub fn new<T: key::secp256k1::ReadOnly>(network_id: u32, seed_keys: &[T]) -> io::Result<Self> {
        // maximize total supply
        let max_total_alloc = u64::MAX;
        let total_keys = seed_keys.len();
        let alloc_per_key = if total_keys > 0 {
            max_total_alloc / total_keys as u64
        } else {
            0u64
        };
        // keep 40%, allow more room for transfers
        let xp_alloc_per_key = (alloc_per_key / 10) * 4;
        log::info!("allocate {} for each key in X/P-chain", xp_alloc_per_key);

        // maximize total supply
        let max_total_alloc = i128::MAX;
        let alloc_per_key = if total_keys > 0 {
            max_total_alloc / total_keys as i128
        } else {
            0i128
        };
        // divide by 2, allow more room for transfers
        let c_alloc_per_key = alloc_per_key / 2;
        log::info!("allocate {} for each key in C-chain", c_alloc_per_key);

        // allocation for C-chain
        let mut default_c_alloc = coreth_genesis::AllocAccount::default();
        default_c_alloc.balance = primitive_types::U256::from(c_alloc_per_key);

        // "initial_staked_funds" addresses use all P-chain balance
        // so keep the remaining balance for other keys than "last" key
        let initial_staked_funds = vec![seed_keys[seed_keys.len() - 1]
            .hrp_address(network_id, "X")
            .unwrap()];

        let mut xp_allocs: Vec<Allocation> = Vec::new();
        let mut c_allocs = BTreeMap::new();

        for k in seed_keys.iter() {
            // allocation for X/P-chain
            let mut xp_alloc = Allocation::default();
            xp_alloc.eth_addr = Some(k.eth_address());
            xp_alloc.avax_addr = Some(k.hrp_address(network_id, "X").unwrap());
            xp_alloc.initial_amount = Some(xp_alloc_per_key);
            xp_alloc.unlock_schedule = Some(vec![LockedAmount {
                amount: Some(xp_alloc_per_key),
                ..Default::default()
            }]);
            xp_allocs.push(xp_alloc);

            c_allocs.insert(
                k.eth_address().trim_start_matches("0x").to_string(),
                default_c_alloc.clone(),
            );
        }

        // make sure to use different network ID than "local" network ID
        // ref. https://github.com/ava-labs/avalanche-ops/issues/8
        let mut c_chain_genesis = coreth_genesis::Genesis::default();
        c_chain_genesis.alloc = Some(c_allocs);

        Ok(Self {
            network_id,
            initial_staked_funds: Some(initial_staked_funds),
            allocations: Some(xp_allocs),
            c_chain_genesis,
            ..Default::default()
        })
    }

    /// Saves the current configuration to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing genesis to '{}'", file_path);
        let path = Path::new(file_path);
        if let Some(parent_dir) = path.parent() {
            log::info!("creating parent dir '{}'", parent_dir.display());
            fs::create_dir_all(parent_dir)?;
        }

        let c_chain_genesis = self.c_chain_genesis.encode_json()?;
        let genesis_file = GenesisFile {
            network_id: self.network_id,
            allocations: self.allocations.clone(),
            start_time: self.start_time,
            initial_stake_duration: self.initial_stake_duration,
            initial_stake_duration_offset: self.initial_stake_duration_offset,
            initial_staked_funds: self.initial_staked_funds.clone(),
            initial_stakers: self.initial_stakers.clone(),

            // the avalanchego can only read string-format c-chain genesis
            c_chain_genesis,

            message: self.message.clone(),
        };

        let d = serde_json::to_vec(&genesis_file)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize JSON {}", e)))?;

        let mut f = File::create(file_path)?;
        f.write_all(&d)?;

        Ok(())
    }

    pub fn load(file_path: &str) -> io::Result<Self> {
        log::info!("loading genesis from {}", file_path);

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

        // load as it is
        let genesis_file: GenesisFile = serde_json::from_reader(f).map_err(|e| {
            return Error::new(ErrorKind::InvalidInput, format!("invalid JSON: {}", e));
        })?;

        // make genesis strictly typed
        let c_chain_genesis: coreth_genesis::Genesis =
            serde_json::from_str(&genesis_file.c_chain_genesis).map_err(|e| {
                return Error::new(ErrorKind::InvalidInput, format!("invalid JSON: {}", e));
            })?;

        let genesis = Genesis {
            network_id: genesis_file.network_id,
            allocations: genesis_file.allocations.clone(),
            start_time: genesis_file.start_time,
            initial_stake_duration: genesis_file.initial_stake_duration,
            initial_stake_duration_offset: genesis_file.initial_stake_duration_offset,
            initial_staked_funds: genesis_file.initial_staked_funds.clone(),
            initial_stakers: genesis_file.initial_stakers.clone(),

            // the avalanchego can only read string-format c-chain genesis
            c_chain_genesis,

            message: genesis_file.message,
        };
        Ok(genesis)
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/genesis#Allocation>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Allocation {
    #[serde(rename = "avaxAddr", skip_serializing_if = "Option::is_none")]
    pub avax_addr: Option<String>,
    /// "eth_addr" can be any value, not used in "avalanchego".
    /// This field is only used for memos.
    #[serde(rename = "ethAddr", skip_serializing_if = "Option::is_none")]
    pub eth_addr: Option<String>,
    /// Initially allocated amount for X-chain.
    /// On the X-Chain, one AVAX is 10^9  units.
    /// On the P-Chain, one AVAX is 10^9  units.
    /// On the C-Chain, one AVAX is 10^18 units.
    /// ref. <https://snowtrace.io/unitconverter>
    #[serde(rename = "initialAmount", skip_serializing_if = "Option::is_none")]
    pub initial_amount: Option<u64>,
    #[serde(rename = "unlockSchedule", skip_serializing_if = "Option::is_none")]
    pub unlock_schedule: Option<Vec<LockedAmount>>,
}

/// On the X-Chain, one AVAX is 10^9  units.
/// On the P-Chain, one AVAX is 10^9  units.
/// On the C-Chain, one AVAX is 10^18 units.
/// 300,000,000 AVAX.
/// ref. <https://snowtrace.io/unitconverter>
pub const DEFAULT_INITIAL_AMOUNT_X_CHAIN: u64 = 300000000000000000;

/// On the X-Chain, one AVAX is 10^9  units.
/// On the P-Chain, one AVAX is 10^9  units.
/// On the C-Chain, one AVAX is 10^18 units.
/// 200,000,000 AVAX.
/// ref. <https://snowtrace.io/unitconverter>
pub const DEFAULT_LOCKED_AMOUNT_P_CHAIN: u64 = 200000000000000000;

impl Default for Allocation {
    fn default() -> Self {
        Self::default()
    }
}

impl Allocation {
    pub fn default() -> Self {
        let unlock_empty = LockedAmount::default();
        Self {
            avax_addr: None,
            eth_addr: None,
            initial_amount: Some(DEFAULT_INITIAL_AMOUNT_X_CHAIN),
            unlock_schedule: Some(vec![unlock_empty]),
        }
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/genesis#LockedAmount>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct LockedAmount {
    /// P-chain amount to lock for the duration of "locktime"
    /// in addition to the initial amount.
    /// On the X-Chain, one AVAX is 10^9  units.
    /// On the P-Chain, one AVAX is 10^9  units.
    /// On the C-Chain, one AVAX is 10^18 units.
    /// ref. <https://snowtrace.io/unitconverter>
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    /// Unix timestamp to unlock the "amount".
    #[serde(rename = "locktime", skip_serializing_if = "Option::is_none")]
    pub locktime: Option<u64>,
}

impl Default for LockedAmount {
    fn default() -> Self {
        Self::default()
    }
}

impl LockedAmount {
    pub fn default() -> Self {
        // NOTE: to place lock-time, use this:
        // let now_unix = SystemTime::now()
        //     .duration_since(SystemTime::UNIX_EPOCH)
        //     .expect("unexpected None duration_since")
        //     .as_secs();
        // unlock_now.locktime = Some(now_unix);

        Self {
            amount: Some(DEFAULT_LOCKED_AMOUNT_P_CHAIN),
            locktime: None, // empty to unlock immediately
        }
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/genesis#Staker>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Staker {
    #[serde(rename = "nodeID", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(rename = "rewardAddress", skip_serializing_if = "Option::is_none")]
    pub reward_address: Option<String>,
    #[serde(rename = "delegationFee", skip_serializing_if = "Option::is_none")]
    pub delegation_fee: Option<u32>,
}

pub const DEFAULT_DELEGATION_FEE: u32 = 62500;

impl Default for Staker {
    fn default() -> Self {
        Self::default()
    }
}

impl Staker {
    pub fn default() -> Self {
        Self {
            node_id: None,
            reward_address: None,
            delegation_fee: Some(DEFAULT_DELEGATION_FEE),
        }
    }
}

#[test]
fn test_genesis() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    use rust_embed::RustEmbed;
    #[derive(RustEmbed)]
    #[folder = "artifacts/"]
    #[prefix = "artifacts/"]
    struct Asset;
    let genesis_json = Asset::get("artifacts/sample.genesis.json").unwrap();
    let genesis_json_contents = std::str::from_utf8(genesis_json.data.as_ref()).unwrap();
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(genesis_json_contents.as_bytes()).unwrap();
    let genesis_file_path = f.path().to_str().unwrap();
    let original_genesis = Genesis::load(genesis_file_path).unwrap();

    let mut alloc = BTreeMap::new();
    alloc.insert(
        String::from("8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC"),
        coreth_genesis::AllocAccount {
            code: None,
            storage: None,
            balance: primitive_types::U256::from_str_radix("0x295BE96E64066972000000", 16).unwrap(),
            mcbalance: None,
            nonce: None,
        },
    );
    let genesis = Genesis {
        network_id: 1337,

        allocations: Some(vec![
            Allocation {
                eth_addr: Some(String::from("0xb3d82b1367d362de99ab59a658165aff520cbd4d")),
                avax_addr: Some(String::from(
                    "X-custom1g65uqn6t77p656w64023nh8nd9updzmxwd59gh",
                )),
                initial_amount: Some(0),
                unlock_schedule: Some(vec![LockedAmount {
                    amount: Some(10000000000000000),
                    locktime: Some(1633824000),
                }]),
            },
            Allocation {
                eth_addr: Some(String::from("0xb3d82b1367d362de99ab59a658165aff520cbd4d")),
                avax_addr: Some(String::from(
                    "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p",
                )),
                initial_amount: Some(300000000000000000),
                unlock_schedule: Some(vec![
                    LockedAmount {
                        amount: Some(20000000000000000),
                        locktime: None,
                    },
                    LockedAmount {
                        amount: Some(10000000000000000),
                        locktime: Some(1633824000),
                    },
                ]),
            },
            Allocation {
                eth_addr: Some(String::from("0xb3d82b1367d362de99ab59a658165aff520cbd4d")),
                avax_addr: Some(String::from(
                    "X-custom16045mxr3s2cjycqe2xfluk304xv3ezhkhsvkpr",
                )),
                initial_amount: Some(10000000000000000),
                unlock_schedule: Some(vec![LockedAmount {
                    amount: Some(10000000000000000),
                    locktime: Some(1633824000),
                }]),
            },
        ]),

        start_time: Some(1630987200),
        initial_stake_duration: Some(31536000),
        initial_stake_duration_offset: Some(5400),
        initial_staked_funds: Some(vec![String::from(
            "X-custom1g65uqn6t77p656w64023nh8nd9updzmxwd59gh",
        )]),
        initial_stakers: Some(vec![
            Staker {
                node_id: Some(String::from("NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg")),
                reward_address: Some(String::from(
                    "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p",
                )),
                delegation_fee: Some(1000000),
            },
            Staker {
                node_id: Some(String::from("NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ")),
                reward_address: Some(String::from(
                    "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p",
                )),
                delegation_fee: Some(500000),
            },
            Staker {
                node_id: Some(String::from("NodeID-NFBbbJ4qCmNaCzeW7sxErhvWqvEQMnYcN")),
                reward_address: Some(String::from(
                    "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p",
                )),
                delegation_fee: Some(250000),
            },
            Staker {
                node_id: Some(String::from("NodeID-GWPcbFJZFfZreETSoWjPimr846mXEKCtu")),
                reward_address: Some(String::from(
                    "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p",
                )),
                delegation_fee: Some(125000),
            },
            Staker {
                node_id: Some(String::from("NodeID-P7oB2McjBGgW2NXXWVYjV8JEDFoW9xDE5")),
                reward_address: Some(String::from(
                    "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p",
                )),
                delegation_fee: Some(62500),
            },
        ]),

        c_chain_genesis: coreth_genesis::Genesis {
            config: Some(coreth_genesis::ChainConfig {
                chain_id: Some(43112),

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
            }),
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
        },

        message: Some(String::from("{{ fun_quote }}")),
    };
    assert_eq!(original_genesis, genesis);

    let p = random_manager::tmp_path(10, Some(".json")).unwrap();
    genesis.sync(&p).unwrap();
    let genesis_loaded = Genesis::load(&p).unwrap();
    assert_eq!(genesis_loaded, genesis);
    assert_eq!(genesis_loaded, original_genesis);

    let d = fs::read_to_string(&p).unwrap();
    log::info!("{}", d);
}
