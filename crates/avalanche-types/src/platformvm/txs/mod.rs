pub mod add_permissionless_validator;
pub mod add_subnet_validator;
pub mod add_validator;
pub mod create_chain;
pub mod create_subnet;
pub mod export;
pub mod import;
pub mod status;

use std::cmp::Ordering;

use crate::{
    codec::{self, serde::hex_0x_bytes::Hex0xBytes},
    ids::{self, node},
    key,
    txs::transferable,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgettx>
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Tx {
    #[serde(rename = "unsignedTx")]
    pub unsigned_tx: UnsignedTx,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Vec<key::secp256k1::txs::Credential>>,
}

impl Default for Tx {
    fn default() -> Self {
        Self::default()
    }
}

impl Tx {
    pub fn default() -> Self {
        Self {
            unsigned_tx: UnsignedTx::default(),
            credentials: None,
        }
    }
}

/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgettx>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UnsignedTx {
    #[serde(rename = "networkID")]
    pub network_id: u32,
    #[serde(rename = "blockchainID")]
    pub blockchain_id: ids::Id,

    #[serde(rename = "outputs")]
    pub transferable_outputs: Option<Vec<transferable::Output>>,
    #[serde(rename = "inputs")]
    pub transferable_inputs: Option<Vec<transferable::Input>>,

    #[serde(rename = "owner")]
    pub output_owners: key::secp256k1::txs::OutputOwners,

    #[serde_as(as = "Option<Hex0xBytes>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<Vec<u8>>,
}

impl Default for UnsignedTx {
    fn default() -> Self {
        Self::default()
    }
}

impl UnsignedTx {
    pub fn default() -> Self {
        Self {
            network_id: 0,
            blockchain_id: ids::Id::empty(),
            transferable_outputs: None,
            transferable_inputs: None,
            output_owners: key::secp256k1::txs::OutputOwners::default(),
            memo: None,
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::test_json_deserialize --exact --show-output
#[test]
fn test_json_deserialize() {
    let parsed_tx: Tx = serde_json::from_str(
        "
    
    {
        \"unsignedTx\": {
            \"networkID\": 1000000,
            \"blockchainID\": \"11111111111111111111111111111111LpoYY\",
            \"outputs\": [
                {
                    \"assetID\": \"u8aaQ7MxyW32iHuP2xMXgYPrWYAsSbh8RJV9C6p1UeuGvqR3\",
                    \"fxID\": \"spdxUxVJQbX85MGxMHbKw1sHxMnSqJ3QBzDyDYEP3h6TLuxqQ\",
                    \"output\": {
                        \"addresses\": [
                            \"P-custom12szthht8tnl455u4mz3ns3nvvkel8ezvw2n8cx\"
                        ],
                        \"amount\": 245952587549460688,
                        \"locktime\": 0,
                        \"threshold\": 1
                    }
                }
            ],
            \"inputs\": [
                {
                    \"txID\": \"nN5QsURgEpM8D3e9q8FonS4EE13mnaBDtnQmgSwwUfBZ6FSW1\",
                    \"outputIndex\": 0,
                    \"assetID\": \"u8aaQ7MxyW32iHuP2xMXgYPrWYAsSbh8RJV9C6p1UeuGvqR3\",
                    \"fxID\": \"spdxUxVJQbX85MGxMHbKw1sHxMnSqJ3QBzDyDYEP3h6TLuxqQ\",
                    \"input\": {
                        \"amount\": 245952587649460688,
                        \"signatureIndices\": [
                            0
                        ]
                    }
                }
            ],
            \"memo\": \"0x\",
            \"owner\": {
                \"addresses\": [
                    \"P-custom12szthht8tnl455u4mz3ns3nvvkel8ezvw2n8cx\"
                ],
                \"locktime\": 0,
                \"threshold\": 1
            }
        },
        \"credentials\": [
            {
                \"signatures\": [
                    \"0xcb356822dc8990672b5777ec50b57da91baf572240e7d4e9e38f26ec9dbdfd8e376fdc5f30769b842668cd8d81bd71db926dfbe326585137d363566ee500369f01\"
                ]
            }
        ]
    }
    
    ",
    )
    .unwrap();

    println!("{:?}", parsed_tx);

    assert_eq!(parsed_tx.unsigned_tx.network_id, 1000000);
    assert_eq!(
        parsed_tx.unsigned_tx.transferable_outputs.clone().unwrap()[0]
            .transfer_output
            .clone()
            .unwrap()
            .amount,
        245952587549460688
    );
    assert_eq!(
        parsed_tx.unsigned_tx.transferable_outputs.clone().unwrap()[0]
            .transfer_output
            .clone()
            .unwrap()
            .output_owners
            .threshold,
        1
    );
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockIn>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct StakeableLockIn {
    pub locktime: u64,
    pub transfer_input: key::secp256k1::txs::transfer::Input,
}

impl Default for StakeableLockIn {
    fn default() -> Self {
        Self::default()
    }
}

impl StakeableLockIn {
    pub fn default() -> Self {
        Self {
            locktime: 0,
            transfer_input: key::secp256k1::txs::transfer::Input::default(),
        }
    }

    pub fn type_name() -> String {
        "platformvm.StakeableLockIn".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::P_TYPES.get(&Self::type_name()).unwrap()) as u32
    }
}

impl Ord for StakeableLockIn {
    fn cmp(&self, other: &StakeableLockIn) -> Ordering {
        self.locktime
            .cmp(&(other.locktime)) // returns when "locktime"s are not Equal
            .then_with(
                || self.transfer_input.cmp(&other.transfer_input), // if "locktime"s are Equal, compare "transfer_input"
            )
    }
}

impl PartialOrd for StakeableLockIn {
    fn partial_cmp(&self, other: &StakeableLockIn) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for StakeableLockIn {
    fn eq(&self, other: &StakeableLockIn) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::test_sort_stakeable_lock_ins --exact --show-output
#[test]
fn test_sort_stakeable_lock_ins() {
    let mut ins: Vec<StakeableLockIn> = Vec::new();
    for i in (0..10).rev() {
        ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 10,
                sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
        });
        ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 5,
                sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9, 9],
            },
        });
        ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 5,
                sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9],
            },
        });
        ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 5,
                sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&ins));
    ins.sort();

    let mut sorted_ins: Vec<StakeableLockIn> = Vec::new();
    for i in 0..10 {
        sorted_ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 5,
                sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
        });
        sorted_ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 5,
                sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9],
            },
        });
        sorted_ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 5,
                sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9, 9],
            },
        });
        sorted_ins.push(StakeableLockIn {
            locktime: i as u64,
            transfer_input: key::secp256k1::txs::transfer::Input {
                amount: 10,
                sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_ins));
    assert_eq!(ins, sorted_ins);
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct StakeableLockOut {
    pub locktime: u64,
    pub transfer_output: key::secp256k1::txs::transfer::Output,
}

impl Default for StakeableLockOut {
    fn default() -> Self {
        Self::default()
    }
}

impl StakeableLockOut {
    pub fn default() -> Self {
        Self {
            locktime: 0,
            transfer_output: key::secp256k1::txs::transfer::Output::default(),
        }
    }

    pub fn type_name() -> String {
        "platformvm.StakeableLockOut".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::P_TYPES.get(&Self::type_name()).unwrap()) as u32
    }
}

impl Ord for StakeableLockOut {
    fn cmp(&self, other: &StakeableLockOut) -> Ordering {
        self.locktime
            .cmp(&(other.locktime)) // returns when "locktime"s are not Equal
            .then_with(
                || self.transfer_output.cmp(&other.transfer_output), // if "locktime"s are Equal, compare "transfer_output"
            )
    }
}

impl PartialOrd for StakeableLockOut {
    fn partial_cmp(&self, other: &StakeableLockOut) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for StakeableLockOut {
    fn eq(&self, other: &StakeableLockOut) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::test_sort_stakeable_lock_outs --exact --show-output
#[test]
fn test_sort_stakeable_lock_outs() {
    use crate::ids::short;
    let mut outs: Vec<StakeableLockOut> = Vec::new();
    for i in (0..10).rev() {
        outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: (i + 1) as u32,
                    addresses: vec![
                        short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                        short::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5]),
                    ],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: (i + 1) as u32,
                    addresses: vec![
                        short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                        short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                    ],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: (i + 1) as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&outs));
    outs.sort();

    let mut sorted_outs: Vec<StakeableLockOut> = Vec::new();
    for i in 0..10 {
        sorted_outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        sorted_outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        sorted_outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        sorted_outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: (i + 1) as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        sorted_outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: (i + 1) as u32,
                    addresses: vec![
                        short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                        short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                    ],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
        sorted_outs.push(StakeableLockOut {
            locktime: i as u64,
            transfer_output: key::secp256k1::txs::transfer::Output {
                amount: (i + 1) as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: (i + 1) as u64,
                    threshold: (i + 1) as u32,
                    addresses: vec![
                        short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                        short::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5]),
                    ],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            },
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_outs));
    assert_eq!(outs, sorted_outs);
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#Validator>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#AddValidatorArgs>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/api#Staker>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Validator {
    pub node_id: node::Id,
    pub start: u64,
    pub end: u64,
    pub weight: u64,
}

impl Default for Validator {
    fn default() -> Self {
        Self::default()
    }
}

impl Validator {
    pub fn default() -> Self {
        Self {
            node_id: node::Id::empty(),
            start: 0,
            end: 0,
            weight: 0,
        }
    }
}
