#![allow(deprecated)]

use std::{env::args, io, str::FromStr};

use avalanche_types::{evm::abi, jsonrpc::client::evm as json_client_evm, key, wallet};
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::{H160, U256},
};

/// cargo run --example evm_contract_simple_registry_register --features="jsonrpc_client evm" -- [HTTP RPC ENDPOINT] [PRIVATE KEY] [CONTRACT ADDRESS]
/// cargo run --example evm_contract_simple_registry_register --features="jsonrpc_client evm" -- http://127.0.0.1:9650/ext/bc/C/rpc 56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027 0x95CA0a568236fC7413Cd2b794A7da24422c2BBb6
///
/// cast call --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x95CA0a568236fC7413Cd2b794A7da24422c2BBb6 "getName(address addr)" "0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC" | sed -r '/^\s*$/d' | tail -1 | xxd -r -p
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let chain_rpc_url = args().nth(1).expect("no chain RPC URL given");
    let private_key = args().nth(2).expect("no private key given");
    let contract_addr = args().nth(3).expect("no contract address given");
    let contract_addr = H160::from_str(contract_addr.trim_start_matches("0x")).unwrap();

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!("running against {chain_rpc_url}, {chain_id} for contract {contract_addr}");

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

    // parsed function of "register(string name)"
    let func = Function {
        name: "register".to_string(),
        inputs: vec![Param {
            name: "name".to_string(),
            kind: ParamType::String,
            internal_type: None,
        }],
        outputs: Vec::new(),
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![Token::String(random_manager::secure_string(10))];
    let calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata.clone()));

    let tx_id = evm_wallet
        .eip1559()
        .recipient(contract_addr) // contract address that this transaction will interact with
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
