mod common;
mod database;
mod shutdown;

use crate::rpc::common::*;
use avalanche_types::subnet::rpc::{
    http::{client::Client as HttpClient, server::Server as HttpServer},
    utils,
};
use jsonrpsee_types::{Response as JsonResp, ResponsePayload};
use serde_json::Value;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_http_service() {
    let handler = TestHandler::new();
    let http_server = HttpServer::new(handler);
    let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();

    tokio::spawn(async move {
        serve_test_http_server(http_server, listener).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client_conn = utils::grpc::default_client("127.0.0.1:1234")
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = HttpClient::new(client_conn);

    let foo_request = generate_http_request("foo", "http://127.0.0.1:1234", &[]);
    let foo_resp = client.serve_http_simple(foo_request).await;
    assert!(foo_resp.is_ok());
    let foo_resp = foo_resp.unwrap();

    assert!(foo_resp.status().is_success());

    let json_str = std::str::from_utf8(foo_resp.body());
    assert!(json_str.is_ok());

    let foo_json_resp: JsonResp<Value> = serde_json::from_str(json_str.unwrap()).unwrap();

    match foo_json_resp.payload {
        ResponsePayload::Result(_) => {}
        ResponsePayload::Error(f) => panic!("inner resp invalid: {f}"),
    }

    let bar_request = generate_http_request("bar", "http://127.0.0.1:1234", &["John"]);
    let bar_resp = client.serve_http_simple(bar_request).await;
    assert!(bar_resp.is_ok());
    let bar_resp = bar_resp.unwrap();

    assert!(bar_resp.status().is_success());

    let json_str = std::str::from_utf8(bar_resp.body());
    assert!(json_str.is_ok());
    let bar_json_resp: JsonResp<Value> = serde_json::from_str(json_str.unwrap()).unwrap();

    match bar_json_resp.payload {
        ResponsePayload::Result(_) => {}
        ResponsePayload::Error(f) => panic!("inner resp invalid: {f}"),
    }
}
