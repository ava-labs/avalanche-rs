use std::error::Error;
use std::time::Duration;
use avalanche_types::ids::node::Id;

pub trait Handler {
    // AppGossip is called when handling an AppGossip message.
    fn app_gossip(
        &self,
        node_id: Id,
        gossip_bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>>;

    // AppRequest is called when handling an AppRequest message.
    // Returns the bytes for the response corresponding to request_bytes
    fn app_request(
        &self,
        node_id: Id,
        deadline: Duration,
        request_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>>;

    // CrossChainAppRequest is called when handling a CrossChainAppRequest message.
    // Returns the bytes for the response corresponding to request_bytes
    fn cross_chain_app_request(
        &self,
        chain_id: Id,
        deadline: Duration,
        request_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>>;
}

// NoOpHandler struct
pub struct NoOpHandler;

impl Handler for NoOpHandler {
    fn app_gossip(&self, _: Id, _: Vec<u8>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn app_request(&self, _: Id, _: Duration, _: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(vec![])
    }
    fn cross_chain_app_request(&self, _: Id, _: Duration, _: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(vec![])
    }
}
