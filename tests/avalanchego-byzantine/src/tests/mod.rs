use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
    time::{Duration, Instant, SystemTime},
};

use avalanche_network_runner_sdk::{Client, GlobalConfig, StartRequest};
use avalanche_types::{
    avm::{self, txs::vertex},
    hash, ids,
    jsonrpc::client::{info as avalanche_sdk_info, x},
    key::{self, secp256k1},
    message,
    packer::Packer,
    txs, utils,
};
use network::peer::outbound;

#[tokio::test]
async fn byzantine() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_network_runner_grpc_endpoint();
    assert!(is_set);

    let cli = Client::new(&ep).await;

    log::info!("ping...");
    let resp = cli.ping().await.expect("failed ping");
    log::info!("network-runner is running (ping response {:?})", resp);

    let (mut exec_path, is_set) = crate::get_network_runner_avalanchego_path();
    if exec_path.is_empty() || !is_set {
        exec_path = avalanche_installer::avalanchego::github::download_latest(None, None)
            .await
            .unwrap();
    }

    log::info!("starting with avalanchego {}...", exec_path);
    let resp = cli
        .start(StartRequest {
            exec_path,
            global_node_config: Some(
                serde_json::to_string(&GlobalConfig {
                    log_level: String::from("info"),
                })
                .unwrap(),
            ),
            ..Default::default()
        })
        .await
        .expect("failed start");
    log::info!(
        "started avalanchego cluster with network-runner: {:?}",
        resp
    );

    // enough time for network-runner to get ready
    thread::sleep(Duration::from_secs(20));

    log::info!("checking cluster healthiness...");
    let mut ready = false;
    let timeout = Duration::from_secs(300);
    let interval = Duration::from_secs(15);
    let start = Instant::now();
    let mut cnt: u128 = 0;
    loop {
        let elapsed = start.elapsed();
        if elapsed.gt(&timeout) {
            break;
        }

        let itv = {
            if cnt == 0 {
                // first poll with no wait
                Duration::from_secs(1)
            } else {
                interval
            }
        };
        thread::sleep(itv);

        ready = {
            match cli.health().await {
                Ok(_) => {
                    log::info!("healthy now!");
                    true
                }
                Err(e) => {
                    log::warn!("not healthy yet {}", e);
                    false
                }
            }
        };
        if ready {
            break;
        }

        cnt += 1;
    }
    assert!(ready);

    log::info!("checking status...");
    let status = cli.status().await.expect("failed status");
    assert!(status.cluster_info.is_some());
    let cluster_info = status.cluster_info.unwrap();
    let mut rpc_eps: Vec<String> = Vec::new();
    for (node_name, iv) in cluster_info.node_infos.into_iter() {
        log::info!("{}: {}", node_name, iv.uri);
        rpc_eps.push(iv.uri.clone());
    }
    log::info!("avalanchego RPC endpoints: {:?}", rpc_eps);

    let resp = avalanche_sdk_info::get_network_id(&rpc_eps[0])
        .await
        .unwrap();
    let network_id = resp.result.unwrap().network_id;

    let resp = avalanche_sdk_info::get_blockchain_id(&rpc_eps[0], "X")
        .await
        .unwrap();
    let x_chain_blockchain_id = resp.result.unwrap().blockchain_id;

    let resp = x::get_asset_description(&rpc_eps[0], "AVAX").await.unwrap();
    let asset_id = resp.result.unwrap().asset_id;

    let resp = avalanche_sdk_info::get_tx_fee(&rpc_eps[0]).await.unwrap();
    let tx_fee = resp.result.unwrap().tx_fee;

    // TODO: check balances for all keys in case TEST_KEYS does not have ewoq key
    let ewoq_key = secp256k1::TEST_KEYS[0].clone();
    let ewoq_keychain = key::secp256k1::keychain::Keychain::new(vec![ewoq_key.clone()]);
    let ewoq_xaddr = ewoq_key
        .to_public_key()
        .to_hrp_address(network_id, "X")
        .unwrap();
    let ewoq_short_addr = ewoq_key.to_public_key().to_short_bytes().unwrap();

    // build vertex and send raw message to the peer bypassing engine
    log::info!("building a vertex with test transactions");
    let resp = x::get_balance(&rpc_eps[0], &ewoq_xaddr).await.unwrap();
    let x_bal = resp.result.unwrap().balance;
    let x_transfer_amount = x_bal / 10;
    log::info!(
        "current balance {}, transfer amount {}",
        x_bal,
        x_transfer_amount
    );

    let conflicting_x_utxos = x::get_utxos(&rpc_eps[0], &ewoq_xaddr).await.unwrap();
    let conflicting_x_utxos = conflicting_x_utxos.result.unwrap().utxos.unwrap();

    let now_unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("unexpected None duration_since")
        .as_secs();

    let mut total_balance_to_spend = 0_u64;
    let mut conflicting_inputs: Vec<txs::transferable::Input> = Vec::new();
    for utxo in conflicting_x_utxos.iter() {
        let (input, _) = ewoq_keychain
            .spend(&utxo.transfer_output.clone().unwrap(), now_unix)
            .unwrap();
        let amount = input.amount;

        conflicting_inputs.push(txs::transferable::Input {
            utxo_id: utxo.utxo_id.clone(),
            asset_id: utxo.asset_id,
            transfer_input: Some(input),
            ..Default::default()
        });

        total_balance_to_spend += amount;
        if total_balance_to_spend > x_transfer_amount + tx_fee {
            break;
        }
    }

    let mut base_txs: Vec<txs::Tx> = Vec::new();
    for i in 0..10 {
        let mut receiver_addr = secp256k1::TEST_KEYS[(i + 1) % secp256k1::TEST_KEYS.len()]
            .to_public_key()
            .to_short_bytes()
            .unwrap();
        if receiver_addr == ewoq_short_addr {
            receiver_addr = secp256k1::TEST_KEYS[1]
                .to_public_key()
                .to_short_bytes()
                .unwrap();
        }
        let outputs = vec![
            // receiver
            txs::transferable::Output {
                asset_id,
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: x_transfer_amount,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![ids::short::Id::from_slice(&receiver_addr)],
                    },
                }),
                ..Default::default()
            },
            // sender
            txs::transferable::Output {
                asset_id,
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: x_bal - x_transfer_amount - tx_fee,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![ids::short::Id::from_slice(&ewoq_short_addr)],
                    },
                }),
                ..Default::default()
            },
        ];
        base_txs.push(txs::Tx {
            network_id,
            blockchain_id: x_chain_blockchain_id,
            transferable_outputs: Some(outputs),
            transferable_inputs: Some(conflicting_inputs.clone()),
            ..Default::default()
        })
    }
    log::info!("sign base transactions to get raw transaction bytes");
    let mut txs: Vec<Vec<u8>> = Vec::new();
    for base_tx in base_txs.iter() {
        let keys: Vec<key::secp256k1::private_key::Key> = vec![ewoq_key.clone()];
        let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys];

        let mut tx = avm::txs::Tx::new(base_tx.clone());
        tx.sign(signers).await.unwrap();

        let tx_bytes_with_signatures = tx
            .base_tx
            .metadata
            .clone()
            .unwrap()
            .tx_bytes_with_signatures;
        txs.push(tx_bytes_with_signatures);
    }
    let mut vtx = vertex::Vertex {
        codec_version: 0,
        chain_id: x_chain_blockchain_id,
        height: 0,
        epoch: 0,
        parent_ids: Vec::new(),
        txs,
    };
    let packer = Packer::new(4096, 0);
    packer.pack_vertex(&mut vtx).unwrap();
    let vtx_bytes = packer.take_bytes();
    let vtx_id = hash::sha256(&vtx_bytes);
    let _vtx_id = ids::Id::from_slice(&vtx_id);

    log::info!("creating test outbound message sender");
    let key_path = random_manager::tmp_path(10, Some(".key")).unwrap();
    let cert_path = random_manager::tmp_path(10, Some(".cert")).unwrap();
    cert_manager::x509::generate_and_write_pem(None, &key_path, &cert_path).unwrap();
    let connector = outbound::Connector::new_from_pem(&key_path, &cert_path).unwrap();

    let (_, host, port, _, _) =
        utils::urls::extract_scheme_host_port_path_chain_alias(&rpc_eps[0]).unwrap();
    log::info!("connecting to the first peer {} with port {:?}", host, port);
    let res = connector.connect(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port.unwrap()),
        Duration::from_secs(10),
    );
    if res.is_err() {
        // TODO: not working "failed to connect TLS connection 'record overflow'"
        log::warn!("failed to connect {:?}", res.err());
        return;
    }
    let mut stream = res.unwrap();

    log::info!("sending test version message");
    let now_unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("unexpected None duration_since")
        .as_secs();
    let msg = message::version::Message::default()
        .network_id(network_id)
        .my_time(now_unix)
        .ip_addr(IpAddr::V4(Ipv4Addr::LOCALHOST))
        .ip_port(port.unwrap() as u32)
        .my_version("avalanche/1.7.11".to_string())
        .my_version_time(now_unix)
        .sig(random_manager::secure_bytes(64).unwrap());
    let msg = msg.serialize().expect("failed serialize");
    stream.write(&msg).unwrap();

    log::info!("sending test push query message");
    let msg = message::push_query::Message::default()
        .chain_id(x_chain_blockchain_id)
        .request_id(random_manager::u32())
        .deadline(random_manager::u64())
        .container(vtx_bytes.as_ref().to_vec());
    let msg = msg.serialize().expect("failed serialize");
    stream.write(&msg).unwrap();

    // enough time for txs processing
    thread::sleep(Duration::from_secs(15));

    // log::info!("checking transaction metrics");
    // let mss = metrics::spawn_get(&rpc_eps[0]).await.unwrap();
    // assert!(mss.avalanche_x_rogue_tx_issued.unwrap_or_default() == 0_f64);
    // assert!(mss.avalanche_x_virtuous_tx_issued.unwrap_or_default() == 0_f64);

    log::info!("stopping...");
    let _resp = cli.stop().await.expect("failed stop");
    log::info!("successfully stopped network");
}
