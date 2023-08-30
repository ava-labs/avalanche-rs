//! Avalanche X-Chain JSON-RPC client.
use std::{collections::HashMap, time::Duration};

use crate::{
    errors::{Error, Result},
    jsonrpc::client::url,
    jsonrpc::{self, avm},
    utils,
};
use reqwest::{header::CONTENT_TYPE, ClientBuilder};

/// e.g., "avm.issueTx" on "http://\[ADDR\]:9650" and "/ext/bc/X" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmissuetx>
pub async fn issue_tx(http_rpc: &str, tx: &str) -> Result<avm::IssueTxResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::X, scheme.as_deref(), host.as_str(), port)?;
    log::info!("issuing a transaction via {url}");

    let mut data = avm::IssueTxRequest::default();
    data.method = String::from("avm.issueTx");
    let params = avm::IssueTxParams {
        tx: prefix_manager::prepend_0x(tx),
        encoding: String::from("hex"), // don't use "cb58"
    };
    data.params = Some(params);
    let d = data.encode_json().map_err(|e| Error::Other {
        message: format!("failed encode_json '{}'", e),
        retryable: false,
    })?;

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
        .post(url.to_string())
        .header(CONTENT_TYPE, "application/json")
        .body(d)
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

/// e.g., "avm.getTxStatus" on "http://\[ADDR\]:9650" and "/ext/bc/X" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgettxstatus>
pub async fn get_tx_status(http_rpc: &str, tx_id: &str) -> Result<avm::GetTxStatusResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::X, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting tx status via {url}");

    let mut data = jsonrpc::Request::default();
    data.method = String::from("avm.getTxStatus");
    let mut params = HashMap::new();
    params.insert(String::from("txID"), String::from(tx_id));
    data.params = Some(params);
    let d = data.encode_json().map_err(|e| Error::Other {
        message: format!("failed encode_json '{}'", e),
        retryable: false,
    })?;

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
        .post(url.to_string())
        .header(CONTENT_TYPE, "application/json")
        .body(d)
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

/// e.g., "avm.getBalance" on "http://\[ADDR\]:9650" and "/ext/bc/X" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/x-chain#avmgetbalance>
pub async fn get_balance(http_rpc: &str, xaddr: &str) -> Result<avm::GetBalanceResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::X, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting balance via {url} for {xaddr}");

    let mut data = jsonrpc::Request::default();
    data.method = String::from("avm.getBalance");
    let mut params = HashMap::new();
    params.insert(String::from("assetID"), String::from("AVAX"));
    params.insert(String::from("address"), xaddr.to_string());
    data.params = Some(params);
    let d = data.encode_json().map_err(|e| Error::Other {
        message: format!("failed encode_json '{}'", e),
        retryable: false,
    })?;

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
        .post(url.to_string())
        .header(CONTENT_TYPE, "application/json")
        .body(d)
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

/// e.g., "avm.getAssetDescription".
/// ref. <https://docs.avax.network/build/avalanchego-apis/x-chain/#avmgetassetdescription>
pub async fn get_asset_description(
    http_rpc: &str,
    asset_id: &str,
) -> Result<avm::GetAssetDescriptionResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::X, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting asset description via {url}");

    let mut data = jsonrpc::Request::default();
    data.method = String::from("avm.getAssetDescription");
    let mut params = HashMap::new();
    params.insert(String::from("assetID"), String::from(asset_id));
    data.params = Some(params);
    let d = data.encode_json().map_err(|e| Error::Other {
        message: format!("failed encode_json '{}'", e),
        retryable: false,
    })?;

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
        .post(url.to_string())
        .header(CONTENT_TYPE, "application/json")
        .body(d)
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

/// e.g., "avm.getUTXOs" on "http://\[ADDR\]:9650" and "/ext/bc/X" path.
/// TODO: support paginated calls
/// ref. <https://docs.avax.network/apis/avalanchego/apis/x-chain/#avmgetutxos>
pub async fn get_utxos(http_rpc: &str, xaddr: &str) -> Result<avm::GetUtxosResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::X, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting UTXOs via {url} for {xaddr}");

    let mut data = avm::GetUtxosRequest::default();
    data.method = String::from("avm.getUTXOs");
    let params = avm::GetUtxosParams {
        addresses: vec![xaddr.to_string()],
        limit: 1024,
        encoding: String::from("hex"), // don't use "cb58"
    };
    data.params = Some(params);
    let d = data.encode_json().map_err(|e| Error::Other {
        message: format!("failed encode_json '{}'", e),
        retryable: false,
    })?;

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
        .post(url.to_string())
        .header(CONTENT_TYPE, "application/json")
        .body(d)
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

/// e.g., "avm.issueStopVertex" on "http://\[ADDR\]:9650" and "/ext/bc/X" path.
/// Issue itself is asynchronous, so the internal error is not exposed!
pub async fn issue_stop_vertex(http_rpc: &str) -> Result<()> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::X, scheme.as_deref(), host.as_str(), port)?;
    log::info!("issuing a stop vertex transaction via {url}");

    let mut data = avm::IssueStopVertexRequest::default();
    data.method = String::from("avm.issueStopVertex");
    let params = avm::IssueStopVertexParams {};
    data.params = Some(params);
    let d = data.encode_json().map_err(|e| Error::Other {
        message: format!("failed encode_json '{}'", e),
        retryable: false,
    })?;

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
        .post(url.to_string())
        .header(CONTENT_TYPE, "application/json")
        .body(d)
        .send()
        .await
        .map_err(|e|
            // TODO: check retryable
            Error::API {
                message: format!("failed reqwest::Client.send '{}'", e),
                retryable: false,
            })?;

    if !resp.status().is_success() {
        return Err(Error::API {
            message: format!("status code non-success {}", resp.status()),
            retryable: false,
        });
    }

    Ok(())
}
