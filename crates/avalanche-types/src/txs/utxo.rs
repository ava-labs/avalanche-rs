use std::cmp::Ordering;

use crate::{
    codec,
    errors::{Error, Result},
    formatting,
    ids::{self, short},
    key, packer, platformvm,
};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#UTXOID>
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Id {
    #[serde(rename = "txID")]
    pub tx_id: ids::Id,
    #[serde(rename = "outputIndex")]
    pub output_index: u32,

    #[serde(skip)]
    pub symbol: bool,
    #[serde(skip)]
    pub id: ids::Id,
}

impl Default for Id {
    fn default() -> Self {
        Self::default()
    }
}

impl Id {
    pub fn default() -> Self {
        Self {
            tx_id: ids::Id::empty(),
            output_index: 0,
            symbol: false,
            id: ids::Id::empty(),
        }
    }

    pub fn new(tx_id: &[u8], output_index: u32, symbol: bool) -> Result<Self> {
        let tx_id = ids::Id::from_slice(tx_id);
        let prefixes: Vec<u64> = vec![output_index as u64];
        let id = tx_id.prefix(&prefixes)?;
        Ok(Self {
            tx_id,
            output_index,
            symbol,
            id,
        })
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Id) -> Ordering {
        self.tx_id
            .cmp(&(other.tx_id)) // returns when "tx_id"s are not Equal
            .then_with(
                || self.output_index.cmp(&other.output_index), // if "tx_id"s are Equal, compare "output_index"
            )
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#SortUTXOIDs>
/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::utxo::test_sort_utxo_ids --exact --show-output
#[test]
fn test_sort_utxo_ids() {
    let mut utxos: Vec<Id> = Vec::new();
    for i in (0..10).rev() {
        utxos.push(Id {
            tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            output_index: (i + 1) as u32,
            ..Id::default()
        });
        utxos.push(Id {
            tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            output_index: i as u32,
            ..Id::default()
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&utxos));
    utxos.sort();

    let mut sorted_utxos: Vec<Id> = Vec::new();
    for i in 0..10 {
        sorted_utxos.push(Id {
            tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            output_index: i as u32,
            ..Id::default()
        });
        sorted_utxos.push(Id {
            tx_id: ids::Id::from_slice(&vec![i as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            output_index: (i + 1) as u32,
            ..Id::default()
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_utxos));
    assert_eq!(utxos, sorted_utxos);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::utxo::test_utxo_id --exact --show-output
/// ref. "avalanchego/vms/components/avax.TestUTXOID"
#[test]
fn test_utxo_id() {
    let tx_id: Vec<u8> = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];
    let utxo_id = Id::new(&tx_id, 0x20212223, false).unwrap();

    let expected_id: Vec<u8> = vec![
        42, 202, 101, 108, 44, 18, 156, 140, 88, 220, 97, 33, 177, 172, 79, 57, 207, 131, 41, 102,
        29, 103, 184, 89, 239, 38, 187, 183, 167, 216, 160, 212,
    ];
    let expected_id = ids::Id::from_slice(&expected_id);
    assert_eq!(utxo_id.id, expected_id);
}

/// Do not parse the internal tests.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#UTXO>
/// TODO: implement ordering?
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Utxo {
    pub utxo_id: Id,
    pub asset_id: ids::Id,

    /// AvalancheGo loads "avax.UTXO" object from the db and
    /// defines the "out" field as an interface "Out verify.State".
    ///
    /// The underlying type is one of the following:
    ///
    /// "*secp256k1fx.TransferOutput"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput>
    ///
    /// "*platformvm.StakeableLockOut" which embeds "*secp256k1fx.TransferOutput"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut>
    ///
    /// MUST: only one of the following can be "Some".
    pub transfer_output: Option<key::secp256k1::txs::transfer::Output>,
    pub stakeable_lock_out: Option<platformvm::txs::StakeableLockOut>,
}

impl Default for Utxo {
    fn default() -> Self {
        Self::default()
    }
}

impl Utxo {
    pub fn default() -> Self {
        Self {
            utxo_id: Id::default(),
            asset_id: ids::Id::empty(),
            transfer_output: None,
            stakeable_lock_out: None,
        }
    }

    /// Hex-encodes the Utxo with the prepended "0x".
    pub fn to_hex(&self) -> Result<String> {
        let packer = self.pack(codec::VERSION)?;
        let b = packer.take_bytes();

        let d = formatting::encode_hex_with_checksum(&b);
        Ok(format!("0x{}", d))
    }

    /// Parses the raw hex-encoded data from the "getUTXOs" API.
    pub fn from_hex(d: &str) -> Result<Self> {
        // ref. "utils/formatting.encode" prepends "0x" for "Hex" encoding
        let d = d.trim_start_matches("0x");

        let decoded =
            formatting::decode_hex_with_checksum(d.as_bytes()).map_err(|e| Error::Other {
                message: format!("failed formatting::decode_hex_with_checksum '{}'", e),
                retryable: false,
            })?;
        Self::unpack(&decoded)
    }

    /// Packes the Utxo.
    pub fn pack(&self, codec_version: u16) -> Result<packer::Packer> {
        // ref. "avalanchego/codec.manager.Marshal", "vms/avm.newCustomCodecs"
        // ref. "math.MaxInt32" and "constants.DefaultByteSliceCap" in Go
        let packer = packer::Packer::new((1 << 31) - 1, 128);

        // codec version
        // ref. "avalanchego/codec.manager.Marshal"
        packer.pack_u16(codec_version)?;

        packer.pack_bytes(self.utxo_id.tx_id.as_ref())?;
        packer.pack_u32(self.utxo_id.output_index)?;

        packer.pack_bytes(self.asset_id.as_ref())?;

        if let Some(out) = &self.transfer_output {
            packer.pack_u32(key::secp256k1::txs::transfer::Output::type_id())?;
            packer.pack_u64(out.amount)?;

            packer.pack_u64(out.output_owners.locktime)?;
            packer.pack_u32(out.output_owners.threshold)?;

            packer.pack_u32(out.output_owners.addresses.len() as u32)?;
            for addr in out.output_owners.addresses.iter() {
                packer.pack_bytes(addr.as_ref())?;
            }
        } else if let Some(lock_out) = &self.stakeable_lock_out {
            packer.pack_u32(platformvm::txs::StakeableLockOut::type_id())?;
            packer.pack_u64(lock_out.locktime)?;

            packer.pack_u32(key::secp256k1::txs::transfer::Output::type_id())?;
            packer.pack_u64(lock_out.transfer_output.amount)?;

            packer.pack_u64(lock_out.transfer_output.output_owners.locktime)?;
            packer.pack_u32(lock_out.transfer_output.output_owners.threshold)?;

            packer.pack_u32(lock_out.transfer_output.output_owners.addresses.len() as u32)?;
            for addr in lock_out.transfer_output.output_owners.addresses.iter() {
                packer.pack_bytes(addr.as_ref())?;
            }
        }

        Ok(packer)
    }

    /// Parses raw bytes to "Utxo".
    /// It assumes the data are already decoded from "hex".
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#UTXO>
    pub fn unpack(d: &[u8]) -> Result<Self> {
        let packer = packer::Packer::load_bytes_for_unpack(d.len() + 1024, d);

        let _codec_version = packer.unpack_u16()?;

        // must unpack in the order of struct
        let tx_id_bytes = packer.unpack_bytes(ids::LEN)?;
        let tx_id = ids::Id::from_slice(&tx_id_bytes);

        let output_index = packer.unpack_u32()?;

        let asset_id_bytes = packer.unpack_bytes(ids::LEN)?;
        let asset_id = ids::Id::from_slice(&asset_id_bytes);

        // "Out verify.State" is an interface
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/avax#UTXO
        //
        // "*secp256k1fx.TransferOutput" -- type ID 7
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
        //
        // "*platformvm.StakeableLockOut" which embeds "*secp256k1fx.TransferOutput"-- type ID 22
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut
        let type_id_verify_state = packer.unpack_u32()?;
        match type_id_verify_state {
            7 => {}
            22 => {}
            _ => {
                return Err(Error::Other {
                    message: format!("unknown type ID for verify.State {}", type_id_verify_state),
                    retryable: false,
                })
            }
        }

        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#StakeableLockOut
        let stakeable_lock_out = {
            if type_id_verify_state == 22 {
                let stakeable_lock_out_locktime = packer.unpack_u64()?;

                // "*secp256k1fx.TransferOutput" -- type ID 7
                // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
                let _type_id_secp256k1fx_transfer_output = packer.unpack_u32()?;

                let mut so = platformvm::txs::StakeableLockOut::default();
                so.locktime = stakeable_lock_out_locktime;

                Some(so)
            } else {
                None
            }
        };

        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#TransferOutput
        let amount = packer.unpack_u64()?;
        let locktime = packer.unpack_u64()?;
        let threshold = packer.unpack_u32()?;
        let addr_len = packer.unpack_u32()?;
        let mut addresses: Vec<short::Id> = Vec::new();
        for _ in 0..addr_len {
            let b = packer.unpack_bytes(short::LEN)?;
            addresses.push(short::Id::from_slice(&b));
        }
        let output_owners = key::secp256k1::txs::OutputOwners {
            locktime,
            threshold,
            addresses,
        };
        let transfer_output = key::secp256k1::txs::transfer::Output {
            amount,
            output_owners,
        };

        let utxo = {
            if let Some(mut stakeable_lock_out) = stakeable_lock_out {
                stakeable_lock_out.transfer_output = transfer_output;
                Utxo {
                    utxo_id: Id {
                        tx_id,
                        output_index,
                        ..Id::default()
                    },
                    asset_id,
                    stakeable_lock_out: Some(stakeable_lock_out),
                    ..Utxo::default()
                }
            } else {
                Utxo {
                    utxo_id: Id {
                        tx_id,
                        output_index,
                        ..Id::default()
                    },
                    asset_id,
                    transfer_output: Some(transfer_output),
                    ..Utxo::default()
                }
            }
        };
        Ok(utxo)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::utxo::test_utxo_unpack_hex --exact --show-output
#[test]
fn test_utxo_unpack_hex() {
    let utxo_hex_1 = "0x000000000000000000000000000000000000000000000000000000000000000000000000000088eec2e099c6a528e689618e8721e04ae85ea574c7a15a7968644d14d54780140000000702c68af0bb1400000000000000000000000000010000000165844a05405f3662c1928142c6c2a783ef871de939b564db";
    let utxo = Utxo::from_hex(utxo_hex_1).unwrap();
    let utxo_hex_2 = utxo.to_hex().unwrap();
    assert_eq!(utxo_hex_1, utxo_hex_2);

    let expected = Utxo {
        utxo_id: Id::default(),
        asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
            136, 238, 194, 224, 153, 198, 165, 40, 230, 137, 97, 142, 135, 33, 224, 74, 232, 94,
            165, 116, 199, 161, 90, 121, 104, 100, 77, 20, 213, 71, 128, 20,
        ])),
        transfer_output: Some(key::secp256k1::txs::transfer::Output {
            amount: 200000000000000000,
            output_owners: key::secp256k1::txs::OutputOwners {
                locktime: 0,
                threshold: 1,
                addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                    101, 132, 74, 5, 64, 95, 54, 98, 193, 146, 129, 66, 198, 194, 167, 131, 239,
                    135, 29, 233,
                ]))],
            },
        }),
        ..Utxo::default()
    };
    assert_eq!(utxo, expected);

    println!("{:?}", utxo);
}
