use avalanche_types::{
    ids,
    message::{get, put},
};
use avalanchego_conformance_sdk::{Client, GetRequest, PutRequest};

#[tokio::test]
async fn get() {
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
    let msg = get::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .container_id(container_id.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .get(GetRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            container_id: container_id.as_ref().to_vec(),
            serialized_msg,
        })
        .await
        .expect("failed get");
    assert!(resp.success);
}

#[tokio::test]
async fn put() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let container_bytes = random_manager::secure_bytes(100).unwrap();
    let msg = put::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .container(container_bytes.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .put(PutRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            container_bytes,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed put");
    assert!(resp.success);
}

#[tokio::test]
async fn put_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let container_bytes = random_manager::secure_bytes(100).unwrap();
    let msg = put::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .container(container_bytes.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .put(PutRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            container_bytes,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed put");
    assert!(resp.success);
}
