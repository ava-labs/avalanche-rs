//! BLS signature module.
use std::io::{self, Error, ErrorKind};

use crate::key::bls::{self, public_key::Key as PublicKey};
use blst::{
    min_pk::{AggregateSignature, Signature},
    BLST_ERROR,
};

#[derive(Debug, Clone)]
pub struct Sig(pub Signature);

pub const LEN: usize = 96;

impl Sig {
    /// Converts the public key to compressed bytes.
    /// ref. "avalanchego/utils/crypto/bls.SignatureToBytes"
    pub fn to_compressed_bytes(&self) -> [u8; LEN] {
        self.0.compress()
    }

    /// Loads the signature from the compressed raw scalar bytes (in big endian).
    pub fn from_bytes(compressed: &[u8]) -> io::Result<Self> {
        let sig = Signature::uncompress(compressed).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed blst::min_pk::Signature::uncompress {:?}", e),
            )
        })?;
        sig.validate(false).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed blst::min_pk::Signature::validate {:?}", e),
            )
        })?;

        Ok(Self(sig))
    }

    /// Verifies the message and the validity of its signature.
    /// Invariant: \[pubkey\] and [self.0] have both been validated.
    /// ref. "avalanchego/utils/crypto/bls.Verify"
    pub fn verify(&self, msg: &[u8], pubkey: &PublicKey) -> bool {
        self.0.verify(
            false,
            msg,
            &bls::private_key::CIPHER_SUITE_SIGNATURE,
            &[],
            &pubkey.0,
            false,
        ) == BLST_ERROR::BLST_SUCCESS
    }

    /// Verifies the message and the validity of its signature.
    /// Invariant: \[pubkey\] and [self.0] have both been validated.
    /// ref. "avalanchego/utils/crypto/bls.VerifyProofOfPossession"
    pub fn verify_proof_of_possession(&self, msg: &[u8], pubkey: &PublicKey) -> bool {
        self.0.verify(
            false,
            msg,
            &bls::private_key::CIPHER_SUITE_PROOF_OF_POSSESSION,
            &[],
            &pubkey.0,
            false,
        ) == BLST_ERROR::BLST_SUCCESS
    }
}

impl From<Signature> for Sig {
    fn from(s: Signature) -> Self {
        Self(s)
    }
}

impl From<Sig> for Signature {
    fn from(s: Sig) -> Self {
        s.0
    }
}

pub fn aggregate(sigs: &[Sig]) -> io::Result<Sig> {
    let mut ss = Vec::with_capacity(sigs.len());
    for s in sigs.iter() {
        ss.push(&s.0);
    }

    let agg_sig = AggregateSignature::aggregate(&ss, false).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed AggregateSignature::aggregate {:?}", e),
        )
    })?;
    Ok(Sig(agg_sig.to_signature()))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::bls::signature::test_signature --exact --show-output
#[test]
fn test_signature() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let msg_to_sign = random_manager::secure_bytes(50).unwrap();

    let sk1 = crate::key::bls::private_key::Key::generate().unwrap();
    let pubkey1 = sk1.to_public_key();

    let sig1 = sk1.sign(&msg_to_sign);
    let sig1_bytes = sig1.to_compressed_bytes();
    assert!(sig1.verify(&msg_to_sign, &pubkey1));
    assert!(!sig1.verify_proof_of_possession(&msg_to_sign, &pubkey1));

    let agg_sig = aggregate(&[sig1.clone()]).unwrap();
    let agg_sig_bytes = agg_sig.to_compressed_bytes();
    assert_eq!(sig1_bytes, agg_sig_bytes);

    let sk2 = crate::key::bls::private_key::Key::generate().unwrap();
    let pubkey2 = sk2.to_public_key();
    let sig2 = sk2.sign(&msg_to_sign);

    let sk3 = crate::key::bls::private_key::Key::generate().unwrap();
    let pubkey3 = sk3.to_public_key();
    let sig3 = sk3.sign(&msg_to_sign);

    let agg_sig = aggregate(&[sig1, sig2, sig3]).unwrap();
    let agg_pubkey = crate::key::bls::public_key::aggregate(&[pubkey1, pubkey2, pubkey3]).unwrap();
    assert!(agg_pubkey.verify(&msg_to_sign, &agg_sig));

    let sig1_pos = sk1.sign_proof_of_possession(&msg_to_sign);
    assert!(!sig1_pos.verify(&msg_to_sign, &pubkey1));
    assert!(sig1_pos.verify_proof_of_possession(&msg_to_sign, &pubkey1));

    let sig2_pos = sk2.sign_proof_of_possession(&msg_to_sign);
    let sig3_pos = sk3.sign_proof_of_possession(&msg_to_sign);

    let agg_sig_pos = aggregate(&[sig1_pos, sig2_pos, sig3_pos]).unwrap();
    assert!(agg_pubkey.verify_proof_of_possession(&msg_to_sign, &agg_sig_pos));
}
