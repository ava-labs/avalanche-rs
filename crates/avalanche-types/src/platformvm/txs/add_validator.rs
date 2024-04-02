use crate::{
    codec,
    errors::Result,
    hash, ids, key,
    packer::{self, Packer},
    platformvm, txs,
};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#AddValidatorTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Default)]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// Tx.ID() is non-empty.
    #[serde(flatten)]
    pub base_tx: txs::Tx,
    pub validator: platformvm::txs::Validator,
    #[serde(rename = "stake")]
    pub stake_transferable_outputs: Option<Vec<txs::transferable::Output>>,
    #[serde(rename = "rewardsOwner")]
    pub rewards_owner: key::secp256k1::txs::OutputOwners,
    pub shares: u32,

    /// To be updated after signing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub creds: Vec<key::secp256k1::txs::Credential>,
}

impl Tx {
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
        let packer = Packer::new();
        packer.pack(self)?;

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

            let cred = key::secp256k1::txs::Credential { signatures: sigs };

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

impl packer::Packable for Tx {
    fn pack(&self, packer: &Packer) -> Result<()> {
        let type_id = Self::type_id();

        packer.pack_u32(type_id)?;
        packer.pack(&self.base_tx)?;

        // pack the second field "validator" in the struct
        packer.pack_bytes(self.validator.node_id.as_ref())?;
        packer.pack_u64(self.validator.start)?;
        packer.pack_u64(self.validator.end)?;
        packer.pack_u64(self.validator.weight)?;

        // pack the third field "stake" in the struct
        packer.pack(&self.stake_transferable_outputs)?;

        // pack the fourth field "reward_owner" in the struct
        // not embedded thus encode struct type id
        let output_owners_type_id = key::secp256k1::txs::OutputOwners::type_id();
        packer.pack_u32(output_owners_type_id)?;
        packer.pack_u64(self.rewards_owner.locktime)?;
        packer.pack_u32(self.rewards_owner.threshold)?;
        packer.pack(&self.rewards_owner.addresses)?;

        // pack the fifth field "shares" in the struct
        packer.pack_u32(self.shares)?;
        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::add_validator::test_add_validator_tx_serialization_with_one_signer --exact --show-output
#[test]
fn test_add_validator_tx_serialization_with_one_signer() {
    use crate::{
        ids::{node, short},
        txs::transferable::TransferableOut,
    };

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
                out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
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
            out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::add_validator::test_json_deserialize --exact --show-output
#[test]
fn test_json_deserialize() {
    use serde_json::json;

    let tx_json = json!({
        "networkID": 10,
        "blockchainID": "11111111111111111111111111111111LpoYY",
        "outputs": [
            {
                "assetID": "Zda4gsqTjRaX6XVZekVNi3ovMFPHDRQiGbzYuAb7Nwqy1rGBc",
                "fxID": "11111111111111111111111111111111LpoYY",
                "output": {
                    "addresses": [
                        "Q4MzFZZDPHRPAHFeDs3NiyyaZDvxHKivf"
                    ],
                    "amount": 1234,
                    "locktime": 0,
                    "threshold": 1
                }
            }
        ],
        "inputs": [
            {
                "txID": "tJ4rZfd5dnsPpWPVYU3skNW8uYNpaS6bmpto3sXMMqFMVpR1f",
                "outputIndex": 2,
                "assetID": "Zda4gsqTjRaX6XVZekVNi3ovMFPHDRQiGbzYuAb7Nwqy1rGBc",
                "fxID": "11111111111111111111111111111111LpoYY",
                "input": {
                    "amount": 5678,
                    "signatureIndices": [
                        0
                    ]
                }
            }
        ],
        "memo": "0x",
        "validator": {
            "nodeID": "NodeID-111111111111111111116DBWJs",
            "start": 1711696405,
            "end": 1711700005,
            "weight": 2022
        },
        "stake": [
            {
                "assetID": "Zda4gsqTjRaX6XVZekVNi3ovMFPHDRQiGbzYuAb7Nwqy1rGBc",
                "fxID": "11111111111111111111111111111111LpoYY",
                "output": {
                    "locktime": 1711696406,
                    "output": {
                        "addresses": [
                            "Q4MzFZZDPHRPAHFeDs3NiyyaZDvxHKivf"
                        ],
                        "amount": 2022,
                        "locktime": 0,
                        "threshold": 1
                    }
                }
            }
        ],
        "rewardsOwner": {
            "addresses": [
                "Q4MzFZZDPHRPAHFeDs3NiyyaZDvxHKivf"
            ],
            "locktime": 0,
            "threshold": 1
        },
        "shares": 1000000
    });

    let tx: Tx = serde_json::from_value(tx_json).expect("parsing tx");
    let packer = Packer::new();
    packer.pack(&tx).expect("packing tx");

    let expected_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // AddValidatorTx type ID
        0x00, 0x00, 0x00, 0x0c, //
        //
        // network id
        0x00, 0x00, 0x00, 0x0a, //
        // blockchainID
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // outputs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // outputs[0].assetID
        0x4a, 0x17, 0x72, 0x05, 0xdf, 0x5c, 0x29, 0x92, 0x9d, 0x06, //
        0xdb, 0x9d, 0x94, 0x1f, 0x83, 0xd5, 0xea, 0x98, 0x5d, 0xe3, //
        0x02, 0x01, 0x5e, 0x99, 0x25, 0x2d, 0x16, 0x46, 0x9a, 0x66, //
        0x10, 0xdb, //
        //
        // outputs[0] type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // outputs[0].output.amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xd2, //
        //
        // outputs[0].output.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // outputs[0].output.threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // outputs[0].output.addresses.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // outputs[0].output.addresses[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        //
        // inputs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // inputs[0].txID
        0x74, 0x78, 0x49, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // inputs[0].outputIndex
        0x00, 0x00, 0x00, 0x02, //
        //
        // inputs[0].assetID
        0x4a, 0x17, 0x72, 0x05, 0xdf, 0x5c, 0x29, 0x92, 0x9d, 0x06, //
        0xdb, 0x9d, 0x94, 0x1f, 0x83, 0xd5, 0xea, 0x98, 0x5d, 0xe3, //
        0x02, 0x01, 0x5e, 0x99, 0x25, 0x2d, 0x16, 0x46, 0x9a, 0x66, //
        0x10, 0xdb, //
        //
        // inputs[0] type ID
        0x00, 0x00, 0x00, 0x05, //
        //
        // inputs[0].input.amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x16, 0x2e, //
        //
        // inputs[0].input.signatureIndices.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // inputs[0].input.signatureIndices[0]
        0x00, 0x00, 0x00, 0x00, //
        //
        // memo.len()
        0x00, 0x00, 0x00, 0x00, //
        // validator.nodeID
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // validator.start
        0x00, 0x00, 0x00, 0x00, 0x66, 0x06, 0x6a, 0x15, //
        //
        // validator.end
        0x00, 0x00, 0x00, 0x00, 0x66, 0x06, 0x78, 0x25, //
        //
        // validator.weight
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xe6, //
        //
        // stake.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // stake[0].assetID
        0x4a, 0x17, 0x72, 0x05, 0xdf, 0x5c, 0x29, 0x92, 0x9d, 0x06, //
        0xdb, 0x9d, 0x94, 0x1f, 0x83, 0xd5, 0xea, 0x98, 0x5d, 0xe3, //
        0x02, 0x01, 0x5e, 0x99, 0x25, 0x2d, 0x16, 0x46, 0x9a, 0x66, //
        0x10, 0xdb, //
        //
        // stake[0] type ID
        0x00, 0x00, 0x00, 0x16, //
        //
        // stake[0].locktime
        0x00, 0x00, 0x00, 0x00, 0x66, 0x06, 0x6a, 0x16, //
        //
        // stake[0].output type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // stake[0].output amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xe6, //
        //
        // stake[0].output.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // stake[0].output.threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // stake[0].output.addresses.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // stake[0].output.addresses[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        //
        // rewardsOwner type ID
        0x00, 0x00, 0x00, 0x0b, //
        //
        // rewardsOwner.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // rewardsOwner.threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // rewardsOwner.addresses.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // rewardsOwner.addresses[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        // shares
        0x00, 0x0f, 0x42, 0x40, //
    ];

    assert_eq!(packer.take_bytes(), expected_bytes);
}
