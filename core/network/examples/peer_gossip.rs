use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use log::{debug, error};
use rand::prelude::SliceRandom;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use network::p2p::gossip::gossip::{Config, Gossiper};
use tokio::sync::mpsc::{channel};
use tracing::field::debug;
use avalanche_types::ids::Id;
use network::p2p::client::{AppResponseCallback, Client};
use network::p2p::gossip::{Gossipable, Set};
use network::p2p::gossip::handler::{Handler, HandlerConfig, new_handler};
use network::p2p::handler::Handler as TraitHandler;

pub struct TestClient {
    pub stream: Arc<Mutex<TcpStream>>,
    pub listener: Arc<Mutex<TcpStream>>,
}

#[async_trait]
#[allow(unused_variables)]
impl Client for TestClient {
    async fn app_gossip(&mut self, request_bytes: Vec<u8>) {
        unimplemented!()
    }

    async fn app_request_any(&mut self, request_bytes: Vec<u8>, on_response: AppResponseCallback) {
        let mut stream_guard = self.stream.try_lock().expect("aa");
        stream_guard.write_all(&*request_bytes).await.unwrap();

        // Lock the listener and wait for a new connection
        let clone = self.listener.clone();
        let mut listener = clone.try_lock().expect("Unable to lock listener");
        debug!("after acquiring lock on listener -- app_request_any -- {:?}", listener);

            let mut buf = [0u8; 1024];
            debug!("Loopity loop");
            match listener.read(&mut buf).await {
                Ok(n) => {
                    debug!("Gossip -- Received a message of length: {} -- {:?}", n, buf);
                    if n == 0 {
                        // Connection was closed
                    }
                    // Handle received data here: buf[0..n]

                    on_response(buf[0..n].to_vec());
                }
                Err(e) => {
                    // Handle the error.
                }
            }
        // match listener.accept().await {
        //     Ok((mut stream, _)) => {
        //         debug!("in peer_gossip - app_request_any");
        //         let mut buf = [0u8; 1024];
        //         match stream.read(&mut buf).await {
        //             Ok(n) => {
        //                 // if n == 0 {
        //                 //     debug!("did we not receive something");
        //                 //     return;
        //                 // }
        //                 debug!("Received a message of length: {}", n);
        //                 on_response(buf[0..n].to_vec());
        //             }
        //             Err(e) => {
        //                 error!("Error reading from stream: {}", e);
        //             }
        //         }
        //     }
        //     Err(e) => {
        //         error!("Error accepting connection: {}", e);
        //     }
        // }

        debug!("End of app_request_any");
    }
}

#[derive(Clone, Hash, Debug)]
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

impl PartialEq for TestGossipableType {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
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
#[derive(Debug, Clone, Hash)]
pub struct MockSet<TestGossipableType> {
    pub set: Vec<TestGossipableType>,
}

impl<T> MockSet<T> {
    pub fn len(&self) -> usize {
        println!("{}", self.set.len());
        self.set.len()
    }
}

impl<T: Gossipable + Sync + Send + Clone + Hash + Debug + PartialEq> Set for MockSet<T> {
    type Item = T;
    fn add(&mut self, _gossipable: T) -> Result<(), Box<dyn Error>> {
        debug!("SET LEN -- {:?}", self.set.len());
        self.set.push(_gossipable.clone());
        Ok(())
    }

    fn has(&self, gossipable: &Self::Item) -> bool {
        self.set.contains(gossipable)
    }

    fn iterate(&self, _f: &mut dyn FnMut(&T) -> bool) {
        debug!("In iteratteeeee");
        debug!("In iterateeeee {}", self.set.len());
        for item in &self.set {
            debug!("In iterateeeee for item {:?}", item);
            if _f(item) {
                debug!("In iterateeeee for item -- false {:?}", item);
                break; // Stop iterating if the closure returns false
            }
        }
        // Do nothing
    }

    fn fetch_elements(&self) -> Self::Item {
        self.set.choose(&mut rand::thread_rng()).cloned().expect("Set is empty")
    }
}

async fn fake_handler_server_logic(mut socket: TcpStream, client_socket: Arc<Mutex<TcpStream>>, handler: Handler<MockSet<TestGossipableType>>) {

    // Initialize a buffer of size 1024.
    let mut buf = [0u8; 1024];
    debug!("Fake Handler Server Logic");
    loop {
        debug!("New loop in fake_handler_server_logic");
        let n = socket.read(&mut buf).await.unwrap();
        debug!("fake_handler_server_logic -- Received {:?} bytes from socket", n);
        // // Empty, wait 5 sec before next attempt
        if n == 0 {
            debug!("received 0 bytes message)");
            tokio::time::sleep(Duration::from_secs(5)).await;
            break;
        }

        // Fake test data.
        let node_id: avalanche_types::ids::node::Id = avalanche_types::ids::node::Id::from_slice(&random_manager::secure_bytes(20).unwrap());
        debug!("fake_handler_server_logic -- Node id {}", node_id);
        let res_bytes = handler.app_gossip(node_id, buf[0..n].to_vec()).await.expect("Issue while attempting to gossip in fake_handler_logic");
        debug!("fake_handler_server_logic -- res_bytes {:?}", res_bytes);

        let mut guard = client_socket.try_lock().expect("Lock of client_socket failed");

        debug!("fake_handler_server_logic -- Send bytes to gossip : {:?} to {:?}", res_bytes, guard);
        let _ = guard.write_all(&res_bytes).await;
        // guard.write_all(random_manager::secure_bytes(32).unwrap().as_slice()).await;

        debug!("fake_handler_server_logic -- End loop in fake_handler_server_logic");
    }
}

async fn start_fake_node(own_handler: String, own_client: String, other_handler: String, other_client: String, vec_gossip_local_client: Vec<TestGossipableType>, vec_gossip_remote_client: Vec<TestGossipableType>) {
    // Initialize the configuration for the gossiper
    let config = Config {
        namespace: "test".to_string(),
        frequency: Duration::from_millis(500),
        poll_size: 1, // As we only have 1 other "node" in our test setup, set it to 1
    };

    // Create a TcpListener to receive messages on.
    // Wrapping it in Arc and Mutex to safely share it between threads.
    let own_handler_listener = Arc::new(Mutex::new(TcpListener::bind(own_handler.clone()).await.unwrap()));
    let own_client_listener_r = Arc::new(Mutex::new(TcpListener::bind(own_client.clone()).await.unwrap()));

    // Create a TcpStream to send messages to.
    // Wrapping it in Arc and Mutex to safely share it between threads.
    let other_client_stream = Arc::new(Mutex::new(TcpStream::connect(other_client).await.unwrap()));
    let other_handler_stream = Arc::new(Mutex::new(TcpStream::connect(other_handler.clone()).await.unwrap()));

    // Initialize the configuration for the handler and create a new handler
    let handler_config = HandlerConfig { namespace: "test".to_string(), target_response_size: 100 };

    let mut set = MockSet { set: Vec::<TestGossipableType>::new() };
    // Generating fake data and pushing to set
    {
        for gossip in &vec_gossip_local_client {
            set.set.push(
                gossip.clone()
            );
        }
    }

    let handler = new_handler(
        handler_config,
        Arc::new(Mutex::new(set)),
    );

    // Clone listener and stream for use inside the spawned task.
    let own_handler_listener_clone = own_handler_listener.clone();
    let other_client_stream_clone = other_client_stream.clone();
    // Spawn an asynchronous task that will handle incoming connections in a loop
    let handler_task = tokio::spawn(async move {
        // Accept incoming connections and spawn a new task to handle each connection
        debug!("Setting up the handler task");
        let guard = own_handler_listener_clone.try_lock().expect("Error acquiring lock on listener_clone");
        debug!("After lock");
        let (listener_socket, _) = guard.accept().await.unwrap();
        debug!("After accept");
        fake_handler_server_logic(listener_socket, other_client_stream_clone.clone(), handler.clone()).await;
    });

    // Initialize a MockSet and populate it with some test data
    let set = Arc::new(Mutex::new(MockSet { set: Vec::<TestGossipableType>::new() }));
    // Generating fake data and pushing to set
    {
        for gossip in &vec_gossip_local_client {
            set.try_lock().expect("Failed to lock").set.push(
                gossip.clone()
            );
        }
    }

    {
        assert_eq!(set.try_lock().expect("Failed to acquire lock").set.len().clone(), 3);
    }

    let (stop_tx, stop_rx) = channel(1);

    // Spawn the gossiping task
    let set_clone = set.clone();
    let gossip_task = tokio::spawn(async move {
        // Initialize a TestClient instance with the given stream and listener
        let (stream, _) = own_client_listener_r.try_lock().expect("Failed to acquire lock").accept().await.expect("Fail");
        debug!("Gossip will be listening on {:?}", stream);
        let gossip_client = Arc::new(Mutex::new(TestClient { stream: other_handler_stream.clone(), listener: Arc::new(Mutex::new(stream)) }));

        // Create a channel for stopping the gossiper

        // Initialize the Gossiper with the provided configuration, set, client, and receiver end of the stop channel
        let mut gossiper = Gossiper::new(config, set_clone, gossip_client.clone(), stop_rx);

        gossiper.gossip().await;
    });

    // Sleep for a few seconds, make sure the whole process ran at least a couple of times
    tokio::time::sleep(Duration::from_secs(1)).await;

    {
        let guard = set.try_lock().expect("Failed to acquire lock");
        // As we have 3 elements in our set pre-gossip loop execution in each one of our fake gossip server, we should end up with 6 gossip at the end of our test run.
        debug!("SET LEN {}", guard.set.len());
        // assert_eq!(guard.set.len(), 6);
        // for gossip in vec_gossip_remote_client {
        //     assert_eq!(guard.set.contains(&gossip), true);
        // }
    }

    // Send the stop signal before awaiting the task.
    if stop_tx.send(()).await.is_err() {
        eprintln!("Failed to send stop signal");
    }
    debug!("Checking if all things good");

    // Await the completion of the gossiping task
    let _ = gossip_task.await.expect("Gossip task failed");
    let _ = handler_task.await.expect("Handler task failed");
}


#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let mut vec_gossip_client_01 = Vec::new();
    let mut vec_gossip_client_02 = Vec::new();

    for _ in 0..3 {
        vec_gossip_client_01.push(
            TestGossipableType { id: Id::from_slice(&random_manager::secure_bytes(32).unwrap()) }
        );
        vec_gossip_client_02.push(
            TestGossipableType { id: Id::from_slice(&random_manager::secure_bytes(32).unwrap()) }
        );
    };


    // Start the client
    // listen on 8080 , send message to 8081
    let client_01_handle = tokio::spawn(start_fake_node("127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string(), "127.0.0.1:8083".to_string(), vec_gossip_client_01.clone(), vec_gossip_client_02.clone()));
    let client_02_handle = tokio::spawn(start_fake_node("127.0.0.1:8082".to_string(), "127.0.0.1:8083".to_string(), "127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string(), vec_gossip_client_02.clone(), vec_gossip_client_01.clone()));

    // Wait for the server and client to complete
    client_01_handle.await.unwrap();
    client_02_handle.await.unwrap();
}

