use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{debug, error};
use prost::Message;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::interval;
use avalanche_types::ids::{Id};
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::client::Client;
use crate::p2p::sdk::{PullGossipRequest, PullGossipResponse};

pub struct Config {
    pub namespace: String,
    pub frequency: Duration,
    pub poll_size: usize,
}


pub struct Gossiper<T: Gossipable>
{
    config: Config,
    set: Arc<Mutex<dyn Set<T>>>,
    client: Arc<Client>,
    stop_rx: Receiver<()>,
}

impl<T> Gossiper<T> where
    T: Gossipable + Default {
    pub fn new(
        config: Config,
        set: Arc<Mutex<dyn Set<T>>>, // Mutex or RWLock here ?
        client: Arc<Client>,
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
    async fn execute_gossip(&self) -> Result<(), Box<dyn Error>> {
        let read_guard = self.set.lock().expect("Failed to acquire lock");
        let (bloom, salt) = read_guard.get_filter()?;
        let request = PullGossipRequest { filter: bloom, salt };

        let mut msg_bytes = vec![];
        request
            .encode(&mut msg_bytes)?;

        for i in 0..self.config.poll_size {
            self.client.app_request_any(); //ToDo out of scope of my PR
        }

        Ok(())
    }

    async fn handle_response(&mut self, node_id: Id, response_bytes: Vec<u8>, err: Option<Box<dyn Error>>) {
        if let Some(e) = err {
            error!("failed gossip request, nodeID: {:?}, error: {:?}", node_id, e);
            return;
        }

        let mut response = PullGossipResponse::default();
        match PullGossipResponse::decode(response_bytes.as_slice()) {
            Ok(res) => response = res,
            Err(e) => {
                error!("failed to unmarshal gossip response, error: {:?}", e);
                return;
            }
        }

        for bytes in response.gossip.iter() {
            let mut gossipable: T = T::default();
            if let Err(e) = gossipable.unmarshal(bytes) {
                error!("failed to unmarshal gossip, nodeID: {:?}, error: {:?}", node_id, e);
                continue;
            }

            let hash = gossipable.get_id();
            debug!("received gossip, nodeID: {:?}, id: {:?}", node_id, hash);


            let mut set_guard = self.set.lock().expect("Failed to acquire lock");
            if let Err(e) = set_guard.add(gossipable) {
                debug!("failed to add gossip to the known set, nodeID: {:?}, id: {:?}, error: {:?}", node_id, hash, e);
                continue;
            }
        }
    }
}

// Mock implementation for the Set trait
//ToDo Should we move all tests to a new file ?
struct MockSet;

impl<T: Gossipable> Set<T> for MockSet {
    fn add(&mut self, _gossipable: T) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn iterate(&self, _f: &dyn Fn(&T) -> bool) {
        // Do nothing
    }

    fn get_filter(&self) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
        Ok((vec![], vec![]))
    }
}

struct MockClient;

struct TestGossipableType;

impl Default for TestGossipableType {
    fn default() -> Self {
        todo!()
    }
}

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


/// RUST_LOG=debug cargo test --package network --lib -- p2p::gossip::test_gossip_shutdown --exact --show-output
#[tokio::test]
async fn test_gossip_shutdown() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init()
        .unwrap();

    let (stop_tx, stop_rx) = channel(1); // Create a new channel

    let mut gossiper: Gossiper<TestGossipableType> = Gossiper::new(
        Config { namespace: "test".to_string(), frequency: Duration::from_millis(200), poll_size: 0 },
        Arc::new(Mutex::new(MockSet)),
        Arc::new(Client {}),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use testing_logger;

    #[tokio::test]
    async fn test_handle_response_with_empty_response_bytes() {
        // Initialize logging capture
        testing_logger::setup();

        let mut set = Arc::new(Mutex::new(HashSet::<u32>::new()));

        let (stop_tx, stop_rx) = channel(1); // Create a new channel

        let mut gossiper: Gossiper<TestGossipableType> = Gossiper::new(
            Config { namespace: "test".to_string(), frequency: Duration::from_millis(200), poll_size: 0 },
            Arc::new(Mutex::new(MockSet)),
            Arc::new(Client {}),
            stop_rx,
        );

        gossiper.handle_response(Id::default(), vec![0u8; 16], None).await;

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, "failed to unmarshal gossip response, error: DecodeError { description: \"invalid tag value: 0\", stack: [] }");
        })
    }


    #[tokio::test]
    async fn test_handle_response_with_empty_response_bytes() {
        // Initialize logging capture
        testing_logger::setup();

        let mut set = Arc::new(Mutex::new(HashSet::<u32>::new()));

        let (stop_tx, stop_rx) = channel(1); // Create a new channel

        let mut gossiper: Gossiper<TestGossipableType> = Gossiper::new(
            Config { namespace: "test".to_string(), frequency: Duration::from_millis(200), poll_size: 0 },
            Arc::new(Mutex::new(MockSet)),
            Arc::new(Client {}),
            stop_rx,
        );

        gossiper.handle_response(Id::default(), vec![0u8; 16], None).await;

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, "failed to unmarshal gossip response, error: DecodeError { description: \"invalid tag value: 0\", stack: [] }");
        })
    }
}
