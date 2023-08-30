use crate::errors::{Error, Result};
use ecdsa::RecoveryId as EcdsaRecoveryId;
use ethers_core::{k256::ecdsa::Signature as KSig, types::Signature as EthSig};
use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    FieldBytes,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zerocopy::AsBytes;

/// The length of recoverable ECDSA signature.
/// "github.com/decred/dcrd/dcrec/secp256k1/v3/ecdsa.SignCompact" outputs
/// 65-byte signature -- see "compactSigSize"
/// ref. "avalanchego/utils/crypto.PrivateKeySECP256K1R.SignHash"
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/crypto#SECP256K1RSigLen>
/// ref. "secp256k1::constants::SCHNORR_SIGNATURE_SIZE" + 1
pub const LEN: usize = 65;

/// Represents Ethereum-style "recoverable signatures". By default
/// serializes as hex string.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sig(pub (Signature, RecoveryId));

impl Sig {
    /// Loads the recoverable signature from the bytes.
    pub fn from_bytes(b: &[u8]) -> Result<Self> {
        if b.len() != LEN {
            return Err(Error::Other {
                message: "invalid signature length".to_string(),
                retryable: false,
            });
        }

        let sig = Signature::try_from(&b[..64]).map_err(|e| Error::Other {
            message: format!("failed to load recoverable signature {}", e),
            retryable: false,
        })?;
        let recid = RecoveryId::try_from(b[64]).map_err(|e| Error::Other {
            message: format!("failed to create recovery Id {}", e),
            retryable: false,
        })?;
        Ok(Self((sig, recid)))
    }

    /// Converts the signature to bytes.
    pub fn to_bytes(&self) -> [u8; LEN] {
        // "elliptic_curve::generic_array::GenericArray"
        let bb = self.0 .0.to_bytes();

        let mut b = [0u8; LEN];
        b.copy_from_slice(&[&bb[..], &[u8::from(self.0 .1)]].concat());
        b
    }

    /// Recovers the public key from the 32-byte SHA256 output message using its signature.
    pub fn recover_public_key(
        &self,
        digest: &[u8],
    ) -> Result<(crate::key::secp256k1::public_key::Key, VerifyingKey)> {
        recover_pubkeys(&self.0 .0, self.0 .1, digest)
    }

    pub fn r(&self) -> primitive_types::U256 {
        let b = self.0 .0.to_vec();
        primitive_types::U256::from_big_endian(&b[0..32])
    }

    pub fn s(&self) -> primitive_types::U256 {
        let b = self.0 .0.to_vec();
        primitive_types::U256::from_big_endian(&b[32..64])
    }

    /// Returns the recovery Id.
    pub fn v(&self) -> u64 {
        // ref. <https://github.com/RustCrypto/elliptic-curves/blob/p384/v0.11.2/k256/src/ecdsa/recoverable.rs> "recovery_id"
        u8::from(self.0 .1) as u64
    }
}

impl<'de> Deserialize<'de> for Sig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let val = String::deserialize(deserializer)
            .and_then(|s| hex::decode(s).map_err(Error::custom))?;
        Self::from_bytes(val.as_bytes()).map_err(Error::custom)
    }
}

impl Serialize for Sig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.to_bytes()))
    }
}

fn recover_pubkeys(
    rsig: &Signature,
    recid: RecoveryId,
    digest: &[u8],
) -> Result<(crate::key::secp256k1::public_key::Key, VerifyingKey)> {
    // ref. <https://github.com/RustCrypto/elliptic-curves/blob/p384/v0.11.2/k256/src/ecdsa/recoverable.rs> "recovery_id"
    // ref. <https://github.com/RustCrypto/elliptic-curves/blob/p384/v0.11.2/k256/src/ecdsa/recoverable.rs> "recover_verifying_key_from_digest_bytes"
    let vkey =
        VerifyingKey::recover_from_prehash(digest, rsig, recid).map_err(|e| Error::Other {
            message: format!("failed recover_verifying_key_from_digest_bytes {}", e),
            retryable: false,
        })?;

    Ok((vkey.into(), vkey))
}

impl From<Sig> for Signature {
    fn from(sig: Sig) -> Self {
        sig.0 .0
    }
}

impl From<Sig> for [u8; LEN] {
    fn from(sig: Sig) -> Self {
        sig.to_bytes()
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::signature::test_signature --exact --show-output
#[test]
fn test_signature() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let pk = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey = pk.to_public_key();

    let msg: Vec<u8> = random_manager::secure_bytes(100).unwrap();
    let hashed = crate::hash::sha256(&msg);

    let sig = pk.sign_digest(&hashed).unwrap();
    assert_eq!(sig.to_bytes().len(), crate::key::secp256k1::signature::LEN);

    let (recovered_pubkey, _) = sig.recover_public_key(&hashed).unwrap();
    assert_eq!(pubkey.to_eth_address(), recovered_pubkey.to_eth_address());
    assert_eq!(pubkey, recovered_pubkey);
}

/// Loads the recoverable signature from the DER-encoded bytes,
/// as defined by ANS X9.62–2005 and RFC 3279 Section 2.2.3.
/// ref. <https://docs.aws.amazon.com/kms/latest/APIReference/API_Sign.html#KMS-Sign-response-Signature>
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-signers/src/aws/utils.rs> "decode_signature"
pub fn decode_signature(b: &[u8]) -> Result<Signature> {
    let sig = Signature::from_der(b).map_err(|e| Error::Other {
        message: format!("failed Signature::from_der {}", e),
        retryable: false,
    })?;

    // EIP-2, not all elliptic curve signatures are accepted
    // "s" needs to be smaller than half of the curve
    // flip "s" if it's greater than half of the curve
    Ok(sig.normalize_s().unwrap_or(sig))
}

/// Converts to recoverable signature of 65-byte.
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-signers/src/aws/utils.rs> "rsig_from_digest_bytes_trial_recovery"
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-signers/src/aws/utils.rs> "rsig_to_ethsig"
/// ref. <https://github.com/gakonst/ethers-rs/pull/2260>
/// ref. <https://docs.rs/ecdsa/latest/ecdsa/struct.RecoveryId.html#method.trial_recovery_from_prehash>
pub fn sig_from_digest_bytes_trial_recovery(
    sig: &KSig,
    digest: &[u8; 32],
    vk: &VerifyingKey,
) -> Result<EthSig> {
    // Checks whether the specified recoverable signature can derive
    // the expected verifying key from the digest bytes.
    // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-signers/src/aws/utils.rs> "check_candidate"
    let recid =
        EcdsaRecoveryId::trial_recovery_from_prehash(vk, digest.as_bytes(), sig).map_err(|e| {
            Error::Other {
                message: format!("failed EcdsaRecoveryId::trial_recovery_from_prehash {}", e),
                retryable: false,
            }
        })?;

    let r_bytes: FieldBytes = sig.r().into();
    let s_bytes: FieldBytes = sig.s().into();

    let r = primitive_types::U256::from_big_endian(r_bytes.as_slice());
    let s = primitive_types::U256::from_big_endian(s_bytes.as_slice());

    Ok(EthSig {
        r,
        s,
        v: recid.to_byte() as u64,
    })
}

/// Modify the v value of a signature to conform to eip155
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-signers/src/aws/utils.rs> "apply_eip155"
/// ref. <https://github.com/gakonst/ethers-rs/pull/2300>
pub fn apply_eip155(sig: &mut ethers_core::types::Signature, chain_id: u64) {
    let v = (chain_id * 2 + 35) + sig.v;
    sig.v = v;
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::signature::test_signature_serialization --exact --show-output
#[test]
fn test_signature_serialization() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        sig: Sig,
    }

    let pk = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey = pk.to_public_key();

    let msg: Vec<u8> = random_manager::secure_bytes(100).unwrap();
    let hashed = crate::hash::sha256(&msg);
    let sig = pk.sign_digest(&hashed).unwrap();
    let d = Data { sig: sig.clone() };

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{}", json_encoded);
    let json_decoded = serde_json::from_str::<Data>(&json_encoded).unwrap();
    assert_eq!(sig, json_decoded.sig);

    let (recovered_pubkey, _) = json_decoded.sig.recover_public_key(&hashed).unwrap();
    assert_eq!(pubkey.to_eth_address(), recovered_pubkey.to_eth_address());
    assert_eq!(pubkey, recovered_pubkey);
}
