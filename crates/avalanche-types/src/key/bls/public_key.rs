//! BLS public key module.
use std::io::{self, Error, ErrorKind};

use crate::key::bls::signature::Sig;
use blst::min_pk::{AggregatePublicKey, PublicKey};

/// Represents "blst::min_pk::PublicKey".
/// By default, serializes as hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Key(pub PublicKey);

pub const LEN: usize = 48;

impl Key {
    /// Converts the public key to compressed bytes.
    /// ref. "avalanchego/utils/crypto/bls.PublicKeyToBytes"
    pub fn to_compressed_bytes(&self) -> [u8; LEN] {
        self.0.compress()
    }

    /// Loads the public key from the compressed raw scalar bytes (in big endian).
    pub fn from_bytes(compressed: &[u8]) -> io::Result<Self> {
        let pubkey = PublicKey::uncompress(compressed).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed blst::min_pk::PublicKey::uncompress {:?}", e),
            )
        })?;
        pubkey.validate().map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed blst::min_pk::PublicKey::validate {:?}", e),
            )
        })?;

        Ok(Self(pubkey))
    }

    /// Verifies the message and the validity of its signature.
    /// Invariant: [self.0] and \[sig\] have both been validated.
    /// ref. "avalanchego/utils/crypto/bls.Verify"
    pub fn verify(&self, msg: &[u8], sig: &Sig) -> bool {
        sig.verify(msg, self)
    }

    /// Verifies the message and the validity of its signature.
    /// Invariant: [self.0] and \[sig\] have both been validated.
    /// ref. "avalanchego/utils/crypto/bls.VerifyProofOfPossession"
    pub fn verify_proof_of_possession(&self, msg: &[u8], sig: &Sig) -> bool {
        sig.verify_proof_of_possession(msg, self)
    }
}

impl From<PublicKey> for Key {
    fn from(pubkey: PublicKey) -> Self {
        Self(pubkey)
    }
}

impl From<Key> for PublicKey {
    fn from(k: Key) -> Self {
        k.0
    }
}

pub fn aggregate(pubkeys: &[Key]) -> io::Result<Key> {
    let ss = pubkeys.iter().map(|s| &s.0).collect::<Vec<_>>();

    let agg_pubkey = AggregatePublicKey::aggregate(&ss, false).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed AggregatePublicKey::aggregate {:?}", e),
        )
    })?;
    Ok(Key(agg_pubkey.to_public_key()))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::bls::public_key::test_key --exact --show-output
#[test]
fn test_key() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let sk = crate::key::bls::private_key::Key::generate().unwrap();
    let pubkey = sk.to_public_key();

    let msg = random_manager::secure_bytes(50).unwrap();
    let sig = sk.sign(&msg);
    assert!(pubkey.verify(&msg, &sig));
    assert!(!pubkey.verify_proof_of_possession(&msg, &sig));

    let sig_pos = sk.sign_proof_of_possession(&msg);
    assert!(!pubkey.verify(&msg, &sig_pos));
    assert!(pubkey.verify_proof_of_possession(&msg, &sig_pos));

    let b = pubkey.to_compressed_bytes();
    let pubkey2 = Key::from_bytes(&b).unwrap();
    assert_eq!(pubkey, pubkey2);
}
