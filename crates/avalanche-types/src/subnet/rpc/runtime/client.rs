use std::io::{Error, ErrorKind, Result};

use tonic::transport::Channel;

use crate::proto::vm::runtime::{runtime_client::RuntimeClient, InitializeRequest};

use super::Initializer;

/// Client is an implementation of [`crate::subnet::rpc::runtime::Initializer`] that talks over RPC.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/rpcchainvm/runtime#Initializer>
#[derive(Debug, Clone)]
pub struct Client {
    inner: RuntimeClient<Channel>,
}

impl Client {
    pub fn new(client_conn: Channel) -> Self {
        Self {
            inner: RuntimeClient::new(client_conn),
        }
    }
}

#[tonic::async_trait]
impl Initializer for Client {
    async fn initialize(&self, protocol_version: u32, vm_server_addr: &str) -> Result<()> {
        let mut client = self.inner.clone();

        client
            .initialize(InitializeRequest {
                protocol_version,
                addr: vm_server_addr.to_owned(),
            })
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("vm initializer failed: {e}")))?;

        Ok(())
    }
}
