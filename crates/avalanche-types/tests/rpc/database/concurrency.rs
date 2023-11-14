use std::time::Duration;

use super::serve_test_database;
use avalanche_types::subnet::rpc::database::{
    memdb::Database as MemDb,
    rpcdb::{client::DatabaseClient, server::Server as RpcDb},
};
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::net::TcpListener;
use tonic::transport::Channel;

#[tokio::test(flavor = "multi_thread")]
async fn rpcdb_mutation_test() {
    let db = MemDb::new();
    let server = RpcDb::new(db);

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

    let client = DatabaseClient::new(client_conn);
    let mut futures = FuturesUnordered::new();
    // 1000 requests
    for i in 0..1000_i32 {
        let mut client = client.clone();
        futures.push(async move {
            client
                .put("foo".as_bytes(), format!("bar-{}", i).as_bytes())
                .await
        });
    }

    while let Some(res) = futures.next().await {
        assert!(res.is_ok());
    }
}
