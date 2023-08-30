use crate::{
    constants,
    errors::{Error, Result},
    formatting, hash,
    ids::short,
    key::{
        self,
        secp256k1::{address, signature::Sig},
    },
};
use k256::{
    ecdsa::{signature::hazmat::PrehashVerifier, VerifyingKey},
    pkcs8::DecodePublicKey,
    PublicKey,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The size (in bytes) of a public key.
/// ref. "secp256k1::constants::PUBLIC_KEY_SIZE"
pub const LEN: usize = 33;

/// The size (in bytes) of an serialized uncompressed public key.
/// ref. "secp256k1::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE"
pub const UNCOMPRESSED_LEN: usize = 65;

/// Represents "k256::PublicKey" and "k256::ecdsa::VerifyingKey".
/// By default serializes as hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(pub PublicKey);

impl Key {
    /// Decodes compressed or uncompressed public key bytes with Elliptic-Curve-Point-to-Octet-String
    /// encoding described in SEC 1: Elliptic Curve Cryptography (Version 2.0) section 2.3.3 (page 10).
    /// ref. <http://www.secg.org/sec1-v2.pdf>
    pub fn from_sec1_bytes(b: &[u8]) -> Result<Self> {
        let pubkey = PublicKey::from_sec1_bytes(b).map_err(|e| Error::Other {
            message: format!("failed PublicKey::from_sec1_bytes {}", e),
            retryable: false,
        })?;
        Ok(Self(pubkey))
    }

    /// Decodes ASN.1 DER-encoded public key bytes.
    pub fn from_public_key_der(b: &[u8]) -> Result<Self> {
        let pubkey = PublicKey::from_public_key_der(b).map_err(|e| Error::Other {
            message: format!("failed PublicKey::from_public_key_der {}", e),
            retryable: false,
        })?;
        Ok(Self(pubkey))
    }

    /// Loads the public key from a message and its recoverable signature.
    /// ref. "fx.SECPFactory.RecoverHashPublicKey"
    pub fn from_signature(digest: &[u8], sig: &[u8]) -> Result<Self> {
        let sig = Sig::from_bytes(sig)?;
        let (pubkey, _) = sig.recover_public_key(digest)?;
        Ok(pubkey)
    }

    pub fn from_verifying_key(verifying_key: &VerifyingKey) -> Self {
        let pubkey: PublicKey = verifying_key.into();
        Self(pubkey)
    }

    pub fn to_verifying_key(&self) -> VerifyingKey {
        self.0.into()
    }

    /// Verifies the message and the validity of its signature with recoverable code.
    pub fn verify(&self, digest: &[u8], sig: &[u8]) -> Result<bool> {
        let sig = Sig::from_bytes(sig).map_err(|e| Error::Other {
            message: format!("failed Sig::from_bytes '{}'", e),
            retryable: false,
        })?;

        let (recovered_pubkey, verifying_key) = sig.recover_public_key(digest)?;
        if verifying_key.verify_prehash(digest, &sig.0 .0).is_err() {
            return Ok(false);
        }

        Ok(*self == recovered_pubkey)
    }

    /// Converts the public key to compressed bytes.
    pub fn to_compressed_bytes(&self) -> [u8; LEN] {
        let vkey: VerifyingKey = self.0.into();

        // ref. <https://github.com/RustCrypto/elliptic-curves/commit/c87d391a107b5bc22f03acf0de4ded988797c4ec> "to_bytes"
        // ref. <https://github.com/RustCrypto/signatures/blob/master/ecdsa/src/verifying.rs> "to_encoded_point"
        let ep = vkey.to_encoded_point(true);
        let bb = ep.as_bytes();

        let mut b = [0u8; LEN];
        b.copy_from_slice(&bb);
        b
    }

    /// Converts the public key to uncompressed bytes.
    pub fn to_uncompressed_bytes(&self) -> [u8; UNCOMPRESSED_LEN] {
        let vkey: VerifyingKey = self.0.into();
        let p = vkey.to_encoded_point(false);

        let mut b = [0u8; UNCOMPRESSED_LEN];
        b.copy_from_slice(&p.to_bytes());
        b
    }

    /// "hashing.PubkeyBytesToAddress"
    ///
    /// ref. "pk.PublicKey().Address().Bytes()"
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    pub fn to_short_id(&self) -> Result<crate::ids::short::Id> {
        let compressed = self.to_compressed_bytes();
        short::Id::from_public_key_bytes(&compressed).map_err(|e| Error::Other {
            message: format!("failed short::Id::from_public_key_bytes '{}'", e),
            retryable: false,
        })
    }

    /// "hashing.PubkeyBytesToAddress" and "ids.ToShortID"
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    pub fn to_short_bytes(&self) -> Result<Vec<u8>> {
        let compressed = self.to_compressed_bytes();
        hash::sha256_ripemd160(&compressed).map_err(|e| Error::Other {
            message: format!("failed to_short_bytes '{}'", e),
            retryable: false,
        })
    }

    pub fn to_h160(&self) -> primitive_types::H160 {
        let uncompressed = self.to_uncompressed_bytes();

        // ref. "Keccak256(pubBytes[1:])[12:]"
        let digest_h256 = hash::keccak256(&uncompressed[1..]);
        let digest_h256 = &digest_h256.0[12..];

        primitive_types::H160::from_slice(digest_h256)
    }

    /// Encodes the public key in ETH address format.
    /// Make sure to not double-hash.
    /// ref. <https://pkg.go.dev/github.com/ethereum/go-ethereum/crypto#PubkeyToAddress>
    /// ref. <https://pkg.go.dev/github.com/ethereum/go-ethereum/common#Address.Hex>
    pub fn to_eth_address(&self) -> String {
        address::h160_to_eth_address(&self.to_h160(), None)
    }

    pub fn to_hrp_address(&self, network_id: u32, chain_id_alias: &str) -> Result<String> {
        let hrp = match constants::NETWORK_ID_TO_HRP.get(&network_id) {
            Some(v) => v,
            None => constants::FALLBACK_HRP,
        };

        // ref. "pk.PublicKey().Address().Bytes()"
        let short_address_bytes = self.to_short_bytes()?;

        // ref. "formatting.FormatAddress(chainIDAlias, hrp, pubBytes)"
        formatting::address(chain_id_alias, hrp, &short_address_bytes).map_err(|e| Error::Other {
            message: format!("failed formatting::address '{}'", e),
            retryable: false,
        })
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let val = String::deserialize(deserializer)
            .and_then(|s| hex::decode(s).map_err(Error::custom))?;
        Self::from_sec1_bytes(&val).map_err(Error::custom)
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.to_compressed_bytes()))
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

impl From<VerifyingKey> for Key {
    fn from(vkey: VerifyingKey) -> Self {
        Self(vkey.into())
    }
}

impl From<Key> for VerifyingKey {
    fn from(k: Key) -> Self {
        k.0.into()
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
///
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
///
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.to_compressed_bytes()))
    }
}

/// ref. <https://doc.rust-lang.org/book/ch10-02-traits.html>
impl key::secp256k1::ReadOnly for Key {
    fn key_type(&self) -> key::secp256k1::KeyType {
        key::secp256k1::KeyType::Hot
    }

    fn hrp_address(&self, network_id: u32, chain_id_alias: &str) -> Result<String> {
        self.to_hrp_address(network_id, chain_id_alias)
    }

    fn short_address(&self) -> Result<short::Id> {
        self.to_short_id()
    }

    fn short_address_bytes(&self) -> Result<Vec<u8>> {
        self.to_short_bytes()
    }

    fn eth_address(&self) -> String {
        self.to_eth_address()
    }

    fn h160_address(&self) -> primitive_types::H160 {
        self.to_h160()
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::public_key::test_public_key --exact --show-output
#[test]
fn test_public_key() {
    use primitive_types::H160;
    use std::str::FromStr;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let pk1 = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey1 = pk1.to_public_key();

    let b = pubkey1.to_compressed_bytes();
    let pubkey2 = Key::from_sec1_bytes(&b).unwrap();

    let b = pubkey1.to_uncompressed_bytes();
    let pubkey3 = Key::from_sec1_bytes(&b).unwrap();

    assert_eq!(pubkey1, pubkey2);
    assert_eq!(pubkey2, pubkey3);

    let msg: Vec<u8> = random_manager::secure_bytes(100).unwrap();
    let hashed = hash::sha256(&msg);

    let sig1 = pk1.sign_digest(&hashed).unwrap();
    assert_eq!(sig1.to_bytes().len(), crate::key::secp256k1::signature::LEN);

    let pubkey4 = Key::from_signature(&hashed, &sig1.to_bytes()).unwrap();
    assert_eq!(pubkey3, pubkey4);

    assert!(pubkey1.verify(&hashed, &sig1.to_bytes()).unwrap());
    assert!(pubkey2.verify(&hashed, &sig1.to_bytes()).unwrap());
    assert!(pubkey3.verify(&hashed, &sig1.to_bytes()).unwrap());
    assert!(pubkey4.verify(&hashed, &sig1.to_bytes()).unwrap());

    log::info!("public key: {}", pubkey1);
    log::info!("to_short_id: {}", pubkey1.to_short_id().unwrap());
    log::info!("to_h160: {}", pubkey1.to_h160());
    log::info!("eth_address: {}", pubkey1.to_eth_address());

    // make sure H160 parses regardless of lower/upper case
    let eth_addr = pubkey1.to_eth_address();
    let eth_to_h160 = H160::from_str(&eth_addr.trim_start_matches("0x")).unwrap();
    assert_eq!(eth_to_h160, pubkey1.to_h160());

    let x_avax_addr = pubkey1.to_hrp_address(1, "X").unwrap();
    let p_avax_addr = pubkey1.to_hrp_address(1, "P").unwrap();
    log::info!("AVAX X address: {}", x_avax_addr);
    log::info!("AVAX P address: {}", p_avax_addr);
}

/// Same as "from_public_key_der".
/// ref. <https://github.com/gakonst/ethers-rs/tree/master/ethers-signers/src/aws> "decode_pubkey"
pub fn load_ecdsa_verifying_key_from_public_key(b: &[u8]) -> Result<VerifyingKey> {
    let spk = spki::SubjectPublicKeyInfoRef::try_from(b).map_err(|e| Error::Other {
        message: format!("failed to load spki::SubjectPublicKeyInfoRef {}", e),
        retryable: false,
    })?;
    VerifyingKey::from_sec1_bytes(spk.subject_public_key.raw_bytes()).map_err(|e| Error::Other {
        message: format!(
            "failed to load k256::ecdsa::VerifyingKey::from_sec1_bytes {}",
            e
        ),
        retryable: false,
    })
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::signature::test_key_serialization --exact --show-output
#[test]
fn test_key_serialization() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        key: Key,
    }

    let pk = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey = pk.to_public_key();
    let d = Data {
        key: pubkey.clone(),
    };

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{}", json_encoded);
    let json_decoded = serde_json::from_str::<Data>(&json_encoded).unwrap();
    assert_eq!(pubkey, json_decoded.key);
}
