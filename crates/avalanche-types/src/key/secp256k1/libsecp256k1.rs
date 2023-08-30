use crate::{
    constants,
    errors::{Error, Result},
    formatting, hash,
    ids::short,
    key::{self, secp256k1::address},
};
use async_trait::async_trait;
use secp256k1 as libsecp256k1;

/// Represents "libsecp256k1::SecretKey" to implement key traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateKey(libsecp256k1::SecretKey);

impl PrivateKey {
    /// Loads the private key from the raw bytes.
    pub fn from_bytes(raw: &[u8]) -> Result<Self> {
        if raw.len() != key::secp256k1::private_key::LEN {
            return Err(Error::Other {
                message: format!(
                    "libsecp256k1::SecretKey must be {}-byte, got {}-byte",
                    key::secp256k1::private_key::LEN,
                    raw.len()
                ),
                retryable: false,
            });
        }

        let sk = libsecp256k1::SecretKey::from_slice(raw).map_err(|e| Error::Other {
            message: format!("failed libsecp256k1::SecretKey::from_slice {}", e),
            retryable: false,
        })?;
        Ok(Self(sk))
    }

    pub fn signing_key(&self) -> Result<k256::ecdsa::SigningKey> {
        let b = self.to_bytes();
        let ga = k256::elliptic_curve::generic_array::GenericArray::from_slice(&b);
        k256::ecdsa::SigningKey::from_bytes(&ga).map_err(|e| Error::Other {
            message: format!("failed k256::ecdsa::SigningKey::from_bytes '{}'", e),
            retryable: false,
        })
    }

    /// Converts the private key to raw bytes.
    pub fn to_bytes(&self) -> [u8; key::secp256k1::private_key::LEN] {
        let b = self.0.secret_bytes();

        let mut bb = [0u8; key::secp256k1::private_key::LEN];
        bb.copy_from_slice(&b);
        bb
    }

    /// Hex-encodes the raw private key to string with "0x" prefix (e.g., Ethereum).
    pub fn to_hex(&self) -> String {
        // ref. https://github.com/rust-bitcoin/rust-secp256k1/pull/396
        let b = self.0.secret_bytes();
        let enc = hex::encode(&b);

        let mut s = String::from(key::secp256k1::private_key::HEX_ENCODE_PREFIX);
        s.push_str(&enc);
        s
    }

    /// Encodes the raw private key to string with "PrivateKey-" prefix (e.g., Avalanche).
    pub fn to_cb58(&self) -> String {
        let b = self.0.secret_bytes();
        let enc = formatting::encode_cb58_with_checksum_string(&b);

        let mut s = String::from(key::secp256k1::private_key::CB58_ENCODE_PREFIX);
        s.push_str(&enc);
        s
    }

    /// Derives the public key from this private key.
    pub fn to_public_key(&self) -> PublicKey {
        let secp = libsecp256k1::Secp256k1::new();
        let pubkey = libsecp256k1::PublicKey::from_secret_key(&secp, &self.0);
        PublicKey(pubkey)
    }

    /// Signs the 32-byte SHA256 output message with the ECDSA private key and the recoverable code.
    /// "github.com/decred/dcrd/dcrec/secp256k1/v3/ecdsa.SignCompact" outputs 65-byte signature.
    /// ref. "avalanchego/utils/crypto.PrivateKeySECP256K1R.SignHash"
    /// ref. <https://github.com/rust-bitcoin/rust-secp256k1/blob/master/src/ecdsa/recovery.rs>
    pub fn sign_digest(&self, digest: &[u8]) -> Result<key::secp256k1::signature::Sig> {
        // ref. "crypto/sha256.Size"
        assert_eq!(digest.len(), hash::SHA256_OUTPUT_LEN);

        let secp = libsecp256k1::Secp256k1::new();
        let m = libsecp256k1::Message::from_slice(digest).map_err(|e| Error::Other {
            message: format!("failed libsecp256k1::Message::from_slice {}", e),
            retryable: false,
        })?;

        // "github.com/decred/dcrd/dcrec/secp256k1/v3/ecdsa.SignCompact" outputs
        // 65-byte signature
        // ref. "avalanchego/utils/crypto.PrivateKeySECP256K1R.SignHash"
        // ref. https://github.com/rust-bitcoin/rust-secp256k1/blob/master/src/ecdsa/recovery.rs
        let sig = secp.sign_ecdsa_recoverable(&m, &self.0);
        let (rec_id, sig) = sig.serialize_compact();

        let mut sig = Vec::from(sig);
        sig.push(rec_id.to_i32() as u8);
        assert_eq!(sig.len(), key::secp256k1::signature::LEN);

        key::secp256k1::signature::Sig::from_bytes(&sig)
    }
}

impl From<libsecp256k1::SecretKey> for PrivateKey {
    fn from(s: libsecp256k1::SecretKey) -> Self {
        Self(s)
    }
}

impl From<PrivateKey> for libsecp256k1::SecretKey {
    fn from(s: PrivateKey) -> Self {
        s.0
    }
}

#[async_trait]
impl key::secp256k1::SignOnly for PrivateKey {
    fn signing_key(&self) -> Result<k256::ecdsa::SigningKey> {
        self.signing_key()
    }

    async fn sign_digest(&self, msg: &[u8]) -> Result<[u8; 65]> {
        let sig = self.sign_digest(msg)?;
        Ok(sig.to_bytes())
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.to_bytes()))
    }
}

/// Represents "secp256k1::PublicKey" to implement key traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(libsecp256k1::PublicKey);

impl PublicKey {
    /// Converts the public key to compressed bytes.
    pub fn to_compressed_bytes(&self) -> [u8; key::secp256k1::public_key::LEN] {
        let bb = self.0.serialize();

        let mut b = [0u8; key::secp256k1::public_key::LEN];
        b.copy_from_slice(&bb);
        b
    }

    /// Converts the public key to uncompressed bytes.
    pub fn to_uncompressed_bytes(&self) -> [u8; key::secp256k1::public_key::UNCOMPRESSED_LEN] {
        let bb = self.0.serialize_uncompressed();

        let mut b = [0u8; key::secp256k1::public_key::UNCOMPRESSED_LEN];
        b.copy_from_slice(&bb);
        b
    }

    /// "hashing.PubkeyBytesToAddress" and "ids.ToShortID"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    pub fn to_short_bytes(&self) -> Result<Vec<u8>> {
        let compressed = self.to_compressed_bytes();
        hash::sha256_ripemd160(&compressed).map_err(|e| Error::Other {
            message: format!("failed hash::sha256_ripemd160 ({})", e),
            retryable: false,
        })
    }

    /// "hashing.PubkeyBytesToAddress"
    /// ref. "pk.PublicKey().Address().Bytes()"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    pub fn to_short_id(&self) -> Result<crate::ids::short::Id> {
        let compressed = self.to_compressed_bytes();
        short::Id::from_public_key_bytes(&compressed).map_err(|e| Error::Other {
            message: format!("failed short::Id::from_public_key_bytes ({})", e),
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
            message: format!("failed formatting::address ({})", e),
            retryable: false,
        })
    }
}

impl From<libsecp256k1::PublicKey> for PublicKey {
    fn from(pubkey: libsecp256k1::PublicKey) -> Self {
        Self(pubkey)
    }
}

impl From<PublicKey> for libsecp256k1::PublicKey {
    fn from(pubkey: PublicKey) -> Self {
        pubkey.0
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.to_compressed_bytes()))
    }
}

/// ref. <https://doc.rust-lang.org/book/ch10-02-traits.html>
impl key::secp256k1::ReadOnly for PublicKey {
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::libsecp256k1::test_key --exact --show-output
#[test]
fn test_key() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let msg: Vec<u8> = random_manager::secure_bytes(100).unwrap();
    let hashed = hash::sha256(&msg);

    let pk1 = key::secp256k1::private_key::Key::generate().unwrap();
    let pk1 = pk1.to_libsecp256k1().unwrap();

    let sig1 = pk1.sign_digest(&hashed).unwrap();
    assert_eq!(sig1.to_bytes().len(), key::secp256k1::signature::LEN);

    let raw_bytes = pk1.to_bytes();
    assert_eq!(raw_bytes.len(), key::secp256k1::private_key::LEN);

    let pk2 = key::secp256k1::private_key::Key::from_bytes(&raw_bytes).unwrap();
    let pk2 = pk2.to_libsecp256k1().unwrap();
    assert_eq!(pk1, pk2);

    let hex1 = pk1.to_hex();
    let hex2 = pk2.to_hex();
    assert_eq!(hex1, hex2);
    log::info!("hex: {}", hex1);

    let pk3 = key::secp256k1::private_key::Key::from_hex(hex1).unwrap();
    let pk3 = pk3.to_libsecp256k1().unwrap();
    assert_eq!(pk1, pk3);

    let cb1 = pk1.to_cb58();
    let cb2 = pk2.to_cb58();
    let cb3 = pk3.to_cb58();
    assert_eq!(cb1, cb2);
    assert_eq!(cb2, cb3);
    log::info!("cb58: {}", cb1);

    let pk4 = key::secp256k1::private_key::Key::from_cb58(cb1).unwrap();
    let pk4 = pk4.to_libsecp256k1().unwrap();
    assert_eq!(pk1, pk2);
    assert_eq!(pk2, pk3);
    assert_eq!(pk3, pk4);
}
