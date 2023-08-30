pub mod eth_signer;

use std::collections::HashMap;

use crate::{
    errors::{Error, Result},
    hash,
    ids::short,
    key,
};
use async_trait::async_trait;
use aws_manager::kms;
use aws_sdk_kms::types::{KeySpec, KeyUsageType};
use ethers_core::types::Signature as EthSig;
use tokio::time::{sleep, Duration, Instant};

/// Represents AWS KMS asymmetric elliptic curve key pair ECC_SECG_P256K1.
/// Note that the actual private key never leaves KMS.
/// Private key signing operation must be done via AWS KMS API.
/// ref. <https://docs.aws.amazon.com/kms/latest/APIReference/API_CreateKey.html>
///
/// "AWS KMS has replaced the term customer master key (CMK) with AWS KMS key and KMS key."
/// ref. <https://docs.aws.amazon.com/kms/latest/APIReference/Welcome.html>
#[derive(Debug, Clone)]
pub struct Key {
    /// AWS KMS API wrapper.
    pub kms_manager: kms::Manager,

    /// Key Id.
    pub id: String,
    /// Key Arn.
    pub arn: String,

    /// Optional. KMS grant token used for signing.
    pub grant_token: Option<String>,

    /// Public key.
    pub public_key: key::secp256k1::public_key::Key,

    /// Total duration for retries.
    pub retry_timeout: Duration,
    /// Interval between retries.
    pub retry_interval: Duration,
}

impl Key {
    /// Generates a new key.
    pub async fn create(kms_manager: kms::Manager, tags: HashMap<String, String>) -> Result<Self> {
        let key = kms_manager
            .create_key(
                KeySpec::EccSecgP256K1,
                KeyUsageType::SignVerify,
                Some(tags),
                false,
            )
            .await
            .map_err(|e| Error::API {
                message: format!(
                    "failed kms.create_key {} (retryable {})",
                    e.message(),
                    e.retryable()
                ),
                retryable: e.retryable(),
            })?;

        Self::from_arn(kms_manager, &key.arn).await
    }

    /// Loads the key from its Arn or Id.
    pub async fn from_arn(kms_manager: kms::Manager, arn: &str) -> Result<Self> {
        let (id, _desc) = kms_manager
            .describe_key(arn)
            .await
            .map_err(|e| Error::API {
                message: format!(
                    "failed kms.describe_key {} (retryable {})",
                    e.message(),
                    e.retryable()
                ),
                retryable: e.retryable(),
            })?;
        log::info!("described key Id '{id}' from '{arn}'");

        // derives the public key from its private key
        let pubkey = kms_manager
            .get_public_key(arn)
            .await
            .map_err(|e| Error::API {
                message: format!(
                    "failed kms.get_public_key {} (retryable {})",
                    e.message(),
                    e.retryable()
                ),
                retryable: e.retryable(),
            })?;

        if let Some(blob) = pubkey.public_key() {
            // same as "key::secp256k1::public_key::Key::from_public_key_der(blob.as_ref())"
            // ref. <https://github.com/gakonst/ethers-rs/tree/master/ethers-signers/src/aws>
            let verifying_key =
                key::secp256k1::public_key::load_ecdsa_verifying_key_from_public_key(
                    blob.as_ref(),
                )?;
            let public_key = key::secp256k1::public_key::Key::from_verifying_key(&verifying_key);
            log::info!(
                "fetched public key with ETH address '{}'",
                public_key.to_eth_address(),
            );

            return Ok(Self {
                kms_manager,
                public_key,
                id,
                arn: arn.to_string(),
                grant_token: None,
                retry_timeout: Duration::from_secs(90),
                retry_interval: Duration::from_secs(10),
            });
        }

        return Err(Error::API {
            message: "public key not found".to_string(),
            retryable: false,
        });
    }

    /// Schedules to delete the KMS key.
    pub async fn delete(&self, pending_window_in_days: i32) -> Result<()> {
        self.kms_manager
            .schedule_to_delete(&self.arn, pending_window_in_days)
            .await
            .map_err(|e| Error::API {
                message: format!(
                    "failed kms.schedule_to_delete {} (retryable {})",
                    e.message(),
                    e.retryable()
                ),
                retryable: e.retryable(),
            })
    }

    pub fn to_public_key(&self) -> key::secp256k1::public_key::Key {
        self.public_key
    }

    /// Converts to Info.
    pub fn to_info(&self, network_id: u32) -> Result<key::secp256k1::Info> {
        let short_addr = self.public_key.to_short_id()?;
        let eth_addr = self.public_key.to_eth_address();
        let h160_addr = self.public_key.to_h160();

        let mut addresses = HashMap::new();
        addresses.insert(
            network_id,
            key::secp256k1::ChainAddresses {
                x: self.public_key.to_hrp_address(network_id, "X")?,
                p: self.public_key.to_hrp_address(network_id, "P")?,
            },
        );

        Ok(key::secp256k1::Info {
            id: Some(self.arn.clone()),
            key_type: key::secp256k1::KeyType::AwsKms,

            addresses,

            short_address: short_addr,
            eth_address: eth_addr,
            h160_address: h160_addr,

            ..Default::default()
        })
    }

    pub async fn sign_digest(&self, digest: &[u8]) -> Result<EthSig> {
        // ref. "crypto/sha256.Size"
        assert_eq!(digest.len(), hash::SHA256_OUTPUT_LEN);

        let (start, mut success) = (Instant::now(), false);
        let mut round = 0_u32;

        // DER-encoded >65-byte signature, need convert to 65-byte recoverable signature
        // ref. <https://docs.aws.amazon.com/kms/latest/APIReference/API_Sign.html#KMS-Sign-response-Signature>
        let mut raw_der = Vec::new();
        loop {
            round = round + 1;
            let elapsed = start.elapsed();
            if elapsed.gt(&self.retry_timeout) {
                break;
            }

            // make sure to use KMS key Arn, not Id
            // in case its key access is granted with a KMS grant token
            raw_der = match self
                .kms_manager
                .sign_digest_secp256k1_ecdsa_sha256(&self.arn, digest, self.grant_token.clone())
                .await
            {
                Ok(raw) => {
                    success = true;
                    raw
                }
                Err(aerr) => {
                    log::warn!(
                        "[round {round}] failed sign {} (retriable {})",
                        aerr,
                        aerr.retryable()
                    );
                    if !aerr.retryable() {
                        return Err(Error::API {
                            message: aerr.message(),
                            retryable: false,
                        });
                    }

                    sleep(self.retry_interval).await;
                    continue;
                }
            };
            break;
        }
        if !success {
            return Err(Error::API {
                message: "failed sign after retries".to_string(),
                retryable: true,
            });
        }

        let sig =
            key::secp256k1::signature::decode_signature(&raw_der).map_err(|e| Error::Other {
                message: format!("failed decode_signature {}", e),
                retryable: false,
            })?;

        let mut fixed_digest = [0u8; hash::SHA256_OUTPUT_LEN];
        fixed_digest.copy_from_slice(digest);

        let eth_sig = key::secp256k1::signature::sig_from_digest_bytes_trial_recovery(
            &sig,
            &fixed_digest,
            &self.public_key.to_verifying_key(),
        )
        .map_err(|e| Error::Other {
            message: format!(
                "failed key::secp256k1::signature::sig_from_digest_bytes_trial_recovery {}",
                e
            ),
            retryable: false,
        })?;
        Ok(eth_sig)
    }
}

#[async_trait]
impl key::secp256k1::SignOnly for Key {
    fn signing_key(&self) -> Result<k256::ecdsa::SigningKey> {
        unimplemented!("signing key not implemented for KMS")
    }

    async fn sign_digest(&self, msg: &[u8]) -> Result<[u8; 65]> {
        let sig = self.sign_digest(msg).await?;

        let mut b = [0u8; key::secp256k1::signature::LEN];
        b.copy_from_slice(&sig.to_vec());

        Ok(b)
    }
}

/// ref. <https://doc.rust-lang.org/book/ch10-02-traits.html>
impl key::secp256k1::ReadOnly for Key {
    fn key_type(&self) -> key::secp256k1::KeyType {
        key::secp256k1::KeyType::AwsKms
    }

    fn hrp_address(&self, network_id: u32, chain_id_alias: &str) -> Result<String> {
        self.to_public_key()
            .to_hrp_address(network_id, chain_id_alias)
    }

    fn short_address(&self) -> Result<short::Id> {
        self.to_public_key().to_short_id()
    }

    fn short_address_bytes(&self) -> Result<Vec<u8>> {
        self.to_public_key().to_short_bytes()
    }

    fn eth_address(&self) -> String {
        self.to_public_key().to_eth_address()
    }

    fn h160_address(&self) -> primitive_types::H160 {
        self.to_public_key().to_h160()
    }
}
