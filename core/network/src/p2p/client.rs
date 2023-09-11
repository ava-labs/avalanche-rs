pub struct Client {}

impl Client {
    pub async fn app_request_any(&self) {}
    pub async fn app_request(&self) {}
    pub async fn app_gossip(&self) {}
    pub async fn app_gossip_specific(&self) {}
    pub async fn cross_chain_app_request(&self) {}
    pub async fn prefix_message(&self) {}
}