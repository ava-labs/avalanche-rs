use std::time::Duration;

use super::serve_test_database;
use avalanche_types::subnet::rpc::{
    database::{
        corruptabledb::Database as CorruptableDb,
        memdb::Database as MemDb,
        rpcdb::{client::DatabaseClient, server::Server as RpcDb},
    },
    errors,
};

use tokio::net::TcpListener;
use tonic::transport::Channel;

// Ensure batched writes work as expected.
#[tokio::test]
async fn batch_put_test() {
    let server = RpcDb::new(MemDb::new());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_test_database(server, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_conn = Channel::builder(format!("http://{}", addr).parse().unwrap())
        .connect()
        .await
        .unwrap();

    let mut db = CorruptableDb::new(DatabaseClient::new(client_conn));

    let key = "hello".as_bytes();
    let value = "world".as_bytes();

    // new batch add key
    let mut batch = db.new_batch().await.unwrap();
    let resp = batch.put(key, value).await;
    assert!(resp.is_ok());
    assert!(!db.has(key).await.unwrap());
    assert!(batch.size().await.unwrap() > 0);

    // write batch
    let resp = batch.write().await;
    assert!(resp.is_ok());
    assert!(db.has(key).await.unwrap());
    assert_eq!(db.get(key).await.unwrap(), value);

    // delete key
    let resp = db.delete(key).await;
    assert!(resp.is_ok());
    assert!(!db.has(key).await.unwrap());

    // close db
    let resp = db.close().await;
    assert!(resp.is_ok());
    let resp = batch.write().await;
    assert!(resp.is_err());
    assert_eq!(
        resp.unwrap_err().to_string(),
        errors::Error::DatabaseClosed.as_str()
    );
}

#[tokio::test]
async fn batch_delete_test() {
    let server = RpcDb::new(MemDb::new());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_test_database(server, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_conn = Channel::builder(format!("http://{}", addr).parse().unwrap())
        .connect()
        .await
        .unwrap();

    let mut db = CorruptableDb::new(DatabaseClient::new(client_conn));

    let key = "hello".as_bytes();
    let value = "world".as_bytes();

    db.put(key, value).await.unwrap();

    // new batch delete key
    let mut batch = db.new_batch().await.unwrap();
    let resp = batch.delete(key).await;
    assert!(resp.is_ok());

    // write batch
    let resp = batch.write().await;
    assert!(resp.is_ok());

    // validate db state
    assert!(!db.has(key).await.unwrap());
    let resp = db.get(key).await;
    assert_eq!(
        resp.unwrap_err().to_string(),
        errors::Error::NotFound.as_str()
    );
    let resp = db.delete(key).await;
    assert!(resp.is_ok());
}

// Tests to make sure that a batch drops un-written operations
// when it is reset.
#[tokio::test]
async fn batch_reset_test() {
    let server = RpcDb::new(MemDb::new());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_test_database(server, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_conn = Channel::builder(format!("http://{}", addr).parse().unwrap())
        .connect()
        .await
        .unwrap();

    let mut db = CorruptableDb::new(DatabaseClient::new(client_conn));

    let key = "hello".as_bytes();
    let value = "world".as_bytes();

    db.put(key, value).await.unwrap();

    // new batch delete key
    let mut batch = db.new_batch().await.unwrap();
    let resp = batch.delete(key).await;
    assert!(resp.is_ok());

    // reset batch
    let _ = batch.reset().await;

    // write batch
    let resp = batch.write().await;
    assert!(resp.is_ok());

    // validate db state
    assert!(db.has(key).await.unwrap());
    let resp = db.get(key).await;
    assert!(resp.is_ok());
    assert_eq!(resp.unwrap(), value);
}
