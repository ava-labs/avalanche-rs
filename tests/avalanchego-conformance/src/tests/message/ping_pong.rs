use avalanche_types::message::{ping, pong};
use avalanchego_conformance_sdk::{Client, PingRequest, PongRequest};

#[tokio::test]
async fn ping() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let msg = ping::Message::default();
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .ping(PingRequest { serialized_msg })
        .await
        .expect("failed ping");
    assert!(resp.success);
}

#[tokio::test]
async fn pong() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let uptime_pct = random_manager::u32();
    let msg = pong::Message::default().uptime_pct(uptime_pct);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let resp = cli
        .pong(PongRequest {
            uptime_pct,
            serialized_msg,
        })
        .await
        .expect("failed message_pong");
    assert!(resp.success);
}
