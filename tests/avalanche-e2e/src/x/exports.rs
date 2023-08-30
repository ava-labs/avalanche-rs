use std::{
    io::{self, Error, ErrorKind},
    sync::Arc,
    time::Duration,
};

use crate::{common, spec::Spec};
use avalanche_types::{
    choices::status::Status,
    jsonrpc::client::{p as avalanche_sdk_p, x as avalanche_sdk_x},
    key, platformvm, wallet,
};
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::sleep};

pub const NAME: &str = "X_EXPORTS";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub rounds: usize,
    pub check_acceptance: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

impl Config {
    pub fn default() -> Self {
        Self {
            rounds: DEFAULT_ROUNDS,
            check_acceptance: DEFAULT_CHECK_ACCEPTANCE,
        }
    }
}

const DEFAULT_ROUNDS: usize = 10;
const DEFAULT_CHECK_ACCEPTANCE: bool = true;

/// Sends a simple X-chain export to P-chain.
/// TODO: make more transfers to evenly distribute funds
pub async fn run(spec: Arc<RwLock<Spec>>) -> io::Result<()> {
    let spec_rlocked = spec.read().await;
    let key_infos = spec_rlocked.key_infos.clone();

    let (rounds, check_acceptance) = if let Some(cfg) = &spec_rlocked.x_exports {
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
        println!(
            "\n\n\n---\n[ROUND #{:02}] making X-chain export to P-chain",
            i
        );
        match make_single_export(
            network_id,
            http_rpc_eps.clone(),
            http_rpc_eps[i % http_rpc_eps.len()].as_str(),
            key_infos.clone(),
            i > 3,
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
async fn make_single_export(
    network_id: u32,
    http_rpc_eps: Vec<String>,
    http_rpc: &str,
    key_infos: Vec<key::secp256k1::Info>,
    permute_keys: bool,
    check_acceptance: bool,
) -> io::Result<()> {
    log::info!("picking address/wallet to transfer");
    let loaded_keys_with_balance =
        common::load_keys_with_balance(key_infos, permute_keys, network_id, http_rpc)
            .await
            .unwrap();
    if loaded_keys_with_balance.key_infos.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "only {} keys (requires >=2 keys",
                loaded_keys_with_balance.key_infos.len()
            ),
        ));
    }
    // ref. https://doc.rust-lang.org/std/fmt/#width
    for (i, _) in loaded_keys_with_balance.key_infos.iter().enumerate() {
        println!(
            "CURRENT BALANCE {:width$} AVAX (SHORT ADDRESS '{}')",
            loaded_keys_with_balance.x_balances[i],
            loaded_keys_with_balance.key_infos[i].short_address.clone(),
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
            "no key found with balance to export",
        ));
    }
    let from_idx = from_idx as usize;

    let w = wallet::Builder::new(&loaded_keys_with_balance.key_infos[from_idx].to_private_key())
        .base_http_url(http_rpc.to_string())
        .build()
        .await
        .unwrap();

    let x_bal = w.x().balance().await.unwrap();
    let p_bal = w.p().balance().await.unwrap();
    log::info!("x balance {}, p balance {}", x_bal, p_bal);

    let tx_id = w
        .x()
        .export()
        .destination_blockchain_id(w.blockchain_id_p.clone())
        .amount(x_bal / 10)
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .unwrap();

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

    // TODO: remove this...
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

    log::info!("SUCCESS with transaction id {}", tx_id);

    let tx_id = w
        .p()
        .import()
        .source_blockchain_id(w.blockchain_id_x.clone())
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .unwrap();

    // enough time for txs processing
    sleep(Duration::from_secs(7)).await;

    let mut accepted = 0_usize;
    for ep in http_rpc_eps.iter() {
        let resp = avalanche_sdk_p::get_tx_status(ep, &tx_id.to_string())
            .await
            .unwrap();

        let status = resp.result.unwrap().status;
        if status == platformvm::txs::status::Status::Committed {
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
            let resp = avalanche_sdk_p::get_tx_status(ep, &tx_id.to_string())
                .await
                .unwrap();

            let status = resp.result.unwrap().status;
            if status == platformvm::txs::status::Status::Committed {
                accepted += 1;
                continue;
            }

            log::warn!("tx not accepted {} in {}!", status, ep);
        }
    }

    let x_bal = w.x().balance().await.unwrap();
    let p_bal = w.p().balance().await.unwrap();
    log::info!("x balance {}, p balance {}", x_bal, p_bal);

    Ok(())
}
