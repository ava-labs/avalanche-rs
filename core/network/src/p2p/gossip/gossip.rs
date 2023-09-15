use crate::p2p::client::Client;
use crate::p2p::gossip::Gossipable;
use crate::p2p::sdk::{PullGossipRequest, PullGossipResponse};
use avalanche_types::ids::Id;
use log::{debug, error};
use prost::Message;
use std::error::Error;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::interval;

pub struct Config {
    pub namespace: String,
    pub frequency: Duration,
    pub poll_size: usize,
}

pub struct Gossiper<T: Gossipable> {
    config: Config,
    client: Arc<Client>,
    stop_rx: Receiver<()>,
    phantom: PhantomData<T>, // Had to use this to please the compiler about T not being used.
}

impl<T> Gossiper<T>
    where
        T: Gossipable + Default,
{
    pub fn new(
        config: Config,
        client: Arc<Client>,
        stop_rx: Receiver<()>,
    ) -> Self {
        Self {
            config,
            client,
            stop_rx,
            phantom: PhantomData,
        }
    }

    pub async fn gossip(&mut self) {
        let mut gossip_ticker = interval(self.config.frequency);

        loop {
            select! {
                _ = gossip_ticker.tick() => {
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

    async fn execute(&self) -> Result<(), Box<dyn Error>> {
        //ToDo Dummy vec<u8> for now.
        let bloom = Vec::new();

        let request = PullGossipRequest {
            filter: bloom,
            salt: Id::default().to_vec(), //ToDo Use default for now
        };

        let mut msg_bytes = vec![];
        request.encode(&mut msg_bytes)?;

        for i in 0..self.config.poll_size {
            self.client.app_request_any(); //ToDo out of scope of my PR
        }

        Ok(())
    }

    async fn handle_response(
        &mut self,
        node_id: Id,
        response_bytes: Vec<u8>,
        err: Option<Box<dyn Error>>,
    ) {
        if let Some(e) = err {
            error!(
                "failed gossip request, nodeID: {:?}, error: {:?}",
                node_id, e
            );
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
            if let Err(e) = gossipable.deserialize(bytes) {
                error!(
                    "failed to unmarshal gossip, nodeID: {:?}, error: {:?}",
                    node_id, e
                );
                continue;
            }

            let hash = gossipable.get_id();
            debug!("received gossip, nodeID: {:?}, id: {:?}", node_id, hash);
        }
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use tokio::sync::mpsc::{channel};
    use std::time::Duration;
    use super::*;
    use testing_logger;
    use avalanche_types::ids::Id;
    use crate::p2p::client::Client;
    use crate::p2p::gossip::gossip::{Config, Gossiper};
    use crate::p2p::sdk::PullGossipResponse;

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
            Config {
                namespace: "test".to_string(),
                frequency: Duration::from_millis(200),
                poll_size: 0,
            },
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

    #[tokio::test]
    async fn test_handle_response_with_empty_response_bytes() {
        // Initialize logging capture
        testing_logger::setup();

        let (stop_tx, stop_rx) = channel(1); // Create a new channel

        let mut gossiper: Gossiper<TestGossipableType> = Gossiper::new(
            Config {
                namespace: "test".to_string(),
                frequency: Duration::from_millis(200),
                poll_size: 0,
            },
            Arc::new(Client {}),
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

        let mut gossiper: Gossiper<TestGossipableType> = Gossiper::new(
            Config {
                namespace: "test".to_string(),
                frequency: Duration::from_millis(200),
                poll_size: 0,
            },
            Arc::new(Client {}),
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
    }
}
