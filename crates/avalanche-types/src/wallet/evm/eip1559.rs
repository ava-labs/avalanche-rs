use std::ops::Mul;

use crate::{
    errors::{Error, Result},
    key,
    wallet::{self, evm},
};
use ethers::{prelude::Eip1559TransactionRequest, utils::Units::Gwei};
use ethers_core::types::transaction::eip2718;
use ethers_providers::Middleware;
use lazy_static::lazy_static;
use primitive_types::{H160, H256, U256};
use tokio::time::Duration;

// With EIP-1559, the fees are: units of gas used * (base fee + priority fee).
// The expensive but highly guaranteed way of getting transaction in is:
// set very high "max_fee_per_gas" and very low "max_priority_fee_per_gas".
// For example, set "max_fee_per_gas" 500 GWEI and "max_priority_fee_per_gas" 10 GWEI.
// If the base fee is 25 GWEI, it will only cost: units of gas used * (25 + 10).
// If the base fee is 200 GWEI, it will only cost: units of gas used * (200 + 10).
// Therefore, we can set the "max_fee_per_gas" to the actual maximum
// we are willing to pay without manual intervention.
// ref. <https://docs.avax.network/quickstart/adjusting-gas-price-during-high-network-activity>
lazy_static! {
    pub static ref URGENT_MAX_FEE_PER_GAS: U256 = {
        let gwei = U256::from(10).checked_pow(Gwei.as_num().into()).unwrap();
        U256::from(700).mul(gwei) // 700 GWEI
    };
    pub static ref URGENT_MAX_PRIORITY_FEE_PER_GAS: U256 = {
        let gwei = U256::from(10).checked_pow(Gwei.as_num().into()).unwrap();
        U256::from(10).mul(gwei) // 10 GWEI
    };
}

impl<T, S> evm::Evm<T, S>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
    S: ethers_signers::Signer + Clone,
    S::Error: 'static,
{
    #[must_use]
    pub fn eip1559(&self) -> Tx<T, S> {
        Tx::new(self)
    }
}
/// Represents an EIP-1559 Ethereum transaction (dynamic fee transaction in coreth/subnet-evm).
/// ref. <https://ethereum.org/en/developers/docs/transactions>
/// ref. <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1559.md>
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip1559.rs>
/// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
/// ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/core/types#DynamicFeeTx>
#[derive(Clone, Debug)]
pub struct Tx<T, S>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
    S: ethers_signers::Signer + Clone,
    S::Error: 'static,
{
    pub inner: wallet::evm::Evm<T, S>,

    /// Sequence number originated from this account to prevent message replay attack
    /// ref. <https://eips.ethereum.org/EIPS/eip-155>
    ///
    /// Must keep track of nonces when creating transactions programmatically.
    /// If two transactions were transmitted with the same nonce,
    /// only one will be confirmed and the other will be rejected.
    ///
    /// Note that nonce increments regardless whether a transaction execution succeeds or not.
    /// The nonce increments when the transaction is included in the block, but
    /// its execution can fail and still pays the gas.
    ///
    /// None for automatically fetching the next available nonce.
    pub signer_nonce: Option<U256>,

    /// Maximum transaction fee as a premium.
    /// Maps to subnet-evm DynamicFeeTx "GasTipCap".
    /// ref. <https://ethereum.org/en/developers/docs/gas/>
    pub max_priority_fee_per_gas: Option<U256>,

    /// Maximum amount that the originator is willing to pay for this transaction.
    /// Maps to subnet-evm DynamicFeeTx "GasFeeCap".
    /// ref. <https://ethereum.org/en/developers/docs/gas/>
    ///
    /// With EIP-1559, the fees are: units of gas used * (base fee + priority fee).
    /// The expensive but highly guaranteed way of getting transaction in is:
    /// set very high "max_fee_per_gas" and very low "max_priority_fee_per_gas".
    /// For example, set "max_fee_per_gas" 500 GWEI and "max_priority_fee_per_gas" 10 GWEI.
    /// If the base fee is 25 GWEI, it will only cost: units of gas used * (25 + 10).
    /// If the base fee is 200 GWEI, it will only cost: units of gas used * (200 + 10).
    /// Therefore, we can set the "max_fee_per_gas" to the actual maximum
    /// we are willing to pay without manual intervention.
    /// ref. <https://docs.avax.network/quickstart/adjusting-gas-price-during-high-network-activity>
    pub max_fee_per_gas: Option<U256>,

    /// Maximum amount of gas that the originator is willing to buy.
    /// Maximum amount of gas that can be consumed by this transaction.
    /// Think of it as a fuel tank capacity for this specific transaction.
    /// The standard gas limit on Ethereum is 21,000 units (e.g., ETH transfer).
    /// If a user puts a gas limit of 30,000 for a simple ETH transfer,
    /// the EVM would only consume 21,000 units, and the user would get back the
    /// remaining 10,000. If the user puts too low gas limit, the EVM would revert
    /// the change (execution failure).
    ///
    /// Before EIP-1559, if a transaction used up all gas units and the current
    /// gas price is 200 GWEI, this transaction fee can cost up to 21,000 * 200
    /// which is 4,200,000 gwei or 0.0042 ETH.
    /// That is, the fees are: Gas units (limit) * Gas price per unit.
    ///
    /// With EIP-1559, the fees are: units of gas used * (base fee + priority fee).
    /// The base fee is set by the protocol (via chain fee configuration).
    /// The priority fee is set by the user (via "max_priority_fee_per_gas").
    ///
    /// In addition, the user can also set "max_fee_per_gas" for the transaction.
    /// The surplus from the max fee and the actual fee is refunded to the user.
    /// For instance, the refunds are: max fee - (base fee + priority fee).
    /// The "max_fee_per_gas" can limit the maximum amount to pay for the transaction.
    /// ref. <https://ethereum.org/en/developers/docs/gas/>
    ///
    /// This is different than "gas limit" in the chain fee configuration.
    /// Which is the maximum amount of gas that can be consumed per block (e.g., 8-million GWEI).
    /// ref. <https://pkg.go.dev/github.com/ava-labs/subnet-evm/params#pkg-variables>
    pub gas_limit: Option<U256>,

    /// If the recipient is an externally-owned account, the transaction will transfer the "value".
    /// If the recipient is a contract account/address, the transaction will execute the contract code.
    /// If the recipient is None, the transaction is for contract creation.
    /// The contract address is created based on the signer address and transaction nonce.
    pub recipient: Option<H160>,

    /// Transfer amount value.
    pub value: Option<U256>,

    /// Arbitrary data.
    pub data: Option<Vec<u8>>,

    /// Set "true" to check whether a transaction is confirmed using "eth_getTransactionReceipt".
    /// If false, returns the transaction Id immediately after signing and sending the transaction.
    /// The transaction may still be pending.
    /// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt>
    pub check_receipt: bool,

    /// Set "true" to poll transfer status after issuance for its acceptance
    /// by calling "eth_getTransactionByHash".
    /// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionbyhash>
    pub check_acceptance: bool,

    /// Initial wait duration before polling for acceptance.
    pub poll_initial_wait: Duration,
    /// Wait between each poll intervals for acceptance.
    pub poll_interval: Duration,
    /// Maximum duration for polling.
    pub poll_timeout: Duration,

    /// Set to true to return transaction Id for "issue" in dry mode.
    pub dry_mode: bool,
}

impl<T, S> Tx<T, S>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
    S: ethers_signers::Signer + Clone,
    S::Error: 'static,
{
    pub fn new(ev: &wallet::evm::Evm<T, S>) -> Self {
        Self {
            inner: ev.clone(),

            signer_nonce: None,

            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_limit: None,

            recipient: None,
            value: None,
            data: None,

            check_receipt: false,
            check_acceptance: false,

            poll_initial_wait: Duration::from_millis(500),
            poll_interval: Duration::from_millis(700),
            poll_timeout: Duration::from_secs(300),

            dry_mode: false,
        }
    }

    #[must_use]
    pub fn signer_nonce(mut self, signer_nonce: impl Into<U256>) -> Self {
        self.signer_nonce = Some(signer_nonce.into());
        self
    }

    /// Same as "GasTipCap" in subnet-evm.
    #[must_use]
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: impl Into<U256>) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas.into());
        self
    }

    /// Same as "GasFeeCap" in subnet-evm.
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

    /// Overwrites all gas and fee parameters to mark this transaction as urgent.
    #[must_use]
    pub fn urgent(mut self) -> Self {
        self.max_priority_fee_per_gas = Some(*URGENT_MAX_PRIORITY_FEE_PER_GAS);
        self.max_fee_per_gas = Some(*URGENT_MAX_FEE_PER_GAS);
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

    /// Sets the check receipt boolean flag.
    #[must_use]
    pub fn check_receipt(mut self, check_receipt: bool) -> Self {
        self.check_receipt = check_receipt;
        self
    }

    /// Sets the check acceptance boolean flag.
    /// If "true", overwrites "check_receipt" with "true".
    /// If "false", does not overwrite "check_receipt" with "false".
    #[must_use]
    pub fn check_acceptance(mut self, check_acceptance: bool) -> Self {
        if check_acceptance {
            self.check_receipt = true;
        }
        self.check_acceptance = check_acceptance;
        self
    }

    /// Sets the initial poll wait time.
    #[must_use]
    pub fn poll_initial_wait(mut self, poll_initial_wait: Duration) -> Self {
        self.poll_initial_wait = poll_initial_wait;
        self
    }

    /// Sets the poll wait time between intervals.
    #[must_use]
    pub fn poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// Sets the poll timeout.
    #[must_use]
    pub fn poll_timeout(mut self, poll_timeout: Duration) -> Self {
        self.poll_timeout = poll_timeout;
        self
    }

    /// Sets the dry mode boolean flag.
    #[must_use]
    pub fn dry_mode(mut self, dry_mode: bool) -> Self {
        self.dry_mode = dry_mode;
        self
    }

    /// Issues the transaction and returns the transaction Id.
    /// ref. "coreth,subnet-evm/internal/ethapi.SubmitTransaction"
    pub async fn submit(&mut self) -> Result<H256> {
        let max_priority_fee_per_gas = if let Some(v) = self.max_priority_fee_per_gas {
            format!("{} GWEI", super::wei_to_gwei(v))
        } else {
            "default".to_string()
        };
        let max_fee_per_gas = if let Some(v) = self.max_fee_per_gas {
            format!("{} GWEI", super::wei_to_gwei(v))
        } else {
            "default".to_string()
        };

        log::info!(
            "submit tx [chain Id {}, value {:?}, from {}, recipient {:?}, chain RPC URL {}, max_priority_fee_per_gas {max_priority_fee_per_gas}, max_fee_per_gas {max_fee_per_gas}, gas_limit {:?}, dry_mode {}]",
            self.inner.chain_id,
            self.value,
            self.inner.inner.h160_address,
            self.recipient,
            self.inner.chain_rpc_url,
            self.gas_limit,
            self.dry_mode,
        );

        let signer_nonce = if let Some(signer_nonce) = self.signer_nonce {
            log::info!("using the existing signer nonce '{}'", signer_nonce);
            signer_nonce
        } else {
            let fetched_nonce =
                self.inner
                    .middleware
                    .initialize_nonce(None)
                    .await
                    .map_err(|e| {
                        // TODO: check retryable
                        Error::Other {
                            message: format!("failed initialize_nonce '{}'", e),
                            retryable: false,
                        }
                    })?;

            log::info!("no signer nonce, thus fetched/cached '{}'", fetched_nonce);
            self.signer_nonce = Some(fetched_nonce);

            fetched_nonce
        };

        // "from" itself is not RLP-encoded field
        // "from" can be simply derived from signature and transaction hash
        // when the RPC decodes the raw transaction
        // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip1559.rs>
        // ref. <https://eips.ethereum.org/EIPS/eip-1559>
        // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip2718.rs>
        // ref. <https://eips.ethereum.org/EIPS/eip-2718>
        let mut tx_request = Eip1559TransactionRequest::new()
            .from(ethers::prelude::H160::from(
                self.inner.inner.h160_address.as_fixed_bytes(),
            ))
            .chain_id(ethers::prelude::U64::from(self.inner.chain_id.as_u64()))
            .nonce(ethers::prelude::U256::from(signer_nonce.as_u128()));

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

        if self.dry_mode {
            // note that the tx hash is only same iff there's no other worker
            // signing/sending the transaction using the same key
            // because tx hash differs for different nonces, different gas
            // if other workers have used the same key (thus incremented the nonce)
            // the hash returned from dry mode will be different
            // ref. "ethers-middleware/signer" "send_transaction"
            let gas_none = tx_request.gas.is_none();
            let mut typed_tx: eip2718::TypedTransaction = tx_request.into();
            if gas_none {
                log::info!("dry-mode estimating gas");
                let estimated_gas = self
                    .inner
                    .provider
                    .estimate_gas(&typed_tx, None)
                    .await
                    .map_err(|e| {
                        // TODO: check retryable
                        Error::API {
                            message: format!("failed estimate_gas '{}' for dry mode", e),
                            retryable: false,
                        }
                    })?;

                log::info!(
                    "dry-mode caching estimated gas limit {} and updating 'gas' in typed tx",
                    estimated_gas
                );
                self.gas_limit = estimated_gas.into();

                typed_tx.set_gas(estimated_gas);
            };

            let signature = self
                .inner
                .eth_signer
                .sign_transaction(&typed_tx)
                .await
                .map_err(|e| {
                    // TODO: check retryable
                    Error::API {
                        message: format!("failed sign_transaction '{}' for dry-mode", e),
                        retryable: false,
                    }
                })?;
            let precomputed_tx_hash = typed_tx.hash(&signature);

            log::info!(
                "dry-mode pre-computed tx hash '0x{:x}'",
                precomputed_tx_hash
            );
            return Ok(precomputed_tx_hash);
        }

        let pending_tx = self
            .inner
            .middleware
            .send_transaction(tx_request, None)
            .await
            .map_err(|e| {
                // e.g., 'Custom { kind: Other, error: "failed to send_transaction '(code: -32000, message: nonce too low: address 0xaa3033DB04bE0C31967bfC9D0D01bF04a0038526 current nonce (1562) > tx nonce (1561), data: None)'" }'
                // e.g., 'Custom { kind: Other, error: "failed to send_transaction '(code: -32000, message: replacement transaction underpriced, data: None)'" }'
                let mut retryable = false;
                if e.to_string().contains("nonce too low")
                    || e.to_string().contains("transaction underpriced")
                    || e.to_string().contains("dropped from mempool")
                {
                    log::warn!("tx submit failed with a retryable error; '{}'", e);
                    retryable = true;
                }
                Error::API {
                    message: format!("failed to send_transaction '{}'", e),
                    retryable,
                }
            })?;
        let sent_tx_hash = H256(pending_tx.tx_hash().0);
        if !self.check_receipt {
            log::info!("sent tx '0x{:x}'", sent_tx_hash);
            return Ok(sent_tx_hash);
        }

        // blocks until "eth_getTransactionReceipt" returns
        // thus this tx is confirmed (not pending)
        log::info!("checking sent tx receipt '0x{:x}'", sent_tx_hash);
        let tx_receipt = pending_tx.await.map_err(|e| {
            // TODO: check retryable
            Error::API {
                message: format!("failed to wait for pending tx '{}'", e),
                retryable: false,
            }
        })?;

        // "receipt is not available for pending transactions"
        // ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt>
        if tx_receipt.is_none() {
            return Err(Error::API {
                message: "tx dropped from mempool or pending".to_string(),
                retryable: true,
            });
        }

        let tx_receipt = tx_receipt.unwrap();
        let tx_hash = H256(tx_receipt.transaction_hash.0);
        log::info!("confirmed sent tx receipt '0x{:x}'", tx_hash);

        if !self.check_acceptance {
            log::debug!("skipping checking acceptance for '0x{:x}'", tx_hash);
            return Ok(tx_hash);
        }

        // calls "eth_getTransactionByHash"; None when the transaction is pending
        let tx = self
            .inner
            .middleware
            .get_transaction(tx_receipt.transaction_hash)
            .await
            .map_err(|e| {
                // TODO: check retryable
                Error::API {
                    message: format!("failed eth_getTransactionByHash '{}'", e),
                    retryable: false,
                }
            })?;

        // serde_json::to_string(&tx).unwrap()
        if let Some(inner) = &tx {
            if inner.hash() != sent_tx_hash {
                return Err(Error::API {
                    message: format!(
                        "eth_getTransactionByHash returned unexpected tx hash '0x{:x}' (expected '0x{:x}')",
                        inner.hash(), sent_tx_hash
                    ),
                    retryable: false,
                });
            }
            if inner.hash() != tx_receipt.transaction_hash {
                return Err(Error::API {
                    message: format!(
                        "eth_getTransactionByHash returned unexpected tx hash '0x{:x}' (expected '0x{:x}')",
                        inner.hash(), tx_receipt.transaction_hash
                    ),
                    retryable: false,
                });
            }
        } else {
            log::warn!("transaction '0x{:x}' still pending", tx_hash);
            return Err(Error::API {
                message: "tx still pending".to_string(),
                retryable: true,
            });
        }

        log::info!("confirmed tx acceptance '0x{:x}'", tx_hash);
        Ok(tx_hash)
    }
}
