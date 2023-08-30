//! Definitions of messages that can be sent between nodes.
pub mod accepted;
pub mod accepted_frontier;
pub mod accepted_state_summary;
pub mod ancestors;
pub mod app_gossip;
pub mod app_request;
pub mod app_response;
pub mod chits;
pub mod compress;
pub mod get;
pub mod get_accepted;
pub mod get_accepted_frontier;
pub mod get_accepted_state_summary;
pub mod get_ancestors;
pub mod get_state_summary_frontier;
pub mod peerlist;
pub mod ping;
pub mod pong;
pub mod pull_query;
pub mod push_query;
pub mod put;
pub mod state_summary_frontier;
pub mod version;

pub fn ip_addr_to_bytes(ip_addr: std::net::IpAddr) -> Vec<u8> {
    match ip_addr {
        std::net::IpAddr::V4(v) => {
            // "avalanchego" encodes IPv4 address as it is
            // (not compatible with IPv6, e.g., prepends 2 "0xFF"s as in Rust)
            let octets = v.octets();
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3],
            ]
        }
        std::net::IpAddr::V6(v) => v.octets().to_vec(),
    }
}
