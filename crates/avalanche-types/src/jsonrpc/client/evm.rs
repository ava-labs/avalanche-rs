//! EVM RPC client.
use std::time::Duration;

use crate::errors::{Error, Result};
use ethers_providers::{Http, Middleware, Provider};
use primitive_types::{H160, U256};

/// Fetches the chain Id from "{http_rpc}/ext/bc/{chain_id_alias}/rpc".
/// "chain_id_alias" is "C" for C-chain, and blockchain Id for subnet-evm.
pub async fn chain_id(rpc_ep: &str) -> Result<U256> {
    let provider = Provider::<Http>::try_from(rpc_ep)
        .map_err(|e| {
            // TODO: check retryable
            Error::API {
                message: format!("failed to create provider '{}'", e),
                retryable: false,
            }
        })?
        .interval(Duration::from_millis(2000u64));

    log::info!("getting chain id via {rpc_ep}");
    provider.get_chainid().await.map_err(|e|
            // TODO: check retryable
            Error::API {
                message: format!("failed to get_chainid '{}'", e),
                retryable: false,
            })
}

/// Fetches the balance from "{http_rpc}/ext/bc/{chain_id_alias}/rpc".
/// "chain_id_alias" is "C" for C-chain, and blockchain Id for subnet-evm.
/// ref. <https://docs.avax.network/build/avalanchego-apis/c-chain#eth_getassetbalance>
pub async fn get_balance(rpc_ep: &str, eth_addr: H160) -> Result<U256> {
    let provider = Provider::<Http>::try_from(rpc_ep)
        .map_err(|e| {
            // TODO: check retryable
            Error::API {
                message: format!("failed to create provider '{}'", e),
                retryable: false,
            }
        })?
        .interval(Duration::from_millis(2000u64));

    log::info!("getting balances for {} via {rpc_ep}", eth_addr);
    provider.get_balance(eth_addr, None).await.map_err(|e|
            // TODO: check retryable
            Error::API {
                message: format!("failed get_balance '{}'", e),
                retryable: false,
            })
}
