pub mod client;
pub mod server;

use std::io::Result;

use crate::ids;

/// AppSender sends application (Vm) level messages.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#AppSender>
#[tonic::async_trait]
pub trait AppSender: Send + Sync + CloneBox {
    async fn send_app_request(
        &self,
        node_ids: ids::node::Set,
        request_id: u32,
        request: Vec<u8>,
    ) -> Result<()>;
    async fn send_app_response(
        &self,
        node_if: ids::node::Id,
        request_id: u32,
        response: Vec<u8>,
    ) -> Result<()>;
    async fn send_app_gossip(&self, msg: Vec<u8>) -> Result<()>;
    async fn send_app_gossip_specific(&self, node_ids: ids::node::Set, msg: Vec<u8>) -> Result<()>;
    async fn send_cross_chain_app_request(
        &self,
        chain_id: ids::Id,
        request_id: u32,
        app_request_bytes: Vec<u8>,
    ) -> Result<()>;
    async fn send_cross_chain_app_response(
        &self,
        chain_id: ids::Id,
        request_id: u32,
        app_response_bytes: Vec<u8>,
    ) -> Result<()>;
}

pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn AppSender + Send + Sync>;
}

impl<T> CloneBox for T
where
    T: 'static + AppSender + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn AppSender + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn AppSender + Send + Sync> {
    fn clone(&self) -> Box<dyn AppSender + Send + Sync> {
        self.clone_box()
    }
}

#[tokio::test]
async fn clone_box_test() {
    use crate::subnet::rpc::snow::engine::common::appsender::client::AppSenderClient;
    use tokio::net::TcpListener;
    use tonic::transport::Channel;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client_conn = Channel::builder(format!("http://{}", addr).parse().unwrap())
        .connect()
        .await
        .unwrap();
    let _app_sender = AppSenderClient::new(client_conn).clone();
}
