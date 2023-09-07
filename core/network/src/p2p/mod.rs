pub mod gossip;

use avalanche_types::ids::Id;

/// Represent the mempool of an avalanche-rs node.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego@v1.10.9/vms/platformvm/txs/mempool>

pub trait Gossipable {
    fn get_id(&self) -> Id;
    fn marshal(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn unmarshal(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Set<T: Gossipable> {
    fn add(&mut self, gossipable: T) -> Result<(), Box<dyn std::error::Error>>;
    fn iterate(&self, f: impl Fn(&T) -> bool);
    fn get_filter(&self) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>>;
}