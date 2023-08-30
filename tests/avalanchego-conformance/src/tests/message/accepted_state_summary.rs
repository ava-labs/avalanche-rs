use avalanche_types::{
    ids,
    message::{accepted_state_summary, get_accepted_state_summary},
};
use avalanchego_conformance_sdk::{
    AcceptedStateSummaryRequest, Client, GetAcceptedStateSummaryRequest,
};
use log::info;

#[tokio::test]
async fn accepted_state_summary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let summary_ids = vec![
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
    ];
    let msg = accepted_state_summary::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .summary_ids(summary_ids.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    info!("sending message ({} bytes)", serialized_msg.len());

    let mut summary_ids_bytes: Vec<Vec<u8>> = Vec::new();
    for id in summary_ids.iter() {
        summary_ids_bytes.push(id.as_ref().to_vec());
    }
    let resp = cli
        .accepted_state_summary(AcceptedStateSummaryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            summary_ids: summary_ids_bytes,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed accepted_state_summary");
    assert!(resp.success);
}

#[tokio::test]
async fn accepted_state_summary_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let summary_ids = vec![
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
    ];
    let msg = accepted_state_summary::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .summary_ids(summary_ids.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    info!("sending message ({} bytes)", serialized_msg.len());

    let mut summary_ids_bytes: Vec<Vec<u8>> = Vec::new();
    for id in summary_ids.iter() {
        summary_ids_bytes.push(id.as_ref().to_vec());
    }
    let resp = cli
        .accepted_state_summary(AcceptedStateSummaryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            summary_ids: summary_ids_bytes,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed accepted_state_summary");
    assert!(resp.success);
}

#[tokio::test]
async fn get_accepted_state_summary() {
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
    let heights: Vec<u64> = vec![random_manager::u64(), random_manager::u64()];
    let msg = get_accepted_state_summary::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .heights(heights.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .get_accepted_state_summary(GetAcceptedStateSummaryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            heights,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed get_accepted_state_summary");
    assert!(resp.success);
}

#[tokio::test]
async fn get_accepted_state_summary_gzip_compress() {
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
    let heights: Vec<u64> = vec![random_manager::u64(), random_manager::u64()];
    let msg = get_accepted_state_summary::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .heights(heights.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .get_accepted_state_summary(GetAcceptedStateSummaryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            heights,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed get_accepted_state_summary");
    assert!(resp.success);
}
