use avalanche_types::{ids, message::push_query};
use avalanchego_conformance_sdk::{Client, PushQueryRequest};

#[tokio::test]
async fn push_query() {
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
    let container_bytes = random_manager::secure_bytes(100).unwrap();
    let msg = push_query::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .container(container_bytes.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .push_query(PushQueryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            container_bytes,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed push_query");
    assert!(resp.success);
}

#[tokio::test]
async fn push_query_gzip_compress() {
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
    let container_bytes = random_manager::secure_bytes(100).unwrap();
    let msg = push_query::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .container(container_bytes.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .push_query(PushQueryRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            container_bytes,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed push_query");
    assert!(resp.success);
}
