use std::error::Error;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use probabilistic_collections::bloom::BloomFilter;
use prost::Message;
use avalanche_types::ids::node::Id;
use crate::p2p;
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::gossip::bloom::{Bloom, Hasher};
use crate::p2p::sdk::PullGossipRequest;

pub struct HandlerConfig {
    pub namespace: String,
    pub target_response_size: usize,
}

pub struct Handler<T: Gossipable, S: Set<T>> {
    pub handler: Arc<dyn p2p::handler::Handler>,
    set: Arc<Mutex<S>>,
    target_response_size: usize,
    phantom: PhantomData<T>,
}

pub fn new<T: Gossipable, S: Set<T>>(
    config: HandlerConfig,
    set: Arc<Mutex<S>>,
) -> Handler<T, S> {
    Handler {
        handler: Arc::new(p2p::handler::NoOpHandler {}),
        set,
        target_response_size: config.target_response_size,
        phantom: PhantomData::default(),
    }
}

impl<T, S> p2p::handler::Handler for Handler<T, S>
    where
        T: Gossipable + Default,
        S: Set<T>,
{
    fn app_gossip(&self, node_id: Id, gossip_bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn app_request(
        &self,
        node_id: Id,
        deadline: Duration,
        request_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut request = PullGossipRequest::default();
        request = PullGossipRequest::decode(request_bytes.as_slice()).expect("Failed to decode request_bytes");

        let salt = avalanche_types::ids::Id::from_slice(request.salt.as_slice());

        //ToDo not sure about this ?
        let mut filter = Bloom::new_bloom_filter_with_salt(100, 0.5, salt);

        //ToDo Am not sure this does exactly what the equivalent gocode does.
        let de_filter: BloomFilter<Hasher> = bincode::deserialize(&request_bytes).unwrap();

        filter.bloom = de_filter;

        let mut response_size = 0_usize;
        let mut gossip_bytes: Vec<Vec<u8>> = Vec::new();

        self.set.lock().expect("Failed to lock").iterate(&|gossipable| {
            if filter.has(gossipable) { return true; }

            let bytes = match gossipable.marshal() {
                Ok(b) => b,
                Err(_) => return false,
            };

            gossip_bytes.push(bytes.clone());
            response_size += bytes.len();

            response_size <= self.target_response_size
        });


        todo!()
    }

    fn cross_chain_app_request(&self, chain_id: Id, deadline: Duration, request_bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }
}