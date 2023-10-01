use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use log::{debug, error};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use network::p2p::gossip::gossip::{Config, Gossiper};
use tokio::sync::mpsc::{channel, Receiver};
use avalanche_types::ids::Id;
use avalanche_types::ids::node::Id as NodeId;
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
        todo!()
    }

    async fn app_request_any(&mut self, request_bytes: Vec<u8>, on_response: AppResponseCallback) -> Result<(), std::io::Error> {
        let mut stream = self.stream.lock().await;
        stream.write_all(&*request_bytes).await?;

        // Lock the listener and wait for a new connection
        let clone = self.listener.clone();
        let mut listener = clone.lock().await;

        let mut buf = [0u8; 1024];
        match listener.read(&mut buf).await {
            Ok(n) => {
                if n == 0 {
                    debug!("Connection to the listener was closed");
                    ()
                }

                on_response(buf[0..n].to_vec());
            }
            Err(e) => {
                error!("Issue occured in app_request_any {:?}", e)
            }
        }
        Ok(())
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct MockSet<TestGossipableType> {
    pub set: Vec<TestGossipableType>,
}

impl<T> MockSet<T> {
    pub fn len(&self) -> usize {
        println!("{}", self.set.len());
        self.set.len()
    }
}

impl<T: Gossipable + Sync + Send + Clone + Hash + Debug + PartialEq + for<'de> Deserialize<'de> + Serialize> Set for MockSet<T> {
    type Item = T;
    fn add(&mut self, gossipable: T) -> Result<(), Box<dyn Error>> {
        // Just for our test purpose am checking manually if we insert an already known gossip.

        if self.set.contains(&gossipable) {
            error!("Cannot insert this item, already known");
        } else {
            self.set.push(gossipable.clone());
        }

        Ok(())
    }

    fn has(&self, gossipable: &Self::Item) -> bool {
        self.set.contains(gossipable)
    }

    fn iterate(&self, _f: &mut dyn FnMut(&T) -> bool) {
        for item in &self.set {
            if !_f(item) {
                println!("Filter sent over network knows about this item already {:?}", item);
                break;
            }
        }
        // Do nothing
    }

    fn fetch_elements(&self) -> Self::Item {
        self.set.choose(&mut rand::thread_rng()).cloned().expect("Set is empty")
    }

    fn fetch_all_elements(&self) -> Vec<Self::Item> {
        self.set.clone()
    }
}

async fn fake_handler_server_logic(mut socket: TcpStream, client_socket: Arc<Mutex<TcpStream>>, handler: Handler<MockSet<TestGossipableType>>, mut stop_handler_rx: Receiver<()>) {

    // Initialize a buffer of size 1024.
    let mut buf = [0u8; 1024];
    loop {
        select! {
            result = socket.read(&mut buf) => {
                debug!("Handler tick");
                match result {
                    Ok(n) if n == 0 => {
                        break; // to check, 0 means connection closed ?
                    }
                    Ok(n) => {
                        let node_id: NodeId = NodeId::from_slice(&random_manager::secure_bytes(20).unwrap()); // Since we fake the network layer, we need to fake this as well
                        let res_bytes = match handler.app_gossip(node_id, buf[0..n].to_vec()).await {
                            Ok(res) => { res}
                            Err(error) => { error!("{:?}", error); continue }
                        };

                        debug!("Before aa");
                        let mut stream = client_socket.lock().await;
                        debug!("After aa {:?}", res_bytes);

                        if res_bytes.is_empty() {
                            // ToDo Whenever the handler return nothing , gossip part hang. Temp dev fix to get pass this
                            let mut temp_vec = Vec::new();
                            temp_vec.push(1);
                            let _ = stream.write_all(temp_vec.as_slice()).await; // todo check, feels ugly
                        } else {
                            let _ = stream.write_all(&res_bytes).await;
                        }

                        }
                        Err(err) => {
                            error!("Error {:?}", err);
                        }
                }
                debug!("End of handler tick run");
            }
            _ = stop_handler_rx.recv() => {
                debug!("Shutting down handler");
                let mut stream = client_socket.lock().await;
                let mut temp_vec = Vec::new();
                temp_vec.push(1);
                let _ = stream.write_all(temp_vec.as_slice()).await; // todo check, feels ugly
                break;
            }
        }
    }

    debug!("Out of handler loop");
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
    let handler_config = HandlerConfig { namespace: "test".to_string(), target_response_size: 1000 };

    let mut mock_set = MockSet { set: Vec::<TestGossipableType>::new() };
    for gossip in &vec_gossip_local_client {
        mock_set.set.push(
            gossip.clone()
        );
    }
    let set = Arc::new(Mutex::new(mock_set ));


    let handler = new_handler(
        handler_config,
        set.clone(),
    );

    // Clone listener and stream for use inside the spawned task.
    let own_handler_listener_clone = own_handler_listener.clone();
    let other_client_stream_clone = other_client_stream.clone();
    let (stop_handler_tx, stop_handler_rx) = channel(1);

    // Spawn an asynchronous task that will handle incoming connections in a loop
    let handler_task = tokio::spawn(async move {
        // Accept incoming connections and spawn a new task to handle each connection
        let listener = own_handler_listener_clone.lock().await;
        let (listener_socket, _) = listener.accept().await.unwrap();
        fake_handler_server_logic(listener_socket, other_client_stream_clone.clone(), handler.clone(), stop_handler_rx).await;
        debug!("HANDLER DONEZO");
    });

    {
        assert_eq!(set.lock().await.set.len().clone(), 3);
    }

    let (stop_tx, stop_rx) = channel(1);

    // Spawn the gossiping task
    let set_clone = set.clone();
    let gossip_task = tokio::spawn(async move {
        // Initialize a TestClient instance with the given stream and listener
        let (stream, _) = own_client_listener_r.lock().await.accept().await.expect("Fail");
        let gossip_client = Arc::new(Mutex::new(TestClient { stream: other_handler_stream.clone(), listener: Arc::new(Mutex::new(stream)) }));

        // Create a channel for stopping the gossiper

        // Initialize the Gossiper with the provided configuration, set, client, and receiver end of the stop channel
        let mut gossiper = Gossiper::new(config, set_clone, gossip_client.clone(), stop_rx);

        gossiper.gossip().await;
        debug!("GOSSIP DONEZO");
    });

    // Sleep for a few seconds, make sure the whole process ran at least a couple of times
    tokio::time::sleep(Duration::from_secs(2)).await;

    {
        let guard = set.lock().await;
        // As we have 3 elements in our set pre-gossip loop execution in each one of our fake gossip server, we should end up with 6 gossip at the end of our test run.
        assert!(guard.set.len() == 6);
        // Need to find them all
        for gossip in vec_gossip_remote_client {
            debug!("Checking if gossip {:?} is present in set {:?}", gossip, guard.set);
            assert!(guard.set.contains(&gossip));
        }
    }
    debug!("Sending stop signal to handler");

    // Send the stop signal before awaiting the task.
    if stop_handler_tx.send(()).await.is_err() {
        eprintln!("Failed to send stop signal");
    }

    debug!("Sending stop signal to gossiper");

    // Send the stop signal before awaiting the task.
    if stop_tx.send(()).await.is_err() {
        eprintln!("Failed to send stop signal");
    }


    tokio::time::sleep(Duration::from_secs(2)).await;

    debug!("Checking if all things good");

    // Await the completion of the gossiping task
    let _ = gossip_task.await.expect("Gossip task failed");
    debug!("Gossip task completed");
    let _ = handler_task.await.expect("Handler task failed");
    debug!("Handler task completed");
}


#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let mut vec_gossip_client_01 = Vec::new();
    let mut vec_gossip_client_02 = Vec::new();
    vec_gossip_client_01.push(TestGossipableType { id: Id::from_slice(&[52, 25, 83, 149, 20, 226, 168, 61, 17, 53, 152, 11, 220, 226, 218, 254, 53, 104, 51, 247, 106, 6, 9, 26, 81, 52, 108, 232, 251, 122, 245, 112]) });
    vec_gossip_client_01.push(TestGossipableType { id: Id::from_slice(&[243, 156, 106, 56, 180, 213, 172, 165, 124, 118, 229, 60, 213, 183, 93, 241, 98, 214, 130, 235, 220, 45, 163, 151, 97, 64, 51, 126, 52, 164, 179, 23]) });
    vec_gossip_client_01.push(TestGossipableType { id: Id::from_slice(&[213, 8, 151, 77, 221, 160, 231, 33, 231, 180, 49, 113, 38, 196, 52, 156, 252, 66, 78, 250, 21, 56, 75, 247, 245, 87, 69, 157, 127, 53, 205, 121]) });

    vec_gossip_client_02.push(TestGossipableType { id: Id::from_slice(&[60, 209, 244, 35, 53, 217, 132, 157, 105, 97, 191, 32, 74, 199, 107, 124, 168, 61, 86, 203, 71, 247, 202, 161, 23, 124, 185, 63, 158, 54, 122, 216]) });
    vec_gossip_client_02.push(TestGossipableType { id: Id::from_slice(&[70, 203, 24, 230, 112, 82, 4, 22, 154, 173, 148, 189, 142, 217, 209, 191, 170, 242, 62, 213, 242, 133, 226, 200, 128, 87, 126, 157, 141, 78, 32, 67]) });
    vec_gossip_client_02.push(TestGossipableType { id: Id::from_slice(&[51, 215, 234, 45, 201, 210, 176, 176, 229, 6, 151, 169, 125, 219, 45, 56, 144, 205, 27, 74, 17, 13, 231, 59, 42, 214, 12, 184, 171, 251, 191, 197]) });


    // Start the client
    // listen on 8080 , send message to 8081
    let client_01_handle = tokio::spawn(start_fake_node("127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string(), "127.0.0.1:8083".to_string(), vec_gossip_client_01.clone(), vec_gossip_client_02.clone()));
    let client_02_handle = tokio::spawn(start_fake_node("127.0.0.1:8082".to_string(), "127.0.0.1:8083".to_string(), "127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string(), vec_gossip_client_02.clone(), vec_gossip_client_01.clone()));

    // Wait for the server and client to complete
    client_01_handle.await.expect("Issue with client01");
    client_02_handle.await.expect("Issue with client02");
}

