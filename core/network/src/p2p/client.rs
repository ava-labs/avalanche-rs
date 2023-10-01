use std::io::Error;
use std::sync::Arc;
use async_trait::async_trait;

pub type AppResponseCallback = Arc<dyn Fn(Vec<u8>) + Send + Sync>;

#[async_trait]
#[allow(unused_variables)]
pub trait Client: Send + Sync {
    async fn app_request_any(&mut self, request_bytes: &Vec<u8>, on_response: AppResponseCallback) -> Result<(), std::io::Error>;
    async fn app_request(&mut self, request_bytes: Vec<u8>) -> Result<(), std::io::Error>;
    async fn app_gossip(&mut self, request_bytes: Vec<u8>) -> Result<(), std::io::Error>;
    async fn app_gossip_specific(&mut self, request_bytes: Vec<u8>) -> Result<(), std::io::Error>;
    async fn cross_chain_app_request(&mut self, request_bytes: Vec<u8>) -> Result<(), std::io::Error>;
    async fn prefix_message(&mut self, request_bytes: Vec<u8>) -> Result<(), std::io::Error>;
}

pub struct NoOpClient;

#[async_trait]
impl Client for NoOpClient {
    async fn app_request_any(&mut self, request_bytes: &Vec<u8>, on_response: AppResponseCallback) -> Result<(), Error> {
        todo!()
    }

    async fn app_request(&mut self, request_bytes: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn app_gossip(&mut self, request_bytes: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn app_gossip_specific(&mut self, request_bytes: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn cross_chain_app_request(&mut self, request_bytes: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn prefix_message(&mut self, request_bytes: Vec<u8>) -> Result<(), Error> {
        todo!()
    }
}
