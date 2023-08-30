use std::{
    collections::HashSet,
    io::{self, Error, ErrorKind},
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};

use crate::spec::Spec;
use avalanche_types::{
    avm,
    choices::status::Status,
    formatting,
    ids::Id,
    jsonrpc::client::{info as avalanche_sdk_info, x as avalanche_sdk_x},
    key, txs,
};
use rand::{seq::SliceRandom, thread_rng};
use tokio::{
    select,
    sync::{mpsc, RwLock},
    time::{self as TokioTime, Duration as TokioDuration},
};

pub const NAME: &str = "X_BYZANTINE_CONFLICTING_PARALLEL_TRANSFERS";

/// Sends multiple transactions with conflicting inputs (attempt to double-spend)
/// and makes sure only virtuous transactions are accepted.
/// Transactions that follow may fail its liveness due to conflicts created from this call.
pub async fn run(spec: Arc<RwLock<Spec>>) -> io::Result<()> {
    let spec_rlocked = spec.read().await;

    let network_id = spec_rlocked.status.clone().unwrap().network_id;
    let rpc_eps = spec_rlocked.rpc_endpoints.clone();

    let resp = avalanche_sdk_info::get_blockchain_id(&rpc_eps[0], "X")
        .await
        .unwrap();
    let x_chain_blockchain_id = resp.result.unwrap().blockchain_id;

    let resp = avalanche_sdk_x::get_asset_description(&rpc_eps[0], "AVAX")
        .await
        .unwrap();
    let asset_id = resp.result.unwrap().asset_id;

    let resp = avalanche_sdk_info::get_tx_fee(&rpc_eps[0]).await.unwrap();
    let tx_fee = resp.result.unwrap().tx_fee;

    let mut from_idx = -1;
    let mut x_balances: Vec<u64> = Vec::new();
    let mut x_addrs: Vec<String> = Vec::new();
    for (i, k) in spec_rlocked.key_infos.iter().enumerate() {
        let x_addr = k.addresses.get(&network_id).unwrap().x.clone();

        let resp = avalanche_sdk_x::get_balance(&rpc_eps[0], &x_addr)
            .await
            .unwrap();
        let x_bal = resp.result.unwrap().balance;

        x_balances.push(x_bal);
        x_addrs.push(x_addr.clone());

        if from_idx < 0 && x_bal > 0 {
            from_idx = i as i32;
            continue;
        }
    }
    if from_idx < 0 {
        return Err(Error::new(
            ErrorKind::Other,
            "no key found with balance to transfer",
        ));
    }
    let from_idx = from_idx as usize;
    let to_idx = (from_idx + 1) % spec_rlocked.key_infos.len();

    let from_x_bal_orig = x_balances[from_idx];
    let from_x_tx_amount = from_x_bal_orig / 10;
    let from_x_bal_after = from_x_bal_orig - from_x_tx_amount - tx_fee;

    log::info!(
        "transferring {} AVAX from {} to {}",
        from_x_tx_amount,
        x_addrs[from_idx],
        x_addrs[to_idx]
    );

    // ref. "avalanchego/vms/avm#Service.SendMultiple"
    let now_unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("unexpected None duration_since")
        .as_secs();

    log::info!("fetching UTXOs for inputs ");
    let x_utxos = avalanche_sdk_x::get_utxos(&rpc_eps[0], &x_addrs[from_idx])
        .await
        .unwrap();
    let x_utxos = x_utxos.result.unwrap().utxos.unwrap();

    let mut total_balance_to_spend = 0_u64;
    let mut conflicting_inputs: Vec<txs::transferable::Input> = Vec::new();
    for utxo in x_utxos.iter() {
        let keychain = key::secp256k1::keychain::Keychain::new(vec![spec_rlocked.key_infos
            [from_idx]
            .to_private_key()]);

        let (input, _) = keychain
            .spend(&utxo.transfer_output.clone().unwrap(), now_unix)
            .unwrap();
        let amount = input.amount;

        conflicting_inputs.push(txs::transferable::Input {
            utxo_id: utxo.utxo_id.clone(),
            asset_id: utxo.asset_id.clone(),
            transfer_input: Some(input),
            ..Default::default()
        });

        total_balance_to_spend += amount;
        if total_balance_to_spend > from_x_tx_amount + tx_fee {
            break;
        }
    }
    conflicting_inputs.sort();

    let sender_short_addr = spec_rlocked.key_infos[from_idx].short_address.clone();

    let mut base_txs: Vec<txs::Tx> = Vec::new();
    for i in 0..5 {
        let mut target_idx = (i + 1) % spec_rlocked.key_infos.len();
        if target_idx == from_idx {
            target_idx = (target_idx + 1) % spec_rlocked.key_infos.len();
        }
        let target_addr = spec_rlocked.key_infos[target_idx].short_address.clone();

        let outputs = vec![
            // receiver
            txs::transferable::Output {
                asset_id: asset_id.clone(),
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: from_x_tx_amount,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![target_addr.clone()],
                    },
                }),
                ..Default::default()
            },
            // sender
            txs::transferable::Output {
                asset_id: asset_id.clone(),
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: from_x_bal_after,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![sender_short_addr.clone()],
                    },
                }),
                ..Default::default()
            },
        ];
        base_txs.push(txs::Tx {
            network_id,
            blockchain_id: x_chain_blockchain_id.clone(),
            transferable_outputs: Some(outputs),
            transferable_inputs: Some(conflicting_inputs.clone()),
            ..Default::default()
        });
    }

    // the engine "Transitive.batch" validates overlapping transactions
    // before issuing the vertex to the consensus ("Transitive.issueBatch")
    // so people cannot issue rogue transactions through wallet/API
    log::info!("issuing transactions via wallet API");

    let (ch_send, mut ch_recv) = mpsc::channel(base_txs.len());

    // parallelize tx issue
    let mut handles = Vec::new();
    for (i, base_tx) in base_txs.iter().enumerate() {
        let keys: Vec<key::secp256k1::private_key::Key> =
            vec![spec_rlocked.key_infos[from_idx].to_private_key()];
        let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys.clone()];

        let mut tx = avm::txs::Tx::new(base_tx.clone());
        tx.sign(signers).await.unwrap();

        let tx_bytes_with_signatures = tx
            .base_tx
            .metadata
            .clone()
            .unwrap()
            .tx_bytes_with_signatures;
        let hex_tx = formatting::encode_hex_with_checksum(&tx_bytes_with_signatures);
        let rpc_ep = rpc_eps[i % rpc_eps.len()].clone();

        let ch = ch_send.clone();
        let handle = tokio::spawn(async move {
            let tx_id = avalanche_sdk_x::issue_tx(&rpc_ep, &hex_tx).await.unwrap();
            match ch.send(tx_id).await {
                Ok(_) => log::info!("issued a tx"),
                Err(e) => log::warn!("failed to send tx {}", e),
            }
        });

        handles.push(handle);

        // don't make all txs rogue
        // TokioTime::sleep(TokioDuration::from_millis(500)).await;
    }

    let mut tx_ids: Vec<Id> = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("handler await failed {}", e),
                ))
            }
        }

        let delay = TokioTime::sleep(TokioDuration::from_secs(5));
        tokio::pin!(delay);

        let (tx_id_res, success) = select! {
                Some(v) = ch_recv.recv() => (Some(v), true),
                _ = &mut delay => {
                    log::warn!("ch recv timeout...");
                    (None, false)
                },
        };
        if !success {
            return Err(Error::new(
                ErrorKind::Other,
                "channel receive await timeout ",
            ));
        }

        let tr = tx_id_res.unwrap();
        tx_ids.push(tr.result.unwrap().tx_id);
    }

    // enough time for txs processing
    thread::sleep(Duration::from_secs(10));

    let mut accepted: HashSet<Id> = HashSet::new();
    for tx_id in tx_ids.iter() {
        log::info!("checking transaction status for {}", tx_id);

        let mut eps = rpc_eps.clone();
        eps.shuffle(&mut thread_rng());

        for ep in eps.iter() {
            let resp = avalanche_sdk_x::get_tx_status(ep, tx_id.to_string().as_str())
                .await
                .unwrap();

            let status = resp.result.unwrap().status;
            if status == Status::Accepted {
                accepted.insert(tx_id.clone());
                break;
            }

            log::warn!("tx not accepted {} in {}!", status, ep);
        }
    }

    if !accepted.is_empty() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "expected all txs to be rogue, but {} accepted",
                accepted.len()
            ),
        ));
    }

    // log::info!("checking transaction metrics");
    // let mss = avalanche_sdk_metrics::spawn_get(&rpc_eps[0]).await.unwrap();
    // assert!(mss.avalanche_x_rogue_tx_issued.unwrap_or_default() == 0_f64);
    // assert!(mss.avalanche_x_virtuous_tx_issued.unwrap_or_default() == 0_f64);

    for k in spec_rlocked.key_infos.iter() {
        let x_addr = k.addresses.get(&network_id).unwrap().x.clone();

        let resp = avalanche_sdk_x::get_balance(&rpc_eps[0], &x_addr)
            .await
            .unwrap();
        let _x_bal = resp.result.unwrap().balance;
    }

    log::info!("SUCCESS");
    Ok(())
}
