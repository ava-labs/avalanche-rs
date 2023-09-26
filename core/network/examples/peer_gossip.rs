use std::error::Error;
use std::hash::Hash;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use network::p2p::gossip::gossip::{Config, Gossiper};
use tokio::sync::mpsc::{channel};
use avalanche_types::ids::Id;
use network::p2p::client::{AppResponseCallback, Client};
use network::p2p::gossip::{Gossipable, Set};
use network::p2p::gossip::handler::{Handler, HandlerConfig, new_handler};
use network::p2p::handler::Handler as TraitHandler;

pub struct TestClient {
    pub stream: Arc<Mutex<TcpStream>>,
    pub listener: Arc<Mutex<TcpListener>>,
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

        loop {
            // Lock the listener and wait for a new connection
            let clone = self.listener.clone();
            let listener = clone.try_lock().expect("Unable to lock listener");

            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 1024];
                    match stream.read(&mut buf).await {
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            println!("Received a message of length: {}", n);
                            on_response(buf[0..n].to_vec());
                        }
                        Err(e) => {
                            eprintln!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                    break;
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    break;
                }
            }
        }
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

impl<T: Gossipable + Sync + Send + Clone + Hash> Set for MockSet<T> {
    type Item = T;
    fn add(&mut self, _gossipable: T) -> Result<(), Box<dyn Error>> {
        self.set.push(_gossipable.clone());
        Ok(())
    }

    fn iterate(&self, _f: &dyn FnMut(&T) -> bool) {
        // Do nothing
    }

    fn fetch_elements(&self) -> Self::Item {
        self.set.get(0).unwrap().clone()
    }
}

async fn fake_handler_server_logic(mut socket: TcpStream, client_socket: Arc<Mutex<TcpStream>>, handler: Handler<MockSet<TestGossipableType>>) {

    // Initialize a buffer of size 1024.
    let mut buf = [0u8; 1024];

    loop {
        let n = socket.read(&mut buf).await.unwrap();

        // Empty, wait 5 sec before next attempt
        if n == 0 {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Fake test data.
        let node_id: avalanche_types::ids::node::Id = avalanche_types::ids::node::Id::from_slice(&random_manager::secure_bytes(20).unwrap());

        let res_bytes = handler.app_gossip(node_id, buf[0..n].to_vec()).await.expect("Issue while attempting to gossip in fake_handler_logic");

        let mut guard = client_socket.try_lock().expect("Lock of client_socket failed");

        let _ = guard.write_all(&res_bytes).await;
    }
}

async fn start_fake_node(gossip_handler_addr: String, listener_handler_addr: String, client_addr: String) {
    // Initialize the configuration for the gossiper
    let config = Config {
        namespace: "test".to_string(),
        frequency: Duration::from_secs(10),
        poll_size: 1, // As we only have 1 other "node" in our test setup, set it to 1
    };

    // Create a TcpListener to receive messages on.
    // Wrapping it in Arc and Mutex to safely share it between threads.
    let handler_listener = Arc::new(Mutex::new(TcpListener::bind(listener_handler_addr.clone()).await.unwrap()));
    let gossip_listener = Arc::new(Mutex::new(TcpListener::bind(gossip_handler_addr.clone()).await.unwrap()));

    // Create a TcpStream to send messages to.
    // Wrapping it in Arc and Mutex to safely share it between threads.
    let stream = Arc::new(Mutex::new(TcpStream::connect(client_addr).await.unwrap()));

    // Initialize the configuration for the handler and create a new handler
    let handler_config = HandlerConfig { namespace: "test".to_string(), target_response_size: 100 };
    let handler = new_handler(
        handler_config,
        Arc::new(Mutex::new(MockSet { set: Vec::<TestGossipableType>::new() })),
    );

    // Clone listener and stream for use inside the spawned task.
    let listener_clone = handler_listener.clone();
    let stream_clone = stream.clone();
    // Spawn an asynchronous task that will handle incoming connections in a loop
    tokio::spawn(async move {
        // Accept incoming connections and spawn a new task to handle each connection
        let (listener_socket, _) = listener_clone.try_lock().expect("Error acquiring lock on listener_clone").accept().await.unwrap();
        fake_handler_server_logic(listener_socket, stream_clone.clone(), handler.clone()).await;
    });

    // Initialize a MockSet and populate it with some test data
    let set = Arc::new(Mutex::new(MockSet { set: Vec::<TestGossipableType>::new() }));
    // Generating fake data and pushing to set
    {
        for _ in 0..3 {
            set.try_lock().expect("Error acquiring lock on set").set.push(
                TestGossipableType { id: Id::from_slice(&random_manager::secure_bytes(32).unwrap()) }
            );
        }
    }

    let (stop_tx, stop_rx) = channel(1);

    // Spawn the gossiping task
    let gossip_task = tokio::spawn(async move {
        // Initialize a TestClient instance with the given stream and listener
        let gossip_client = Arc::new(Mutex::new(TestClient { stream: stream.clone(), listener: gossip_listener.clone() }));

        // Create a channel for stopping the gossiper

        // Initialize the Gossiper with the provided configuration, set, client, and receiver end of the stop channel
        let mut gossiper = Gossiper::new(config, set.clone(), gossip_client.clone(), stop_rx);

        gossiper.gossip().await;
    });

    // Sleep for a few seconds, make sure the whole process ran
    tokio::time::sleep(Duration::from_secs(1)).await;
    // Send the stop signal before awaiting the task.
    if stop_tx.send(()).await.is_err() {
        eprintln!("Failed to send stop signal");
    }

    // Await the completion of the gossiping task
    let _ = gossip_task.await.expect("Gossip task failed");
}


#[tokio::main]
async fn main() {

    // Start the client
    // listen on 8080 , send message to 8081
    let client_01_handle = tokio::spawn(start_fake_node("127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string()));
    let client_02_handle = tokio::spawn(start_fake_node("127.0.0.1:8082".to_string(), "127.0.0.1:8083".to_string(), "127.0.0.1:8080".to_string()));

    // Wait for the server and client to complete
    client_01_handle.await.unwrap();
    client_02_handle.await.unwrap();
}

