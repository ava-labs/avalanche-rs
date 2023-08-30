use std::{env::args, io};

use avalanche_types::jsonrpc::client::x as jsonrpc_client_x;

/// cargo run --example jsonrpc_client_x --features="jsonrpc_client" -- [HTTP RPC ENDPOINT] X-custom152qlr6zunz7nw2kc4lfej3cn3wk46u3002k4w5
/// cargo run --example jsonrpc_client_x --features="jsonrpc_client" -- http://44.230.236.23:9650 X-custom152qlr6zunz7nw2kc4lfej3cn3wk46u3002k4w5
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let url = args().nth(1).expect("no url given");
    let xaddr = args().nth(2).expect("no x-chain address given");

    let resp = jsonrpc_client_x::get_balance(&url, &xaddr).await.unwrap();
    log::info!(
        "get_balance response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    let resp = jsonrpc_client_x::get_asset_description(&url, "AVAX")
        .await
        .unwrap();
    log::info!(
        "get_asset_description response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    Ok(())
}
