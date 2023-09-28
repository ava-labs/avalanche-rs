//! Avalanche JSON-RPC client for P-Chain.
use std::{collections::HashMap, time::Duration};

use crate::{
    errors::{Error, Result},
    ids,
    jsonrpc::client::url,
    jsonrpc::{self, platformvm},
    utils,
};
use reqwest::{header::CONTENT_TYPE, ClientBuilder};

/// "platform.issueTx" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetcurrentvalidators>
pub async fn issue_tx(http_rpc: &str, tx: &str) -> Result<platformvm::IssueTxResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("issuing a transaction via {url}");

    let method = String::from("platform.issueTx");
    let params = platformvm::IssueTxParams {
        tx: prefix_manager::prepend_0x(tx),
        encoding: String::from("hex"), // don't use "cb58"
    }
    .into();

    let data = platformvm::IssueTxRequest {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getTx" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain/#platformgettx>
pub async fn get_tx(http_rpc: &str, tx_id: &str) -> Result<platformvm::GetTxResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting tx via {url}");

    let method = String::from("platform.getTx");
    // TODO: use "hex"?
    let params = HashMap::from([
        (String::from("txID"), String::from(tx_id)),
        (String::from("encoding"), String::from("json")),
    ])
    .into();

    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };
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

/// "platform.getTxStatus" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain/#platformgettxstatus>
pub async fn get_tx_status(http_rpc: &str, tx_id: &str) -> Result<platformvm::GetTxStatusResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting tx status via {url}");

    let method = String::from("platform.getTxStatus");
    let params = HashMap::from([(String::from("txID"), String::from(tx_id))]).into();

    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getHeight" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetheight>
pub async fn get_height(http_rpc: &str) -> Result<platformvm::GetHeightResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting height via {url}");

    let method = String::from("platform.getHeight");
    let params = HashMap::new().into();

    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getBalance" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetbalance>
/// ref. <https://github.com/ava-labs/avalanchego/blob/45ec88151f8a0e3bca1d43fe902fd632c41cd956/vms/platformvm/service.go#L192-L194>
pub async fn get_balance(http_rpc: &str, paddr: &str) -> Result<platformvm::GetBalanceResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting balance via {url} for {}", paddr);

    let method = String::from("platform.getBalance");
    let params = HashMap::from([(String::from("addresses"), vec![paddr.to_string()])]).into();

    let data = jsonrpc::RequestWithParamsHashMapToArray {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getUTXOs" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetutxos>
pub async fn get_utxos(http_rpc: &str, paddr: &str) -> Result<platformvm::GetUtxosResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting UTXOs via {url} for {}", paddr);

    let method = String::from("platform.getUTXOs");
    let params = platformvm::GetUtxosParams {
        addresses: vec![paddr.to_string()],
        limit: 100,
        encoding: String::from("hex"), // don't use "cb58"
    }
    .into();

    let data = platformvm::GetUtxosRequest {
        method,
        params,
        ..Default::default()
    };
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

/// "platform.getCurrentValidators" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetcurrentvalidators>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#ClientPermissionlessValidator>
pub async fn get_primary_network_validators(
    http_rpc: &str,
) -> Result<platformvm::GetCurrentValidatorsResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting primary network validators via {url}");

    let method = String::from("platform.getCurrentValidators");
    let params = HashMap::new().into();

    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };
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

/// "platform.getCurrentValidators" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/build/avalanchego-apis/p-chain/#platformgetcurrentvalidators>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#ClientPermissionlessValidator>
pub async fn get_subnet_validators(
    http_rpc: &str,
    subnet_id: &str,
) -> Result<platformvm::GetCurrentValidatorsResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting subnet validators via {url} for {subnet_id}");

    let method = String::from("platform.getCurrentValidators");
    let params = HashMap::from([(String::from("subnetID"), subnet_id.to_string())]).into();

    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getSubnets" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgetsubnets>
pub async fn get_subnets(
    http_rpc: &str,
    subnet_ids: Option<Vec<ids::Id>>,
) -> Result<platformvm::GetSubnetsResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting subnets via {url}");

    let method = String::from("platform.getSubnets");

    let ids = subnet_ids
        .iter()
        .flat_map(|ids| ids.iter().map(|id| id.to_string()))
        .collect();
    let params = HashMap::from([(String::from("ids"), ids)]).into();

    let data = jsonrpc::RequestWithParamsHashMapToArray {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getBlockchains" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgetblockchains>
pub async fn get_blockchains(http_rpc: &str) -> Result<platformvm::GetBlockchainsResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting blockchain via {url}");

    let method = String::from("platform.getBlockchains");
    let params = HashMap::new().into();
    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };

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

/// "platform.getBlockchainStatus" on "http://\[ADDR\]:9650" and "/ext/P" path.
/// ref. <https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgetblockchainstatus>
pub async fn get_blockchain_status(
    http_rpc: &str,
    blockchain_id: ids::Id,
) -> Result<platformvm::GetBlockchainStatusResponse> {
    let (scheme, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).map_err(|e| {
            Error::Other {
                message: format!("failed extract_scheme_host_port_path_chain_alias '{}'", e),
                retryable: false,
            }
        })?;
    let url = url::try_create_url(url::Path::P, scheme.as_deref(), host.as_str(), port)?;
    log::info!("getting blockchain status via {url} for {blockchain_id}");

    let method = String::from("platform.getBlockchainStatus");
    let params = HashMap::from([(String::from("blockchainID"), blockchain_id.to_string())]).into();

    let data = jsonrpc::Request {
        method,
        params,
        ..Default::default()
    };

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
