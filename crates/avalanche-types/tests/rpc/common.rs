#[cfg(any(test, feature = "proto"))]
use std::io::{self, Error, ErrorKind};

use avalanche_types::{
    proto::pb::rpcdb::database_server::{Database, DatabaseServer},
    subnet::rpc::utils::grpc::default_server,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

pub async fn serve_test_database<D>(database: D, listener: TcpListener) -> io::Result<()>
where
    D: Database,
{
    default_server()
        .add_service(DatabaseServer::new(database))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to serve test database service: {}", e),
            )
        })
}
