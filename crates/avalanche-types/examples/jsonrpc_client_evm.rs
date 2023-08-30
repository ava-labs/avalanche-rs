use std::{env::args, io, str::FromStr};

use avalanche_types::jsonrpc::client::evm;

/// cargo run --example jsonrpc_client_evm --features="jsonrpc_client evm" -- [HTTP RPC ENDPOINT] 0x613040a239BDfCF110969fecB41c6f92EA3515C0
/// cargo run --example jsonrpc_client_evm --features="jsonrpc_client evm" -- http://localhost:9650 0x613040a239BDfCF110969fecB41c6f92EA3515C0
/// cargo run --example jsonrpc_client_evm --features="jsonrpc_client evm" -- http://44.230.236.23:9650 0x613040a239BDfCF110969fecB41c6f92EA3515C0
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let http_rpc = args().nth(1).expect("no http_rpc given");
    let caddr = args().nth(2).expect("no C-chain address given");

    let chain_id = evm::chain_id(format!("{http_rpc}/ext/bc/C/rpc").as_str())
        .await
        .unwrap();
    log::info!("chain_id: {:?}", chain_id);

    let balance = evm::get_balance(
        format!("{http_rpc}/ext/bc/C/rpc").as_str(),
        primitive_types::H160::from_str(caddr.trim_start_matches("0x")).unwrap(),
    )
    .await
    .unwrap();
    log::info!("balance: {:?}", balance);

    Ok(())
}
