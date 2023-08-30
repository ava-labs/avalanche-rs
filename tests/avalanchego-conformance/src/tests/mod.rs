pub mod key;
pub mod packer;

// TODO: add it back... failing
// pub mod message;

use avalanchego_conformance_sdk::Client;

#[tokio::test]
async fn ping() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let resp = cli.ping_service().await.expect("failed ping_service");
    log::info!(
        "conformance test server is running (ping_service response {:?})",
        resp
    );
}
