use std::cmp::Ordering;

use crate::{
    errors::Result,
    ids, key,
    packer::{Packable, Packer},
    platformvm, txs,
};
use serde::{Deserialize, Serialize};

/// Implementation of "*components.avax.TransferOut"
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOut>
/// which is either:
///
/// "*secp256k1fx.TransferOutput"
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput>
///
/// "*platformvm.StakeableLockOut" which embeds "*secp256k1fx.TransferOutput"
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut>
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, PartialOrd)]
#[serde(untagged)]
pub enum TransferableOut {
    TransferOutput(key::secp256k1::txs::transfer::Output),
    StakeableLockOut(platformvm::txs::StakeableLockOut),
}

impl TransferableOut {
    pub fn type_id(&self) -> u32 {
        match self {
            TransferableOut::TransferOutput(_out) => {
                key::secp256k1::txs::transfer::Output::type_id()
            }
            TransferableOut::StakeableLockOut(_out) => platformvm::txs::StakeableLockOut::type_id(),
        }
    }
}

impl Packable for TransferableOut {
    fn pack(&self, packer: &Packer) -> Result<()> {
        match self {
            TransferableOut::TransferOutput(transfer_output) => {
                packer.pack(transfer_output)?;
            }
            TransferableOut::StakeableLockOut(stakeable_lock_out) => {
                // marshal type ID "platformvm::txs::StakeableLockOut"
                packer.pack_u32(platformvm::txs::StakeableLockOut::type_id())?;

                // "platformvm::txs::StakeableLockOut"
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut

                // marshal "platformvm::txs::StakeableLockOut.locktime" field
                packer.pack_u64(stakeable_lock_out.locktime)?;
                packer.pack(&stakeable_lock_out.transfer_output)?;
            }
        }
        Ok(())
    }
}

impl Ord for TransferableOut {
    fn cmp(&self, other: &TransferableOut) -> Ordering {
        // unlike "avalanchego", we want ordering without "codec" dependency and marshal-ing
        // just check type ID header
        let type_id_ord = self.type_id().cmp(&other.type_id());
        if type_id_ord != Ordering::Equal {
            // no need to compare further
            return type_id_ord;
        }

        match (self, other) {
            (
                TransferableOut::TransferOutput(out_self),
                TransferableOut::TransferOutput(out_other),
            ) => out_self.cmp(out_other),
            (
                TransferableOut::StakeableLockOut(out_self),
                TransferableOut::StakeableLockOut(out_other),
            ) => out_self.cmp(out_other),
            (_, _) => self.type_id().cmp(&other.type_id()),
        }
    }
}

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

    #[serde(rename = "output")]
    pub out: TransferableOut,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            asset_id: ids::Id::empty(),
            fx_id: None,
            out: TransferableOut::TransferOutput(Default::default()),
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

        self.out.cmp(&other.out)
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
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5]),
                        ],
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5]),
                        ],
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: i as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: i as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
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
            asset_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: i as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
                amount: i as u64,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: i as u64,
                    threshold: i as u32,
                    addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: i as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5])],
                    },
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5]),
                        ],
                    },
                },
            }),
            ..Output::default()
        });
        sorted_outputs.push(Output {
            asset_id: ids::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5, 6, 7, 8, 9]),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut {
                locktime: i as u64,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: (i + 1) as u64,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: i as u64,
                        threshold: i as u32,
                        addresses: vec![
                            short::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5]),
                            short::Id::from_slice(&[i as u8, 2, 2, 3, 4, 5]),
                        ],
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
                tx_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: (i + 1) as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
        inputs.push(Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
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
                tx_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: i as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
        sorted_inputs.push(Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&[i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
                output_index: (i + 1) as u32,
                ..txs::utxo::Id::default()
            },
            ..Input::default()
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_inputs));
    assert_eq!(inputs, sorted_inputs);
}
