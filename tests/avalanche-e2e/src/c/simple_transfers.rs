use std::{
    io::{self, Error, ErrorKind},
    ops::Div,
    sync::Arc,
};

use crate::{common, spec::Spec};
use avalanche_types::{
    jsonrpc::client::evm as client_evm,
    key::{self, secp256k1::kms::aws::eth_signer::Signer as KmsAwsSigner},
    wallet,
};
use aws_manager::kms;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub const NAME: &str = "C_SIMPLE_TRANSFERS";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub rounds: usize,
    pub gas_price: Option<u64>,
    pub gas_limit: u64,
    pub check_acceptance: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rounds: DEFAULT_ROUNDS,
            gas_price: None,
            gas_limit: DEFAULT_GAS_LIMIT,
            check_acceptance: true,
        }
    }
}

const DEFAULT_ROUNDS: usize = 10;
const DEFAULT_GAS_LIMIT: u64 = 21000;

/// Sends a simple C-chain transfer between two addresses
/// and makes sure the AVAX is transferred as requested.
/// TODO: make more transfers to evenly distribute funds
pub async fn run(spec: Arc<RwLock<Spec>>) -> io::Result<()> {
    let spec_rlocked = spec.read().await;

    let (rounds, gas_limit, gas_price, check_acceptance) =
        if let Some(cfg) = &spec_rlocked.c_simple_transfers {
            (
                cfg.rounds,
                cfg.gas_limit,
                cfg.gas_price,
                cfg.check_acceptance,
            )
        } else {
            (DEFAULT_ROUNDS, DEFAULT_GAS_LIMIT, None, false)
        };

    let network_id = spec_rlocked.status.clone().unwrap().network_id;
    let http_rpc_eps = spec_rlocked.rpc_endpoints.clone();

    let chain_id = client_evm::chain_id(format!("{}/ext/bc/C/rpc", http_rpc_eps[0]).as_str())
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to get chainId for C-chain '{}'", e),
            )
        })?;

    log::info!(
        "{}: network Id {}, chain Id {}, rpc endpoints {:?}",
        NAME,
        network_id,
        chain_id,
        http_rpc_eps
    );

    let (mut success, mut failure) = (0_u64, 0_u64);
    for i in 0..rounds {
        let to_permute = i > 5;
        println!(
            "\n\n\n---\n[ROUND #{:02}] making C-chain transfer (to permute {to_permute})",
            i
        );
        match make_single_transfer(
            network_id,
            chain_id,
            i % http_rpc_eps.len(),
            http_rpc_eps.clone(),
            spec_rlocked.key_infos.clone(),
            to_permute,
            gas_price,
            gas_limit,
            check_acceptance,
        )
        .await
        {
            Ok(_) => {
                success += 1;
            }
            Err(e) => {
                failure += 1;
                if !spec_rlocked.ignore_errors {
                    return Err(e);
                } else {
                    log::warn!("ignoring error {}", e);
                }
            }
        }
    }

    log::info!(
        "DONE ROUNDS {}, SUCCESS {}, FAILURE {}",
        rounds,
        success,
        failure
    );
    Ok(())
}

/// Set "permute_keys" to "true" to randomly distribute funds from second runs.
#[allow(clippy::too_many_arguments)]
async fn make_single_transfer(
    network_id: u32,
    chain_id: primitive_types::U256,
    ep_idx: usize,
    http_rpc_eps: Vec<String>,
    key_infos: Vec<key::secp256k1::Info>,
    permute_keys: bool,
    _gas_price: Option<u64>,
    _gas_limit: u64,
    check_acceptance: bool,
) -> io::Result<()> {
    if key_infos.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("only {} keys (requires >=2 keys)", key_infos.len()),
        ));
    }

    log::info!("picking address/wallet to transfer");
    let loaded_keys_with_balance = common::load_keys_with_balance(
        key_infos,
        permute_keys,
        network_id,
        http_rpc_eps[ep_idx].as_str(),
    )
    .await
    .unwrap();

    // ref. https://doc.rust-lang.org/std/fmt/#width
    for (i, _) in loaded_keys_with_balance.key_infos.iter().enumerate() {
        println!(
            "CURRENT BALANCE {:width$} AVAX (ETH ADDRESS '{}')",
            loaded_keys_with_balance.c_balances[i],
            loaded_keys_with_balance.key_infos[i].eth_address,
            width = 25,
        )
    }

    let mut from_idx = -1;
    for (i, _) in loaded_keys_with_balance.key_infos.iter().enumerate() {
        if from_idx < 0 && !loaded_keys_with_balance.c_balances[i].is_zero() {
            from_idx = i as i32;
            break;
        }
    }
    if from_idx < 0 {
        return Err(Error::new(
            ErrorKind::Other,
            "no key found with balance to transfer",
        ));
    }
    let from_idx = from_idx as usize;

    let mut to_idx = -1;
    for (i, _) in loaded_keys_with_balance.key_infos.iter().enumerate() {
        // prioritize the address with zero balance
        if to_idx < 0 && i != from_idx && loaded_keys_with_balance.c_balances[i].is_zero() {
            to_idx = i as i32;
            break;
        }
    }
    let to_idx = {
        if to_idx < 0 {
            // no zero balance address, so just transfer between any two addresses
            (from_idx + 1) % loaded_keys_with_balance.key_infos.len()
        } else {
            to_idx as usize
        }
    };

    let target_h160_addr = loaded_keys_with_balance.key_infos[to_idx].h160_address;

    if loaded_keys_with_balance.key_infos[from_idx].key_type == key::secp256k1::KeyType::AwsKms {
        let shared_config = aws_manager::load_config(None, None, None).await;
        let kms_manager = kms::Manager::new(&shared_config);

        let key = avalanche_types::key::secp256k1::kms::aws::Key::from_arn(
            kms_manager.clone(),
            &loaded_keys_with_balance.key_infos[from_idx]
                .id
                .clone()
                .unwrap(),
        )
        .await
        .unwrap();

        let w = wallet::Builder::new(&key)
            .base_http_url(http_rpc_eps[ep_idx].clone())
            .build()
            .await
            .unwrap();
        let signer = KmsAwsSigner::new(key, chain_id).unwrap();
        let evm_wallet = w
            .evm(
                &signer,
                format!("{}/ext/bc/C/rpc", http_rpc_eps[ep_idx]).as_str(),
                chain_id,
            )
            .unwrap();

        let c_bal = evm_wallet.balance().await.unwrap();
        let transfer_amount = c_bal.div(primitive_types::U256::from(10));

        let mut tx = evm_wallet
            .eip1559()
            .recipient(target_h160_addr)
            .value(transfer_amount)
            .urgent()
            .dry_mode(true)
            .check_acceptance(check_acceptance);

        let txid1 = tx.submit().await.unwrap();
        let txid2 = tx.dry_mode(false).submit().await.unwrap();
        if txid1 != txid2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("dry-mode tx hash 0x{:x} != 0x{:x}", txid1, txid2),
            ));
        }
        log::info!(
            "evm ethers wallet SUCCESS with transaction id 0x{:x}",
            txid2
        );
    } else {
        let w =
            wallet::Builder::new(&loaded_keys_with_balance.key_infos[from_idx].to_private_key())
                .base_http_url(http_rpc_eps[ep_idx].clone())
                .build()
                .await
                .unwrap();

        let pk = loaded_keys_with_balance.key_infos[from_idx].to_private_key();
        let signer: ethers_signers::LocalWallet = pk.to_ethers_core_signing_key().into();

        let evm_wallet = w
            .evm(
                &signer,
                format!("{}/ext/bc/C/rpc", http_rpc_eps[ep_idx]).as_str(),
                chain_id,
            )
            .unwrap();

        let c_bal = evm_wallet.balance().await.unwrap();
        let transfer_amount = c_bal.div(primitive_types::U256::from(10));

        let mut tx = evm_wallet
            .eip1559()
            .recipient(target_h160_addr)
            .value(transfer_amount)
            .urgent()
            .dry_mode(true)
            .check_acceptance(check_acceptance);

        let txid1 = tx.submit().await.unwrap();
        let txid2 = tx.dry_mode(false).submit().await.unwrap();
        if txid1 != txid2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("dry-mode tx hash 0x{:x} != 0x{:x}", txid1, txid2),
            ));
        }

        log::info!("evm ethers wallet SUCCESS with transaction id {}", txid2);
    };

    Ok(())
}
