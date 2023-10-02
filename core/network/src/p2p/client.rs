use async_trait::async_trait;
use std::io::Error;

pub type AppResponseCallback = Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>;

#[async_trait]
pub trait Client: Send + Sync {
    async fn app_request_any(
        &mut self,
        request_bytes: &Vec<u8>,
        on_response: AppResponseCallback,
    ) -> Result<(), Error>;
    async fn app_request(&mut self, request_bytes: Vec<u8>) -> Result<(), Error>;
    async fn app_gossip(&mut self, request_bytes: Vec<u8>) -> Result<(), Error>;
    async fn app_gossip_specific(&mut self, request_bytes: Vec<u8>) -> Result<(), Error>;
    async fn cross_chain_app_request(&mut self, request_bytes: Vec<u8>) -> Result<(), Error>;
    async fn prefix_message(&mut self, request_bytes: Vec<u8>) -> Result<(), Error>;
}

pub struct NoOpClient;

#[async_trait]
impl Client for NoOpClient {
    async fn app_request_any(&mut self, _: &Vec<u8>, _: AppResponseCallback) -> Result<(), Error> {
        todo!()
    }

    async fn app_request(&mut self, _: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn app_gossip(&mut self, _: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn app_gossip_specific(&mut self, _: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn cross_chain_app_request(&mut self, _: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    async fn prefix_message(&mut self, _: Vec<u8>) -> Result<(), Error> {
        todo!()
    }
}
