pub mod gossip;
pub mod handler;

use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait Gossipable {
    fn get_id(&self) -> Id;
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn deserialize(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Set: Send + Sync {
    type Item: Gossipable + ?Sized + Debug + Serialize + for<'de> Deserialize<'de> + PartialEq;
    fn add(&mut self, gossipable: Self::Item) -> Result<(), Box<dyn std::error::Error>>;
    fn has(&self, gossipable: &Self::Item) -> bool;
    fn iterate(&self, f: &mut dyn FnMut(&Self::Item) -> bool);
    fn fetch_elements(&self) -> Self::Item;
    fn fetch_all_elements(&self) -> Vec<Self::Item>
    where
        <Self as Set>::Item: Sized;
}
