use std::io::Result;

use crate::ids;
use chrono::{DateTime, Utc};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#NetworkAppHandler>
#[tonic::async_trait]
pub trait NetworkAppHandler {
    async fn app_request(
        &self,
        node_id: &ids::node::Id,
        request_id: u32,
        deadline: DateTime<Utc>,
        request: &[u8],
    ) -> Result<()>;
    async fn app_request_failed(&self, node_id: &ids::node::Id, request_id: u32) -> Result<()>;
    async fn app_response(
        &self,
        node_id: &ids::node::Id,
        request_id: u32,
        response: &[u8],
    ) -> Result<()>;
    async fn app_gossip(&self, node_id: &ids::node::Id, msg: &[u8]) -> Result<()>;
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#CrossChainAppHandler>
#[tonic::async_trait]
pub trait CrossChainAppHandler {
    async fn cross_chain_app_request(
        &self,
        chain_id: &ids::Id,
        request_id: u32,
        deadline: DateTime<Utc>,
        request: &[u8],
    ) -> Result<()>;
    async fn cross_chain_app_request_failed(
        &self,
        chain_id: &ids::Id,
        request_id: u32,
    ) -> Result<()>;
    async fn cross_chain_app_response(
        &self,
        chain_id: &ids::Id,
        request_id: u32,
        response: &[u8],
    ) -> Result<()>;
}

// Defines how a consensus engine reacts to app specific messages.
// Functions only return fatal errors.
pub trait AppHandler: NetworkAppHandler + CrossChainAppHandler {}
