use std::env::args;

use avalanchego_conformance_sdk::Client;
use tokio::runtime::Runtime;

/// cargo run --example ping_service -- [HTTP RPC ENDPOINT]
/// cargo run --example ping_service -- http://127.0.0.1:22342
fn main() {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let url = args().nth(1).expect("no url given");
    let rt = Runtime::new().unwrap();

    log::info!("creating client");
    let cli = rt.block_on(Client::new(&url));

    let resp = rt
        .block_on(cli.ping_service())
        .expect("failed ping_service");
    log::info!("ping_service response: {:?}", resp);
}
