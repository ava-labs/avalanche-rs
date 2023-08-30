use std::cmp::Ordering;

use crate::{ids, key, platformvm, txs};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOutput>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOut>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Output {
    #[serde(rename = "assetID")]
    pub asset_id: ids::Id,

    /// Packer skips serialization due to serialize:"false" in avalanchego.
    #[serde(rename = "fxID", skip_serializing_if = "Option::is_none")]
    pub fx_id: Option<ids::Id>,

    /// The underlying type is one of the following:
    ///
    /// "*secp256k1fx.TransferOutput"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput>
    ///
    /// "*platformvm.StakeableLockOut" which embeds "*secp256k1fx.TransferOutput"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut>
    ///
    /// MUST: only one of the following can be "Some".
    #[serde(rename = "output")]
    pub transfer_output: Option<key::secp256k1::txs::transfer::Output>,
    pub stakeable_lock_out: Option<platformvm::txs::StakeableLockOut>,
}

impl Default for Output {
    fn default() -> Self {
        Self::default()
    }
}

impl Output {
    pub fn default() -> Self {
        Self {
            asset_id: ids::Id::empty(),
            fx_id: None,
            transfer_output: None,
            stakeable_lock_out: None,
        }
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortTransferableOutputs>
impl Ord for Output {
    fn cmp(&self, other: &Output) -> Ordering {
        let asset_id_ord = self.asset_id.cmp(&(other.asset_id));
        if asset_id_ord != Ordering::Equal {
            // no need to compare further
            return asset_id_ord;
        }
        if self.transfer_output.is_none()
            && self.stakeable_lock_out.is_none()
            && other.transfer_output.is_none()
            && other.stakeable_lock_out.is_none()
        {
            // should never happen but sorting/ordering shouldn't worry about this...
            // can't compare anymore, so thus return here...
            return Ordering::Equal;
        }

        // unlike "avalanchego", we want ordering without "codec" dependency and marshal-ing
        // just check type ID header
        let type_id_self = {
            if self.transfer_output.is_some() {
                key::secp256k1::txs::transfer::Output::type_id()
            } else {
                platformvm::txs::StakeableLockOut::type_id()
            }
        };
        let type_id_other = {
            if other.transfer_output.is_some() {
                key::secp256k1::txs::transfer::Output::type_id()
            } else {
                platformvm::txs::StakeableLockOut::type_id()
            }
        };
        let type_id_ord = type_id_self.cmp(&type_id_other);
        if type_id_ord != Ordering::Equal {
            // no need to compare further
            return type_id_ord;
        }

        // both instances have the same type!!!
        // just use the ordering of underlying types
        match type_id_self {
            7 => {
                // "key::secp256k1::txs::transfer::Output"
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
                let transfer_output_self = self.transfer_output.clone().unwrap();
                let transfer_output_other = other.transfer_output.clone().unwrap();
                transfer_output_self.cmp(&transfer_output_other)
            }
            22 => {
                // "platformvm::txs::StakeableLockOut"
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut
                let stakeable_lock_out_self = self.stakeable_lock_out.clone().unwrap();
                let stakeable_lock_out_other = other.stakeable_lock_out.clone().unwrap();
                stakeable_lock_out_self.cmp(&stakeable_lock_out_other)
            }
            _ => {
                // should never happen
                panic!("unexpected type ID {} for TransferableOutput", type_id_self);
            }
        }
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

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortTransferableOutputs>
/// ref. "avalanchego/vms/components/avax.TestTransferableOutputSorting"
/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::transferable::test_sort_transferable_outputs --exact --show-output
#[test]
fn test_sort_transferable_outputs() {
    use crate::ids::short;

    let mut outputs: Vec<Output> = Vec::new();
    for i in (0..10).rev() {
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5]),
                        ],
                        ..key::secp256k1::txs::OutputOwners::default()
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                        ],
                        ..key::secp256k1::txs::OutputOwners::default()
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            transfer_output: Some(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            transfer_output: Some(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                },
            }),
            ..Output::default()
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&outputs));
    outputs.sort();

    let mut sorted_outputs: Vec<Output> = Vec::new();
    for i in 0..10 {
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            transfer_output: Some(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            transfer_output: Some(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5])],
                    ..key::secp256k1::txs::OutputOwners::default()
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
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
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                        ],
                        ..key::secp256k1::txs::OutputOwners::default()
                    },
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&vec![i as u8, 2, 2, 3, 4, 5]),
                        ],
                        ..key::secp256k1::txs::OutputOwners::default()
                    },
                },
            }),
            ..Output::default()
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&outputs));
    assert_eq!(outputs, sorted_outputs);
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableInput>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableIn>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Input {
    #[serde(flatten)]
    pub utxo_id: txs::utxo::Id,

    #[serde(rename = "assetID")]
    pub asset_id: ids::Id,

    #[serde(rename = "fxID")]
    pub fx_id: ids::Id, // skip serialization due to serialize:"false"

    /// The underlying type is one of the following:
    ///
    /// "*secp256k1fx.TransferInput"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput>
    ///
    /// "*platformvm.StakeableLockIn" which embeds "*secp256k1fx.TransferInput"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockIn>
    ///
    /// MUST: only one of the following can be "Some".
    #[serde(rename = "input")]
    pub transfer_input: Option<key::secp256k1::txs::transfer::Input>,
    pub stakeable_lock_in: Option<platformvm::txs::StakeableLockIn>,
}

impl Default for Input {
    fn default() -> Self {
        Self::default()
    }
}

impl Input {
    pub fn default() -> Self {
        Self {
            utxo_id: txs::utxo::Id::default(),
            asset_id: ids::Id::empty(),
            fx_id: ids::Id::empty(),
            transfer_input: None,
            stakeable_lock_in: None,
        }
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortTransferableInputs>
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortTransferableInputsWithSigners>
impl Ord for Input {
    fn cmp(&self, other: &Input) -> Ordering {
        self.utxo_id
            .tx_id
            .cmp(&(other.utxo_id.tx_id)) // returns when "utxo_id.tx_id"s are not Equal
            .then_with(
                || self.utxo_id.output_index.cmp(&other.utxo_id.output_index), // if "utxo_id.tx_id"s are Equal, compare "output_index"
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

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortTransferableInputs>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortTransferableInputsWithSigners>
/// ref. "avalanchego/vms/components/avax.TestTransferableInputSorting"
/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::transferable::test_sort_transferable_inputs --exact --show-output
#[test]
fn test_sort_transferable_inputs() {
    let mut inputs: Vec<Input> = Vec::new();
    for i in (0..10).rev() {
        inputs.push(Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: (i + 1) as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
        inputs.push(Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: i as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&inputs));
    inputs.sort();

    let mut sorted_inputs: Vec<Input> = Vec::new();
    for i in 0..10 {
        sorted_inputs.push(Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: i as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
        sorted_inputs.push(Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: (i + 1) as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_inputs));
    assert_eq!(inputs, sorted_inputs);
}
