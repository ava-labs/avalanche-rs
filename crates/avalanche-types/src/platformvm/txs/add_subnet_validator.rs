use crate::{codec, errors::Result, hash, ids, key, platformvm, txs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Validator {
    pub validator: platformvm::txs::Validator,
    pub subnet_id: ids::Id,
}

impl Default for Validator {
    fn default() -> Self {
        Self::default()
    }
}

impl Validator {
    pub fn default() -> Self {
        Self {
            validator: platformvm::txs::Validator::default(),
            subnet_id: ids::Id::empty(),
        }
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#AddSubnetValidatorTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// Tx.ID() is non-empty.
    pub base_tx: txs::Tx,
    pub validator: Validator,
    pub subnet_auth: key::secp256k1::txs::Input,

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
            validator: Validator::default(),
            subnet_auth: key::secp256k1::txs::Input::default(),
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
        "platformvm.AddSubnetValidatorTx".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::P_TYPES.get(&Self::type_name()).unwrap()) as u32
    }

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx.Sign>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/crypto#PrivateKeyED25519.SignHash>
    pub async fn sign<T: key::secp256k1::SignOnly>(&mut self, signers: Vec<Vec<T>>) -> Result<()> {
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
        packer.pack_bytes(self.validator.validator.node_id.as_ref())?;
        packer.pack_u64(self.validator.validator.start)?;
        packer.pack_u64(self.validator.validator.end)?;
        packer.pack_u64(self.validator.validator.weight)?;
        packer.pack_bytes(self.validator.subnet_id.as_ref())?;

        // pack the third field "subnet_auth" in the struct
        let subnet_auth_type_id = key::secp256k1::txs::Input::type_id();
        packer.pack_u32(subnet_auth_type_id)?;
        packer.pack_u32(self.subnet_auth.sig_indices.len() as u32)?;
        for sig_idx in self.subnet_auth.sig_indices.iter() {
            packer.pack_u32(*sig_idx)?;
        }

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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::add_subnet_validator::test_add_subnet_validator_tx_serialization_with_one_signer --exact --show-output
#[test]
fn test_add_subnet_validator_tx_serialization_with_one_signer() {
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
                    amount: 0x2c6874d5c56f500,
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
                    output_index: 0,
                    tx_id: ids::Id::from_slice(&<Vec<u8>>::from([
                        0xdd, 0x91, 0x70, 0x54, 0x1a, 0xf4, 0x4b, 0x08, //
                        0x54, 0x4d, 0xae, 0x2c, 0x5e, 0x6f, 0x2b, 0xd9, //
                        0x1e, 0xd4, 0x1e, 0x72, 0x22, 0x44, 0x73, 0x56, //
                        0x1f, 0x50, 0xe8, 0xeb, 0xfc, 0xba, 0x59, 0xb9, //
                    ])),
                    ..txs::utxo::Id::default()
                },
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                    0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                    0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                    0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
                ])),
                transfer_input: Some(key::secp256k1::txs::transfer::Input {
                    amount: 0x2c6874d5c663740,
                    sig_indices: vec![0],
                }),
                ..txs::transferable::Input::default()
            }]),
            ..txs::Tx::default()
        },
        validator: Validator {
            validator: platformvm::txs::Validator {
                node_id: node::Id::from_slice(&<Vec<u8>>::from([
                    0xca, 0xc3, 0x1b, 0x23, 0x7f, 0x96, 0x40, 0xd5, 0x01, 0x11, //
                    0xbe, 0x86, 0xb9, 0x58, 0x73, 0x0a, 0xfb, 0x70, 0x5e, 0x0f, //
                ])),
                start: 0x623d424b,
                end: 0x641e6651,
                weight: 0x3e8,
            },
            subnet_id: ids::Id::from_slice(&<Vec<u8>>::from([
                0xdd, 0x91, 0x70, 0x54, 0x1a, 0xf4, 0x4b, 0x08, 0x54, 0x4d, //
                0xae, 0x2c, 0x5e, 0x6f, 0x2b, 0xd9, 0x1e, 0xd4, 0x1e, 0x72, //
                0x22, 0x44, 0x73, 0x56, 0x1f, 0x50, 0xe8, 0xeb, 0xfc, 0xba, //
                0x59, 0xb9,
            ])),
        },
        subnet_auth: key::secp256k1::txs::Input {
            sig_indices: vec![0_u32],
        },
        ..Tx::default()
    };

    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-2kqWNDaqUKQyE4ZsV5GLCGeizE6sHAJVyjnfjXoXrtcZpK9M67",
    )
    .expect("failed to load private key");
    let keys1: Vec<key::secp256k1::private_key::Key> = vec![test_key.clone()];
    let keys2: Vec<key::secp256k1::private_key::Key> = vec![test_key];
    let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys1, keys2];
    ab!(tx.sign(signers)).expect("failed to sign");
    let tx_metadata = tx.base_tx.metadata.clone().unwrap();
    let tx_bytes_with_signatures = tx_metadata.tx_bytes_with_signatures;
    assert_eq!(
        tx.tx_id().to_string(),
        "2bAuXK8TGqehHQCSaFkg4tSf7BX91aXM4qP3vX2Y62d4hg22T5"
    );

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // platformvm.UnsignedAddSubnetValidatorTx type ID
        0x00, 0x00, 0x00, 0x0d, //
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
        0x02, 0xc6, 0x87, 0x4d, 0x5c, 0x56, 0xf5, 0x00, //
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
        0xdd, 0x91, 0x70, 0x54, 0x1a, 0xf4, 0x4b, 0x08, 0x54, 0x4d, //
        0xae, 0x2c, 0x5e, 0x6f, 0x2b, 0xd9, 0x1e, 0xd4, 0x1e, 0x72, //
        0x22, 0x44, 0x73, 0x56, 0x1f, 0x50, 0xe8, 0xeb, 0xfc, 0xba, //
        0x59, 0xb9, //
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
        0x02, 0xc6, 0x87, 0x4d, 0x5c, 0x66, 0x37, 0x40, //
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
        0xca, 0xc3, 0x1b, 0x23, 0x7f, 0x96, 0x40, 0xd5, 0x01, 0x11, //
        0xbe, 0x86, 0xb9, 0x58, 0x73, 0x0a, 0xfb, 0x70, 0x5e, 0x0f, //
        //
        // Validator.validator.start
        0x00, 0x00, 0x00, 0x00, 0x62, 0x3d, 0x42, 0x4b, //
        //
        // Validator.validator.end
        0x00, 0x00, 0x00, 0x00, 0x64, 0x1e, 0x66, 0x51, //
        //
        // Validator.validator.weight
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, //
        //
        // Validator.subnet_id
        0xdd, 0x91, 0x70, 0x54, 0x1a, 0xf4, 0x4b, 0x08, //
        0x54, 0x4d, 0xae, 0x2c, 0x5e, 0x6f, 0x2b, 0xd9, //
        0x1e, 0xd4, 0x1e, 0x72, 0x22, 0x44, 0x73, 0x56, //
        0x1f, 0x50, 0xe8, 0xeb, 0xfc, 0xba, 0x59, 0xb9, //
        //
        // "secp256k1fx.Input" type ID
        0x00, 0x00, 0x00, 0x0a, //
        //
        // "secp256k1fx.Input.sig_indices.len()"
        0x00, 0x00, 0x00, 0x01, //
        //
        // "secp256k1fx.Input.sig_indices[0]"
        0x00, 0x00, 0x00, 0x00,
        //
        //
        // number of of credentials (avax.Tx.creds.len())
        0x00, 0x00, 0x00, 0x02, //
        //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // struct field type ID "fx::Credential.cred"
        // "secp256k1fx.Credential" type ID
        0x00, 0x00, 0x00, 0x09, //
        //
        // number of signers ("fx::Credential.cred.sigs.len()")
        0x00, 0x00, 0x00, 0x01, //
        //
        // first 65-byte signature
        0x12, 0x51, 0x43, 0xaf, 0xa0, 0xd1, 0x5b, 0xe6, 0x06, 0xe2, //
        0xc5, 0x50, 0xe1, 0x09, 0xac, 0x86, 0xcd, 0x55, 0x45, 0xeb, //
        0x86, 0x5d, 0x8e, 0x19, 0xf0, 0x37, 0x28, 0x62, 0x8e, 0xaf, //
        0xac, 0x52, 0x3a, 0x2c, 0xe3, 0xde, 0x22, 0xa1, 0x3d, 0x3b, //
        0xfb, 0x67, 0x2b, 0x03, 0xa8, 0x29, 0xd7, 0xbd, 0x1d, 0x10, //
        0x06, 0x34, 0xbd, 0x2b, 0x4a, 0xf5, 0x3d, 0xb9, 0x0d, 0x2a, //
        0x63, 0x71, 0x38, 0x5a, 0x00, //
        //
        // struct field type ID "fx::Credential.cred"
        // "secp256k1fx.Credential" type ID
        0x00, 0x00, 0x00, 0x09, //
        //
        // number of signers ("fx::Credential.cred.sigs.len()")
        0x00, 0x00, 0x00, 0x01, //
        //
        // second 65-byte signature
        0x12, 0x51, 0x43, 0xaf, 0xa0, 0xd1, 0x5b, 0xe6, 0x06, 0xe2, //
        0xc5, 0x50, 0xe1, 0x09, 0xac, 0x86, 0xcd, 0x55, 0x45, 0xeb, //
        0x86, 0x5d, 0x8e, 0x19, 0xf0, 0x37, 0x28, 0x62, 0x8e, 0xaf, //
        0xac, 0x52, 0x3a, 0x2c, 0xe3, 0xde, 0x22, 0xa1, 0x3d, 0x3b, //
        0xfb, 0x67, 0x2b, 0x03, 0xa8, 0x29, 0xd7, 0xbd, 0x1d, 0x10, //
        0x06, 0x34, 0xbd, 0x2b, 0x4a, 0xf5, 0x3d, 0xb9, 0x0d, 0x2a, //
        0x63, 0x71, 0x38, 0x5a, 0x00, //
    ];
    // for c in &signed_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}
