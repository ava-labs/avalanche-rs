use std::error::Error;
use std::fmt::Debug;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use prost::Message;
use avalanche_types::ids::node::Id;
use crate::p2p;
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::sdk::{PullGossipRequest, PullGossipResponse};

pub struct HandlerConfig {
    pub namespace: String,
    pub target_response_size: usize,
}

#[derive(Debug, Clone)]
pub struct Handler<S: Set + Debug> {
    pub set: Arc<Mutex<S>>,
    pub target_response_size: usize,

}

pub fn new_handler<S: Set + Debug>(
    config: HandlerConfig,
    set: Arc<Mutex<S>>,
) -> Handler<S> {
    Handler {
        set,
        target_response_size: config.target_response_size,
    }
}

#[async_trait]
#[allow(unused_variables)]
impl<S> p2p::handler::Handler for Handler<S>
    where
        S: Set + Debug,
        S::Item: Default
{
    async fn app_gossip(&self, node_id: Id, gossip_bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let request = PullGossipRequest::decode(gossip_bytes.as_slice()).expect("Failed to decode request_bytes into PullGossipRequest");

        let mut response_size = 0_usize;
        let mut gossip_bytes: Vec<Vec<u8>> = Vec::new();
        let guard = self.set.try_lock().expect("Lock failed on set");
        guard.iterate(&|gossipable| {
            let bytes = match gossipable.serialize() {
                Ok(b) => {
                    b
                }
                Err(_) => {
                    return false;
                }
            };

            gossip_bytes.push(bytes.clone());
            response_size += bytes.len();

            response_size <= self.target_response_size
        });
        let mut response = PullGossipResponse::default();
        response.gossip = gossip_bytes;

        let mut response_bytes = vec![];
        response.encode(&mut response_bytes).expect("Failed to encode response_bytes into PullGossipResponse");

        Ok(response_bytes)
    }

    async fn app_request(
        &self,
        _: Id,
        _: Duration,
        request_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let request = PullGossipRequest::decode(request_bytes.as_slice()).expect("Failed to decode request_bytes");

        let mut response_size = 0_usize;
        let mut gossip_bytes: Vec<Vec<u8>> = Vec::new();

        self.set.lock().await.iterate(&|gossipable| {
            let bytes = match gossipable.serialize() {
                Ok(b) => b,
                Err(_) => return false,
            };

            gossip_bytes.push(bytes.clone());
            response_size += bytes.len();

            response_size <= self.target_response_size
        });

        let mut response = PullGossipResponse::default();
        response.gossip = gossip_bytes;

        let mut response_bytes = vec![];
        response.encode(&mut response_bytes).expect("s");

        Ok(response_bytes)
    }

    async fn cross_chain_app_request(&self, chain_id: Id, deadline: Duration, request_bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        unimplemented!()
    }
}