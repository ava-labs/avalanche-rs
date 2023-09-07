use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use log::{debug, error};
use tokio::time::interval;
use avalanche_types::ids::Id;
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::client::Client;

pub struct Config {
    pub frequency: std::time::Duration,
    pub poll_size: usize,
}

pub struct Gossiper<T: Gossipable<T> + ?Sized> {
    config: Config,
    set: Arc<dyn Set<T>>,
    client: Arc<dyn Client>,
}

impl<T: Gossipable<T>> Gossiper<T> {
    pub fn new(
        config: Config,
        set: Arc<dyn Set<T>>,
        client: Arc<dyn Client>,
    ) -> Self {
        Self {
            config,
            set,
            client,
        }
    }

    pub async fn gossip(&self) {
        let mut gossip_ticker = interval(self.config.frequency);

        loop {
            gossip_ticker.tick().await;
            // This will pause the loop for `self.config.frequency`
            if let Err(e) = self.single_gossip().await {
                error!("Failed to Gossip : {:?}", e)
            }
        }
    }

    async fn single_gossip(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (bloom, salt) = self.set.get_filter()?;
        // ... Perform the gossip operation, involving self.client, and return a Result
        // (Left as an exercise for the reader)
        debug!("In single_gossip");
        Ok(())
    }
}

// Mock implementation for the Set trait
struct MockSet;

impl<T: Gossipable<T>> Set<T> for MockSet {
    fn add(&mut self, _gossipable: T) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn iterate(&self, _f: &dyn Fn(&T) -> bool) {
        // Do nothing
    }

    fn get_filter(&self) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        Ok((vec![], vec![]))
    }
}

// Mock implementation for the Client trait
struct MockClient;

impl Client for MockClient {
    // Implement the methods of the Client trait here...
}

struct TestGossipableType;

impl<T> Gossipable<T> for TestGossipableType {
    fn get_id(&self) -> Id {
        todo!()
    }

    fn marshal(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }

    fn unmarshal(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    // ... your methods here
}


/// RUST_LOG=debug cargo test --package network --lib -- p2p::gossip::test_gossip --exact --show-output
#[tokio::test]
async fn test_gossip() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init()
        .unwrap();

    let gossiper: Gossiper<TestGossipableType> = Gossiper::new(
        Config { frequency: Duration::from_millis(200), poll_size: 0 },
        Arc::new(MockSet),  // Replace with your real Set implementation
        Arc::new(MockClient), // Replace with your real Client implementation
    );

    // Spawn the gossiping task
    let gossip_task = tokio::spawn(async move {
        gossiper.gossip().await;
    });

    // Wait some time to let a few cycles of gossiping happen
    tokio::time::sleep(Duration::from_secs(5)).await;

    gossip_task.abort();
}
