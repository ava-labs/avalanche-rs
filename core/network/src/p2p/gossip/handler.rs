use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use prost::Message;
use avalanche_types::ids::node::Id;
use crate::p2p;
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::sdk::{PullGossipRequest, PullGossipResponse};

pub struct HandlerConfig {
    pub namespace: String,
    pub target_response_size: usize,
}

pub struct Handler<S: Set> {
    pub handler: Arc<dyn p2p::handler::Handler>,
    set: Arc<Mutex<S>>,
    target_response_size: usize,

}

pub fn new<S: Set>(
    config: HandlerConfig,
    set: Arc<Mutex<S>>,
) -> Handler<S> {
    Handler {
        handler: Arc::new(p2p::handler::NoOpHandler {}),
        set,
        target_response_size: config.target_response_size,
    }
}

impl<S> p2p::handler::Handler for Handler<S>
    where
        S: Set,
        S::Item: Default
{
    fn app_gossip(&self, node_id: Id, gossip_bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    fn app_request(
        &self,
        _: Id,
        _: Duration,
        request_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut request = PullGossipRequest::default();
        request = PullGossipRequest::decode(request_bytes.as_slice()).expect("Failed to decode request_bytes");

        let mut response_size = 0_usize;
        let mut gossip_bytes: Vec<Vec<u8>> = Vec::new();

        self.set.lock().expect("Failed to lock").iterate(&|gossipable| {
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

    fn cross_chain_app_request(&self, chain_id: Id, deadline: Duration, request_bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        unimplemented!()
    }
}