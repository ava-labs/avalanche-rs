use std::{convert::TryFrom, env::args, io};

use avalanche_types::{
    evm::eip1559,
    key::{self, secp256k1::kms::aws::eth_signer::Signer as KmsAwsSigner},
};
use aws_manager::kms;
use ethers_providers::{Http, Middleware, Provider};
use primitive_types::U256;

/// cargo run --example evm_send_raw_transaction_eip1559_kms_aws -- [HTTP RPC ENDPOINT] [KMS_KEY_ARN]
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

    let kms_key_arn = args().nth(2).expect("no KMS key ARN given");
    log::info!("running with {kms_key_arn}");

    let chain_id = random_manager::u64() % 3000;
    let signer_nonce = U256::from(random_manager::u64() % 10);
    let gas_limit = U256::from(random_manager::u64() % 10000);
    let max_fee_per_gas = U256::from(random_manager::u64() % 10000);
    let value = U256::from(random_manager::u64() % 100000);

    let shared_config = aws_manager::load_config(None, None, None).await;
    let kms_manager = kms::Manager::new(&shared_config);
    let k1 =
        avalanche_types::key::secp256k1::kms::aws::Key::from_arn(kms_manager.clone(), &kms_key_arn)
            .await
            .unwrap();

    let key_info1 = k1.to_info(1).unwrap();
    log::info!("loaded key\n\n{}\n(network Id 1)\n", key_info1);

    let k1_signer = KmsAwsSigner::new(k1, U256::from(chain_id)).unwrap();

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
