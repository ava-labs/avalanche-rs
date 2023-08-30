use std::time::Duration;

use avalanche_types::{
    proto::pb::rpcdb::database_server::DatabaseServer,
    subnet::rpc::{
        database::{
            memdb::Database as MemDb,
            rpcdb::{client::DatabaseClient, server::Server as RpcDb},
        },
        utils,
    },
};
use tokio::sync::broadcast::{Receiver, Sender};
use tonic::transport::Channel;

#[tokio::test]
async fn test_shutdown() {
    // initialize broadcast channel
    let (tx, _rx): (Sender<()>, Receiver<()>) = tokio::sync::broadcast::channel(1);

    // setup rpcdb service
    let memdb = MemDb::new();
    let server = RpcDb::new(memdb);
    let svc = DatabaseServer::new(server);
    let addr = utils::new_socket_addr();

    // start gRPC service
    let server = utils::grpc::Server::new(addr, tx.subscribe());
    let resp = server.serve(svc);
    assert!(resp.is_ok());
    tokio::time::sleep(Duration::from_millis(100)).await;

    // setup gRPC client
    let client_conn = Channel::builder(format!("http://{}", addr).parse().unwrap())
        .connect()
        .await
        .unwrap();
    let mut client = DatabaseClient::new(client_conn);

    // client request ok
    let resp = client.put("foo".as_bytes(), "bar".as_bytes()).await;
    assert!(resp.is_ok());

    // broadcast shutdown to server
    let _ = tx.send(());
    tokio::time::sleep(Duration::from_millis(100)).await;

    // client request fail
    let resp = client.put("foo".as_bytes(), "bar".as_bytes()).await;
    assert!(resp.is_err());
}
