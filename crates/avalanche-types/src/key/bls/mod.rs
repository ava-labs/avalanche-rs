//! BLS key module.
pub mod private_key;
pub mod public_key;
pub mod signature;

use std::io::{self, Error, ErrorKind};

use crate::codec::serde::hex_0x_bytes::Hex0xBytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// ref. "avalanchego"/vms/platformvm/signer.ProofOfPossession"
/// ref. <https://docs.avax.network/build/avalanchego-apis/info/#infogetnodeid>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ProofOfPossession {
    #[serde(rename = "publicKey")]
    #[serde_as(as = "Hex0xBytes")]
    pub public_key: Vec<u8>,
    #[serde(rename = "proofOfPossession")]
    #[serde_as(as = "Hex0xBytes")]
    pub proof_of_possession: Vec<u8>,

    #[serde(skip)]
    pub pubkey: Option<public_key::Key>,
}

impl Default for ProofOfPossession {
    fn default() -> Self {
        Self::default()
    }
}

impl ProofOfPossession {
    pub fn default() -> Self {
        Self {
            public_key: Vec::new(),
            proof_of_possession: Vec::new(),
            pubkey: None,
        }
    }
}

impl ProofOfPossession {
    pub fn new(public_key: &[u8], proof_of_possession: &[u8]) -> io::Result<Self> {
        let pubkey = public_key::Key::from_bytes(public_key)?;
        let sig = signature::Sig::from_bytes(proof_of_possession)?;

        if !pubkey.verify_proof_of_possession(public_key, &sig) {
            return Err(Error::new(
                ErrorKind::Other,
                "failed verify_proof_of_possession",
            ));
        }

        Ok(Self {
            public_key: public_key.to_vec(),
            proof_of_possession: proof_of_possession.to_vec(),
            pubkey: Some(pubkey),
        })
    }

    /// ref. "avalanchego"/vms/platformvm/signer.ProofOfPossession.Verify"
    pub fn verify(&self) -> io::Result<bool> {
        let pubkey = public_key::Key::from_bytes(&self.public_key)?;
        let pubkey_bytes = pubkey.to_compressed_bytes();

        let sig = signature::Sig::from_bytes(&self.proof_of_possession)?;

        Ok(pubkey.verify_proof_of_possession(&pubkey_bytes, &sig))
    }

    pub fn load_pubkey(&self) -> io::Result<public_key::Key> {
        // ref. "avalanchego/vms/platformvm/signer.ProofOfPossession.UnmarshalJSON"
        let pubkey = public_key::Key::from_bytes(&self.public_key)?;
        Ok(pubkey)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --all-features --lib -- key::bls::test_proof_of_possession --exact --show-output
#[test]
fn test_proof_of_possession() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let sk = private_key::Key::generate().unwrap();
    let pubkey = sk.to_public_key();

    let msg = random_manager::secure_bytes(50).unwrap();
    let sig = sk.sign(&msg);
    assert!(sig.verify(&msg, &pubkey));
    assert!(!sig.verify_proof_of_possession(&msg, &pubkey));

    let sig_pop = sk.sign_proof_of_possession(&msg);
    assert!(!sig_pop.verify(&msg, &pubkey));
    assert!(sig_pop.verify_proof_of_possession(&msg, &pubkey));

    let b = sk.to_bytes();
    let sk2 = private_key::Key::from_bytes(&b).unwrap();
    assert_eq!(sk.to_bytes(), sk2.to_bytes());

    let key_path = random_manager::tmp_path(10, None).unwrap();
    let generated_key = private_key::Key::generate_to_file(&key_path).unwrap();
    let loaded_key = private_key::Key::from_file(&key_path).unwrap();
    assert_eq!(generated_key.to_bytes(), loaded_key.to_bytes());

    let (loaded_key_2, generated) = private_key::Key::load_or_generate(&key_path).unwrap();
    assert!(!generated);
    assert_eq!(loaded_key.to_bytes(), loaded_key_2.to_bytes());

    std::fs::remove_file(&key_path).unwrap();

    let pop = sk.to_proof_of_possession();
    log::info!(
        "proof-of-possession: {}",
        serde_json::to_string_pretty(&pop).unwrap()
    );
    assert!(pop.verify().unwrap());

    let pop =
        ProofOfPossession::new(&pubkey.to_compressed_bytes(), &pop.proof_of_possession).unwrap();
    assert!(pop.verify().unwrap());
    assert_eq!(pop.public_key, pubkey.to_compressed_bytes());
    assert_eq!(
        pop.pubkey.unwrap().to_compressed_bytes(),
        pubkey.to_compressed_bytes()
    );
}
