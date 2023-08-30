use avalanche_types::{
    ids,
    message::{app_gossip, app_request, app_response},
};
use avalanchego_conformance_sdk::{
    AppGossipRequest, AppRequestRequest, AppResponseRequest, Client,
};

#[tokio::test]
async fn app_gossip() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let app_bytes = random_manager::secure_bytes(60).unwrap();
    let msg = app_gossip::Message::default()
        .chain_id(chain_id.clone())
        .app_bytes(app_bytes.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .app_gossip(AppGossipRequest {
            chain_id: chain_id.as_ref().to_vec(),
            app_bytes,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed app_gossip");
    assert!(resp.success);
}

#[tokio::test]
async fn app_gossip_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let app_bytes = random_manager::secure_bytes(60).unwrap();
    let msg = app_gossip::Message::default()
        .chain_id(chain_id.clone())
        .app_bytes(app_bytes.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .app_gossip(AppGossipRequest {
            chain_id: chain_id.as_ref().to_vec(),
            app_bytes,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed app_gossip");
    assert!(resp.success);
}

#[tokio::test]
async fn app_request() {
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
    let app_bytes = random_manager::secure_bytes(60).unwrap();
    let msg = app_request::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .app_bytes(app_bytes.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .app_request(AppRequestRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            app_bytes,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed app_request");
    assert!(resp.success);
}

#[tokio::test]
async fn app_request_gzip_compress() {
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
    let app_bytes = random_manager::secure_bytes(60).unwrap();
    let msg = app_request::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .deadline(deadline)
        .app_bytes(app_bytes.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .app_request(AppRequestRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            deadline,
            app_bytes,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed app_request");
    assert!(resp.success);
}

#[tokio::test]
async fn app_response() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let app_bytes = random_manager::secure_bytes(60).unwrap();
    let msg = app_response::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .app_bytes(app_bytes.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .app_response(AppResponseRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            app_bytes,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed app_response");
    assert!(resp.success);
}

#[tokio::test]
async fn app_response_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let chain_id = ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap());
    let request_id = random_manager::u32();
    let app_bytes = random_manager::secure_bytes(60).unwrap();
    let msg = app_response::Message::default()
        .chain_id(chain_id.clone())
        .request_id(request_id)
        .app_bytes(app_bytes.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .app_response(AppResponseRequest {
            chain_id: chain_id.as_ref().to_vec(),
            request_id,
            app_bytes,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed app_response");
    assert!(resp.success);
}
