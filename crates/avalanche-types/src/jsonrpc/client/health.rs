//! Avalanche JSON-RPC Health API endpoints.
use std::{sync::Arc, time::Duration};

use crate::{
    errors::{Error, Result},
    jsonrpc::client::url,
    jsonrpc::health,
    utils::urls::extract_scheme_host_port_path_chain_alias,
};
use reqwest::ClientBuilder;

/// "If a single piece of data must be accessible from more than one task
/// concurrently, then it must be shared using synchronization primitives such as Arc."
/// ref. <https://tokio.rs/tokio/tutorial/spawning>
pub async fn check(http_rpc: Arc<String>, liveness: bool) -> Result<health::Response> {
    let (scheme, host, port, _, _) =
        extract_scheme_host_port_path_chain_alias(&http_rpc).map_err(|e| Error::Other {
            message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
            retryable: false,
        })?;

    let mut url = url::try_create_url(url::Path::Health, scheme.as_deref(), host.as_str(), port)?;
    if liveness {
        url = url::try_create_url(url::Path::Liveness, scheme.as_deref(), host.as_str(), port)?;
    }
    log::info!("getting network name for {url}");

    let req_cli_builder = ClientBuilder::new()
        .user_agent(env!("CARGO_PKG_NAME"))
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(15))
        .connection_verbose(true)
        .build()
        .map_err(|e| {
            // TODO: check retryable
            Error::Other {
                message: format!("failed reqwest::ClientBuilder.build '{}'", e),
                retryable: false,
            }
        })?;
    let resp = req_cli_builder
        .get(url.to_string())
        .send()
        .await
        .map_err(|e|
            // TODO: check retryable
            Error::API {
                message: format!("failed reqwest::Client.send '{}'", e),
                retryable: false,
            })?;
    let out = resp.bytes().await.map_err(|e| {
        // TODO: check retryable
        Error::Other {
            message: format!("failed reqwest response bytes '{}'", e),
            retryable: false,
        }
    })?;
    let out: Vec<u8> = out.into();

    serde_json::from_slice(&out).map_err(|e| Error::Other {
        message: format!("failed serde_json::from_slice '{}'", e),
        retryable: false,
    })
}

pub async fn spawn_check(http_rpc: &str, liveness: bool) -> Result<health::Response> {
    let ep_arc = Arc::new(http_rpc.to_string());
    tokio::spawn(async move { check(ep_arc, liveness).await })
        .await
        .expect("failed spawn await")
}
