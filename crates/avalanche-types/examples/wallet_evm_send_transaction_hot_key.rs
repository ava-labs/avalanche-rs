use std::{env::args, io, ops::Div};

use avalanche_types::{jsonrpc::client::evm as json_client_evm, key, wallet};
use ethers::utils::format_units;
use ethers_providers::Middleware;
use primitive_types::U256;

/// cargo run --example wallet_evm_send_transaction_hot_key -- [HTTP RPC ENDPOINT] [PRIVATE KEY]
/// cargo run --example wallet_evm_send_transaction_hot_key -- http://3.37.240.20:9650/ext/bc/C/rpc 56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027
/// cargo run --example wallet_evm_send_transaction_hot_key -- http://3.37.240.20:9650/ext/bc/jyMffWvvB6Jd6C3ZqSuz67dMQUsMSmvZyLKLu26MrgFhjinst/rpc 56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let chain_rpc_url = args().nth(1).expect("no chain RPC URL given");
    let private_key = args().nth(2).expect("no private key given");

    let chain_id = json_client_evm::chain_id(&chain_rpc_url).await.unwrap();
    log::info!("running against {chain_rpc_url}, {chain_id}");

    let k1 = key::secp256k1::private_key::Key::from_hex(private_key).unwrap();
    let key_info1 = k1.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", key_info1);

    let k1_signer: ethers_signers::LocalWallet = k1.to_ethers_core_signing_key().into();

    let k2 = key::secp256k1::private_key::Key::generate().unwrap();
    let key_info2 = k2.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", key_info2);

    let w = wallet::Builder::new(&k1)
        .base_http_url(chain_rpc_url.clone())
        .build()
        .await
        .unwrap();
    let evm_wallet = w
        .evm(&k1_signer, chain_rpc_url.as_str(), U256::from(chain_id))
        .unwrap();

    let c_bal = evm_wallet.balance().await.unwrap();
    let transfer_amount = c_bal.div(U256::from(10));

    let (max_fee_per_gas, max_priority_fee_per_gas) = evm_wallet
        .middleware
        .estimate_eip1559_fees(None)
        .await
        .unwrap();
    let max_fee_per_gas_in_gwei = wallet::evm::wei_to_gwei(max_fee_per_gas);
    let max_fee_per_gas_in_avax = format_units(max_fee_per_gas, "ether").unwrap();
    let max_priority_fee_per_gas_in_gwei = wallet::evm::wei_to_gwei(max_priority_fee_per_gas);
    let max_priority_fee_per_gas_in_avax = format_units(max_priority_fee_per_gas, "ether").unwrap();

    log::info!(
        "[estimated] max_fee_per_gas: {max_fee_per_gas_in_gwei} GWEI, {max_fee_per_gas_in_avax} AVAX"
    );
    log::info!("[estimated] max_priority_fee_per_gas: {max_priority_fee_per_gas_in_gwei} GWEI, {max_priority_fee_per_gas_in_avax} AVAX");

    let tx_id = evm_wallet
        .eip1559()
        .recipient(key_info2.h160_address)
        .value(transfer_amount)
        .urgent()
        .check_receipt(true)
        .check_acceptance(true)
        .submit()
        .await
        .unwrap();
    log::info!("evm ethers wallet SUCCESS with transaction id {}", tx_id);

    Ok(())
}
