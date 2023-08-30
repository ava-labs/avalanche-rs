//! Definitions of Avalanche transaction types.
pub mod raw;
pub mod transferable;
pub mod utxo;

use super::{
    codec::{self, serde::hex_0x_bytes::Hex0xBytes},
    errors::{Error, Result},
    hash, ids, key, packer, platformvm,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#BaseTx>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Tx {
    #[serde(skip)]
    pub metadata: Option<Metadata>, // skip serialization due to serialize:"false"

    #[serde(rename = "networkID")]
    pub network_id: u32,
    #[serde(rename = "blockchainID")]
    pub blockchain_id: ids::Id,

    #[serde(rename = "inputs")]
    pub transferable_inputs: Option<Vec<transferable::Input>>,
    #[serde(rename = "outputs")]
    pub transferable_outputs: Option<Vec<transferable::Output>>,

    #[serde_as(as = "Option<Hex0xBytes>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<Vec<u8>>,
}

impl Default for Tx {
    fn default() -> Self {
        Self::default()
    }
}

impl Tx {
    pub fn default() -> Self {
        Self {
            metadata: None,
            network_id: 0,
            blockchain_id: ids::Id::empty(),
            transferable_inputs: None,
            transferable_outputs: None,
            memo: None,
        }
    }

    pub fn type_name() -> String {
        "avm.BaseTx".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::X_TYPES.get(&Self::type_name()).unwrap()) as u32
    }

    /// "Tx.Unsigned" is implemented by "avax.BaseTx"
    /// but for marshal, it's passed as an interface.
    /// Then marshaled via "avalanchego/codec/linearcodec.linearCodec"
    /// which then calls "genericCodec.marshal".
    /// ref. "avalanchego/vms/avm.Tx.SignSECP256K1Fx"
    /// ref. "avalanchego/codec.manager.Marshal"
    /// ref. "avalanchego/codec.manager.Marshal(codecVersion, &t.UnsignedTx)"
    /// ref. "avalanchego/codec/linearcodec.linearCodec.MarshalInto"
    /// ref. "avalanchego/codec/reflectcodec.genericCodec.MarshalInto"
    /// ref. "avalanchego/codec/reflectcodec.genericCodec.marshal"
    ///
    /// Returns the packer itself so that the following marshals can reuse.
    ///
    /// "BaseTx" is an interface in Go (reflect.Interface)
    /// thus the underlying type must be specified by the caller
    /// TODO: can we do better in Rust? Go does so with reflect...
    /// e.g., pack prefix with the type ID for "avm.BaseTx" (linearCodec.PackPrefix)
    /// ref. "avalanchego/codec/linearcodec.linearCodec.MarshalInto"
    /// ref. "avalanchego/codec/reflectcodec.genericCodec.MarshalInto"
    pub fn pack(&self, codec_version: u16, type_id: u32) -> Result<packer::Packer> {
        // ref. "avalanchego/codec.manager.Marshal", "vms/avm.newCustomCodecs"
        // ref. "math.MaxInt32" and "constants.DefaultByteSliceCap" in Go
        let packer = packer::Packer::new((1 << 31) - 1, 128);

        // codec version
        // ref. "avalanchego/codec.manager.Marshal"
        packer.pack_u16(codec_version)?;
        packer.pack_u32(type_id)?;

        // marshal the actual struct "avm.BaseTx"
        // "BaseTx.Metadata" is not serialize:"true" thus skipping serialization!!!
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#BaseTx
        // ref. "avalanchego/codec/reflectcodec.structFielder"
        packer.pack_u32(self.network_id)?;
        packer.pack_bytes(self.blockchain_id.as_ref())?;

        // "transferable_outputs" field; pack the number of slice elements
        if self.transferable_outputs.is_some() {
            let transferable_outputs = self.transferable_outputs.as_ref().unwrap();
            packer.pack_u32(transferable_outputs.len() as u32)?;

            for transferable_output in transferable_outputs.iter() {
                // "TransferableOutput.Asset" is struct and serialize:"true"
                // but embedded inline in the struct "TransferableOutput"
                // so no need to encode type ID
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOutput
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#Asset
                packer.pack_bytes(transferable_output.asset_id.as_ref())?;

                // fx_id is serialize:"false" thus skipping serialization

                // decide the type
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableOutput
                if transferable_output.transfer_output.is_none()
                    && transferable_output.stakeable_lock_out.is_none()
                {
                    return Err(Error::Other {
                        message: "unexpected Nones in TransferableOutput transfer_output and stakeable_lock_out".to_string(),
                        retryable: false,
                    });
                }
                let type_id_transferable_out = {
                    if transferable_output.transfer_output.is_some() {
                        key::secp256k1::txs::transfer::Output::type_id()
                    } else {
                        platformvm::txs::StakeableLockOut::type_id()
                    }
                };
                // marshal type ID for "key::secp256k1::txs::transfer::Output" or "platformvm::txs::StakeableLockOut"
                packer.pack_u32(type_id_transferable_out)?;

                match type_id_transferable_out {
                    7 => {
                        // "key::secp256k1::txs::transfer::Output"
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
                        let transfer_output = transferable_output.transfer_output.clone().unwrap();

                        // marshal "secp256k1fx.TransferOutput.Amt" field
                        packer.pack_u64(transfer_output.amount)?;

                        // "secp256k1fx.TransferOutput.OutputOwners" is struct and serialize:"true"
                        // but embedded inline in the struct "TransferOutput"
                        // so no need to encode type ID
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#OutputOwners
                        packer.pack_u64(transfer_output.output_owners.locktime)?;
                        packer.pack_u32(transfer_output.output_owners.threshold)?;
                        packer.pack_u32(transfer_output.output_owners.addresses.len() as u32)?;
                        for addr in transfer_output.output_owners.addresses.iter() {
                            packer.pack_bytes(addr.as_ref())?;
                        }
                    }
                    22 => {
                        // "platformvm::txs::StakeableLockOut"
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut
                        let stakeable_lock_out =
                            transferable_output.stakeable_lock_out.clone().unwrap();

                        // marshal "platformvm::txs::StakeableLockOut.locktime" field
                        packer.pack_u64(stakeable_lock_out.locktime)?;

                        // secp256k1fx.TransferOutput type ID
                        packer.pack_u32(7)?;

                        // "platformvm.StakeableLockOut.TransferOutput" is struct and serialize:"true"
                        // but embedded inline in the struct "StakeableLockOut"
                        // so no need to encode type ID
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#OutputOwners
                        //
                        // marshal "secp256k1fx.TransferOutput.Amt" field
                        packer.pack_u64(stakeable_lock_out.transfer_output.amount)?;
                        packer
                            .pack_u64(stakeable_lock_out.transfer_output.output_owners.locktime)?;
                        packer
                            .pack_u32(stakeable_lock_out.transfer_output.output_owners.threshold)?;
                        packer.pack_u32(
                            stakeable_lock_out
                                .transfer_output
                                .output_owners
                                .addresses
                                .len() as u32,
                        )?;
                        for addr in stakeable_lock_out
                            .transfer_output
                            .output_owners
                            .addresses
                            .iter()
                        {
                            packer.pack_bytes(addr.as_ref())?;
                        }
                    }
                    _ => {
                        return Err(Error::Other {
                            message: format!(
                                "unexpected type ID {} for TransferableOutput",
                                type_id_transferable_out
                            ),
                            retryable: false,
                        })
                    }
                }
            }
        } else {
            packer.pack_u32(0_u32)?;
        }

        // "transferable_inputs" field; pack the number of slice elements
        if self.transferable_inputs.is_some() {
            let transferable_inputs = self.transferable_inputs.as_ref().unwrap();
            packer.pack_u32(transferable_inputs.len() as u32)?;

            for transferable_input in transferable_inputs.iter() {
                // "TransferableInput.UTXOID" is struct and serialize:"true"
                // but embedded inline in the struct "TransferableInput"
                // so no need to encode type ID
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableInput
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#UTXOID
                packer.pack_bytes(transferable_input.utxo_id.tx_id.as_ref())?;
                packer.pack_u32(transferable_input.utxo_id.output_index)?;

                // "TransferableInput.Asset" is struct and serialize:"true"
                // but embedded inline in the struct "TransferableInput"
                // so no need to encode type ID
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableInput
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#Asset
                packer.pack_bytes(transferable_input.asset_id.as_ref())?;

                // fx_id is serialize:"false" thus skipping serialization

                // decide the type
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#TransferableInput
                if transferable_input.transfer_input.is_none()
                    && transferable_input.stakeable_lock_in.is_none()
                {
                    return Err(Error::Other {
                        message: "unexpected Nones in TransferableInput transfer_input and stakeable_lock_in".to_string(),
                        retryable: false,
                    });
                }
                let type_id_transferable_in = {
                    if transferable_input.transfer_input.is_some() {
                        key::secp256k1::txs::transfer::Input::type_id()
                    } else {
                        platformvm::txs::StakeableLockIn::type_id()
                    }
                };
                // marshal type ID for "key::secp256k1::txs::transfer::Input" or "platformvm::txs::StakeableLockIn"
                packer.pack_u32(type_id_transferable_in)?;

                match type_id_transferable_in {
                    5 => {
                        // "key::secp256k1::txs::transfer::Input"
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput
                        let transfer_input = transferable_input.transfer_input.clone().unwrap();

                        // marshal "secp256k1fx.TransferInput.Amt" field
                        packer.pack_u64(transfer_input.amount)?;

                        // "secp256k1fx.TransferInput.Input" is struct and serialize:"true"
                        // but embedded inline in the struct "TransferInput"
                        // so no need to encode type ID
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Input
                        packer.pack_u32(transfer_input.sig_indices.len() as u32)?;
                        for idx in transfer_input.sig_indices.iter() {
                            packer.pack_u32(*idx)?;
                        }
                    }
                    21 => {
                        // "platformvm::txs::StakeableLockIn"
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockIn
                        let stakeable_lock_in =
                            transferable_input.stakeable_lock_in.clone().unwrap();

                        // marshal "platformvm::txs::StakeableLockIn.locktime" field
                        packer.pack_u64(stakeable_lock_in.locktime)?;

                        // "platformvm.StakeableLockIn.TransferableIn" is struct and serialize:"true"
                        // but embedded inline in the struct "StakeableLockIn"
                        // so no need to encode type ID
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockIn
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Input
                        //
                        // marshal "secp256k1fx.TransferInput.Amt" field
                        packer.pack_u64(stakeable_lock_in.transfer_input.amount)?;
                        //
                        // "secp256k1fx.TransferInput.Input" is struct and serialize:"true"
                        // but embedded inline in the struct "TransferInput"
                        // so no need to encode type ID
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferInput
                        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Input
                        packer
                            .pack_u32(stakeable_lock_in.transfer_input.sig_indices.len() as u32)?;
                        for idx in stakeable_lock_in.transfer_input.sig_indices.iter() {
                            packer.pack_u32(*idx)?;
                        }
                    }
                    _ => {
                        return Err(Error::Other {
                            message: format!(
                                "unexpected type ID {} for TransferableInput",
                                type_id_transferable_in
                            ),
                            retryable: false,
                        })
                    }
                }
            }
        } else {
            packer.pack_u32(0_u32)?;
        }

        // marshal "BaseTx.memo"
        if self.memo.is_some() {
            let memo = self.memo.as_ref().unwrap();
            packer.pack_u32(memo.len() as u32)?;
            packer.pack_bytes(memo)?;
        } else {
            packer.pack_u32(0_u32)?;
        }

        Ok(packer)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::test_base_tx_serialization --exact --show-output
/// ref. "avalanchego/vms/avm.TestBaseTxSerialization"
#[test]
fn test_base_tx_serialization() {
    use crate::{ids::short, key};

    // ref. "avalanchego/vms/avm/vm_test.go"
    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-24jUJ9vZexUM6expyMcT48LBx27k1m7xpraoV62oSQAHdziao5",
    )
    .expect("failed to load private key");
    let test_key_short_addr = test_key
        .to_public_key()
        .to_short_bytes()
        .expect("failed to_short_bytes");
    let test_key_short_addr = short::Id::from_slice(&test_key_short_addr);

    let unsigned_tx = Tx {
        network_id: 10,
        blockchain_id: ids::Id::from_slice(&<Vec<u8>>::from([5, 4, 3, 2, 1])),
        transferable_outputs: Some(vec![transferable::Output {
            asset_id: ids::Id::from_slice(&<Vec<u8>>::from([1, 2, 3])),
            transfer_output: Some(key::secp256k1::txs::transfer::Output {
                amount: 12345,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: 0,
                    threshold: 1,
                    addresses: vec![test_key_short_addr],
                },
            }),
            ..transferable::Output::default()
        }]),
        transferable_inputs: Some(vec![transferable::Input {
            utxo_id: utxo::Id {
                tx_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, //
                    0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf2, 0xf1, 0xf0, //
                    0xef, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe8, //
                    0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xe0, //
                ])),
                output_index: 1,
                ..utxo::Id::default()
            },
            asset_id: ids::Id::from_slice(&<Vec<u8>>::from([1, 2, 3])),
            transfer_input: Some(key::secp256k1::txs::transfer::Input {
                amount: 54321,
                sig_indices: vec![2],
            }),
            ..transferable::Input::default()
        }]),
        memo: Some(vec![0x00, 0x01, 0x02, 0x03]),
        ..Tx::default()
    };
    let unsigned_tx_packer = unsigned_tx
        .pack(0, Tx::type_id())
        .expect("failed to pack unsigned_tx");
    let unsigned_tx_bytes = unsigned_tx_packer.take_bytes();

    let expected_unsigned_tx_bytes: Vec<u8> = vec![
        // codec version
        0x00, 0x00, //
        //
        // avm.BaseTx type ID
        0x00, 0x00, 0x00, 0x00, //
        //
        // network id
        0x00, 0x00, 0x00, 0x0a, //
        //
        // blockchain id
        0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // outs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.asset_id
        0x01, 0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // "outs[0]" secp256k1fx.TransferOutput type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x39, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.addrs[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, //
        0x61, 0x4b, 0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, //
        0x93, 0x07, 0x76, 0x26, //
        //
        // ins.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.utxo_id.tx_id
        0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, //
        0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf2, 0xf1, 0xf0, //
        0xef, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe8, //
        0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xe0, //
        //
        // "ins[0]" TransferableInput.utxo_id.output_index
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.asset_id
        0x01, 0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // "ins[0]" secp256k1fx.TransferInput type ID
        0x00, 0x00, 0x00, 0x05, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd4, 0x31, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices[0]
        0x00, 0x00, 0x00, 0x02, //
        //
        // memo.len()
        0x00, 0x00, 0x00, 0x04, //
        //
        // memo
        0x00, 0x01, 0x02, 0x03, //
    ];
    // for c in &unsigned_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        &expected_unsigned_tx_bytes,
        &unsigned_tx_bytes
    ));
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#Metadata>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Metadata {
    pub id: ids::Id,
    pub tx_bytes_with_no_signature: Vec<u8>,
    pub tx_bytes_with_signatures: Vec<u8>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::default()
    }
}

impl Metadata {
    pub fn default() -> Self {
        Self {
            id: ids::Id::empty(),
            tx_bytes_with_no_signature: Vec::new(),
            tx_bytes_with_signatures: Vec::new(),
        }
    }

    pub fn new(tx_bytes_with_no_signature: &[u8], tx_bytes_with_signatures: &[u8]) -> Self {
        let id = hash::sha256(tx_bytes_with_signatures);
        let id = ids::Id::from_slice(&id);
        Self {
            id,
            tx_bytes_with_no_signature: Vec::from(tx_bytes_with_no_signature),
            tx_bytes_with_signatures: Vec::from(tx_bytes_with_signatures),
        }
    }

    pub fn verify(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::Other {
                message: "metadata was never initialized and is not valid".to_string(), // ref. "errMetadataNotInitialize"
                retryable: false,
            });
        }
        Ok(())
    }
}
