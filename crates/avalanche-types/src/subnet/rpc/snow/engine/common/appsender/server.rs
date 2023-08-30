use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use crate::{
    ids,
    proto::pb::{
        self,
        appsender::{
            SendAppGossipMsg, SendAppGossipSpecificMsg, SendAppRequestMsg, SendAppResponseMsg,
            SendCrossChainAppRequestMsg, SendCrossChainAppResponseMsg,
        },
        google::protobuf::Empty,
    },
};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Server {
    pub inner: Arc<RwLock<Box<dyn super::AppSender + Send + Sync>>>,
}

/// A gRPC server which wraps a subnet::rpc::database::Database impl allowing client control over over RPC.
impl Server {
    pub fn new(
        appsender: Box<dyn super::AppSender + Send + Sync>,
    ) -> impl pb::appsender::app_sender_server::AppSender {
        Server {
            inner: Arc::new(RwLock::new(appsender)),
        }
    }
}

#[tonic::async_trait]
impl pb::appsender::app_sender_server::AppSender for Server {
    async fn send_app_request(
        &self,
        request: Request<SendAppRequestMsg>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let appsender = self.inner.read().await;

        let mut node_ids = ids::node::new_set(req.node_ids.len());
        for node_id_bytes in req.node_ids.iter() {
            let node_id = ids::node::Id::from_slice(node_id_bytes);
            node_ids.insert(node_id);
        }

        appsender
            .send_app_request(node_ids, req.request_id, req.request.to_vec())
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_app_request failed: {:?}", e),
                )
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn send_app_response(
        &self,
        request: Request<SendAppResponseMsg>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        let appsender = self.inner.read().await;
        let node_id = ids::node::Id::from_slice(&req.node_id);

        appsender
            .send_app_response(node_id, req.request_id, req.response.to_vec())
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_app_response failed: {:?}", e),
                )
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn send_app_gossip(
        &self,
        request: Request<SendAppGossipMsg>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        let appsender = self.inner.read().await;
        appsender
            .send_app_gossip(req.msg.to_vec())
            .await
            .map_err(|e| {
                Error::new(ErrorKind::Other, format!("send_app_gossip failed: {:?}", e))
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn send_app_gossip_specific(
        &self,
        request: Request<SendAppGossipSpecificMsg>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        let appsender = self.inner.read().await;

        let mut node_ids = ids::node::new_set(req.node_ids.len());
        for node_id_bytes in req.node_ids.iter() {
            let node_id = ids::node::Id::from_slice(node_id_bytes);
            node_ids.insert(node_id);
        }

        appsender
            .send_app_gossip_specific(node_ids, req.msg.to_vec())
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_app_gossip_specific failed: {:?}", e),
                )
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn send_cross_chain_app_request(
        &self,
        request: Request<SendCrossChainAppRequestMsg>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        let appsender = self.inner.read().await;
        appsender
            .send_cross_chain_app_request(
                ids::Id::from_slice(&req.chain_id),
                req.request_id,
                req.request.to_vec(),
            )
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_cross_chain_app_request failed: {:?}", e),
                )
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn send_cross_chain_app_response(
        &self,
        request: Request<SendCrossChainAppResponseMsg>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        let appsender = self.inner.read().await;
        appsender
            .send_cross_chain_app_response(
                ids::Id::from_slice(&req.chain_id),
                req.request_id,
                req.response.to_vec(),
            )
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("send_cross_chain_app_response failed: {:?}", e),
                )
            })?;

        Ok(Response::new(Empty {}))
    }
}
