pub mod gossip;
pub mod bloom;

use avalanche_types::ids::Id;

pub trait Gossipable {
    fn get_id(&self) -> Id;
    fn marshal(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn unmarshal(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Set<T: Gossipable + ?Sized>: Send + Sync {
    fn add(&mut self, gossipable: T) -> Result<(), Box<dyn std::error::Error>>;
    fn iterate(&self, f: &dyn Fn(&T) -> bool);
    fn get_filter(&self) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>>;
}