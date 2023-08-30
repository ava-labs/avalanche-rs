use avalanche_types::{ids, message::pull_query};
use avalanchego_conformance_sdk::{Client, PullQueryRequest};

#[tokio::test]
async fn pull_query() {
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
    let container_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let msg = pull_query::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .container_id(container_id.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .pull_query(PullQueryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            container_id: container_id.as_ref().to_vec(),
            serialized_msg,
        })
        .await
        .expect("failed pull_query");
    assert!(resp.success);
}
