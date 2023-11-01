#![allow(deprecated)]

use std::{env::args, io, str::FromStr, sync::Arc};

use avalanche_types::{
    evm::{abi, eip712::gsn::Tx},
    jsonrpc::client::evm as json_client_evm,
    key,
    wallet::evm as wallet_evm,
};
use ethers::prelude::Eip1559TransactionRequest;
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::transaction::eip2718::TypedTransaction,
    types::{H160, U256},
};
use ethers_providers::Middleware;
use tokio::time::Duration;

/// cargo run --example evm_contract_counter_increment_forwarder_relay_eip712 --features="jsonrpc_client evm" -- [RELAY SERVER HTTP RPC ENDPOINT] [EVM HTTP RPC ENDPOINT] [FORWARDER CONTRACT ADDRESS] [DOMAIN NAME] [DOMAIN VERSION] [TYPE TYPE NAME] [TYPE SUFFIX DATA] [RECIPIENT CONTRACT ADDRESS]
/// cargo run --example evm_contract_counter_increment_forwarder_relay_eip712 --features="jsonrpc_client evm" -- http://127.0.0.1:9876/rpc http://127.0.0.1:9650/ext/bc/C/rpc 0x52C84043CD9c865236f11d9Fc9F56aa003c1f922 "my domain name" "1" "my type name" "my suffix data" 0x5DB9A7629912EBF95876228C24A848de0bfB43A9
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let relay_server_rpc_url = args().nth(1).expect("no relay server RPC URL given");
    let relay_server_provider = wallet_evm::new_provider(
        &relay_server_rpc_url,
        Duration::from_secs(15),
        Duration::from_secs(30),
        10,
        Duration::from_secs(3),
    )
    .unwrap();
    log::info!("created relay server provider for {relay_server_rpc_url}");

    let chain_rpc_url = args().nth(2).expect("no chain RPC URL given");
    let chain_rpc_provider = wallet_evm::new_provider(
        &chain_rpc_url,
        Duration::from_secs(15),
        Duration::from_secs(30),
        10,
        Duration::from_secs(3),
    )
    .unwrap();
    log::info!("created chain rpc server provider for {chain_rpc_url}");

    let forwarder_contract_addr = args().nth(3).expect("no forwarder contract address given");
    let forwarder_contract_addr =
        H160::from_str(forwarder_contract_addr.trim_start_matches("0x")).unwrap();

    let domain_name = args().nth(4).expect("no domain name given");
    let domain_version = args().nth(5).expect("no domain version given");
    let type_name = args().nth(6).expect("no type suffix data given");
    let type_suffix_data = args().nth(7).expect("no type suffix data given");

    let recipient_contract_addr = args().nth(8).expect("no recipient contract address given");
    let recipient_contract_addr =
        H160::from_str(recipient_contract_addr.trim_start_matches("0x")).unwrap();

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!(
        "running against {chain_rpc_url}, {chain_id} for forwarder contract {forwarder_contract_addr}, recipient contract {recipient_contract_addr}"
    );

    let no_gas_key = key::secp256k1::private_key::Key::generate().unwrap();
    let no_gas_key_info = no_gas_key.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", no_gas_key_info);
    let no_gas_key_signer: ethers_signers::LocalWallet =
        no_gas_key.to_ethers_core_signing_key().into();

    let tx = Eip1559TransactionRequest::new()
        .chain_id(chain_id.as_u64())
        .to(ethers::prelude::H160::from(
            forwarder_contract_addr.as_fixed_bytes(),
        ))
        .data(get_nonce_calldata(no_gas_key_info.h160_address));
    let tx: TypedTransaction = tx.into();
    let output = chain_rpc_provider.call(&tx, None).await.unwrap();
    let forwarder_nonce_no_gas_key = U256::from_big_endian(&output);
    log::info!(
        "forwarder_nonce_no_gas_key: {} {}",
        no_gas_key_info.h160_address,
        forwarder_nonce_no_gas_key
    );

    // parsed function of "increment()"
    let func = Function {
        name: "increment".to_string(),
        inputs: vec![],
        outputs: Vec::new(),
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![];
    let no_gas_recipient_contract_calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!(
        "no gas recipient contract calldata: 0x{}",
        hex::encode(no_gas_recipient_contract_calldata.clone())
    );

    let mut relay_tx = Tx::new()
        //
        // make sure this matches with "registerDomainSeparator" call
        .domain_name(&domain_name)
        //
        .domain_version(&domain_version)
        //
        // local network
        .domain_chain_id(chain_id)
        //
        // trusted forwarder contract address
        .domain_verifying_contract(forwarder_contract_addr)
        .from(no_gas_key_info.h160_address)
        //
        // contract address that this gasless transaction will interact with
        .to(recipient_contract_addr)
        //
        // just some random value, otherwise, estimate gas fails
        .gas(U256::from(30000))
        //
        // contract call needs no value
        .value(U256::zero())
        //
        .nonce(forwarder_nonce_no_gas_key)
        //
        // calldata for contract calls
        .data(no_gas_recipient_contract_calldata)
        //
        .valid_until_time(U256::MAX)
        //
        .type_name(&type_name)
        //
        .type_suffix_data(&type_suffix_data);

    let chain_rpc_provider_arc = Arc::new(chain_rpc_provider);
    let relay_tx_request = relay_tx
        .sign_to_request_with_estimated_gas_with_retries(
            no_gas_key_signer,
            Arc::clone(&chain_rpc_provider_arc),
            Duration::from_secs(30),
            Duration::from_millis(100),
            U256::from(10000),
        )
        .await
        .unwrap();
    log::info!("relay_tx_request: {:?}", relay_tx_request);

    let signed_bytes: ethers_core::types::Bytes =
        serde_json::to_vec(&relay_tx_request).unwrap().into();

    let pending = relay_server_provider
        .send_raw_transaction(signed_bytes)
        .await
        .unwrap();
    log::info!(
        "pending tx hash 0x{:x} using no gas key 0x{:x}",
        pending.tx_hash(),
        no_gas_key_info.h160_address
    );

    Ok(())
}

fn get_nonce_calldata(addr: H160) -> Vec<u8> {
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
    let arg_tokens = vec![Token::Address(addr)];
    abi::encode_calldata(func, &arg_tokens).unwrap()
}
