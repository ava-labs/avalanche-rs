use crate::{
    codec,
    errors::Result,
    hash, ids, key,
    txs::{self},
};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#CreateSubnetTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Default)]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// Tx.ID() is non-empty.
    pub base_tx: txs::Tx,
    pub owner: key::secp256k1::txs::OutputOwners,

    /// To be updated after signing.
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
        "platformvm.CreateSubnetTx".to_string()
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
        let base = packer.take_bytes();
        packer.set_bytes(&base);

        // pack the second field "owner" in the struct
        // not embedded thus encode struct type id
        let output_owners_type_id = key::secp256k1::txs::OutputOwners::type_id();
        packer.pack_u32(output_owners_type_id)?;
        packer.pack_u64(self.owner.locktime)?;
        packer.pack_u32(self.owner.threshold)?;
        packer.pack_u32(self.owner.addresses.len() as u32)?;
        for addr in self.owner.addresses.iter() {
            packer.pack_bytes(addr.as_ref())?;
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::create_subnet::test_create_subnet_tx_serialization_with_one_signer --exact --show-output
#[test]
fn test_create_subnet_tx_serialization_with_one_signer() {
    use crate::{ids::short, txs::transferable::TransferableOut};

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let mut tx = Tx {
        base_tx: txs::Tx {
            network_id: 1337,
            transferable_outputs: Some(vec![txs::transferable::Output {
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x17, 0xcc, 0x8b, 0x15, 0x78, 0xba, 0x38, 0x35, //
                    0x44, 0xd1, 0x63, 0x95, 0x88, 0x22, 0xd8, 0xab, //
                    0xd3, 0x84, 0x9b, 0xb9, 0xdf, 0xab, 0xe3, 0x9f, //
                    0xcb, 0xc3, 0xe7, 0xee, 0x88, 0x11, 0xfe, 0x2f, //
                ])),
                out: TransferableOut::TransferOutput(key::secp256k1::txs::transfer::Output {
                    amount: 0x2386f269cb1f00,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0x00,
                        threshold: 0x01,
                        addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                            0x3c, 0xb7, 0xd3, 0x84, 0x2e, 0x8c, 0xee, 0x6a, 0x0e, 0xbd, 0x09, 0xf1,
                            0xfe, 0x88, 0x4f, 0x68, 0x61, 0xe1, 0xb2, 0x9c,
                        ]))],
                    },
                }),
                ..txs::transferable::Output::default()
            }]),
            transferable_inputs: Some(vec![txs::transferable::Input {
                utxo_id: txs::utxo::Id {
                    output_index: 1,
                    ..txs::utxo::Id::default()
                },
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x17, 0xcc, 0x8b, 0x15, 0x78, 0xba, 0x38, 0x35, //
                    0x44, 0xd1, 0x63, 0x95, 0x88, 0x22, 0xd8, 0xab, //
                    0xd3, 0x84, 0x9b, 0xb9, 0xdf, 0xab, 0xe3, 0x9f, //
                    0xcb, 0xc3, 0xe7, 0xee, 0x88, 0x11, 0xfe, 0x2f, //
                ])),
                transfer_input: Some(key::secp256k1::txs::transfer::Input {
                    amount: 0x2386f26fc10000,
                    sig_indices: vec![0],
                }),
                ..txs::transferable::Input::default()
            }]),
            ..txs::Tx::default()
        },
        owner: key::secp256k1::txs::OutputOwners {
            locktime: 0x00,
            threshold: 0x01,
            addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                0x3c, 0xb7, 0xd3, 0x84, 0x2e, 0x8c, 0xee, 0x6a, 0x0e, 0xbd, //
                0x09, 0xf1, 0xfe, 0x88, 0x4f, 0x68, 0x61, 0xe1, 0xb2, 0x9c, //
            ]))],
        },
        ..Tx::default()
    };

    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN",
    )
    .expect("failed to load private key");
    let keys1: Vec<key::secp256k1::private_key::Key> = vec![test_key];
    let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys1];
    ab!(tx.sign(signers)).expect("failed to sign");
    let tx_metadata = tx.base_tx.metadata.clone().unwrap();
    let tx_bytes_with_signatures = tx_metadata.tx_bytes_with_signatures;
    assert_eq!(
        tx.tx_id().to_string(),
        "24tZhrm8j8GCJRE9PomW8FaeqbgGS4UAQjJnqqn8pq5NwYSYV1"
    );

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // platformvm.UnsignedCreateSubnetTx type ID
        0x00, 0x00, 0x00, 0x10, //
        //
        // network id
        0x00, 0x00, 0x05, 0x39, //
        //
        // blockchain id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // outs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.asset_id
        0x17, 0xcc, 0x8b, 0x15, 0x78, 0xba, 0x38, 0x35, //
        0x44, 0xd1, 0x63, 0x95, 0x88, 0x22, 0xd8, 0xab, //
        0xd3, 0x84, 0x9b, 0xb9, 0xdf, 0xab, 0xe3, 0x9f, //
        0xcb, 0xc3, 0xe7, 0xee, 0x88, 0x11, 0xfe, 0x2f, //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // "outs[0]" secp256k1fx.TransferOutput type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.amount
        0x00, 0x23, 0x86, 0xf2, 0x69, 0xcb, 0x1f, 0x00, //
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
        0x3c, 0xb7, 0xd3, 0x84, 0x2e, 0x8c, 0xee, 0x6a, 0x0e, 0xbd, //
        0x09, 0xf1, 0xfe, 0x88, 0x4f, 0x68, 0x61, 0xe1, 0xb2, 0x9c, //
        //
        // ins.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.utxo_id.tx_id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // "ins[0]" TransferableInput.utxo_id.output_index
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.asset_id
        0x17, 0xcc, 0x8b, 0x15, 0x78, 0xba, 0x38, 0x35, //
        0x44, 0xd1, 0x63, 0x95, 0x88, 0x22, 0xd8, 0xab, //
        0xd3, 0x84, 0x9b, 0xb9, 0xdf, 0xab, 0xe3, 0x9f, //
        0xcb, 0xc3, 0xe7, 0xee, 0x88, 0x11, 0xfe, 0x2f, //
        //
        // "ins[0]" secp256k1fx.TransferInput type ID
        0x00, 0x00, 0x00, 0x05, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.amount
        0x00, 0x23, 0x86, 0xf2, 0x6f, 0xc1, 0x00, 0x00, //
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
        // secp256k1fx.OutputOwner type ID
        0x00, 0x00, 0x00, 0x0b, //
        //
        // output_owners.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //
        // output_owners.threshold
        0x00, 0x00, 0x00, 0x01, //
        // output_owners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // output_owners.addrs[0]
        0x3c, 0xb7, 0xd3, 0x84, 0x2e, 0x8c, 0xee, 0x6a, 0x0e, 0xbd, //
        0x09, 0xf1, 0xfe, 0x88, 0x4f, 0x68, 0x61, 0xe1, 0xb2, 0x9c, //
        //
        // number of of credentials (avax.Tx.fx_creds.len())
        0x00, 0x00, 0x00, 0x01, //
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
        0xbb, 0xd0, 0x6b, 0xf2, 0x62, 0x71, 0x49, 0x07, 0x83, 0x52, //
        0x07, 0x30, 0xa1, 0x12, 0x1f, 0x9c, 0x8e, 0x60, 0x2b, 0xf8, //
        0x75, 0xae, 0x07, 0x5e, 0x1c, 0xe4, 0xd6, 0xbc, 0x21, 0x9b, //
        0xac, 0xb8, 0x71, 0xb8, 0xf2, 0x0f, 0x9c, 0x1f, 0xcf, 0x88, //
        0xe8, 0xa3, 0x0c, 0x71, 0x53, 0x5f, 0xe2, 0xde, 0x36, 0x84, //
        0x49, 0x8e, 0x7f, 0x5f, 0xf8, 0xbb, 0x40, 0x14, 0xf4, 0xb8, //
        0xc8, 0x2e, 0x3a, 0x0e, 0x00,
    ];
    // for c in &signed_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::create_subnet::test_create_subnet_tx_serialization_with_custom_network --exact --show-output
#[test]
fn test_create_subnet_tx_serialization_with_custom_network() {
    use crate::{ids::short, txs::transferable::TransferableOut};

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
                    amount: 0x2c6874d5c663740,
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
                        0x7c, 0x63, 0x55, 0x9f, 0xf6, 0x61, 0xf9, 0x8e, //
                        0x75, 0x4d, 0xb1, 0x5f, 0xe6, 0xd5, 0x50, 0x71, //
                        0x25, 0x49, 0x1c, 0x1d, 0xbc, 0xf9, 0x67, 0xd4, //
                        0x69, 0x73, 0xfc, 0x89, 0x67, 0xf7, 0xa3, 0xdc, //
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
                    amount: 0x2c6874d625c1840,
                    sig_indices: vec![0],
                }),
                ..txs::transferable::Input::default()
            }]),
            ..txs::Tx::default()
        },
        owner: key::secp256k1::txs::OutputOwners {
            locktime: 0x00,
            threshold: 0x01,
            addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
                0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
            ]))],
        },
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
        "2gafJ6qhw4dastVU3XZmte5C2SsooL4avkPr1qMfc3rhJgBkty"
    );

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // platformvm.UnsignedCreateSubnetTx type ID
        0x00, 0x00, 0x00, 0x10, //
        //
        // network id
        0x00, 0x0f, 0x42, 0x40, //
        //
        // blockchain id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
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
        0x02, 0xc6, 0x87, 0x4d, 0x5c, 0x66, 0x37, 0x40, //
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
        0x7c, 0x63, 0x55, 0x9f, 0xf6, 0x61, 0xf9, 0x8e, 0x75, 0x4d, //
        0xb1, 0x5f, 0xe6, 0xd5, 0x50, 0x71, 0x25, 0x49, 0x1c, 0x1d, //
        0xbc, 0xf9, 0x67, 0xd4, 0x69, 0x73, 0xfc, 0x89, 0x67, 0xf7, //
        0xa3, 0xdc, //
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
        0x02, 0xc6, 0x87, 0x4d, 0x62, 0x5c, 0x18, 0x40, //
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
        // secp256k1fx.OutputOwner type ID
        0x00, 0x00, 0x00, 0x0b, //
        //
        // output_owners.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //
        // output_owners.threshold
        0x00, 0x00, 0x00, 0x01, //
        // output_owners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // output_owners.addrs[0]
        0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
        0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
        //
        // number of of credentials (avax.Tx.fx_creds.len())
        0x00, 0x00, 0x00, 0x01, //
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
        0xee, 0x3e, 0x13, 0x18, 0xca, 0x62, 0x9b, 0x00, 0x42, 0x82, //
        0x4b, 0x6a, 0x12, 0x20, 0xd3, 0xfc, 0xda, 0x63, 0xdb, 0x51, //
        0xf5, 0xd0, 0xe2, 0x62, 0x63, 0x43, 0x11, 0x07, 0xdb, 0x70, //
        0x53, 0xf6, 0x0c, 0x34, 0x80, 0xf5, 0x2a, 0x93, 0x68, 0x28, //
        0xc5, 0xeb, 0x1b, 0x41, 0xdd, 0x7b, 0x3d, 0x6d, 0x08, 0x35, //
        0x7c, 0x03, 0xd9, 0xed, 0xe6, 0x90, 0x68, 0xff, 0x00, 0x70, //
        0x9d, 0x15, 0x03, 0x44, 0x00,
    ];
    // for c in &signed_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}
