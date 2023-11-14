use std::time::Duration;

use super::serve_test_database;
use avalanche_types::subnet::rpc::database::{
    corruptabledb::Database as CorruptableDb,
    memdb::Database as MemDb,
    rpcdb::{client::DatabaseClient, server::Server as RpcDb},
};

use tokio::net::TcpListener;
use tonic::transport::Channel;

// Test to make sure the database iterates over the database
// contents lexicographically.
#[tokio::test]
async fn iterator_test() {
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

    let key1 = "hello1".as_bytes();
    let value1 = "world1".as_bytes();
    let key2 = "hello2".as_bytes();
    let value2 = "world2".as_bytes();

    db.put(key1, value1).await.unwrap();
    db.put(key2, value2).await.unwrap();

    let resp = db.new_iterator().await;
    assert!(resp.is_ok());

    let mut iterator = resp.unwrap();

    // first
    assert!(iterator.next().await.unwrap());
    assert_eq!(iterator.key().await.unwrap(), key1);
    assert_eq!(iterator.value().await.unwrap(), value1);

    // second
    assert!(iterator.next().await.unwrap());
    assert_eq!(iterator.key().await.unwrap(), key2);
    assert_eq!(iterator.value().await.unwrap(), value2);

    assert!(!iterator.next().await.unwrap());

    // cleanup
    let _ = iterator.release().await;
}

// Test to make sure the the iterator can be configured to
// start mid way through the database.
#[tokio::test]
async fn iterator_start_test() {
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

    let key1 = "hello1".as_bytes();
    let value1 = "world1".as_bytes();
    let key2 = "goodbye".as_bytes();
    let value2 = "world2".as_bytes();

    db.put(key1, value1).await.unwrap();
    db.put(key2, value2).await.unwrap();

    let resp = db.new_iterator_with_start(key2).await;
    assert!(resp.is_ok());

    let mut iterator = resp.unwrap();

    assert!(iterator.next().await.unwrap());
    assert_eq!(iterator.key().await.unwrap(), key2);
    assert_eq!(iterator.value().await.unwrap(), value2);

    // cleanup
    let _ = iterator.release().await;
}

// Test to make sure the iterator can be configured to skip
// keys missing the provided prefix.
#[tokio::test]
async fn iterator_prefix_test() {
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

    let key1 = "hello1".as_bytes();
    let value1 = "world1".as_bytes();
    let key2 = "goodbye".as_bytes();
    let value2 = "world2".as_bytes();
    let key3 = "joy".as_bytes();
    let value3 = "world3".as_bytes();

    db.put(key1, value1).await.unwrap();
    db.put(key2, value2).await.unwrap();
    db.put(key3, value3).await.unwrap();

    let resp = db.new_iterator_with_prefix("h".as_bytes()).await;
    assert!(resp.is_ok());

    let mut iterator = resp.unwrap();

    assert!(iterator.next().await.unwrap());
    assert_eq!(iterator.key().await.unwrap(), key1);
    assert_eq!(iterator.value().await.unwrap(), value1);

    assert!(!iterator.next().await.unwrap());

    // cleanup
    let _ = iterator.release().await;
}

// Tests to make sure that an iterator on a database will report itself as being
// exhausted and return [ErrClosed] to indicate that the iteration was not
// successful. Additionally tests that an iterator that has already called
// next() can still serve its current value after the underlying DB was closed.
#[tokio::test]
async fn iterator_error_test() {
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

    let key1 = "hello1".as_bytes();
    let value1 = "world1".as_bytes();
    let key2 = "hello2".as_bytes();
    let value2 = "world2".as_bytes();

    db.put(key1, value1).await.unwrap();
    db.put(key2, value2).await.unwrap();

    let resp = db.new_iterator().await;
    assert!(resp.is_ok());

    let mut iterator = resp.unwrap();
    assert!(iterator.next().await.unwrap());

    let resp = db.close().await;
    assert!(resp.is_ok());

    assert_eq!(iterator.key().await.unwrap(), key1);
    assert_eq!(iterator.value().await.unwrap(), value1);

    // Subsequent calls to the iterator should return false and report an error
    assert!(!iterator.next().await.unwrap());

    let resp = iterator.error().await;
    assert!(resp.is_err());
    assert!(resp.unwrap_err().to_string().contains("database closed"));
}

// Tests to make sure that an iterator that was
// released still reports the error correctly.
#[tokio::test]
async fn iterator_error_after_release_test() {
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

    let key1 = "hello1".as_bytes();
    let value1 = "world1".as_bytes();

    db.put(key1, value1).await.unwrap();

    let resp = db.close().await;
    assert!(resp.is_ok());

    let resp = db.new_iterator().await;
    assert!(resp.is_ok());

    let mut iterator = resp.unwrap();
    let _ = iterator.release().await;

    assert!(!iterator.next().await.unwrap());

    assert!(iterator.key().await.unwrap().is_empty());
    assert!(iterator.value().await.unwrap().is_empty());

    let resp = iterator.error().await;
    assert!(resp.is_err());
    assert!(resp.unwrap_err().to_string().contains("database closed"));
}
