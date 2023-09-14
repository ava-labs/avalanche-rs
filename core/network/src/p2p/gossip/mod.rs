pub mod bloom;
pub mod gossip;
pub mod handler;

use avalanche_types::ids::Id;

pub trait Gossipable {
    fn get_id(&self) -> Id;
    fn marshal(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn unmarshal(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Set<T: Gossipable + ?Sized>: Send + Sync {
    fn add(&mut self, gossipable: T) -> Result<(), Box<dyn std::error::Error>>;
    fn iterate(&self, f: &dyn FnMut(&T) -> bool);
    fn get_filter(&self) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>>;
}