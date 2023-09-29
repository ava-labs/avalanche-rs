//! BLS private key module.
use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Read, Write},
    path::Path,
};

use crate::key::bls::{
    ProofOfPossession,
    {public_key::Key as PublicKey, signature::Sig as Signature},
};
use blst::min_pk::SecretKey;
use lazy_static::lazy_static;
use zeroize::Zeroize;

#[cfg(not(windows))]
use ring::rand::{SecureRandom, SystemRandom};

/// The size (in bytes) of a secret key.
/// At least 32-byte.
/// ref. "blst::BLST_ERROR::BLST_BAD_ENCODING"
/// ref. "avalanchego/utils/crypto/bls.SecretKeyLen"
pub const LEN: usize = 32;

/// Represents "k256::SecretKey" and "k256::ecdsa::SigningKey".
#[derive(Debug, Clone, Zeroize)]
pub struct Key(SecretKey);

#[cfg(not(windows))]
fn secure_random() -> &'static dyn SecureRandom {
    use std::ops::Deref;
    lazy_static! {
        static ref RANDOM: SystemRandom = SystemRandom::new();
    }
    RANDOM.deref()
}

lazy_static! {
    /// The ciphersuite is more commonly known as G2ProofOfPossession.
    /// There are two digests to ensure that that message space for normal
    /// signatures and the proof of possession are distinct.
    /// ref. "avalanchego/utils/crypto/bls.ciphersuiteSignature"
    pub static ref CIPHER_SUITE_SIGNATURE: Vec<u8> =
        b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_".to_vec();
    pub static ref CIPHER_SUITE_PROOF_OF_POSSESSION: Vec<u8> =
        b"BLS_POP_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_".to_vec();
}

impl Key {
    /// Generates a private key from random bytes.
    #[cfg(not(windows))]
    pub fn generate() -> io::Result<Self> {
        let mut b = [0u8; LEN];
        secure_random()
            .fill(&mut b)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed secure_random {}", e)))?;

        let sk = SecretKey::key_gen(&b, &[]).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed blst::min_pk::SecretKey::key_gen {:?}", e),
            )
        })?;
        Ok(Self(sk))
    }

    #[cfg(windows)]
    pub fn generate() -> io::Result<Self> {
        unimplemented!("not implemented")
    }

    /// Generates and writes the key to a file.
    #[cfg(not(windows))]
    pub fn generate_to_file(key_path: &str) -> io::Result<Self> {
        log::info!("generating staking signer key file to {}", key_path);
        if Path::new(key_path).exists() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("key path {} already exists", key_path),
            ));
        }

        let sk = Key::generate()?;
        let key_contents = sk.to_bytes();

        let mut key_file = File::create(key_path)?;
        key_file.write_all(&key_contents)?;
        log::info!(
            "saved staking signer key {} ({}-byte)",
            key_path,
            key_contents.len()
        );

        Ok(sk)
    }

    #[cfg(windows)]
    pub fn generate_to_file(key_path: &str) -> io::Result<Self> {
        unimplemented!("not implemented")
    }

    /// Loads the key.
    pub fn from_file(key_path: &str) -> io::Result<Key> {
        log::info!("loading staking signer key {}", key_path);
        if !Path::new(key_path).exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("key path {} does not exists", key_path),
            ));
        }

        let raw = read_vec(key_path)?;
        Self::from_bytes(&raw)
    }

    /// Loads the existing staking certificates if exists,
    /// and returns the loaded or generated the key.
    /// Returns "true" if generated.
    pub fn load_or_generate(key_path: &str) -> io::Result<(Self, bool)> {
        let key_exists = Path::new(&key_path).exists();
        log::info!(
            "staking signer key file {} exists? {}",
            key_path,
            key_exists
        );

        if !key_exists {
            log::info!(
                "generating staking signer key file (key exists {})",
                key_exists
            );
            Ok((Self::generate_to_file(key_path)?, true))
        } else {
            log::info!("loading existing staking signer key from '{}'", key_path);
            Ok((Self::from_file(key_path)?, false))
        }
    }

    /// Loads the private key from the raw scalar bytes (in big endian).
    pub fn from_bytes(raw: &[u8]) -> io::Result<Self> {
        let sk = SecretKey::from_bytes(raw).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed blst::min_pk::SecretKey::from_bytes {:?}", e),
            )
        })?;

        Ok(Self(sk))
    }

    /// Converts the private key to raw scalar bytes in big endian.
    pub fn to_bytes(&self) -> [u8; LEN] {
        self.0.serialize()
    }

    /// Derives the public key from this private key.
    pub fn to_public_key(&self) -> PublicKey {
        PublicKey::from(self.0.sk_to_pk())
    }

    /// ref. "avalanchego/utils/crypto/bls.SecretKey.Sign"
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.0.sign(msg, &CIPHER_SUITE_SIGNATURE, &[]).into()
    }

    /// ref. "avalanchego/utils/crypto/bls.SecretKey.SignProofOfPossession"
    pub fn sign_proof_of_possession(&self, msg: &[u8]) -> Signature {
        self.0
            .sign(msg, &CIPHER_SUITE_PROOF_OF_POSSESSION, &[])
            .into()
    }

    /// ref. "avalanchego"/vms/platformvm/signer.NewProofOfPossession"
    pub fn to_proof_of_possession(&self) -> ProofOfPossession {
        let pubkey = self.to_public_key();
        let pubkey_bytes = pubkey.to_compressed_bytes();

        let sig = self.sign_proof_of_possession(&pubkey_bytes);
        let sig_bytes = sig.to_compressed_bytes();

        ProofOfPossession {
            public_key: pubkey_bytes.to_vec(),
            proof_of_possession: sig_bytes.to_vec(),
            pubkey: Some(pubkey),
        }
    }
}

/// ref. <https://doc.rust-lang.org/std/fs/fn.read.html>
fn read_vec(p: &str) -> io::Result<Vec<u8>> {
    let mut f = File::open(p)?;
    let metadata = fs::metadata(p)?;
    let mut buffer = vec![0; metadata.len() as usize];
    let _read_bytes = f.read(&mut buffer)?;
    Ok(buffer)
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::bls::private_key::test_key --exact --show-output
#[test]
fn test_key() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let sk = Key::generate().unwrap();
    let pubkey = sk.to_public_key();

    let msg = random_manager::secure_bytes(50).unwrap();
    let sig = sk.sign(&msg);
    assert!(sig.verify(&msg, &pubkey));
    assert!(!sig.verify_proof_of_possession(&msg, &pubkey));

    let sig_pop = sk.sign_proof_of_possession(&msg);
    assert!(!sig_pop.verify(&msg, &pubkey));
    assert!(sig_pop.verify_proof_of_possession(&msg, &pubkey));

    let b = sk.to_bytes();
    let sk2 = Key::from_bytes(&b).unwrap();
    assert_eq!(sk.to_bytes(), sk2.to_bytes());

    let key_path = random_manager::tmp_path(10, None).unwrap();
    let generated_key = Key::generate_to_file(&key_path).unwrap();
    let loaded_key = Key::from_file(&key_path).unwrap();
    assert_eq!(generated_key.to_bytes(), loaded_key.to_bytes());

    let (loaded_key_2, generated) = Key::load_or_generate(&key_path).unwrap();
    assert!(!generated);
    assert_eq!(loaded_key.to_bytes(), loaded_key_2.to_bytes());

    std::fs::remove_file(&key_path).unwrap();
}
