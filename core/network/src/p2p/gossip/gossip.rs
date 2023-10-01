use crate::p2p::client::{AppResponseCallback, Client};
use crate::p2p::gossip::{Gossipable, Set};
use crate::p2p::sdk::{PullGossipRequest, PullGossipResponse};
use avalanche_types::ids::Id;
use log::{debug, error};
use prost::Message;
use std::error::Error;
use std::hash::Hash;
use std::ops::Deref;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use serde::Serialize;
use serde_json::to_vec;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::interval;

pub struct Config {
    pub namespace: String,
    pub frequency: Duration,
    pub poll_size: usize,
}

pub struct Gossiper<S: Set + 'static> {
    config: Config,
    set: Arc<Mutex<S>>,
    client: Arc<Mutex<dyn Client>>,
    stop_rx: Receiver<()>,
}

unsafe impl<S: Set> Sync for Gossiper<S> {}

unsafe impl<S: Set> Send for Gossiper<S> {}

impl<S> Gossiper<S>
    where
        S: Set,
        S::Item: Default
{
    pub fn new(
        config: Config,
        set: Arc<Mutex<S>>, // Mutex or RWLock here ?
        client: Arc<Mutex<dyn Client>>,
        stop_rx: Receiver<()>,
    ) -> Self {
        Self {
            config,
            set,
            client,
            stop_rx,
        }
    }

    pub async fn gossip(&mut self) where <S as Set>::Item: Clone, <S as Set>::Item: Serialize {
        let mut gossip_ticker = interval(self.config.frequency);

        loop {
            select! {
                _ = gossip_ticker.tick() => {
                    debug!("Gossip tick");
                    if let Err(e) = self.execute().await {
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

    async fn execute(&mut self) -> Result<(), Box<dyn Error>> where <S as Set>::Item: Clone, <S as Set>::Item: Serialize {
        //ToDo Dummy vec<u8> for now.
        let bloom = Vec::new();

        let mut request = PullGossipRequest {
            filter: bloom,
            salt: Id::default().to_vec(), //ToDo Use default for now
        };

        let mut msg_bytes = vec![];

        // We scope the lock here to avoid issue later on
        {
            let set_guard = self.set.lock().await;

            let elem = set_guard.fetch_all_elements();

            request.filter = to_vec(&elem)?;

            // debug!("TTTTT {:?}", t);
            request.encode(&mut msg_bytes)?;
        }

        for _ in 0..self.config.poll_size {
            {
                let set = Arc::clone(&self.set.clone());

                // Initialize the callback that will be used upon receiving a response from our gossip attempt
                let on_response: AppResponseCallback = Arc::new({
                    move |response_bytes| {
                        let response = match PullGossipResponse::decode(response_bytes.as_slice()) {
                            Ok(res) => {
                                res
                            }
                            Err(e) => {
                                return;
                            }
                        };

                        // We iterate over the response's gossip
                        for bytes in response.gossip.iter() {
                            let mut gossipable: S::Item = S::Item::default();
                            gossipable.deserialize(bytes).unwrap();

                            let hash = gossipable.get_id();

                            let mut set_guard = set.try_lock().expect("Failed to acquire lock on set");
                            if let Err(e) = set_guard.add(gossipable) {
                                error!(
                            "failed to add gossip to the known set, id: {:?}, error: {:?}"
                            , hash, e
                        );
                                continue;
                            }
                        }
                    }
                });

                let mut guard = self.client.try_lock().expect("Failed to acquire a lock on client");
                guard.app_request_any(msg_bytes.clone(), on_response).await;
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tokio::sync::mpsc::{channel};
    use std::time::Duration;
    use super::*;
    use testing_logger;
    use avalanche_types::ids::Id;
    use crate::p2p::client::Client;
    use crate::p2p::gossip::gossip::{Config, Gossiper};
    use crate::p2p::sdk::PullGossipResponse;
    use crate::p2p::client::NoOpClient;

    struct MockClient;

    #[derive(Clone, Hash)]
    struct TestGossipableType {
        pub id: Id,
    }

    impl Default for TestGossipableType {
        fn default() -> Self {
            TestGossipableType {
                id: Default::default(),
            }
        }
    }

    impl Gossipable for TestGossipableType {
        fn get_id(&self) -> Id {
            self.id
        }

        fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
            Ok(self.id.to_vec())
        }

        fn deserialize(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
            self.id = Id::from_slice(bytes);
            Ok(())
        }
    }

    // Mock implementation for the Set trait
//ToDo Should we move all tests to a new file ?
    pub struct MockSet<TestGossipableType> {
        pub set: Vec<TestGossipableType>,
    }

    impl<T> MockSet<T> {
        pub fn len(&self) -> usize {
            println!("{}", self.set.len());
            self.set.len()
        }
    }

    impl<T: Gossipable + Sync + Send + Clone + Hash> Set for MockSet<T> {
        type Item = T;
        fn add(&mut self, gossipable: T) -> Result<(), Box<dyn Error>> {
            self.set.push(gossipable.clone());
            Ok(())
        }

        fn iterate(&self, _f: &mut dyn FnMut(&T) -> bool) {
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
        let noopclient = NoOpClient {};
        noopclient.app_request(Vec::new());
        let (stop_tx, stop_rx) = channel(1); // Create a new channel

        let mut gossiper: Gossiper<MockSet<TestGossipableType>> = Gossiper::new(
            Config {
                namespace: "test".to_string(),
                frequency: Duration::from_millis(200),
                poll_size: 0,
            },
            Arc::new(Mutex::new(MockSet {
                set: Vec::new(),
            })),
            Arc::new(NoOpClient {}),
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

    #[tokio::test]
    async fn test_handle_response_with_empty_response_bytes() {
        // Initialize logging capture
        testing_logger::setup();

        let (stop_tx, stop_rx) = channel(1); // Create a new channel

        let mut gossiper: Gossiper<MockSet<TestGossipableType>> = Gossiper::new(
            Config {
                namespace: "test".to_string(),
                frequency: Duration::from_millis(200),
                poll_size: 0,
            },
            Arc::new(Mutex::new(MockSet {
                set: Vec::new(),
            })),
            Arc::new(NoOpClient {}),
            stop_rx,
        );

        gossiper
            .handle_response(Id::default(), vec![0u8; 16], None)
            .await;

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, "failed to unmarshal gossip response, error: DecodeError { description: \"invalid tag value: 0\", stack: [] }");
        })
    }

    #[tokio::test]
    async fn test_handle_response_with_non_empty_response_bytes() {
        // Initialize logging capture
        testing_logger::setup();

        let (stop_tx, stop_rx) = channel(1); // Create a new channel

        let mut gossiper: Gossiper<MockSet<TestGossipableType>> = Gossiper::new(
            Config {
                namespace: "test".to_string(),
                frequency: Duration::from_millis(200),
                poll_size: 0,
            },
            Arc::new(Mutex::new(MockSet {
                set: Vec::new(),
            })),
            Arc::new(NoOpClient {}),
            stop_rx,
        );

        let mut pull_gossip_response = PullGossipResponse::default();
        let gossip_data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let another_gossip_data: Vec<u8> = vec![6, 7, 8, 9, 10];
        pull_gossip_response.gossip.push(gossip_data);
        pull_gossip_response.gossip.push(another_gossip_data);
        let mut response_bytes: Vec<u8> = vec![];
        pull_gossip_response
            .encode(&mut response_bytes)
            .expect("Encoding failed");

        gossiper
            .handle_response(Id::default(), response_bytes, None)
            .await;

        let read_guard = gossiper.set.lock().expect("Failed to acquire lock");

        assert!(read_guard.len() == 2);
    }
}
