use std::io::{Error, ErrorKind, Result};

use crate::{
    ids,
    proto::pb::appsender::{
        app_sender_client, SendAppGossipMsg, SendAppGossipSpecificMsg, SendAppRequestMsg,
        SendAppResponseMsg, SendCrossChainAppRequestMsg, SendCrossChainAppResponseMsg,
    },
};
use prost::bytes::Bytes;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct AppSenderClient {
    inner: app_sender_client::AppSenderClient<Channel>,
}

/// A gRPC client which manages the app sender server instances.
impl AppSenderClient {
    pub fn new(client_conn: Channel) -> Self {
        Self {
            inner: app_sender_client::AppSenderClient::new(client_conn),
        }
    }
}

#[tonic::async_trait]
impl super::AppSender for AppSenderClient {
    /// Send an application-level request.
    /// A nil return value guarantees that for each nodeID in \[nodeIDs\],
    /// the VM corresponding to this AppSender eventually receives either:
    /// * An AppResponse from nodeID with ID \[requestID\]
    /// * An AppRequestFailed from nodeID with ID \[requestID\]
    /// Exactly one of the above messages will eventually be received per nodeID.
    /// A non-nil error should be considered fatal.
    async fn send_app_request(
        &self,
        node_ids: ids::node::Set,
        request_id: u32,
        request: Vec<u8>,
    ) -> Result<()> {
        let mut client = self.inner.clone();

        let mut id_bytes: Vec<Bytes> = Vec::with_capacity(node_ids.len());
        for node_id in node_ids.iter() {
            let node_id = node_id;
            id_bytes.push(Bytes::from(node_id.to_vec()))
        }

        client
            .send_app_request(SendAppRequestMsg {
                node_ids: id_bytes,
                request_id,
                request: Bytes::from(request),
            })
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_app_request failed: {:?}", e),
                )
            })?;

        Ok(())
    }

    /// Send an application-level response to a request.
    /// This response must be in response to an AppRequest that the VM corresponding
    /// to this AppSender received from \[nodeID\] with ID \[requestID\].
    /// A non-nil error should be considered fatal.
    async fn send_app_response(
        &self,
        node_id: ids::node::Id,
        request_id: u32,
        response: Vec<u8>,
    ) -> Result<()> {
        let mut client = self.inner.clone();

        client
            .send_app_response(SendAppResponseMsg {
                node_id: Bytes::from(node_id.to_vec()),
                request_id,
                response: Bytes::from(response),
            })
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_app_response failed: {:?}", e),
                )
            })?;

        Ok(())
    }

    /// Gossip an application-level message.
    /// A non-nil error should be considered fatal.
    async fn send_app_gossip(&self, msg: Vec<u8>) -> Result<()> {
        let mut client = self.inner.clone();

        client
            .send_app_gossip(SendAppGossipMsg {
                msg: Bytes::from(msg),
            })
            .await
            .map_err(|e| {
                Error::new(ErrorKind::Other, format!("send_app_gossip failed: {:?}", e))
            })?;

        Ok(())
    }

    /// Gossip an application-level message to a list of nodeIds.
    async fn send_app_gossip_specific(&self, node_ids: ids::node::Set, msg: Vec<u8>) -> Result<()> {
        let mut client = self.inner.clone();

        let mut node_id_bytes: Vec<Bytes> = Vec::with_capacity(node_ids.len());
        for node_id in node_ids.iter() {
            node_id_bytes.push(Bytes::from(node_id.to_vec()))
        }

        client
            .send_app_gossip_specific(SendAppGossipSpecificMsg {
                node_ids: node_id_bytes,
                msg: Bytes::from(msg),
            })
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_app_gossip_specific failed: {:?}", e),
                )
            })?;

        Ok(())
    }

    async fn send_cross_chain_app_request(
        &self,
        chain_id: ids::Id,
        request_id: u32,
        app_request_bytes: Vec<u8>,
    ) -> Result<()> {
        let mut client = self.inner.clone();

        client
            .send_cross_chain_app_request(SendCrossChainAppRequestMsg {
                chain_id: Bytes::from(chain_id.to_vec()),
                request_id,
                request: Bytes::from(app_request_bytes),
            })
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_cross_chain_app_request failed: {:?}", e),
                )
            })?;

        Ok(())
    }

    async fn send_cross_chain_app_response(
        &self,
        chain_id: ids::Id,
        request_id: u32,
        app_response_bytes: Vec<u8>,
    ) -> Result<()> {
        let mut client = self.inner.clone();

        client
            .send_cross_chain_app_response(SendCrossChainAppResponseMsg {
                chain_id: Bytes::from(chain_id.to_vec()),
                request_id,
                response: Bytes::from(app_response_bytes),
            })
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_cross_chain_app_response failed: {:?}", e),
                )
            })?;

        Ok(())
    }
}
