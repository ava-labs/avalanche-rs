pub mod gossip;
pub mod handler;

use avalanche_types::ids::Id;

pub trait Gossipable {
    fn get_id(&self) -> Id;
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn deserialize(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Set: Send + Sync {
    type Item: Gossipable + ?Sized;
    fn add(&mut self, gossipable: Self::Item) -> Result<(), Box<dyn std::error::Error>>;
    fn iterate(&self, f: &dyn FnMut(&Self::Item) -> bool);
    fn fetch_elements(&self) -> Self::Item;
}
