use std::{convert::TryFrom, env::args, io};

use avalanche_types::{evm::eip1559, key};
use ethers_providers::{Http, Middleware, Provider};

/// cargo run --example evm_send_raw_transaction_eip1559_hot_key -- [HTTP RPC ENDPOINT]
/// cargo run --example evm_send_raw_transaction_eip1559_hot_key -- http://localhost:9876/rpc
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let chain_rpc_url = args().nth(1).expect("no chain rpc url given");
    let chain_rpc_provider = Provider::<Http>::try_from(chain_rpc_url.clone())
        .expect("could not instantiate HTTP Provider");
    log::info!("created chain rpc server provider for {chain_rpc_url}");

    let chain_id = random_manager::u64() % 3000;
    let signer_nonce = primitive_types::U256::from(random_manager::u64() % 10);
    let gas_limit = primitive_types::U256::from(random_manager::u64() % 10000);
    let max_fee_per_gas = primitive_types::U256::from(random_manager::u64() % 10000);
    let value = primitive_types::U256::from(random_manager::u64() % 100000);

    let k1 = key::secp256k1::TEST_KEYS[0].clone();
    let key_info1 = k1.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", key_info1);
    let k1_signer: ethers_signers::LocalWallet = k1.to_ethers_core_signing_key().into();

    let k2 = key::secp256k1::private_key::Key::generate().unwrap();
    let key_info2 = k2.to_info(1).unwrap();
    log::info!("created hot key:\n\n{}\n", key_info2);

    let tx = eip1559::Transaction::new()
        .chain_id(chain_id)
        .from(key_info1.h160_address)
        .recipient(key_info2.h160_address)
        .signer_nonce(signer_nonce)
        .max_fee_per_gas(max_fee_per_gas)
        .gas_limit(gas_limit)
        .value(value);

    let signed_bytes = tx.sign_as_typed_transaction(k1_signer).await.unwrap();
    log::info!("signed_bytes: {}", signed_bytes);

    let pending = chain_rpc_provider
        .send_raw_transaction(signed_bytes)
        .await
        .unwrap();
    log::info!("pending tx hash 0x{:x}", pending.tx_hash());

    Ok(())
}
