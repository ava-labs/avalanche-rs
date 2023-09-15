use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use prost::Message;
use avalanche_types::ids::node::Id;
use crate::p2p;
use crate::p2p::gossip::Gossipable;
use crate::p2p::sdk::{PullGossipRequest, PullGossipResponse};
pub struct HandlerConfig {
    pub namespace: String,
    pub target_response_size: usize,
}

pub struct Handler<T: Gossipable> {
    pub handler: Arc<dyn p2p::handler::Handler>,
    target_response_size: usize,
    phantom: PhantomData<T>,
}

pub fn new<T: Gossipable>(
    config: HandlerConfig,
) -> Handler<T> {
    Handler {
        handler: Arc::new(p2p::handler::NoOpHandler {}),
        target_response_size: config.target_response_size,
        phantom: PhantomData::default(),
    }
}

impl<T> p2p::handler::Handler for Handler<T>
    where
        T: Gossipable + Default,
{
    fn app_gossip(&self, node_id: Id, gossip_bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn app_request(
        &self,
        _: Id,
        _: Duration,
        request_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut request = PullGossipRequest::default();
        request = PullGossipRequest::decode(request_bytes.as_slice()).expect("Failed to decode request_bytes");

        let gossip_bytes: Vec<Vec<u8>> = Vec::new();

        let mut response = PullGossipResponse::default();
        response.gossip = gossip_bytes;

        let mut response_bytes = vec![];
        response.encode(&mut response_bytes).expect("s");

        Ok(response_bytes)
    }

    fn cross_chain_app_request(&self, chain_id: Id, deadline: Duration, request_bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }
}