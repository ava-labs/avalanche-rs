use crate::{codec, errors::Result, hash, ids, key, packer::Packable, platformvm, txs};
use serde::{Deserialize, Serialize};

/// ref. <https://github.com/ava-labs/avalanchego/blob/master/vms/platformvm/txs/add_permissionless_validator_tx.go>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#AddPermissionlessValidatorTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// Tx.ID() is non-empty.
    pub base_tx: txs::Tx,
    pub validator: platformvm::txs::Validator,

    /// ID of the subnet this validator is validating.
    /// ref. "github.com/ava-labs/avalanchego/utils/constants.PrimaryNetworkID" (ids.Empty).
    #[serde(rename = "subnetID")]
    pub subnet_id: ids::Id,
    /// If the \[subnet_id\] is the primary network,
    /// \[signer\] is the BLS key for this validator.
    /// If the \[subnet_id\] is not the primary network,
    /// \[signer\] is empty.
    pub signer: Option<key::bls::ProofOfPossession>,

    #[serde(rename = "stake")]
    pub stake_transferable_outputs: Option<Vec<txs::transferable::Output>>,

    pub validator_rewards_owner: key::secp256k1::txs::OutputOwners,
    pub delegator_rewards_owner: key::secp256k1::txs::OutputOwners,

    #[serde(rename = "shares")]
    pub delegation_shares: u32,

    /// To be updated after signing.
    pub creds: Vec<key::secp256k1::txs::Credential>,
}

impl Default for Tx {
    fn default() -> Self {
        Self {
            base_tx: txs::Tx::default(),
            validator: platformvm::txs::Validator::default(),
            subnet_id: ids::Id::empty(), // primary network
            signer: None,
            stake_transferable_outputs: None,
            validator_rewards_owner: key::secp256k1::txs::OutputOwners::default(),
            delegator_rewards_owner: key::secp256k1::txs::OutputOwners::default(),
            delegation_shares: 0,
            creds: Vec::new(),
        }
    }
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
        "platformvm.AddPermissionlessValidatorTx".to_string()
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
        let packer = self.base_tx.pack(type_id)?;

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

        // pack the third field "subnet_id" in the struct
        packer.pack_bytes(self.subnet_id.as_ref())?;

        // pack the fourth field "signer"
        if let Some(signer) = &self.signer {
            let type_id_signer: u32 = 28;
            packer.pack_u32(type_id_signer)?;
            packer.pack_bytes(&signer.public_key)?;
            packer.pack_bytes(&signer.proof_of_possession)?;
        } else {
            // empty signer for non-primary network
            let type_id_signer: u32 = 27;
            packer.pack_u32(type_id_signer)?;
        }

        // pack the third field "stake" in the struct
        self.stake_transferable_outputs.pack(&packer)?;

        // pack the fourth field "reward_owner" in the struct
        // not embedded thus encode struct type id
        let output_owners_type_id = key::secp256k1::txs::OutputOwners::type_id();
        packer.pack_u32(output_owners_type_id)?;
        packer.pack_u64(self.validator_rewards_owner.locktime)?;
        packer.pack_u32(self.validator_rewards_owner.threshold)?;
        self.validator_rewards_owner.addresses.pack(&packer)?;

        packer.pack_u32(output_owners_type_id)?;
        packer.pack_u64(self.delegator_rewards_owner.locktime)?;
        packer.pack_u32(self.delegator_rewards_owner.threshold)?;
        self.delegator_rewards_owner.addresses.pack(&packer)?;

        // pack the fifth field "shares" in the struct
        packer.pack_u32(self.delegation_shares)?;

        // take bytes just for hashing computation
        let tx_bytes_with_no_signature = packer.take_bytes();
        packer.set_bytes(&tx_bytes_with_no_signature);

        // compute sha256 for marshaled "unsigned tx" bytes
        // IMPORTANT: take the hash only for the type "platformvm.AddPermissionlessValidatorTx" unsigned tx
        // not other fields -- only hash "platformvm.AddPermissionlessValidatorTx.*" but not "platformvm.Tx.Creds"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#UnsignedAddPermissionlessValidatorTx
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::add_permissionless_validator::test_add_permissionless_validator_tx_serialization_with_one_signer --exact --show-output
#[test]
fn test_add_permissionless_validator_tx_serialization_with_one_signer() {
    use crate::{
        ids::{node, short},
        txs::transferable::TransferableOut,
    };
    use std::str::FromStr;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let sk = key::secp256k1::private_key::Key::from_cb58(
        "24jUJ9vZexUM6expyMcT48LBx27k1m7xpraoV62oSQAHdziao5",
    )
    .unwrap();

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
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![sk.to_public_key().to_short_id().unwrap()],
                    },
                }),
                ..txs::transferable::Output::default()
            }]),
            transferable_inputs: Some(vec![txs::transferable::Input {
                utxo_id: txs::utxo::Id {
                    tx_id: ids::Id::from_slice(&<Vec<u8>>::from([
                        0x74, 0x78, 0x49, 0x44,
                    ])),
                    output_index: 2,
                    ..txs::utxo::Id::default()
                },
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                    0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                    0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                    0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
                ])),
                transfer_input: Some(key::secp256k1::txs::transfer::Input {
                    amount: 5678,
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
            weight: 0x7e7,
        },
        subnet_id: ids::Id::from_str("2u5EYNkXMDFNi4pL9eGBt2F5DnXLGriecu7Ctje8jK155FFkPx").unwrap(),
        signer: Some(key::bls::ProofOfPossession{
            public_key: hex::decode("0x8f95423f7142d00a48e1014a3de8d28907d420dc33b3052a6dee03a3f2941a393c2351e354704ca66a3fc29870282e15".trim_start_matches("0x")).unwrap(),
            proof_of_possession: hex::decode("0x86a3ab4c45cfe31cae34c1d06f212434ac71b1be6cfe046c80c162e057614a94a5bc9f1ded1a7029deb0ba4ca7c9b71411e293438691be79c2dbf19d1ca7c3eadb9c756246fc5de5b7b89511c7d7302ae051d9e03d7991138299b5ed6a570a98".trim_start_matches("0x")).unwrap(),
            ..Default::default()
        }),
        stake_transferable_outputs: Some(vec![txs::transferable::Output {
            asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
            ])),
            out: TransferableOut::StakeableLockOut(platformvm::txs::StakeableLockOut{
                locktime:0,
                transfer_output: key::secp256k1::txs::transfer::Output {
                    amount: 0x7e7,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                            0xfc,0xed,0xa8,0xf9,0x0f,0xcb,0x5d,0x30,0x61,0x4b, //
                            0x99,0xd7,0x9f,0xc4,0xba,0xa2,0x93,0x07,0x76,0x26, //
                        ]))],
                    },
                },
            }),
            ..txs::transferable::Output::default()
        }]),
        validator_rewards_owner: key::secp256k1::txs::OutputOwners {
            locktime: 0,
            threshold: 1,
            addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                            0xfc,0xed,0xa8,0xf9,0x0f,0xcb,0x5d,0x30,0x61,0x4b, //
                            0x99,0xd7,0x9f,0xc4,0xba,0xa2,0x93,0x07,0x76,0x26, //
                        ]))],
        },
        delegator_rewards_owner: key::secp256k1::txs::OutputOwners {
            locktime: 0,
            threshold: 1,
            addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                            0xfc,0xed,0xa8,0xf9,0x0f,0xcb,0x5d,0x30,0x61,0x4b, //
                            0x99,0xd7,0x9f,0xc4,0xba,0xa2,0x93,0x07,0x76,0x26, //
                        ]))],
        },
        delegation_shares: 1_000_000,
        ..Tx::default()
    };

    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-24jUJ9vZexUM6expyMcT48LBx27k1m7xpraoV62oSQAHdziao5",
    )
    .expect("failed to load private key");
    let keys1: Vec<key::secp256k1::private_key::Key> = vec![test_key];
    let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys1];
    ab!(tx.sign(signers)).expect("failed to sign");
    let tx_metadata = tx.base_tx.metadata.clone().unwrap();
    let tx_bytes_with_signatures = tx_metadata.tx_bytes_with_signatures;
    log::info!("tx id {}", tx.tx_id().to_string());
    assert_eq!(
        tx.tx_id().to_string(),
        "22tDNpLuSpTfv8dweokq22KCo8hVTK4o2mgBESg1XQGHJegve5"
    );

    // for (i, c) in tx_bytes_with_signatures.iter().enumerate() {
    //     print!("0x{:02x},", *c);
    //     if i > 0 && i % 20 == 0 {
    //         println!();
    //     }
    // }

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // platformvm.UnsignedAddPermissionlessValidatorTx type ID
        0x00, 0x00, 0x00, 0x19, //
        //
        // network id
        0x00, 0x0f, 0x42, 0x40, //
        //
        // blockchain id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // outs.len()
        0x00, 0x00, 0x00, 0x01, //
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
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        //
        // ins.len()
        0x00, 0x00, 0x00, 0x01, //
        // "ins[0]" TransferableInput.utxo_id.tx_id
        0x74, 0x78, 0x49, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // "ins[0]" TransferableInput.utxo_id.output_index
        0x00, 0x00, 0x00, 0x02, //
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
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x16, 0x2e, //
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
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xe7, //
        // subnet id
        0xf9, 0xef, 0x27, 0x25, 0xf6, 0x61, 0x9b, 0x92, 0x3f, 0x1e, 0x84, 0xbf, 0x34, 0x81, 0xd5,
        0x3f, 0xd0, 0x7e, 0x2b, 0xa4, 0xbc, 0x49, 0xcc, 0xf5, 0xa6, 0x9e, 0x9a, 0xc7, 0x36, 0x73,
        0x4e, 0x1a, //
        // proof of possession type id
        0x00, 0x00, 0x00, 0x1c, //
        //
        // proof of possession public key 48-byte
        0x8f, 0x95, 0x42, 0x3f, 0x71, 0x42, 0xd0, 0x0a, 0x48, 0xe1, //
        0x01, 0x4a, 0x3d, 0xe8, 0xd2, 0x89, 0x07, 0xd4, 0x20, 0xdc, //
        0x33, 0xb3, 0x05, 0x2a, 0x6d, 0xee, 0x03, 0xa3, 0xf2, 0x94, //
        0x1a, 0x39, 0x3c, 0x23, 0x51, 0xe3, 0x54, 0x70, 0x4c, 0xa6, //
        0x6a, 0x3f, 0xc2, 0x98, 0x70, 0x28, 0x2e, 0x15, //
        //
        // proof of possession, 96-byte
        0x86, 0xa3, 0xab, 0x4c, 0x45, 0xcf, 0xe3, 0x1c, 0xae, 0x34, //
        0xc1, 0xd0, 0x6f, 0x21, 0x24, 0x34, 0xac, 0x71, 0xb1, 0xbe, //
        0x6c, 0xfe, 0x04, 0x6c, 0x80, 0xc1, 0x62, 0xe0, 0x57, 0x61, //
        0x4a, 0x94, 0xa5, 0xbc, 0x9f, 0x1d, 0xed, 0x1a, 0x70, 0x29, //
        0xde, 0xb0, 0xba, 0x4c, 0xa7, 0xc9, 0xb7, 0x14, 0x11, 0xe2, //
        0x93, 0x43, 0x86, 0x91, 0xbe, 0x79, 0xc2, 0xdb, 0xf1, 0x9d, //
        0x1c, 0xa7, 0xc3, 0xea, 0xdb, 0x9c, 0x75, 0x62, 0x46, 0xfc, //
        0x5d, 0xe5, 0xb7, 0xb8, 0x95, 0x11, 0xc7, 0xd7, 0x30, 0x2a, //
        0xe0, 0x51, 0xd9, 0xe0, 0x3d, 0x79, 0x91, 0x13, 0x82, 0x99, //
        0xb5, 0xed, 0x6a, 0x57, 0x0a, 0x98, //
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
        // platformvm.StakeableLockOut type ID
        0x00, 0x00, 0x00, 0x16, //
        //
        // locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // secp256k1fx.TransferOutput type ID
        0x00, 0x00, 0x00, 0x07, //
        // amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xe7, //
        //
        // secp256k1fx.OutputOwners.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //
        // secp256k1fx.OutputOwners.threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.OutputOwners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.OutputOwners.addrs[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        //
        // secp256k1fx.OutputOwners type id
        0x00, 0x00, 0x00, 0x0b, //
        //
        // secp256k1fx.OutputOwners locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // secp256k1fx.OutputOwners threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.OutputOwners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.OutputOwners.addrs[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        //
        // secp256k1fx.OutputOwners type id
        0x00, 0x00, 0x00, 0x0b, //
        // secp256k1fx.OutputOwners locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // secp256k1fx.OutputOwners threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.OutputOwners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.OutputOwners.addrs[0]
        0xfc, 0xed, 0xa8, 0xf9, 0x0f, 0xcb, 0x5d, 0x30, 0x61, 0x4b, //
        0x99, 0xd7, 0x9f, 0xc4, 0xba, 0xa2, 0x93, 0x07, 0x76, 0x26, //
        //
        // delegation_shares
        0x00, 0x0f, 0x42, 0x40, //
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
        0xfc, 0x13, 0x6a, 0x2d, 0x14, 0x0d, 0x7e, 0xdf, 0xdc, 0x87, //
        0xa4, 0x13, 0xcd, 0x8f, 0xdf, 0xa6, 0x80, 0xdd, 0x07, 0x69, //
        0xf3, 0x61, 0xdc, 0x22, 0x7f, 0xe4, 0x84, 0x53, 0x47, 0xec, //
        0xda, 0xd7, 0x06, 0x93, 0x96, 0x9a, 0x45, 0x35, 0xe2, 0x51, //
        0x71, 0x94, 0x84, 0xe2, 0xe5, 0x52, 0xb1, 0x53, 0xe7, 0x66, //
        0xde, 0x74, 0x2b, 0x3c, 0x24, 0x52, 0x66, 0xc9, 0x29, 0x45, //
        0xe7, 0x98, 0x99, 0xac, 0x00, //
    ];
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}
