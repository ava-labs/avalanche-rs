use std::sync::Arc;
use async_trait::async_trait;

pub type AppResponseCallback = Arc<dyn Fn(Vec<u8>) + Send + Sync>;
#[async_trait]
#[allow(unused_variables)]
pub trait Client: Send + Sync {
    async fn app_request_any(&mut self, request_bytes: Vec<u8>, on_response: AppResponseCallback)  -> Result<(), std::io::Error> { Ok(()) }
    async fn app_request(&mut self, request_bytes: Vec<u8>) {}
    async fn app_gossip(&mut self, request_bytes: Vec<u8>) {}
    async fn app_gossip_specific(&mut self, request_bytes: Vec<u8>) {}
    async fn cross_chain_app_request(&mut self, request_bytes: Vec<u8>) {}
    async fn prefix_message(&mut self, request_bytes: Vec<u8>) {}
}

pub struct NoOpClient;

unsafe impl Sync for NoOpClient {}
unsafe impl Send for NoOpClient {}
impl Client for NoOpClient {

}
