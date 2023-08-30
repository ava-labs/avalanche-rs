//! Base import transaction type.
use crate::{
    avm::txs::fx,
    codec,
    errors::{Error, Result},
    hash, ids, key, platformvm, txs,
};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#ImportTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// Tx.ID() is non-empty.
    pub base_tx: txs::Tx,
    pub source_chain_id: ids::Id,
    pub source_chain_transferable_inputs: Option<Vec<txs::transferable::Input>>,
    pub fx_creds: Vec<fx::Credential>,
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
            source_chain_id: ids::Id::default(),
            source_chain_transferable_inputs: None,
            fx_creds: Vec::new(),
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
        "avm.ImportTx".to_string()
    }

    pub fn type_id() -> u32 {
        *(codec::X_TYPES.get(&Self::type_name()).unwrap()) as u32
    }

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#Tx.SignSECP256K1Fx>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/crypto#PrivateKeyED25519.SignHash>
    pub async fn sign<T: key::secp256k1::SignOnly>(&mut self, signers: Vec<Vec<T>>) -> Result<()> {
        // marshal "unsigned tx" with the codec version
        let type_id = Self::type_id();
        let packer = self.base_tx.pack(codec::VERSION, type_id)?;

        // "avalanchego" marshals the whole struct again for signed bytes
        // even when the underlying "unsigned_tx" is already once marshaled
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#Tx.SignSECP256K1Fx
        //
        // reuse the underlying packer to avoid marshaling the unsigned tx twice
        // just marshal the next fields in the struct and pack them all together
        // in the existing packer
        let b = packer.take_bytes();
        packer.set_bytes(&b);

        // pack the second field in the struct
        packer.pack_bytes(self.source_chain_id.as_ref())?;

        // pack the third field in the struct
        if self.source_chain_transferable_inputs.is_some() {
            let source_chain_transferable_inputs =
                self.source_chain_transferable_inputs.as_ref().unwrap();
            packer.pack_u32(source_chain_transferable_inputs.len() as u32)?;

            for transferable_input in source_chain_transferable_inputs.iter() {
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
                        });
                    }
                }
            }
        } else {
            packer.pack_u32(0_u32)?;
        }

        // take bytes just for hashing computation
        let tx_bytes_with_no_signature = packer.take_bytes();
        packer.set_bytes(&tx_bytes_with_no_signature);

        // compute sha256 for marshaled "unsigned tx" bytes
        // IMPORTANT: take the hash only for the type "avm.ImportTx" unsigned tx
        // not other fields -- only hash "avm.ImportTx.*" but not "avm.Tx.Creds"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#ImportTx
        let tx_bytes_hash = hash::sha256(&tx_bytes_with_no_signature);

        // number of of credentials
        let fx_creds_len = signers.len() as u32;
        // pack the fourth field in the struct
        packer.pack_u32(fx_creds_len)?;

        // sign the hash with the signers (in case of multi-sig)
        // and combine all signatures into a secp256k1fx credential
        self.fx_creds = Vec::new();
        for keys in signers.iter() {
            let mut sigs: Vec<Vec<u8>> = Vec::new();
            for k in keys.iter() {
                let sig = k.sign_digest(&tx_bytes_hash).await?;
                sigs.push(Vec::from(sig));
            }

            let mut cred = key::secp256k1::txs::Credential::default();
            cred.signatures = sigs;

            let mut fx_cred = fx::Credential::default();
            fx_cred.cred = cred;

            // add a new credential to "Tx"
            self.fx_creds.push(fx_cred);
        }
        if fx_creds_len > 0 {
            // pack each "fx_cred" which is "secp256k1fx.Credential"
            // marshal type ID for "secp256k1fx.Credential"
            let cred_type_id = key::secp256k1::txs::Credential::type_id();
            for fx_cred in self.fx_creds.iter() {
                packer.pack_u32(cred_type_id)?;
                packer.pack_u32(fx_cred.cred.signatures.len() as u32)?;
                for sig in fx_cred.cred.signatures.iter() {
                    packer.pack_bytes(sig)?;
                }
            }
        }
        let tx_bytes_with_signatures = packer.take_bytes();
        let tx_id = hash::sha256(&tx_bytes_with_signatures);

        // update "BaseTx.Metadata" with id/unsigned bytes/bytes
        // ref. "avalanchego/vms/avm.Tx.SignSECP256K1Fx"
        // ref. "avalanchego/vms/components/avax.BaseTx.Metadata.Initialize"
        self.base_tx.metadata = Some(txs::Metadata {
            id: ids::Id::from_slice(&tx_id),
            tx_bytes_with_no_signature: tx_bytes_with_no_signature.to_vec(),
            tx_bytes_with_signatures: tx_bytes_with_signatures.to_vec(),
        });

        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- avm::txs::import::test_import_tx_serialization_with_two_signers --exact --show-output
/// ref. "avalanchego/vms/avm.TestImportTxSerialization"
#[test]
fn test_import_tx_serialization_with_two_signers() {
    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let mut tx = Tx {
        base_tx: txs::Tx {
            network_id: 2,
            blockchain_id: ids::Id::from_slice(&<Vec<u8>>::from([
                0xff, 0xff, 0xff, 0xff, 0xee, 0xee, 0xee, 0xee, //
                0xdd, 0xdd, 0xdd, 0xdd, 0xcc, 0xcc, 0xcc, 0xcc, //
                0xbb, 0xbb, 0xbb, 0xbb, 0xaa, 0xaa, 0xaa, 0xaa, //
                0x99, 0x99, 0x99, 0x99, 0x88, 0x88, 0x88, 0x88, //
            ])),
            memo: Some(vec![0x00, 0x01, 0x02, 0x03]),
            ..txs::Tx::default()
        },
        source_chain_id: ids::Id::from_slice(&<Vec<u8>>::from([
            0x1f, 0x8f, 0x9f, 0x0f, 0x1e, 0x8e, 0x9e, 0x0e, //
            0x2d, 0x7d, 0xad, 0xfd, 0x2c, 0x7c, 0xac, 0xfc, //
            0x3b, 0x6b, 0xbb, 0xeb, 0x3a, 0x6a, 0xba, 0xea, //
            0x49, 0x59, 0xc9, 0xd9, 0x48, 0x58, 0xc8, 0xd8, //
        ])),
        source_chain_transferable_inputs: Some(vec![txs::transferable::Input {
            utxo_id: txs::utxo::Id {
                tx_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x0f, 0x2f, 0x4f, 0x6f, 0x8e, 0xae, 0xce, 0xee, //
                    0x0d, 0x2d, 0x4d, 0x6d, 0x8c, 0xac, 0xcc, 0xec, //
                    0x0b, 0x2b, 0x4b, 0x6b, 0x8a, 0xaa, 0xca, 0xea, //
                    0x09, 0x29, 0x49, 0x69, 0x88, 0xa8, 0xc8, 0xe8, //
                ])),
                ..txs::utxo::Id::default()
            },
            asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                0x1f, 0x3f, 0x5f, 0x7f, 0x9e, 0xbe, 0xde, 0xfe, //
                0x1d, 0x3d, 0x5d, 0x7d, 0x9c, 0xbc, 0xdc, 0xfc, //
                0x1b, 0x3b, 0x5b, 0x7b, 0x9a, 0xba, 0xda, 0xfa, //
                0x19, 0x39, 0x59, 0x79, 0x98, 0xb8, 0xd8, 0xf8, //
            ])),
            transfer_input: Some(key::secp256k1::txs::transfer::Input {
                amount: 1000,
                sig_indices: vec![0],
            }),
            ..txs::transferable::Input::default()
        }]),
        ..Tx::default()
    };

    // ref. "avalanchego/vms/avm/vm_test.go"
    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-24jUJ9vZexUM6expyMcT48LBx27k1m7xpraoV62oSQAHdziao5",
    )
    .expect("failed to load private key");
    let keys1: Vec<key::secp256k1::private_key::Key> = vec![test_key.clone(), test_key.clone()];
    let keys2: Vec<key::secp256k1::private_key::Key> = vec![test_key.clone(), test_key.clone()];
    let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys1, keys2];
    ab!(tx.sign(signers)).expect("failed to sign");
    let tx_metadata = tx.base_tx.metadata.clone().unwrap();
    let tx_bytes_with_signatures = tx_metadata.tx_bytes_with_signatures;
    assert_eq!(
        tx.tx_id().to_string(),
        "pCW7sVBytzdZ1WrqzGY1DvA2S9UaMr72xpUMxVyx1QHBARNYx"
    );

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // avm.ImportTx type ID
        0x00, 0x00, 0x00, 0x03, //
        //
        // network id
        0x00, 0x00, 0x00, 0x02, //
        //
        // blockchain id
        0xff, 0xff, 0xff, 0xff, 0xee, 0xee, 0xee, 0xee, //
        0xdd, 0xdd, 0xdd, 0xdd, 0xcc, 0xcc, 0xcc, 0xcc, //
        0xbb, 0xbb, 0xbb, 0xbb, 0xaa, 0xaa, 0xaa, 0xaa, //
        0x99, 0x99, 0x99, 0x99, 0x88, 0x88, 0x88, 0x88, //
        //
        // outs.len()
        0x00, 0x00, 0x00, 0x00, //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // ins.len()
        0x00, 0x00, 0x00, 0x00, //
        //
        // memo.len()
        0x00, 0x00, 0x00, 0x04, //
        //
        // memo
        0x00, 0x01, 0x02, 0x03, //
        //
        // Tx.source_chain
        0x1f, 0x8f, 0x9f, 0x0f, 0x1e, 0x8e, 0x9e, 0x0e, //
        0x2d, 0x7d, 0xad, 0xfd, 0x2c, 0x7c, 0xac, 0xfc, //
        0x3b, 0x6b, 0xbb, 0xeb, 0x3a, 0x6a, 0xba, 0xea, //
        0x49, 0x59, 0xc9, 0xd9, 0x48, 0x58, 0xc8, 0xd8, //
        //
        // Tx.source_chain_ins.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "Tx.source_chain_ins[0]" TransferableInput.utxo_id.tx_id
        0x0f, 0x2f, 0x4f, 0x6f, 0x8e, 0xae, 0xce, 0xee, //
        0x0d, 0x2d, 0x4d, 0x6d, 0x8c, 0xac, 0xcc, 0xec, //
        0x0b, 0x2b, 0x4b, 0x6b, 0x8a, 0xaa, 0xca, 0xea, //
        0x09, 0x29, 0x49, 0x69, 0x88, 0xa8, 0xc8, 0xe8, //
        //
        // "Tx.source_chain_ins[0]" TransferableInput.utxo_id.output_index
        0x00, 0x00, 0x00, 0x00, //
        //
        // "Tx.source_chain_ins[0]" TransferableInput.asset_id
        0x1f, 0x3f, 0x5f, 0x7f, 0x9e, 0xbe, 0xde, 0xfe, //
        0x1d, 0x3d, 0x5d, 0x7d, 0x9c, 0xbc, 0xdc, 0xfc, //
        0x1b, 0x3b, 0x5b, 0x7b, 0x9a, 0xba, 0xda, 0xfa, //
        0x19, 0x39, 0x59, 0x79, 0x98, 0xb8, 0xd8, 0xf8, //
        //
        // "Tx.source_chain_ins[0]" secp256k1fx.TransferInput type ID
        0x00, 0x00, 0x00, 0x05, //
        //
        // "Tx.source_chain_ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.amount
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, //
        //
        // "Tx.source_chain_ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "Tx.source_chain_ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices[0]
        0x00, 0x00, 0x00, 0x00, //
        //
        // number of of credentials (avax.Tx.fx_creds.len())
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
        0x00, 0x00, 0x00, 0x02, //
        //
        // first 65-byte signature
        0x8c, 0xc7, 0xdc, 0x8c, 0x11, 0xd3, 0x75, 0x9e, 0x16, 0xa5, //
        0x9f, 0xd2, 0x9c, 0x64, 0xd7, 0x1f, 0x9b, 0xad, 0x1a, 0x62, //
        0x33, 0x98, 0xc7, 0xaf, 0x67, 0x02, 0xc5, 0xe0, 0x75, 0x8e, //
        0x62, 0xcf, 0x15, 0x6d, 0x99, 0xf5, 0x4e, 0x71, 0xb8, 0xf4, //
        0x8b, 0x5b, 0xbf, 0x0c, 0x59, 0x62, 0x79, 0x34, 0x97, 0x1a, //
        0x1f, 0x49, 0x9b, 0x0a, 0x4f, 0xbf, 0x95, 0xfc, 0x31, 0x39, //
        0x46, 0x4e, 0xa1, 0xaf, 0x00, //
        //
        // second 65-byte signature
        0x8c, 0xc7, 0xdc, 0x8c, 0x11, 0xd3, 0x75, 0x9e, 0x16, 0xa5, //
        0x9f, 0xd2, 0x9c, 0x64, 0xd7, 0x1f, 0x9b, 0xad, 0x1a, 0x62, //
        0x33, 0x98, 0xc7, 0xaf, 0x67, 0x02, 0xc5, 0xe0, 0x75, 0x8e, //
        0x62, 0xcf, 0x15, 0x6d, 0x99, 0xf5, 0x4e, 0x71, 0xb8, 0xf4, //
        0x8b, 0x5b, 0xbf, 0x0c, 0x59, 0x62, 0x79, 0x34, 0x97, 0x1a, //
        0x1f, 0x49, 0x9b, 0x0a, 0x4f, 0xbf, 0x95, 0xfc, 0x31, 0x39, //
        0x46, 0x4e, 0xa1, 0xaf, 0x00, //
        //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // struct field type ID "fx::Credential.cred"
        // "secp256k1fx.Credential" type ID
        0x00, 0x00, 0x00, 0x09, //
        //
        // number of signers ("fx::Credential.cred.sigs.len()")
        0x00, 0x00, 0x00, 0x02, //
        //
        // first 65-byte signature
        0x8c, 0xc7, 0xdc, 0x8c, 0x11, 0xd3, 0x75, 0x9e, 0x16, 0xa5, //
        0x9f, 0xd2, 0x9c, 0x64, 0xd7, 0x1f, 0x9b, 0xad, 0x1a, 0x62, //
        0x33, 0x98, 0xc7, 0xaf, 0x67, 0x02, 0xc5, 0xe0, 0x75, 0x8e, //
        0x62, 0xcf, 0x15, 0x6d, 0x99, 0xf5, 0x4e, 0x71, 0xb8, 0xf4, //
        0x8b, 0x5b, 0xbf, 0x0c, 0x59, 0x62, 0x79, 0x34, 0x97, 0x1a, //
        0x1f, 0x49, 0x9b, 0x0a, 0x4f, 0xbf, 0x95, 0xfc, 0x31, 0x39, //
        0x46, 0x4e, 0xa1, 0xaf, 0x00, //
        //
        // second 65-byte signature
        0x8c, 0xc7, 0xdc, 0x8c, 0x11, 0xd3, 0x75, 0x9e, 0x16, 0xa5, //
        0x9f, 0xd2, 0x9c, 0x64, 0xd7, 0x1f, 0x9b, 0xad, 0x1a, 0x62, //
        0x33, 0x98, 0xc7, 0xaf, 0x67, 0x02, 0xc5, 0xe0, 0x75, 0x8e, //
        0x62, 0xcf, 0x15, 0x6d, 0x99, 0xf5, 0x4e, 0x71, 0xb8, 0xf4, //
        0x8b, 0x5b, 0xbf, 0x0c, 0x59, 0x62, 0x79, 0x34, 0x97, 0x1a, //
        0x1f, 0x49, 0x9b, 0x0a, 0x4f, 0xbf, 0x95, 0xfc, 0x31, 0x39, //
        0x46, 0x4e, 0xa1, 0xaf, 0x00, //
    ];
    // for c in &signed_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}
