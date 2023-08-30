#![allow(deprecated)]

use std::{env::args, io, str::FromStr};

use avalanche_types::{evm::abi, jsonrpc::client::evm as json_client_evm};
use ethers::prelude::Eip1559TransactionRequest;
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability},
    types::transaction::eip2718::TypedTransaction,
    types::H160,
};
use ethers_providers::{Http, Middleware, Provider};

/// cargo run --example evm_contract_counter_get_last --features="jsonrpc_client evm" -- [HTTP RPC ENDPOINT] [CONTRACT ADDRESS]
/// cargo run --example evm_contract_counter_get_last --features="jsonrpc_client evm" -- http://127.0.0.1:9650/ext/bc/C/rpc 0x5DB9A7629912EBF95876228C24A848de0bfB43A9
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let chain_rpc_url = args().nth(1).expect("no chain RPC URL given");
    let chain_rpc_provider = Provider::<Http>::try_from(chain_rpc_url.clone())
        .expect("could not instantiate HTTP Provider");
    log::info!("created chain rpc server provider for {chain_rpc_url}");

    let contract_addr = args().nth(2).expect("no contract address given");
    let contract_addr = H160::from_str(contract_addr.trim_start_matches("0x")).unwrap();

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!("running against {chain_rpc_url}, {chain_id} for contract {contract_addr}");

    // parsed function of "getLast() public view returns (address)"
    let func = Function {
        name: "getLast".to_string(),
        inputs: vec![],
        outputs: vec![Param {
            name: "address".to_string(),
            kind: ParamType::Address,
            internal_type: None,
        }],
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![];
    let calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata.clone()));

    let tx = Eip1559TransactionRequest::new()
        .chain_id(chain_id.as_u64())
        .to(ethers::prelude::H160::from(contract_addr.as_fixed_bytes()))
        .data(calldata);
    let tx: TypedTransaction = tx.into();

    let output = chain_rpc_provider.call(&tx, None).await.unwrap();
    log::info!("output: {:?}", output);

    Ok(())
}
