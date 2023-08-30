use crate::{errors::Result, key};
use async_trait::async_trait;
use ethers_core::types::{
    transaction::{eip2718::TypedTransaction, eip712::Eip712},
    Address, Signature,
};

#[derive(Clone, Debug)]
pub struct Signer {
    pub inner: super::Key,
    pub chain_id: primitive_types::U256,
    pub address: Address,
}

impl Signer {
    pub fn new(inner: super::Key, chain_id: primitive_types::U256) -> Result<Self> {
        let address: Address = inner.to_public_key().to_h160().into();
        Ok(Self {
            inner,
            chain_id,
            address,
        })
    }

    async fn sign_digest_with_eip155(
        &self,
        digest: ethers_core::types::H256,
        chain_id: u64,
    ) -> Result<Signature> {
        let mut sig = self.inner.sign_digest(digest.as_ref()).await?;
        key::secp256k1::signature::apply_eip155(&mut sig, chain_id);
        Ok(sig)
    }
}

#[async_trait]
impl<'a> ethers_signers::Signer for Signer {
    type Error = aws_manager::errors::Error;

    /// Implements "eth_sign" using "ethers_core::utils::hash_message".
    /// ref. <https://eips.ethereum.org/EIPS/eip-191>
    /// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sign>
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> std::result::Result<Signature, Self::Error> {
        let message = message.as_ref();
        let message_hash = ethers_core::utils::hash_message(message);

        self.sign_digest_with_eip155(message_hash, self.chain_id.as_u64())
            .await
            .map_err(|e| Self::Error::API {
                message: format!("failed sign_digest_with_eip155 {}", e),
                retryable: e.retryable(),
            })
    }

    async fn sign_transaction(
        &self,
        tx: &TypedTransaction,
    ) -> std::result::Result<Signature, Self::Error> {
        let mut tx_with_chain = tx.clone();
        let chain_id = tx_with_chain
            .chain_id()
            .map(|id| id.as_u64())
            .unwrap_or(self.chain_id.as_u64());
        tx_with_chain.set_chain_id(chain_id);

        let sighash = tx_with_chain.sighash();
        self.sign_digest_with_eip155(sighash, chain_id)
            .await
            .map_err(|e| Self::Error::API {
                message: format!("failed sign_digest_with_eip155 {}", e),
                retryable: e.retryable(),
            })
    }

    /// Implements "eth_signTypedData".
    /// ref. <https://eips.ethereum.org/EIPS/eip-712>
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> std::result::Result<Signature, Self::Error> {
        let digest = payload.encode_eip712().map_err(|e| Self::Error::Other {
            message: format!("failed encode_eip712 {}", e),
            retryable: false,
        })?;
        self.inner
            .sign_digest(digest.as_ref())
            .await
            .map_err(|e| Self::Error::API {
                message: format!("failed sign_digest {}", e),
                retryable: e.retryable(),
            })
    }

    fn address(&self) -> Address {
        self.address
    }

    fn chain_id(&self) -> u64 {
        self.chain_id.as_u64()
    }

    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        let chain_id: u64 = chain_id.into();
        self.chain_id = primitive_types::U256::from(chain_id);
        self
    }
}
