use std::{
    collections::HashMap,
    io::{self, Error, ErrorKind},
    sync::Arc,
    time::Duration,
};

use crate::{common, spec::Spec};
use avalanche_types::{
    choices::status::Status, jsonrpc::client::x as avalanche_sdk_x, key, wallet,
};
use aws_manager::kms;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::sleep};

pub const NAME: &str = "X_SIMPLE_TRANSFERS";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub rounds: usize,
    pub check_acceptance: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rounds: DEFAULT_ROUNDS,
            check_acceptance: DEFAULT_CHECK_ACCEPTANCE,
        }
    }
}

const DEFAULT_ROUNDS: usize = 10;
const DEFAULT_CHECK_ACCEPTANCE: bool = true;

/// Sends a simple X-chain transfer between two addresses
/// and makes sure the AVAX is transferred as requested.
/// TODO: make more transfers to evenly distribute funds
pub async fn run(spec: Arc<RwLock<Spec>>) -> io::Result<()> {
    let spec_rlocked = spec.read().await;
    let key_infos = spec_rlocked.key_infos.clone();

    let (rounds, check_acceptance) = if let Some(cfg) = &spec_rlocked.x_simple_transfers {
        (cfg.rounds, cfg.check_acceptance)
    } else {
        (DEFAULT_ROUNDS, DEFAULT_CHECK_ACCEPTANCE)
    };

    let http_rpc_eps = spec_rlocked.rpc_endpoints.clone();

    let network_id = spec_rlocked.status.clone().unwrap().network_id;
    log::info!(
        "{}: network id {}, rpc endpoints {:?}",
        NAME,
        network_id,
        http_rpc_eps
    );

    let (mut success, mut failure) = (0_u64, 0_u64);
    for i in 0..rounds {
        let to_permute = i > 5;
        println!(
            "\n\n\n---\n[ROUND #{:02}] making X-chain transfer (to permute {to_permute})",
            i
        );
        match make_single_transfer(
            network_id,
            http_rpc_eps.clone(),
            http_rpc_eps[i % http_rpc_eps.len()].as_str(),
            key_infos.clone(),
            to_permute,
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
/// Returns "true" to retry.
///
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/examples/local_signer.rs>
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/examples/transfer_eth.rs>
///
async fn make_single_transfer(
    network_id: u32,
    http_rpc_eps: Vec<String>,
    http_rpc: &str,
    key_infos: Vec<key::secp256k1::Info>,
    permute_keys: bool,
    check_acceptance: bool,
) -> io::Result<()> {
    if key_infos.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("only {} keys (requires >=2 keys)", key_infos.len()),
        ));
    }

    log::info!("picking address/wallet to transfer");
    let loaded_keys_with_balance =
        common::load_keys_with_balance(key_infos, permute_keys, network_id, http_rpc)
            .await
            .unwrap();

    // ref. https://doc.rust-lang.org/std/fmt/#width
    for (i, _) in loaded_keys_with_balance.key_infos.iter().enumerate() {
        println!(
            "CURRENT BALANCE {:width$} AVAX (SHORT ADDRESS '{}')",
            loaded_keys_with_balance.x_balances[i],
            loaded_keys_with_balance.key_infos[i].short_address,
            width = 25,
        )
    }

    let mut from_idx = -1;
    for (i, _) in loaded_keys_with_balance.key_infos.iter().enumerate() {
        if from_idx < 0 && loaded_keys_with_balance.x_balances[i] > 0 {
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
        if to_idx < 0 && i != from_idx && loaded_keys_with_balance.x_balances[i] == 0 {
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

    let tx_id = if loaded_keys_with_balance.key_infos[from_idx].key_type
        == key::secp256k1::KeyType::AwsKms
    {
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
            .base_http_url(http_rpc.to_string())
            .build()
            .await
            .unwrap();

        let x_bal = w.x().balance().await.unwrap();
        log::info!("sender from_idx {} has balance {}", from_idx, x_bal);

        w.x()
            .transfer()
            .receiver(
                loaded_keys_with_balance.key_infos[to_idx]
                    .short_address
                    .clone(),
            )
            .amount(x_bal / 10)
            .check_acceptance(check_acceptance)
            .issue()
            .await
            .unwrap()
    } else {
        let w =
            wallet::Builder::new(&loaded_keys_with_balance.key_infos[from_idx].to_private_key())
                .base_http_url(http_rpc.to_string())
                .build()
                .await
                .unwrap();

        let x_bal = w.x().balance().await.unwrap();
        log::info!("sender from_idx {} has balance {}", from_idx, x_bal);

        w.x()
            .transfer()
            .receiver(
                loaded_keys_with_balance.key_infos[to_idx]
                    .short_address
                    .clone(),
            )
            .amount(x_bal / 10)
            .check_acceptance(check_acceptance)
            .issue()
            .await
            .unwrap()
    };

    if !check_acceptance {
        log::info!("skipping checking acceptance...");
        return Ok(());
    }

    // enough time for txs processing
    sleep(Duration::from_secs(7)).await;

    let mut accepted = 0_usize;
    for ep in http_rpc_eps.iter() {
        let resp = avalanche_sdk_x::get_tx_status(ep, &tx_id.to_string())
            .await
            .unwrap();

        let status = resp.result.unwrap().status;
        if status == Status::Accepted {
            accepted += 1;
            continue;
        }

        log::warn!("tx not accepted {} in {}!", status, ep);
    }

    if accepted != http_rpc_eps.len() {
        log::warn!("tx fetching retries...");

        // enough time for txs processing
        sleep(Duration::from_secs(5)).await;

        accepted = 0_usize;
        for ep in http_rpc_eps.iter() {
            let resp = avalanche_sdk_x::get_tx_status(ep, &tx_id.to_string())
                .await
                .unwrap();

            let status = resp.result.unwrap().status;
            if status == Status::Accepted {
                accepted += 1;
                continue;
            }

            log::warn!("tx not accepted {} in {}!", status, ep);
        }
    }

    if accepted != http_rpc_eps.len() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "accepted only found {}, expected {}",
                accepted,
                http_rpc_eps.len()
            ),
        ));
    }

    log::info!("make sure all nodes accept the tx");
    for ep in http_rpc_eps.iter() {
        let req_cli_builder = ClientBuilder::new()
            .user_agent(env!("CARGO_PKG_NAME"))
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(15))
            .connection_verbose(true)
            .build()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed ClientBuilder build {}", e),
                )
            })?;
        let resp = req_cli_builder
            .get(format!("{ep}/ext/metrics").as_str())
            .send()
            .await
            .map_err(|e| {
                Error::new(ErrorKind::Other, format!("failed ClientBuilder send {}", e))
            })?;
        let out = resp.bytes().await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed ClientBuilder bytes {}", e),
            )
        })?;
        let out: Vec<u8> = out.into();

        let s = match prometheus_manager::Scrape::from_bytes(&out) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, format!("failed scrape {}", e)));
            }
        };
        let matched = prometheus_manager::find_all(&s.metrics, |s| {
            s.metric
                .contains("avalanche_X_vm_avalanche_base_txs_accepted")
        });
        let mut cur: HashMap<String, f64> = HashMap::new();
        for m in matched {
            cur.insert(m.metric.clone(), m.value.to_f64());
        }

        if *cur
            .get("avalanche_X_vm_avalanche_base_txs_accepted")
            .unwrap()
            == 0_f64
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "{} unexpected 'avalanche_X_vm_avalanche_base_txs_accepted' {}, expected >0",
                    ep.as_str(),
                    *cur.get("avalanche_X_vm_avalanche_base_txs_accepted")
                        .unwrap(),
                ),
            ));
        }
    }

    log::info!("SUCCESS with transaction id {}", tx_id);
    Ok(())
}
