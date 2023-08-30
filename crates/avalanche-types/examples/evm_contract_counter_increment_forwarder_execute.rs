#![allow(deprecated)]

use std::{env::args, io, str::FromStr};

use avalanche_types::{
    evm::{abi, eip712::gsn::Tx},
    jsonrpc::client::evm as json_client_evm,
    key, wallet,
};
use ethers::prelude::Eip1559TransactionRequest;
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::transaction::eip2718::TypedTransaction,
    types::{H160, U256},
};
use ethers_providers::{Http, Middleware, Provider};

/// Sends a request to the forwarder.
///
/// cargo run --example evm_contract_counter_increment_forwarder_execute --features="jsonrpc_client evm" -- [HTTP RPC ENDPOINT] [GAS PAYER PRIVATE KEY] [FORWARDER CONTRACT ADDRESS] [DOMAIN NAME] [DOMAIN VERSION] [TYPE NAME] [TYPE SUFFIX DATA] [RECIPIENT CONTRACT ADDRESS]
/// cargo run --example evm_contract_counter_increment_forwarder_execute --features="jsonrpc_client evm" -- http://127.0.0.1:9650/ext/bc/C/rpc 56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027 0x52C84043CD9c865236f11d9Fc9F56aa003c1f922 "my domain name" "1" "my type name" "my suffix data" 0x5DB9A7629912EBF95876228C24A848de0bfB43A9
///
/// cast send --gas-price 700000000000 --priority-gas-price 10000000000 --private-key=56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027 --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x5DB9A7629912EBF95876228C24A848de0bfB43A9 "increment()"
/// cast call --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x5DB9A7629912EBF95876228C24A848de0bfB43A9 "getNumber()" | sed -r '/^\s*$/d' | tail -1
/// cast call --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x5DB9A7629912EBF95876228C24A848de0bfB43A9 "getLast()"
///
/// cast receipt --rpc-url=http://127.0.0.1:9650/ext/bc/C/rpc 0x31b977eff419b20c7f0e1c612530258e65cf51a38676b4c7930060ec3b9f10ee
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

    let private_key = args().nth(2).expect("no private key given");

    let forwarder_contract_addr = args().nth(3).expect("no forwarder contract address given");
    let forwarder_contract_addr =
        H160::from_str(forwarder_contract_addr.trim_start_matches("0x")).unwrap();

    let domain_name = args().nth(4).expect("no domain name given");
    let domain_version = args().nth(5).expect("no domain version given");
    let type_suffix_data = args().nth(6).expect("no type suffix data given");

    let recipient_contract_addr = args().nth(7).expect("no recipient contract address given");
    let recipient_contract_addr =
        H160::from_str(recipient_contract_addr.trim_start_matches("0x")).unwrap();

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!(
        "running against {chain_rpc_url}, {chain_id} for forwarder contract {forwarder_contract_addr}, recipient contract {recipient_contract_addr}"
    );

    let no_gas_key = key::secp256k1::private_key::Key::generate().unwrap();
    let no_gas_key_info = no_gas_key.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", no_gas_key_info);
    let no_gas_signer: ethers_signers::LocalWallet = no_gas_key.to_ethers_core_signing_key().into();

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

    let rr_tx = Tx::new()
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
        //
        // gasless transaction signer
        .from(no_gas_key_info.h160_address.clone())
        //
        // contract address that this gasless transaction will interact with
        .to(recipient_contract_addr)
        //
        // fails if zero (e.g., "out of gas")
        // TODO: better estimate gas based on "RelayHub"
        .gas(U256::from(30000))
        //
        // contract call needs no value
        .value(U256::zero())
        //
        // initial nonce is zero
        .nonce(forwarder_nonce_no_gas_key)
        //
        .data(no_gas_recipient_contract_calldata)
        //
        .valid_until_time(U256::MAX)
        //
        .type_name(&domain_name)
        //
        .type_suffix_data(&type_suffix_data);

    let no_gas_sig = rr_tx.sign(no_gas_signer.clone()).await.unwrap();
    log::info!("gas payer sig: 0x{}", hex::encode(no_gas_sig.clone()));

    // zero gas -> immediately error
    let gas_payer_calldata = rr_tx.encode_execute_call(no_gas_sig).unwrap();
    log::info!(
        "gas payer calldata: 0x{}",
        hex::encode(gas_payer_calldata.clone())
    );

    let gas_payer_key = key::secp256k1::private_key::Key::from_hex(private_key).unwrap();
    let gas_payer_key_info = gas_payer_key.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", gas_payer_key_info);
    let gas_payer_signer: ethers_signers::LocalWallet =
        gas_payer_key.to_ethers_core_signing_key().into();
    let w = wallet::Builder::new(&gas_payer_key)
        .base_http_url(chain_rpc_url.clone())
        .build()
        .await
        .unwrap();
    let gas_payer_evm_wallet = w
        .evm(
            &gas_payer_signer,
            chain_rpc_url.as_str(),
            U256::from(chain_id),
        )
        .unwrap();

    let tx_id = gas_payer_evm_wallet
        .eip1559()
        .recipient(forwarder_contract_addr)
        .data(gas_payer_calldata)
        .urgent()
        .check_receipt(true)
        .check_acceptance(true)
        .submit()
        .await
        .unwrap();
    log::info!("evm ethers wallet SUCCESS with transaction id {}", tx_id);

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
