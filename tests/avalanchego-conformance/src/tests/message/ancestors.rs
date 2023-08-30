use avalanche_types::{
    ids,
    message::{ancestors, get_ancestors},
};
use avalanchego_conformance_sdk::{AncestorsRequest, Client, GetAncestorsRequest};

#[tokio::test]
async fn ancestors() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let containers = vec![
        random_manager::secure_bytes(11).unwrap(),
        random_manager::secure_bytes(12).unwrap(),
        random_manager::secure_bytes(13).unwrap(),
        random_manager::secure_bytes(14).unwrap(),
        random_manager::secure_bytes(15).unwrap(),
    ];
    let msg = ancestors::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .containers(containers.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .ancestors(AncestorsRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            containers,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed ancestors");
    assert!(resp.success);
}

#[tokio::test]
async fn ancestors_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let containers = vec![
        random_manager::secure_bytes(11).unwrap(),
        random_manager::secure_bytes(12).unwrap(),
        random_manager::secure_bytes(13).unwrap(),
        random_manager::secure_bytes(14).unwrap(),
        random_manager::secure_bytes(15).unwrap(),
    ];
    let msg = ancestors::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .containers(containers.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .ancestors(AncestorsRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            containers,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed ancestors");
    assert!(resp.success);
}

#[tokio::test]
async fn get_ancestors() {
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
    let msg = get_ancestors::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .container_id(container_id.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .get_ancestors(GetAncestorsRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            container_id: container_id.as_ref().to_vec(),
            serialized_msg,
        })
        .await
        .expect("failed get_ancestors");
    assert!(resp.success);
}
