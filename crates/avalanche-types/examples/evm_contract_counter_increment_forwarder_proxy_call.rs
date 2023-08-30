#![allow(deprecated)]

use std::{env::args, io, str::FromStr};

use avalanche_types::{evm::abi, jsonrpc::client::evm as json_client_evm, key, wallet};
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::{H160, U256},
};

/// cargo run --example evm_contract_counter_increment_forwarder_proxy_call --features="jsonrpc_client evm" -- [HTTP RPC ENDPOINT] [PRIVATE KEY] [FORWARDER CONTRACT ADDRESS] [RECIPIENT CONTRACT ADDRESS]
/// cargo run --example evm_contract_counter_increment_forwarder_proxy_call --features="jsonrpc_client evm" -- http://127.0.0.1:9650/ext/bc/C/rpc 56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027 0x7466154c5DE2680Ee2767C763546F052DC7bC393 0x59289F9Ea2432226c8430e3057E2642aD5f979aE
///
/// cast send --gas-price 700000000000 --priority-gas-price 10000000000 --private-key=56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027 --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x59289F9Ea2432226c8430e3057E2642aD5f979aE "increment()"
/// cast call --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x59289F9Ea2432226c8430e3057E2642aD5f979aE "getNumber()" | sed -r '/^\s*$/d' | tail -1
/// cast call --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x59289F9Ea2432226c8430e3057E2642aD5f979aE "getLast()"
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let chain_rpc_url = args().nth(1).expect("no chain RPC URL given");
    let private_key = args().nth(2).expect("no private key given");

    let forwarder_contract_addr = args().nth(3).expect("no contract address given");
    let forwarder_contract_addr =
        H160::from_str(forwarder_contract_addr.trim_start_matches("0x")).unwrap();

    let recipient_contract_addr = args().nth(4).expect("no contract address given");
    let recipient_contract_addr =
        H160::from_str(recipient_contract_addr.trim_start_matches("0x")).unwrap();

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!(
        "running against {chain_rpc_url}, {chain_id} for forwarder contract {forwarder_contract_addr}, recipient contract {recipient_contract_addr}"
    );

    let k = key::secp256k1::private_key::Key::from_hex(private_key).unwrap();
    let key_info = k.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", key_info);
    let signer: ethers_signers::LocalWallet = k.to_ethers_core_signing_key().into();

    let w = wallet::Builder::new(&k)
        .base_http_url(chain_rpc_url.clone())
        .build()
        .await
        .unwrap();
    let evm_wallet = w
        .evm(&signer, chain_rpc_url.as_str(), U256::from(chain_id))
        .unwrap();

    // parsed function of "increment()"
    let func = Function {
        name: "increment".to_string(),
        inputs: vec![],
        outputs: Vec::new(),
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![];
    let increment_calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(increment_calldata.clone()));

    // parsed function of "proxy_call(address target, bytes calldata func)"
    let func = Function {
        name: "proxy_call".to_string(),
        inputs: vec![
            Param {
                name: "target".to_string(),
                kind: ParamType::Address,
                internal_type: None,
            },
            Param {
                name: "func".to_string(),
                kind: ParamType::Bytes,
                internal_type: None,
            },
        ],
        outputs: Vec::new(),
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![
        Token::Address(recipient_contract_addr),
        Token::Bytes(increment_calldata),
    ];
    let calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata.clone()));

    let tx_id = evm_wallet
        .eip1559()
        .recipient(forwarder_contract_addr) // contract address that this transaction will interact with
        .data(calldata)
        .urgent()
        .check_receipt(true)
        .check_acceptance(true)
        .submit()
        .await
        .unwrap();
    log::info!("evm ethers wallet SUCCESS with transaction id {}", tx_id);

    Ok(())
}
