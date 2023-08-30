use avalanche_types::{ids, message::chits};

use avalanchego_conformance_sdk::{ChitsRequest, Client};

#[tokio::test]
async fn chits() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let container_ids = vec![
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
    ];
    let msg = chits::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .container_ids(container_ids.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let mut container_ids_bytes: Vec<Vec<u8>> = Vec::new();
    for id in container_ids.iter() {
        container_ids_bytes.push(id.as_ref().to_vec());
    }
    let resp = cli
        .chits(ChitsRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            container_ids: container_ids_bytes,
            serialized_msg,
        })
        .await
        .expect("failed chits");
    assert!(resp.success);
}
