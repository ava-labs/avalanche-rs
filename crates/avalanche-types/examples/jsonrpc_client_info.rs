use std::{
    str::FromStr,
    {env::args, io},
};

use avalanche_types::{ids, jsonrpc::client::info as jsonrpc_client_info};

/// cargo run --example jsonrpc_client_info --features="jsonrpc_client" -- [HTTP RPC ENDPOINT]
/// cargo run --example jsonrpc_client_info --features="jsonrpc_client" -- http://localhost:9650
/// cargo run --example jsonrpc_client_info --features="jsonrpc_client" -- http://52.42.183.125:9650
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let url = args().nth(1).expect("no url given");

    println!();
    let resp = jsonrpc_client_info::get_network_name(&url).await.unwrap();
    log::info!(
        "get_network_name response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::get_network_id(&url).await.unwrap();
    log::info!(
        "get_network_id response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::get_blockchain_id(&url, "X")
        .await
        .unwrap();
    log::info!(
        "get_blockchain_id for X response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );
    log::info!(
        "blockchain_id for X: {}",
        resp.result.unwrap().blockchain_id
    );

    println!();
    let resp = jsonrpc_client_info::get_blockchain_id(&url, "P")
        .await
        .unwrap();
    log::info!("get_blockchain_id for P response: {:?}", resp);
    log::info!(
        "blockchain_id for P: {}",
        resp.result.unwrap().blockchain_id
    );

    println!();
    let resp = jsonrpc_client_info::get_blockchain_id(&url, "C")
        .await
        .unwrap();
    log::info!(
        "get_blockchain_id for C response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );
    log::info!(
        "blockchain_id for C: {}",
        resp.result.unwrap().blockchain_id
    );

    println!();
    let resp = jsonrpc_client_info::get_node_id(&url).await.unwrap();
    log::info!(
        "get_node_id response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );
    assert_eq!(
        resp.result
            .unwrap()
            .node_pop
            .unwrap()
            .pubkey
            .unwrap()
            .to_compressed_bytes()
            .len(),
        48
    );

    println!();
    let resp = jsonrpc_client_info::get_node_version(&url).await.unwrap();
    log::info!(
        "get_node_version response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::get_vms(&url).await.unwrap();
    log::info!(
        "get_vms response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::is_bootstrapped(&url).await.unwrap();
    log::info!(
        "get_bootstrapped response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::get_tx_fee(&url).await.unwrap();
    log::info!(
        "get_tx_fee response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::peers(&url, None).await.unwrap();
    log::info!(
        "peers response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    println!();
    let resp = jsonrpc_client_info::peers(
        &url,
        Some(vec![
            ids::node::Id::from_str("NodeID-BGWCnLx5ZtoAG5iRzTsnihoPNBFxHQApV").unwrap(),
            ids::node::Id::from_str("NodeID-5gjqXKiVoPuDtmbPcUk8vRsFwv7CXgz4U").unwrap(),
            ids::node::Id::from_str("NodeID-4th1GdGLafMcrvd6FJ4p1UQAgU6knpzAZ").unwrap(),
            ids::node::Id::from_str("NodeID-9N65G7BiCi1kjBqwH32p72uboKonjxBGw").unwrap(),
            ids::node::Id::from_str("NodeID-JoG6qMe8mcqSSJNqeZdqeLvePTDfSacwy").unwrap(),
            ids::node::Id::from_str("NodeID-GaTZDZD8wn6GRFohrMcjLECxHgPvdt3iM").unwrap(),
            ids::node::Id::from_str("NodeID-4VULBj2cySv8sf7D7ckajcffCrHKx74Ao").unwrap(),
            ids::node::Id::from_str("NodeID-8HsTgBQ4ruXFqW4Ap8Tm1kVvZ2hLjSXke").unwrap(),
            ids::node::Id::from_str("NodeID-9CtJZ3HeoDtzSArtqYy4b6qCUFo82PvzL").unwrap(),
            ids::node::Id::from_str("NodeID-PC2kBs3FTFccbvdjnX8XJnkEqq4jk4k5R").unwrap(),
            ids::node::Id::from_str("NodeID-P29Bc8DpQ566aBjjxwxnQKArbuMwCw7KR").unwrap(),
            ids::node::Id::from_str("NodeID-M1RFLFjWUEkQZmPKpVMcrEdKFCYNZgkNn").unwrap(),
            ids::node::Id::from_str("NodeID-EqSLQp3e2Cj4BdvM3cy3a2ByVsLwJ6A7T").unwrap(),
            ids::node::Id::from_str("NodeID-EXHG1H7EqtGtEJ5ujZXnjYnnjviz5N9Zh").unwrap(),
            ids::node::Id::from_str("NodeID-6TFpmcUzpi7CuvkGA7ggPWTi69YNT2rTK").unwrap(),
            ids::node::Id::from_str("NodeID-LzfpdvmtxynCC4ZNLQfdP5G4kWnK2YjXw").unwrap(),
            ids::node::Id::from_str("NodeID-4noEHwixf71REyCxEjAr9GVLS7MPw6kEv").unwrap(),
            ids::node::Id::from_str("NodeID-Hnh4TiMEqR4sZoAUdwNcmBQK8LJu1zKb3").unwrap(),
            ids::node::Id::from_str("NodeID-6twDwuqGYuVdJnSYhvW35F951yQZqE5Dp").unwrap(),
        ]),
    )
    .await
    .unwrap();
    log::info!(
        "peers response: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );

    Ok(())
}
