use avalanche_types::{
    ids,
    message::{get_state_summary_frontier, state_summary_frontier},
};
use avalanchego_conformance_sdk::{
    Client, GetStateSummaryFrontierRequest, StateSummaryFrontierRequest,
};

#[tokio::test]
async fn state_summary_frontier() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let summary = random_manager::secure_bytes(100).unwrap();
    let msg = state_summary_frontier::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .summary(summary.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .state_summary_frontier(StateSummaryFrontierRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            summary,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed state_summary_frontier");
    assert!(resp.success);
}

#[tokio::test]
async fn state_summary_frontier_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let summary = random_manager::secure_bytes(100).unwrap();
    let msg = state_summary_frontier::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .summary(summary.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .state_summary_frontier(StateSummaryFrontierRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            summary,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed state_summary_frontier");
    assert!(resp.success);
}

#[tokio::test]
async fn get_state_summary_frontier() {
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
    let msg = get_state_summary_frontier::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .get_state_summary_frontier(GetStateSummaryFrontierRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            serialized_msg,
        })
        .await
        .expect("failed get_state_summary_frontier");
    assert!(resp.success);
}
