use crate::{
    codec,
    errors::{Error, Result},
    hash, ids, key, platformvm, txs,
};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#AddValidatorTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// Tx.ID() is non-empty.
    pub base_tx: txs::Tx,
    pub validator: platformvm::txs::Validator,
    pub stake_transferable_outputs: Option<Vec<txs::transferable::Output>>,
    pub rewards_owner: key::secp256k1::txs::OutputOwners,
    pub shares: u32,

    /// To be updated after signing.
    pub creds: Vec<key::secp256k1::txs::Credential>,
}

impl Default for Tx {
    fn default() -> Self {
        Self::default()
    }
}

impl Tx {
    pub fn default() -> Self {
        Self {
            base_tx: txs::Tx::default(),
            validator: platformvm::txs::Validator::default(),
            stake_transferable_outputs: None,
            rewards_owner: key::secp256k1::txs::OutputOwners::default(),
            shares: 0,
            creds: Vec::new(),
        }
    }

    pub fn new(base_tx: txs::Tx) -> Self {
        Self {
            base_tx,
            ..Self::default()
        }
    }

    /// Returns the transaction ID.
    /// Only non-empty if the embedded metadata is updated
    /// with the signing process.
    pub fn tx_id(&self) -> ids::Id {
        if self.base_tx.metadata.is_some() {
            let m = self.base_tx.metadata.clone().unwrap();
            m.id
        } else {
            ids::Id::default()
        }
    }

    pub fn type_name() -> String {
        "platformvm.AddValidatorTx".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::P_TYPES.get(&Self::type_name()).unwrap()) as u32
    }

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx.Sign>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/crypto#PrivateKeyED25519.SignHash>
    pub async fn sign<T: key::secp256k1::SignOnly + Clone>(
        &mut self,
        signers: Vec<Vec<T>>,
    ) -> Result<()> {
        // marshal "unsigned tx" with the codec version
        let type_id = Self::type_id();
        let packer = self.base_tx.pack(codec::VERSION, type_id)?;

        // "avalanchego" marshals the whole struct again for signed bytes
        // even when the underlying "unsigned_tx" is already once marshaled
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#Tx.Sign
        //
        // reuse the underlying packer to avoid marshaling the unsigned tx twice
        // just marshal the next fields in the struct and pack them all together
        // in the existing packer
        let unsigned_tx_bytes = packer.take_bytes();
        packer.set_bytes(&unsigned_tx_bytes);

        // pack the second field "validator" in the struct
        packer.pack_bytes(self.validator.node_id.as_ref())?;
        packer.pack_u64(self.validator.start)?;
        packer.pack_u64(self.validator.end)?;
        packer.pack_u64(self.validator.weight)?;

        // pack the third field "stake" in the struct
        if self.stake_transferable_outputs.is_some() {
            let stake_transferable_outputs = self.stake_transferable_outputs.as_ref().unwrap();
            packer.pack_u32(stake_transferable_outputs.len() as u32)?;

            for transferable_output in stake_transferable_outputs.iter() {
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
                        });
                    }
                }
            }
        } else {
            packer.pack_u32(0_u32)?;
        }

        // pack the fourth field "reward_owner" in the struct
        // not embedded thus encode struct type id
        let output_owners_type_id = key::secp256k1::txs::OutputOwners::type_id();
        packer.pack_u32(output_owners_type_id)?;
        packer.pack_u64(self.rewards_owner.locktime)?;
        packer.pack_u32(self.rewards_owner.threshold)?;
        packer.pack_u32(self.rewards_owner.addresses.len() as u32)?;
        for addr in self.rewards_owner.addresses.iter() {
            packer.pack_bytes(addr.as_ref())?;
        }

        // pack the fifth field "shares" in the struct
        packer.pack_u32(self.shares)?;

        // take bytes just for hashing computation
        let tx_bytes_with_no_signature = packer.take_bytes();
        packer.set_bytes(&tx_bytes_with_no_signature);

        // compute sha256 for marshaled "unsigned tx" bytes
        // IMPORTANT: take the hash only for the type "platformvm.AddValidatorTx" unsigned tx
        // not other fields -- only hash "platformvm.AddValidatorTx.*" but not "platformvm.Tx.Creds"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#UnsignedAddValidatorTx
        let tx_bytes_hash = hash::sha256(&tx_bytes_with_no_signature);

        // number of of credentials
        let creds_len = signers.len() as u32;
        // pack the fourth field in the struct
        packer.pack_u32(creds_len)?;

        // sign the hash with the signers (in case of multi-sig)
        // and combine all signatures into a secp256k1fx credential
        self.creds = Vec::new();
        for keys in signers.iter() {
            let mut sigs: Vec<Vec<u8>> = Vec::new();
            for k in keys.iter() {
                let sig = k.sign_digest(&tx_bytes_hash).await?;
                sigs.push(Vec::from(sig));
            }

            let mut cred = key::secp256k1::txs::Credential::default();
            cred.signatures = sigs;

            // add a new credential to "Tx"
            self.creds.push(cred);
        }
        if creds_len > 0 {
            // pack each "cred" which is "secp256k1fx.Credential"
            // marshal type ID for "secp256k1fx.Credential"
            let cred_type_id = key::secp256k1::txs::Credential::type_id();
            for cred in self.creds.iter() {
                // marshal type ID for "secp256k1fx.Credential"
                packer.pack_u32(cred_type_id)?;

                // marshal fields for "secp256k1fx.Credential"
                packer.pack_u32(cred.signatures.len() as u32)?;
                for sig in cred.signatures.iter() {
                    packer.pack_bytes(sig)?;
                }
            }
        }
        let tx_bytes_with_signatures = packer.take_bytes();
        let tx_id = hash::sha256(&tx_bytes_with_signatures);

        // update "BaseTx.Metadata" with id/unsigned bytes/bytes
        // ref. "avalanchego/vms/platformvm.Tx.Sign"
        // ref. "avalanchego/vms/components/avax.BaseTx.Metadata.Initialize"
        self.base_tx.metadata = Some(txs::Metadata {
            id: ids::Id::from_slice(&tx_id),
            tx_bytes_with_no_signature: tx_bytes_with_no_signature.to_vec(),
            tx_bytes_with_signatures: tx_bytes_with_signatures.to_vec(),
        });

        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::add_validator::test_add_validator_tx_serialization_with_one_signer --exact --show-output
#[test]
fn test_add_validator_tx_serialization_with_one_signer() {
    use crate::ids::{node, short};

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let mut tx = Tx {
        base_tx: txs::Tx {
            network_id: 1000000,
            transferable_outputs: Some(vec![txs::transferable::Output {
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                    0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                    0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                    0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
                ])),
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: 0x2c6874d687fc000,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0x00,
                        threshold: 0x01,
                        addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                            0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
                            0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
                        ]))],
                    },
                }),
                ..txs::transferable::Output::default()
            }]),
            transferable_inputs: Some(vec![txs::transferable::Input {
                utxo_id: txs::utxo::Id {
                    tx_id: ids::Id::from_slice(&<Vec<u8>>::from([
                        0x78, 0x3b, 0x22, 0xc6, 0xa8, 0xd6, 0x83, 0x4c, 0x89, 0x30, //
                        0xae, 0xac, 0x3d, 0xb6, 0x02, 0x63, 0xc1, 0x2e, 0x98, 0x16, //
                        0x0e, 0xf7, 0x22, 0x1b, 0x4d, 0x5e, 0x62, 0x2e, 0x87, 0x0f, //
                        0x92, 0xd9,
                    ])),
                    output_index: 0,
                    ..txs::utxo::Id::default()
                },
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                    0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                    0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                    0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
                ])),
                transfer_input: Some(key::secp256k1::txs::transfer::Input {
                    amount: 0x2c6891f11c9e000,
                    sig_indices: vec![0],
                }),
                ..txs::transferable::Input::default()
            }]),
            ..txs::Tx::default()
        },
        validator: platformvm::txs::Validator {
            node_id: node::Id::from_slice(&<Vec<u8>>::from([
                0x9c, 0xd7, 0xb3, 0xe4, 0x79, 0x04, 0xf6, 0x7c, 0xc4, 0x8e, //
                0xb5, 0xb9, 0xaf, 0xdb, 0x03, 0xe6, 0xd1, 0x8a, 0xcf, 0x6c, //
            ])),
            start: 0x623d7267,
            end: 0x63c91062,
            weight: 0x1d1a94a2000,
        },
        stake_transferable_outputs: Some(vec![txs::transferable::Output {
            asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
            ])),
            transfer_output: Some(key::secp256k1::txs::transfer::Output {
                amount: 0x1d1a94a2000,
                output_owners: key::secp256k1::txs::OutputOwners {
                    locktime: 0x00,
                    threshold: 0x01,
                    addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                        0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
                        0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
                    ]))],
                },
            }),
            ..txs::transferable::Output::default()
        }]),
        rewards_owner: key::secp256k1::txs::OutputOwners {
            locktime: 0x00,
            threshold: 0x01,
            addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
                0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
            ]))],
        },
        shares: 0x4e20,
        ..Tx::default()
    };

    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-2kqWNDaqUKQyE4ZsV5GLCGeizE6sHAJVyjnfjXoXrtcZpK9M67",
    )
    .expect("failed to load private key");
    let keys1: Vec<key::secp256k1::private_key::Key> = vec![test_key];
    let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys1];
    ab!(tx.sign(signers)).expect("failed to sign");
    let tx_metadata = tx.base_tx.metadata.clone().unwrap();
    let tx_bytes_with_signatures = tx_metadata.tx_bytes_with_signatures;
    assert_eq!(
        tx.tx_id().to_string(),
        "SPG7CSVMSkXSxnCWQnaENXFHKuzxuCYDGBSKVqsQtqx7WvwJ8"
    );

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // platformvm.UnsignedAddValidatorTx type ID
        0x00, 0x00, 0x00, 0x0c, //
        //
        // network id
        0x00, 0x0f, 0x42, 0x40, //
        //
        // blockchain id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // outs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.asset_id
        0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, 0xe6, 0x89, //
        0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, 0xe8, 0x5e, 0xa5, 0x74, //
        0xc7, 0xa1, 0x5a, 0x79, 0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, //
        0x80, 0x14, //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // "outs[0]" secp256k1fx.TransferOutput type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.amount
        0x02, 0xc6, 0x87, 0x4d, 0x68, 0x7f, 0xc0, 0x00, //
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
        0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
        0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
        //
        // ins.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.utxo_id.tx_id
        0x78, 0x3b, 0x22, 0xc6, 0xa8, 0xd6, 0x83, 0x4c, 0x89, 0x30, //
        0xae, 0xac, 0x3d, 0xb6, 0x02, 0x63, 0xc1, 0x2e, 0x98, 0x16, //
        0x0e, 0xf7, 0x22, 0x1b, 0x4d, 0x5e, 0x62, 0x2e, 0x87, 0x0f, //
        0x92, 0xd9, //
        //
        // "ins[0]" TransferableInput.utxo_id.output_index
        0x00, 0x00, 0x00, 0x00, //
        //
        // "ins[0]" TransferableInput.asset_id
        0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, 0xe6, 0x89, //
        0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, 0xe8, 0x5e, 0xa5, 0x74, //
        0xc7, 0xa1, 0x5a, 0x79, 0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, //
        0x80, 0x14, //
        //
        // "ins[0]" secp256k1fx.TransferInput type ID
        0x00, 0x00, 0x00, 0x05, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.amount
        0x02, 0xc6, 0x89, 0x1f, 0x11, 0xc9, 0xe0, 0x00, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices[0]
        0x00, 0x00, 0x00, 0x00, //
        //
        // memo.len()
        0x00, 0x00, 0x00, 0x00, //
        //
        // Validator.validator.node_id
        0x9c, 0xd7, 0xb3, 0xe4, 0x79, 0x04, 0xf6, 0x7c, 0xc4, 0x8e, //
        0xb5, 0xb9, 0xaf, 0xdb, 0x03, 0xe6, 0xd1, 0x8a, 0xcf, 0x6c, //
        //
        // Validator.validator.start
        0x00, 0x00, 0x00, 0x00, 0x62, 0x3d, 0x72, 0x67, //
        //
        // Validator.validator.end
        0x00, 0x00, 0x00, 0x00, 0x63, 0xc9, 0x10, 0x62, //
        //
        // Validator.validator.weight
        0x00, 0x00, 0x01, 0xd1, 0xa9, 0x4a, 0x20, 0x00, //
        //
        // stake_outputs.len
        0x00, 0x00, 0x00, 0x01, //
        //
        // stake_outputs[0].asset_id
        0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, 0xe6, 0x89, //
        0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, 0xe8, 0x5e, 0xa5, 0x74, //
        0xc7, 0xa1, 0x5a, 0x79, 0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, //
        0x80, 0x14, //
        //
        // secp256k1fx.TransferOutput type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // stake_outputs[0].amount
        0x00, 0x00, 0x01, 0xd1, 0xa9, 0x4a, 0x20, 0x00, //
        //
        // locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // addrs[0]
        0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
        0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
        //
        // secp256k1fx.OutputOwners type id
        0x00, 0x00, 0x00, 0x0b, //
        //
        // locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // addrs[0]
        0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
        0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
        //
        // reward shares
        0x00, 0x00, 0x4e, 0x20, //
        //
        // number of credentials
        0x00, 0x00, 0x00, 0x01, //
        //
        // struct field type ID "fx::Credential.cred"
        // "secp256k1fx.Credential" type ID
        0x00, 0x00, 0x00, 0x09, //
        //
        // number of signers ("fx::Credential.cred.sigs.len()")
        0x00, 0x00, 0x00, 0x01, //
        //
        // first 65-byte signature
        0x83, 0xa8, 0x63, 0xc8, 0x90, 0x02, 0xab, 0x70, 0xa1, 0x2c, //
        0x37, 0x80, 0x22, 0x84, 0xb7, 0x03, 0xc1, 0x65, 0x3a, 0x93, //
        0xa0, 0xa2, 0x5e, 0x04, 0x51, 0xf0, 0xda, 0xa0, 0x79, 0x16, //
        0xa3, 0x24, 0x71, 0xb1, 0x65, 0xbb, 0x4b, 0x1b, 0xd1, 0xb6, //
        0xed, 0xc6, 0xb4, 0x94, 0xbc, 0x6a, 0xac, 0x63, 0xc2, 0x4f, //
        0xcc, 0xfd, 0x9a, 0x54, 0x7b, 0x5f, 0x03, 0xa6, 0x02, 0x52, //
        0xd4, 0x5c, 0x24, 0x80, 0x00,
    ];
    // for c in &signed_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}
