use avalanche_types::{
    ids,
    message::{accepted_frontier, get_accepted_frontier},
};
use avalanchego_conformance_sdk::{AcceptedFrontierRequest, Client, GetAcceptedFrontierRequest};

#[tokio::test]
async fn accepted_frontier() {
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
    let msg = accepted_frontier::Message::default()
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
        .accepted_frontier(AcceptedFrontierRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            container_ids: container_ids_bytes,
            serialized_msg,
        })
        .await
        .expect("failed accepted_frontier");
    assert!(resp.success);
}

#[tokio::test]
async fn get_accepted_frontier() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let deadline = random_manager::u64();
    let msg = get_accepted_frontier::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .get_accepted_frontier(GetAcceptedFrontierRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            serialized_msg,
        })
        .await
        .expect("failed get_accepted_frontier");
    assert!(resp.success);
}
