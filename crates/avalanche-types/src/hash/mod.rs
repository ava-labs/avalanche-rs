//! Hashing utilities.
//!
//! Includes SHA256, SHA256+RIPEMD160, and Keccak256.
use std::io::{self, Error, ErrorKind};

use ripemd::{Digest, Ripemd160};
use sha3::Keccak256;

#[cfg(all(not(windows)))]
use ring::digest::{digest, SHA256};

#[cfg(all(not(windows)))]
pub const SHA256_OUTPUT_LEN: usize = ring::digest::SHA256_OUTPUT_LEN;

#[cfg(all(windows))]
pub const SHA256_OUTPUT_LEN: usize = 32;

/// Returns SHA256 digest of the given data.
#[cfg(all(not(windows)))]
pub fn sha256(d: impl AsRef<[u8]>) -> Vec<u8> {
    digest(&SHA256, d.as_ref()).as_ref().into()
}

/// Returns SHA256 digest of the given data.
/// TODO: implement this
#[cfg(all(windows))]
pub fn sha256(b: impl AsRef<[u8]>) -> Vec<u8> {
    panic!("unimplemented")
}

/// Converts bytes to the short address bytes (20-byte).
/// e.g., "hashing.PubkeyBytesToAddress" and "ids.ToShortID"
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
pub fn sha256_ripemd160<B>(b: B) -> io::Result<Vec<u8>>
where
    B: AsRef<[u8]>,
{
    let digest_sha256 = sha256(b);

    // "hashing.PubkeyBytesToAddress"
    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 20]
    // already in "type ShortID [20]byte" format
    let sha256_ripemd160 = Ripemd160::digest(&digest_sha256);

    // "ids.ToShortID" merely enforces "ripemd160" size!
    // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ToShortID
    if sha256_ripemd160.len() != 20 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "ripemd160 of sha256 must be 20-byte, got {}",
                sha256_ripemd160.len()
            ),
        ));
    }

    Ok(sha256_ripemd160.to_vec())
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- hash::test_sha256_ripemd160 --exact --show-output
#[test]
fn test_sha256_ripemd160() {
    let d = sha256_ripemd160(&<Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
        0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
        0x59, 0xa7,
    ]))
    .unwrap();
    assert_eq!(d.len(), 20);
}

/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/utils/hash.rs> "keccak256"
pub fn keccak256(b: impl AsRef<[u8]>) -> primitive_types::H256 {
    primitive_types::H256::from_slice(&Keccak256::digest(b.as_ref()))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- hash::test_keccak256 --exact --show-output
#[test]
fn test_keccak256() {
    let digest_input = <Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
        0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
        0x59, 0xa7,
    ]);

    let d1 = keccak256(&digest_input);
    assert_eq!(d1.0.len(), 32);

    // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/utils/hash.rs> "keccak256"
    let d2 = ethers_core::utils::keccak256(&digest_input);
    let d2 = primitive_types::H256::from_slice(&d2);
    assert_eq!(d1, d2);
}
