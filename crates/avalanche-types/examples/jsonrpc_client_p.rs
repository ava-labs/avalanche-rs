use std::{env::args, io};

use avalanche_types::jsonrpc::client::p as jsonrpc_client_p;

/// cargo run --example jsonrpc_client_p --features="jsonrpc_client" -- [HTTP RPC ENDPOINT] P-custom1qwmslrrqdv4slxvynhy9csq069l0u8mqwjzmcd
/// cargo run --example jsonrpc_client_p --features="jsonrpc_client" -- http://52.42.183.125:9650
/// cargo run --example jsonrpc_client_p --features="jsonrpc_client" -- http://52.42.183.125:9650 P-custom1qwmslrrqdv4slxvynhy9csq069l0u8mqwjzmcd
///
/// ```
/// # or run this
/// avalanche-cli-rust get-utxos \
/// --http-rpc-endpoint [HTTP RPC ENDPOINT] \
/// --p-chain-address P-custom1qwmslrrqdv4slxvynhy9csq069l0u8mqwjzmcd
/// ```
///
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let url = args().nth(1).expect("no url given");
    println!("{}", url);

    if let Some(paddr) = &args().nth(2) {
        println!("{}", paddr);
        let resp = jsonrpc_client_p::get_balance(&url, paddr).await.unwrap();
        log::info!(
            "get_balance response: {}",
            serde_json::to_string_pretty(&resp).unwrap()
        );

        let resp = jsonrpc_client_p::get_utxos(&url, paddr).await.unwrap();
        log::info!(
            "get_utxos response: {}",
            serde_json::to_string_pretty(&resp).unwrap()
        );
    }

    let resp = jsonrpc_client_p::get_height(&url).await.unwrap();
    log::info!(
        "get_height response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    let resp = jsonrpc_client_p::get_primary_network_validators(&url)
        .await
        .unwrap();
    log::info!(
        "get_current_validators response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    let resp = jsonrpc_client_p::get_subnets(&url, None).await.unwrap();
    log::info!(
        "get_subnets response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    let resp = jsonrpc_client_p::get_blockchains(&url).await.unwrap();
    log::info!(
        "get_blockchains response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    Ok(())
}
