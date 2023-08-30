//! EIP-1559 transaction type.
use std::io::{self, Error, ErrorKind};

use ethers::prelude::Eip1559TransactionRequest;
use ethers_core::types::{transaction::eip2718::TypedTransaction, RecoveryMessage, Signature};
use primitive_types::{H160, H256, U256};

/// Transaction but without provider.
#[derive(Clone, Debug)]
pub struct Transaction {
    pub chain_id: u64,
    pub signer_nonce: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub gas_limit: Option<U256>,

    // "from" itself is not RLP-encoded field
    // "from" can be simply derived from signature and transaction hash
    // when the RPC decodes the raw transaction
    pub from: H160,
    pub recipient: Option<H160>,

    pub value: Option<U256>,
    pub data: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            chain_id: 0,
            signer_nonce: None,

            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_limit: None,

            from: H160::zero(),
            recipient: None,
            value: None,
            data: None,
        }
    }

    #[must_use]
    pub fn chain_id(mut self, chain_id: impl Into<u64>) -> Self {
        self.chain_id = chain_id.into();
        self
    }

    #[must_use]
    pub fn signer_nonce(mut self, signer_nonce: impl Into<U256>) -> Self {
        self.signer_nonce = Some(signer_nonce.into());
        self
    }

    #[must_use]
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: impl Into<U256>) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas.into());
        self
    }

    #[must_use]
    pub fn max_fee_per_gas(mut self, max_fee_per_gas: impl Into<U256>) -> Self {
        self.max_fee_per_gas = Some(max_fee_per_gas.into());
        self
    }

    #[must_use]
    pub fn gas_limit(mut self, gas_limit: impl Into<U256>) -> Self {
        self.gas_limit = Some(gas_limit.into());
        self
    }

    #[must_use]
    pub fn from(mut self, from: impl Into<H160>) -> Self {
        self.from = from.into();
        self
    }

    #[must_use]
    pub fn recipient(mut self, to: impl Into<H160>) -> Self {
        self.recipient = Some(to.into());
        self
    }

    #[must_use]
    pub fn value(mut self, value: impl Into<U256>) -> Self {
        self.value = Some(value.into());
        self
    }

    #[must_use]
    pub fn data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Signs the transaction as "ethers_core::types::transaction::eip2718::TypedTransaction"
    /// and returns the rlp-encoded bytes that can be sent via "eth_sendRawTransaction".
    /// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
    pub async fn sign_as_typed_transaction(
        &self,
        eth_signer: impl ethers_signers::Signer + Clone,
    ) -> io::Result<ethers_core::types::Bytes> {
        let mut tx_request = Eip1559TransactionRequest::new()
            .from(ethers::prelude::H160::from(self.from.as_fixed_bytes()))
            .chain_id(ethers::prelude::U64::from(self.chain_id));

        if let Some(signer_nonce) = self.signer_nonce {
            tx_request = tx_request.nonce(signer_nonce);
        }

        if let Some(to) = &self.recipient {
            tx_request = tx_request.to(ethers::prelude::H160::from(to.as_fixed_bytes()));
        }

        if let Some(value) = &self.value {
            let converted: ethers::prelude::U256 = value.into();
            tx_request = tx_request.value(converted);
        }

        if let Some(max_priority_fee_per_gas) = &self.max_priority_fee_per_gas {
            let converted: ethers::prelude::U256 = max_priority_fee_per_gas.into();
            tx_request = tx_request.max_priority_fee_per_gas(converted);
        }

        if let Some(max_fee_per_gas) = &self.max_fee_per_gas {
            let converted: ethers::prelude::U256 = max_fee_per_gas.into();
            tx_request = tx_request.max_fee_per_gas(converted);
        }

        if let Some(gas_limit) = &self.gas_limit {
            let converted: ethers::prelude::U256 = gas_limit.into();
            tx_request = tx_request.gas(converted);
        }

        if let Some(data) = &self.data {
            tx_request = tx_request.data(data.clone());
        }

        let tx: TypedTransaction = tx_request.into();
        let sig = eth_signer.sign_transaction(&tx).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to sign_transaction '{}'", e),
            )
        })?;

        Ok(tx.rlp_signed(&sig))
    }
}

/// Decodes the RLP-encoded signed "ethers_core::types::transaction::eip2718::TypedTransaction" bytes.
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
pub fn decode_signed_rlp(b: impl AsRef<[u8]>) -> io::Result<(TypedTransaction, Signature)> {
    let r = rlp::Rlp::new(b.as_ref());
    TypedTransaction::decode_signed(&r)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed decode_signed '{}'", e)))
}

/// Decodes the RLP-encoded signed "ethers_core::types::transaction::eip2718::TypedTransaction" bytes.
/// And verifies the decoded signature.
/// It returns the typed transaction, transaction hash, its signer address, and the signature.
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
pub fn decode_and_verify_signed_rlp(
    b: impl AsRef<[u8]>,
) -> io::Result<(TypedTransaction, H256, H160, Signature)> {
    let r = rlp::Rlp::new(b.as_ref());
    let (decoded_tx, sig) = TypedTransaction::decode_signed(&r)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed decode_signed '{}'", e)))?;

    let tx_hash = decoded_tx.sighash();
    log::debug!("decoded signed transaction hash: 0x{:x}", tx_hash);

    let signer_addr = sig.recover(RecoveryMessage::Hash(tx_hash)).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!(
                "failed to recover signer address from signature and signed transaction hash '{}'",
                e
            ),
        )
    })?;

    sig.verify(RecoveryMessage::Hash(tx_hash), signer_addr)
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!(
                    "failed to verify signature against the signed transaction hash '{}'",
                    e
                ),
            )
        })?;
    log::info!(
        "verified signer address '{}' against signature and transaction hash",
        signer_addr
    );

    Ok((decoded_tx, tx_hash, signer_addr, sig))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- evm::eip1559::test_transaction --exact --show-output
#[test]
fn test_transaction() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let k1 = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let key_info1 = k1.to_info(1234).unwrap();
    log::info!("created {}", key_info1.h160_address);
    let k1_signer: ethers_signers::LocalWallet = k1.to_ethers_core_signing_key().into();

    let k2 = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let key_info2 = k2.to_info(1234).unwrap();
    log::info!("created {}", key_info2.h160_address);

    let chain_id = random_manager::u64() % 3000;
    let signer_nonce = U256::from(random_manager::u64() % 10);
    let gas_limit = U256::from(random_manager::u64() % 10000);
    let max_fee_per_gas = U256::from(random_manager::u64() % 10000);
    let value = U256::from(random_manager::u64() % 100000);

    let tx = Transaction::new()
        .chain_id(chain_id)
        .from(key_info1.h160_address)
        .recipient(key_info2.h160_address)
        .signer_nonce(signer_nonce)
        .max_fee_per_gas(max_fee_per_gas)
        .gas_limit(gas_limit)
        .value(value);

    let signed_bytes = ab!(tx.sign_as_typed_transaction(k1_signer)).unwrap();
    log::info!("signed_bytes: {}", signed_bytes);

    let (decoded_tx, sig) = decode_signed_rlp(&signed_bytes).unwrap();
    let (decoded_tx2, _tx_hash, signer_addr, sig2) =
        decode_and_verify_signed_rlp(&signed_bytes).unwrap();

    assert_eq!(decoded_tx, decoded_tx2);
    assert_eq!(sig, sig2);
    assert_eq!(decoded_tx.chain_id().unwrap().as_u64(), chain_id);
    assert_eq!(*decoded_tx.from().unwrap(), key_info1.h160_address);
    assert_eq!(signer_addr, key_info1.h160_address);
    assert_eq!(*decoded_tx.to_addr().unwrap(), key_info2.h160_address);
    assert_eq!(decoded_tx.nonce().unwrap().as_u64(), signer_nonce.as_u64());
    assert_eq!(decoded_tx.gas().unwrap().as_u64(), gas_limit.as_u64());
    assert_eq!(decoded_tx.value().unwrap().as_u64(), value.as_u64());
}
