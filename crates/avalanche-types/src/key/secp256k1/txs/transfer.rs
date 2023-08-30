use std::{
    cmp::Ordering,
    io::{self, Error, ErrorKind},
};

use crate::{codec, key};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOutput>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOut>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput>
/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgettx>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Output {
    pub amount: u64,

    /// The custom de/serializer embeds "output_owners" at the same level as "amount" as in avalanchego.
    #[serde(flatten)]
    pub output_owners: key::secp256k1::txs::OutputOwners,
}

impl Default for Output {
    fn default() -> Self {
        Self::default()
    }
}

impl Output {
    pub fn default() -> Self {
        Self {
            amount: 0,
            output_owners: key::secp256k1::txs::OutputOwners::default(),
        }
    }

    pub fn new(amount: u64, output_owners: key::secp256k1::txs::OutputOwners) -> Self {
        Self {
            amount,
            output_owners,
        }
    }

    pub fn type_name() -> String {
        "secp256k1fx.TransferOutput".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::X_TYPES.get(&Self::type_name()).unwrap()) as u32
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::txs::transfer::test_transfer_output_custom_de_serializer --exact --show-output
#[test]
fn test_transfer_output_custom_de_serializer() {
    use crate::ids::short;

    let d = Output {
        amount: 1234,
        output_owners: key::secp256k1::txs::OutputOwners {
            locktime: 1,
            threshold: 2,
            addresses: vec![short::Id::empty()],
        },
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    println!("yaml_encoded:\n{}", yaml_encoded);
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{}", json_encoded);
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);
}

impl Ord for Output {
    fn cmp(&self, other: &Output) -> Ordering {
        self.amount
            .cmp(&(other.amount)) // returns when "amount"s are not Equal
            .then_with(
                || self.output_owners.cmp(&(other.output_owners)), // if "amount"s are Equal, compare "output_owners"
            )
    }
}

impl PartialOrd for Output {
    fn partial_cmp(&self, other: &Output) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Output {
    fn eq(&self, other: &Output) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::txs::transfer::test_sort_transfer_outputs --exact --show-output
#[test]
fn test_sort_transfer_outputs() {
    use crate::ids::short;

    let mut outputs: Vec<Output> = Vec::new();
    for i in (0..10).rev() {
        outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: (i + 1) as u32,
                addresses: vec![
                    short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                    short::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5]),
                ],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: (i + 1) as u32,
                addresses: vec![
                    short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                    short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                ],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: (i + 1) as u32,
                addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: i as u32,
                addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: i as u64,
                threshold: i as u32,
                addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&outputs));
    outputs.sort();

    let mut sorted_outputs: Vec<Output> = Vec::new();
    for i in 0..10 {
        sorted_outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: i as u64,
                threshold: i as u32,
                addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        sorted_outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: i as u32,
                addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        sorted_outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: (i + 1) as u32,
                addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        sorted_outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: (i + 1) as u32,
                addresses: vec![
                    short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                    short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                ],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
        sorted_outputs.push(Output {
            amount: i as u64,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: (i + 1) as u64,
                threshold: (i + 1) as u32,
                addresses: vec![
                    short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                    short::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5]),
                ],
                ..key::secp256k1::txs::OutputOwners::default()
            },
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_outputs));
    assert_eq!(outputs, sorted_outputs);
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableInput>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableIn>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Input>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Input {
    pub amount: u64,
    #[serde(rename = "signatureIndices")]
    pub sig_indices: Vec<u32>,
}

impl Default for Input {
    fn default() -> Self {
        Self::default()
    }
}

impl Input {
    pub fn default() -> Self {
        Self {
            amount: 0,
            sig_indices: Vec::new(),
        }
    }

    pub fn new(amount: u64, sig_indices: Vec<u32>) -> Self {
        Self {
            amount,
            sig_indices,
        }
    }

    pub fn type_name() -> String {
        "secp256k1fx.TransferInput".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::X_TYPES.get(&Self::type_name()).unwrap()) as u32
    }

    pub fn verify(&self) -> io::Result<()> {
        if self.amount == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "input has no value", // ref. "errNoValueInput"
            ));
        }
        if !cmp_manager::is_sorted_and_unique(&self.sig_indices) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "signatures not sorted and unique", // ref. "errNotSortedUnique"
            ));
        }
        Ok(())
    }

    /// ref. "vms/secp256k1fx.Input.Cost"
    pub fn sig_costs(&self) -> u64 {
        let sigs = self.sig_indices.len();
        (sigs as u64) * 1000
    }
}

impl Ord for Input {
    fn cmp(&self, other: &Input) -> Ordering {
        self.amount
            .cmp(&(other.amount)) // returns when "amount"s are not Equal
            .then_with(
                || {
                    key::secp256k1::txs::SigIndices::new(&self.sig_indices)
                        .cmp(&key::secp256k1::txs::SigIndices::new(&other.sig_indices))
                }, // if "amount"s are Equal, compare "sig_indices"
            )
    }
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Input) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Input {
    fn eq(&self, other: &Input) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::txs::transfer::test_sort_transfer_inputs --exact --show-output
#[test]
fn test_sort_transfer_inputs() {
    let mut inputs: Vec<Input> = Vec::new();
    for i in (0..10).rev() {
        inputs.push(Input {
            amount: 5,
            sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        inputs.push(Input {
            amount: 5,
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        inputs.push(Input {
            amount: 50,
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5],
        });
        inputs.push(Input {
            amount: 5,
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5],
        });
        inputs.push(Input {
            amount: 1,
            sig_indices: vec![(i + 100) as u32, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&inputs));
    inputs.sort();

    let mut sorted_inputs: Vec<Input> = Vec::new();
    for i in 0..10 {
        sorted_inputs.push(Input {
            amount: 1,
            sig_indices: vec![(i + 100) as u32, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
        });
    }
    for i in 0..10 {
        sorted_inputs.push(Input {
            amount: 5,
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5],
        });
    }
    for i in 0..10 {
        sorted_inputs.push(Input {
            amount: 5,
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        sorted_inputs.push(Input {
            amount: 5,
            sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9],
        });
    }
    for i in 0..10 {
        sorted_inputs.push(Input {
            amount: 50,
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5],
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_inputs));
    assert_eq!(inputs, sorted_inputs);
}
