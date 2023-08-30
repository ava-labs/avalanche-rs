use std::net::{IpAddr, Ipv6Addr};

use avalanche_types::{ids, message::version};
use avalanchego_conformance_sdk::{Client, VersionRequest};
use log::info;

#[tokio::test]
async fn version() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let my_time = random_manager::u64();
    let sig = random_manager::secure_bytes(64).unwrap();
    let tracked_subnets = vec![
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
    ];
    let mut tracked_subnets_bytes: Vec<Vec<u8>> = Vec::new();
    for id in tracked_subnets.iter() {
        tracked_subnets_bytes.push(id.as_ref().to_vec());
    }

    let msg = version::Message::default()
        .network_id(9999)
        .my_time(my_time)
        .ip_addr(IpAddr::V6(Ipv6Addr::LOCALHOST))
        .ip_port(8080)
        .my_version(String::from("v1.2.3"))
        .my_version_time(my_time)
        .sig(sig.clone())
        .tracked_subnets(tracked_subnets.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .version(VersionRequest {
            network_id: 9999,
            my_time: my_time,
            ip_addr: Ipv6Addr::LOCALHOST.octets().to_vec(),
            ip_port: 8080,
            my_version: String::from("v1.2.3"),
            my_version_time: my_time,
            sig,
            tracked_subnets: tracked_subnets_bytes,
            serialized_msg,
        })
        .await
        .expect("failed version");
    assert!(resp.success);
}
