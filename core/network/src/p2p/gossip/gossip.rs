use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use log::{debug, error};
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::interval;
use avalanche_types::ids::Id;
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::client::Client;

pub struct Config {
    pub frequency: Duration,
    pub poll_size: usize,
}

pub struct Gossiper<T: Gossipable + ?Sized> {
    config: Config,
    set: Arc<dyn Set<T>>,
    client: Arc<dyn Client>,
    stop_rx: Receiver<()>,
}

impl<T: Gossipable> Gossiper<T> {
    pub fn new(
        config: Config,
        set: Arc<dyn Set<T>>,
        client: Arc<dyn Client>,
        stop_rx: Receiver<()>,
    ) -> Self {
        Self {
            config,
            set,
            client,
            stop_rx,
        }
    }

    pub async fn gossip(&mut self) {
        let mut gossip_ticker = interval(self.config.frequency);

        loop {
            select! {
                _ = gossip_ticker.tick() => {
                    debug!("Tick!");
                    if let Err(e) = self.execute_gossip().await {
                        error!("Failed to Gossip : {:?}", e);
                        //ToDo

                    }
                }
                _ = self.stop_rx.recv() => {
                    debug!("Shutting down gossip");
                    break;
                }
            }
        }
    }

    // ToDo Maybe there is a better name here
    async fn execute_gossip(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (bloom, salt) = self.set.get_filter()?;

        debug!("In single_gossip");
        // ToDo Implement logic
        Ok(())
    }

    async fn handle_response(&self, node_id: Id, response_bytes: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let (bloom, salt) = self.set.get_filter()?;

        debug!("In handle_response");
        // ToDo Implement logic

        Ok(())
    }
}

// Mock implementation for the Set trait
struct MockSet;

impl<T: Gossipable> Set<T> for MockSet {
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

struct MockClient;

impl Client for MockClient {}

struct TestGossipableType;

impl Gossipable for TestGossipableType {
    fn get_id(&self) -> Id {
        todo!()
    }

    fn marshal(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }

    fn unmarshal(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}


/// RUST_LOG=debug cargo test --package network --lib -- p2p::gossip::test_gossip --exact --show-output
#[tokio::test]
async fn test_gossip() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init()
        .unwrap();

    let (stop_tx, stop_rx) = channel(1); // Create a new channel

    let mut gossiper: Gossiper<TestGossipableType> = Gossiper::new(
        Config { frequency: Duration::from_millis(200), poll_size: 0 },
        Arc::new(MockSet),
        Arc::new(MockClient),
        stop_rx,
    );

    // Spawn the gossiping task
    let gossip_task = tokio::spawn(async move {
        gossiper.gossip().await;
    });

    // Wait some time to let a few cycles of gossiping happen
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Send the stop signal before awaiting the task.
    if stop_tx.send(()).await.is_err() {
        panic!("Failed to send stop signal");
    }

    // Await the gossip task.
    let _ = gossip_task.await.expect("Gossip task failed");
}
