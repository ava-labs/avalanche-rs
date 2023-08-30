pub mod eip1559;

use std::{ops::Div, sync::Arc, time::Duration};

use crate::{
    errors::{Error, Result},
    jsonrpc::client::evm as jsonrpc_client_evm,
    key, wallet,
};
use ethers::{
    prelude::{
        gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
        NonceManagerMiddleware, SignerMiddleware,
    },
    utils::Units::Gwei,
};
use ethers_providers::{Http, HttpRateLimitRetryPolicy, Provider, RetryClient};
use lazy_static::lazy_static;
use primitive_types::U256;
use reqwest::ClientBuilder;
use url::Url;

#[derive(Clone, Debug)]
pub struct Evm<T, S>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
    S: ethers_signers::Signer + Clone,
    S::Error: 'static,
{
    pub inner: wallet::Wallet<T>,

    pub chain_rpc_url: String,
    pub provider: Arc<Provider<RetryClient<Http>>>,

    pub eth_signer: S,

    /// Middleware created on the picked RPC endpoint and signer address.
    /// ref. "ethers-middleware::signer::SignerMiddleware"
    /// ref. "ethers-signers::LocalWallet"
    /// ref. "ethers-signers::wallet::Wallet"
    /// ref. "ethers-signers::wallet::Wallet::sign_transaction_sync"
    /// ref. <https://github.com/giantbrain0216/ethers_rs/blob/master/ethers-middleware/tests/nonce_manager.rs>
    pub middleware: Arc<
        NonceManagerMiddleware<
            SignerMiddleware<GasEscalatorMiddleware<Arc<Provider<RetryClient<Http>>>>, S>,
        >,
    >,

    pub chain_id: U256,
}

impl<T, S> Evm<T, S>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
    S: ethers_signers::Signer + Clone,
    S::Error: 'static,
{
    /// Fetches the current balance of the wallet owner.
    pub async fn balance(&self) -> Result<U256> {
        let cur_balance =
            jsonrpc_client_evm::get_balance(&self.chain_rpc_url, self.inner.h160_address).await?;
        Ok(cur_balance)
    }
}

impl<T> wallet::Wallet<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    /// Sets the chain RPC URLs (can be different than base HTTP URLs).
    /// e.g., "{base_http_url}/ext/bc/{chain_id_alias}/rpc"
    /// Set "chain_id_alias" to either "C" or subnet-evm chain Id.
    #[must_use]
    pub fn evm<S>(&self, eth_signer: &S, chain_rpc_url: &str, chain_id: U256) -> Result<Evm<T, S>>
    where
        S: ethers_signers::Signer + Clone,
        S::Error: 'static,
    {
        // TODO: make timeouts + retries configurable
        let provider = new_provider(
            chain_rpc_url,
            Duration::from_secs(15),
            Duration::from_secs(30),
            5,
            Duration::from_secs(3),
        )?;
        let provider_arc = Arc::new(provider);

        let nonce_middleware = new_middleware(Arc::clone(&provider_arc), eth_signer, chain_id)?;
        let middleware = Arc::new(nonce_middleware);

        Ok(Evm::<T, S> {
            inner: self.clone(),

            chain_rpc_url: chain_rpc_url.to_string(),
            provider: provider_arc,

            eth_signer: eth_signer.clone(),

            middleware,

            chain_id,
        })
    }
}

/// Make sure to not create multiple providers for the ease of nonce management.
/// ref. "`Provider::<RetryClient<Http>>::new_client`".
pub fn new_provider(
    chain_rpc_url: &str,
    connect_timeout: Duration,
    request_timeout: Duration,
    max_retries: u32,
    backoff_timeout: Duration,
) -> Result<Provider<RetryClient<Http>>> {
    let u = Url::parse(chain_rpc_url).map_err(|e| Error::Other {
        message: format!("failed to parse chain RPC URL {}", e),
        retryable: false,
    })?;

    let http_cli = ClientBuilder::new()
        .user_agent(env!("CARGO_PKG_NAME"))
        .connect_timeout(connect_timeout)
        .connection_verbose(true)
        .timeout(request_timeout)
        .danger_accept_invalid_certs(true) // make this configurable
        .build()
        .map_err(|e| {
            // TODO: check retryable
            Error::Other {
                message: format!("failed reqwest::ClientBuilder.build '{}'", e),
                retryable: false,
            }
        })?;

    // TODO: make "HttpRateLimitRetryPolicy" configurable
    let retry_client = RetryClient::new(
        Http::new_with_client(u, http_cli),
        Box::new(HttpRateLimitRetryPolicy),
        max_retries,
        backoff_timeout.as_millis() as u64,
    );

    let provider = Provider::new(retry_client).interval(Duration::from_millis(2000u64));
    Ok(provider)
}

/// Make sure to not create multiple providers for the ease of nonce management.
pub fn new_middleware<S>(
    provider: Arc<Provider<RetryClient<Http>>>,
    eth_signer: &S,
    chain_id: U256,
) -> Result<
    NonceManagerMiddleware<
        SignerMiddleware<GasEscalatorMiddleware<Arc<Provider<RetryClient<Http>>>>, S>,
    >,
>
where
    S: ethers_signers::Signer + Clone,
    S::Error: 'static,
{
    // TODO: make this configurable
    let escalator = GeometricGasPrice::new(5.0, 10u64, None::<u64>);

    // TODO: this can lead to file descriptor leaks!!!
    // ref. <https://github.com/gakonst/ethers-rs/issues/2269>
    let gas_escalator_middleware =
        GasEscalatorMiddleware::new(Arc::clone(&provider), escalator, Frequency::PerBlock);

    let signer_middleware = SignerMiddleware::new(
        gas_escalator_middleware,
        eth_signer.clone().with_chain_id(chain_id.as_u64()),
    );

    let nonce_middleware = NonceManagerMiddleware::new(signer_middleware, eth_signer.address());
    Ok(nonce_middleware)
}

lazy_static! {
    pub static ref GWEI: U256 = U256::from(10).checked_pow(Gwei.as_num().into()).unwrap();
}

/// Converts WEI to GWEI.
pub fn wei_to_gwei(wei: impl Into<U256>) -> U256 {
    let wei: U256 = wei.into();
    if wei.is_zero() {
        U256::zero()
    } else {
        wei.div(*GWEI)
    }
}
