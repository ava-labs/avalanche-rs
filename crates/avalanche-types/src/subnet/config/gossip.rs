use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SenderConfig {
    pub gossip_accepted_frontier_validator_size: u32,
    pub gossip_accepted_frontier_non_validator_size: u32,
    pub gossip_accepted_frontier_peer_size: u32,
    pub gossip_on_accept_validator_size: u32,
    pub gossip_on_accept_non_validator_size: u32,
    pub gossip_on_accept_peer_size: u32,
    pub app_gossip_validator_size: u32,
    pub app_gossip_non_validator_size: u32,
    pub app_gossip_peer_size: u32,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self::default()
    }
}

impl SenderConfig {
    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default() -> Self {
        Self {
            gossip_accepted_frontier_validator_size: 0,
            gossip_accepted_frontier_non_validator_size: 0,
            gossip_accepted_frontier_peer_size: 15,
            gossip_on_accept_validator_size: 0,
            gossip_on_accept_non_validator_size: 0,
            gossip_on_accept_peer_size: 10,
            app_gossip_validator_size: 10,
            app_gossip_non_validator_size: 0,
            app_gossip_peer_size: 0,
        }
    }
}
