//! AWS KMS support for secp256k1 keys.
#[cfg(feature = "kms_aws")]
#[cfg_attr(docsrs, doc(cfg(feature = "kms_aws")))]
pub mod aws;
