#![allow(deprecated)]

use std::{env::args, io, str::FromStr};

use avalanche_types::{evm::abi, jsonrpc::client::evm as json_client_evm};
use ethers::prelude::Eip1559TransactionRequest;
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::transaction::eip2718::TypedTransaction,
    types::H160,
};
use ethers_providers::{Http, Middleware, Provider};

/// cargo run --example evm_contract_forwarder_get_nonce --features="jsonrpc_client evm" -- [HTTP RPC ENDPOINT] [FORWARDER CONTRACT ADDRESS] [ADDRESS TO GET NONCE FOR]
/// cargo run --example evm_contract_forwarder_get_nonce --features="jsonrpc_client evm" -- http://127.0.0.1:9650/ext/bc/C/rpc 0x52C84043CD9c865236f11d9Fc9F56aa003c1f922 0x52aE9944e80F7Fa6b6F79294F736b5e7c1671f7A
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

    let forwarder_contract_addr = args().nth(2).expect("no contract address given");
    let forwarder_contract_addr =
        H160::from_str(forwarder_contract_addr.trim_start_matches("0x")).unwrap();

    let nonce_addr = args().nth(3).expect("no address given");
    let nonce_addr = H160::from_str(nonce_addr.trim_start_matches("0x")).unwrap();

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!(
        "running against {chain_rpc_url}, {chain_id} for contract {forwarder_contract_addr}"
    );

    // parsed function of "getNonce(address from)"
    let func = Function {
        name: "getNonce".to_string(),
        inputs: vec![Param {
            name: "from".to_string(),
            kind: ParamType::Address,
            internal_type: None,
        }],
        outputs: vec![Param {
            name: "nonce".to_string(),
            kind: ParamType::Uint(256),
            internal_type: None,
        }],
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![Token::Address(nonce_addr)];
    let calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata.clone()));

    let tx = Eip1559TransactionRequest::new()
        .chain_id(chain_id.as_u64())
        .to(ethers::prelude::H160::from(
            forwarder_contract_addr.as_fixed_bytes(),
        ))
        .data(calldata);
    let tx: TypedTransaction = tx.into();

    let output = chain_rpc_provider.call(&tx, None).await.unwrap();
    log::info!("output: {:?}", output);

    Ok(())
}
