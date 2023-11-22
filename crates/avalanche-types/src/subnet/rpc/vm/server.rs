//! RPC Chain VM Server.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    ids,
    packer::U32_LEN,
    proto::pb::{
        self,
        aliasreader::alias_reader_client::AliasReaderClient,
        google::protobuf::Empty,
        keystore::keystore_client::KeystoreClient,
        messenger::{messenger_client::MessengerClient, NotifyRequest},
        sharedmemory::shared_memory_client::SharedMemoryClient,
        vm,
    },
    subnet::rpc::{
        consensus::snowman::{Block, Decidable},
        context::Context,
        database::rpcdb::{client::DatabaseClient, error_to_error_code},
        database::{
            corruptabledb,
            manager::{versioned_database, DatabaseManager},
        },
        errors,
        http::server::Server as HttpServer,
        snow::{
            engine::common::{appsender::client::AppSenderClient, message::Message},
            validators::client::ValidatorStateClient,
            State,
        },
        snowman::block::ChainVm,
        utils::{
            self,
            grpc::{self, timestamp_from_time},
        },
    },
};
use chrono::{TimeZone, Utc};
use pb::vm::vm_server::Vm;
use prost::bytes::Bytes;
use semver::Version;
use tokio::sync::{broadcast, mpsc, RwLock};
use tonic::{Request, Response};
use crate::warp::client::WarpSignerClient;

pub struct Server<V> {
    /// Underlying Vm implementation.
    pub vm: Arc<RwLock<V>>,

    #[cfg(feature = "subnet_metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "subnet_metrics")))]
    /// Subnet Prometheus process metrics.
    pub process_metrics: Arc<RwLock<prometheus::Registry>>,

    /// Stop channel broadcast producer.
    pub stop_ch: broadcast::Sender<()>,
}

impl<V: ChainVm> Server<V> {
    pub fn new(vm: V, stop_ch: broadcast::Sender<()>) -> Self {
        Self {
            vm: Arc::new(RwLock::new(vm)),
            #[cfg(feature = "subnet_metrics")]
            #[cfg_attr(docsrs, doc(cfg(feature = "subnet_metrics")))]
            process_metrics: Arc::new(RwLock::new(prometheus::default_registry().to_owned())),
            stop_ch,
        }
    }

    /// Attempts to get the ancestors of a block from the underlying Vm.
    pub async fn vm_ancestors(
        &self,
        block_id_bytes: &[u8],
        max_block_num: i32,
        max_block_size: i32,
        max_block_retrival_time: Duration,
    ) -> std::io::Result<Vec<Bytes>> {
        let inner_vm = self.vm.read().await;
        inner_vm
            .get_ancestors(
                ids::Id::from_slice(block_id_bytes),
                max_block_num,
                max_block_size,
                max_block_retrival_time,
            )
            .await
    }
}

#[tonic::async_trait]
impl<V> Vm for Server<V>
where
    V: ChainVm<
            DatabaseManager = DatabaseManager,
            AppSender = AppSenderClient,
            ValidatorState = ValidatorStateClient,
            WarpSigner=WarpSignerClient,
        > + Send
        + Sync
        + 'static,
{
    async fn initialize(
        &self,
        req: Request<vm::InitializeRequest>,
    ) -> std::result::Result<Response<vm::InitializeResponse>, tonic::Status> {
        log::info!("initialize called");

        let req = req.into_inner();
        let server_addr = req.server_addr.as_str();
        let client_conn = utils::grpc::default_client(server_addr)?
            .connect()
            .await
            .map_err(|e| {
                tonic::Status::unknown(format!(
                    "failed to create client conn from: {server_addr}: {e}",
                ))
            })?;

        // Multiplexing in tonic is done by cloning the client which is very cheap.
        // ref. https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html#multiplexing-requests
        let mut message = MessengerClient::new(client_conn.clone());
        let keystore = KeystoreClient::new(client_conn.clone());
        let shared_memory = SharedMemoryClient::new(client_conn.clone());
        let bc_lookup = AliasReaderClient::new(client_conn.clone());

        let ctx: Option<Context<ValidatorStateClient>> = Some(Context {
            network_id: req.network_id,
            subnet_id: ids::Id::from_slice(&req.subnet_id),
            chain_id: ids::Id::from_slice(&req.chain_id),
            node_id: ids::node::Id::from_slice(&req.node_id),
            x_chain_id: ids::Id::from_slice(&req.x_chain_id),
            c_chain_id: ids::Id::from_slice(&req.c_chain_id),
            avax_asset_id: ids::Id::from_slice(&req.avax_asset_id),
            keystore,
            shared_memory,
            bc_lookup,
            chain_data_dir: req.chain_data_dir,
            validator_state: ValidatorStateClient::new(client_conn.clone()),
        });

        let mut versioned_dbs = Vec::with_capacity(req.db_servers.len());
        for db_server in req.db_servers.iter() {
            let semver = db_server.version.trim_start_matches('v');
            let version =
                Version::parse(semver).map_err(|e| tonic::Status::unknown(e.to_string()))?;
            let server_addr = db_server.server_addr.as_str();

            // Create a client connection with the server address
            let client_conn = utils::grpc::default_client(server_addr)?
                .connect()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(format!(
                        "failed to create client conn from: {server_addr}: {e}",
                    ))
                })?;

            let vdb = versioned_database::VersionedDatabase::new(
                corruptabledb::Database::new(DatabaseClient::new(client_conn)),
                version,
            );
            versioned_dbs.push(vdb);
        }

        let (tx_engine, mut rx_engine): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(100);
        tokio::spawn(async move {
            loop {
                if let Some(msg) = rx_engine.recv().await {
                    log::debug!("message received: {msg:?}");
                    let _ = message
                        .notify(NotifyRequest {
                            message: msg as i32,
                        })
                        .await
                        .map_err(|s| tonic::Status::unknown(s.to_string()));
                    continue;
                }

                log::error!("engine receiver closed unexpectedly");
                return tonic::Status::unknown("engine receiver closed unexpectedly");
            }
        });
        let warp_signer = WarpSignerClient::new(client_conn.clone());
        let mut inner_vm = self.vm.write().await;
        inner_vm
            .initialize(
                ctx,
                DatabaseManager::from_databases(versioned_dbs),
                &req.genesis_bytes,
                &req.upgrade_bytes,
                &req.config_bytes,
                tx_engine,
                &[()],
                AppSenderClient::new(client_conn.clone()),
                warp_signer
            )
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        // Get last accepted block on the chain
        let last_accepted = inner_vm.last_accepted().await?;

        let last_accepted_block = inner_vm
            .get_block(last_accepted)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        log::debug!("last_accepted_block id: {last_accepted:?}");

        Ok(Response::new(vm::InitializeResponse {
            last_accepted_id: Bytes::from(last_accepted.to_vec()),
            last_accepted_parent_id: Bytes::from(last_accepted_block.parent().await.to_vec()),
            bytes: Bytes::from(last_accepted_block.bytes().await.to_vec()),
            height: last_accepted_block.height().await,
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(last_accepted_block.timestamp().await as i64, 0)
                    .unwrap(),
            )),
        }))
    }

    async fn shutdown(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("shutdown called");

        // notify all gRPC servers to shutdown
        self.stop_ch
            .send(())
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    /// Creates the HTTP handlers for custom chain network calls.
    /// This creates and exposes handlers that the outside world can use to communicate
    /// with the chain. Each handler has the path:
    /// `\[Address of node]/ext/bc/[chain ID]/[extension\]`
    ///
    /// Returns a mapping from \[extension\]s to HTTP handlers.
    /// Each extension can specify how locking is managed for convenience.
    ///
    /// For example, if this VM implements an account-based payments system,
    /// it have an extension called `accounts`, where clients could get
    /// information about their accounts.
    async fn create_handlers(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::CreateHandlersResponse>, tonic::Status> {
        log::debug!("create_handlers called");

        // get handlers from underlying vm
        let mut inner_vm = self.vm.write().await;
        let handlers = inner_vm
            .create_handlers()
            .await
            .map_err(|e| tonic::Status::unknown(format!("failed to create handlers: {e}")))?;

        // create and start gRPC server serving HTTP service for each handler
        let mut resp_handlers: Vec<vm::Handler> = Vec::with_capacity(handlers.keys().len());
        for (prefix, http_handler) in handlers {
            let server_addr = utils::new_socket_addr();
            let server = grpc::Server::new(server_addr, self.stop_ch.subscribe());

            server
                .serve(pb::http::http_server::HttpServer::new(HttpServer::new(
                    http_handler.handler,
                )))
                .map_err(|e| {
                    tonic::Status::unknown(format!("failed to create http service: {e}"))
                })?;

            let resp_handler = vm::Handler {
                prefix,
                server_addr: server_addr.to_string(),
            };
            resp_handlers.push(resp_handler);
        }

        Ok(Response::new(vm::CreateHandlersResponse {
            handlers: resp_handlers,
        }))
    }

    /// Creates the HTTP handlers for custom VM network calls.
    ///
    /// This creates and exposes handlers that the outside world can use to communicate
    /// with a static reference to the VM. Each handler has the path:
    /// `\[Address of node]/ext/VM/[VM ID]/[extension\]`
    ///
    /// Returns a mapping from \[extension\]s to HTTP handlers.
    ///
    /// Each extension can specify how locking is managed for convenience.
    ///
    /// For example, it might make sense to have an extension for creating
    /// genesis bytes this VM can interpret.
    async fn create_static_handlers(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::CreateStaticHandlersResponse>, tonic::Status> {
        log::debug!("create_static_handlers called");

        // get handlers from underlying vm
        let mut inner_vm = self.vm.write().await;
        let handlers = inner_vm.create_static_handlers().await.map_err(|e| {
            tonic::Status::unknown(format!("failed to create static handlers: {e}"))
        })?;

        // create and start gRPC server serving HTTP service for each handler
        let mut resp_handlers: Vec<vm::Handler> = Vec::with_capacity(handlers.keys().len());
        for (prefix, http_handler) in handlers {
            let server_addr = utils::new_socket_addr();
            let server = grpc::Server::new(server_addr, self.stop_ch.subscribe());

            server
                .serve(pb::http::http_server::HttpServer::new(HttpServer::new(
                    http_handler.handler,
                )))
                .map_err(|e| {
                    tonic::Status::unknown(format!("failed to create http service: {e}"))
                })?;

            let resp_handler = vm::Handler {
                prefix,
                server_addr: server_addr.to_string(),
            };
            resp_handlers.push(resp_handler);
        }

        Ok(Response::new(vm::CreateStaticHandlersResponse {
            handlers: resp_handlers,
        }))
    }

    async fn build_block(
        &self,
        _req: Request<vm::BuildBlockRequest>,
    ) -> std::result::Result<Response<vm::BuildBlockResponse>, tonic::Status> {
        log::debug!("build_block called");

        let inner_vm = self.vm.write().await;
        let block = inner_vm
            .build_block()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::BuildBlockResponse {
            id: Bytes::from(block.id().await.to_vec()),
            parent_id: Bytes::from(block.parent().await.to_vec()),
            bytes: Bytes::from(block.bytes().await.to_vec()),
            height: block.height().await,
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(block.timestamp().await as i64, 0)
                    .unwrap(),
            )),
            verify_with_context: false,
        }))
    }

    async fn parse_block(
        &self,
        req: Request<vm::ParseBlockRequest>,
    ) -> std::result::Result<Response<vm::ParseBlockResponse>, tonic::Status> {
        log::debug!("parse_block called");

        let req = req.into_inner();
        let inner_vm = self.vm.write().await;
        let block = inner_vm
            .parse_block(req.bytes.as_ref())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::ParseBlockResponse {
            id: Bytes::from(block.id().await.to_vec()),
            parent_id: Bytes::from(block.parent().await.to_vec()),
            status: block.status().await.to_i32(),
            height: block.height().await,
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(block.timestamp().await as i64, 0)
                    .unwrap(),
            )),
            verify_with_context: false,
        }))
    }

    /// Attempt to load a block.
    ///
    /// If the block does not exist, an empty GetBlockResponse is returned with
    /// an error code.
    ///
    /// It is expected that blocks that have been successfully verified should be
    /// returned correctly. It is also expected that blocks that have been
    /// accepted by the consensus engine should be able to be fetched. It is not
    /// required for blocks that have been rejected by the consensus engine to be
    /// able to be fetched.
    ///
    /// ref: <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#Getter>
    async fn get_block(
        &self,
        req: Request<vm::GetBlockRequest>,
    ) -> std::result::Result<Response<vm::GetBlockResponse>, tonic::Status> {
        log::debug!("get_block called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;

        // determine if response is an error or not
        match inner_vm.get_block(ids::Id::from_slice(&req.id)).await {
            Ok(block) => Ok(Response::new(vm::GetBlockResponse {
                parent_id: Bytes::from(block.parent().await.to_vec()),
                bytes: Bytes::from(block.bytes().await.to_vec()),
                status: block.status().await.to_i32(),
                height: block.height().await,
                timestamp: Some(timestamp_from_time(
                    &Utc.timestamp_opt(block.timestamp().await as i64, 0)
                        .unwrap(),
                )),
                err: 0, // return 0 indicating no error
                verify_with_context: false,
            })),
            // if an error was found, generate empty response with ErrNotFound code
            // ref: https://github.com/ava-labs/avalanchego/blob/master/vms/
            Err(e) => {
                log::debug!("Error getting block");
                Ok(Response::new(vm::GetBlockResponse {
                    parent_id: Bytes::new(),
                    bytes: Bytes::new(),
                    status: 0,
                    height: 0,
                    timestamp: Some(timestamp_from_time(&Utc.timestamp_opt(0, 0).unwrap())),
                    err: error_to_error_code(&e.to_string()),
                    verify_with_context: false,
                }))
            }
        }
    }

    async fn set_state(
        &self,
        req: Request<vm::SetStateRequest>,
    ) -> std::result::Result<Response<vm::SetStateResponse>, tonic::Status> {
        log::debug!("set_state called");

        let req = req.into_inner();
        let inner_vm = self.vm.write().await;
        let state = State::try_from(req.state)
            .map_err(|_| tonic::Status::unknown("failed to convert to vm state"))?;

        inner_vm
            .set_state(state)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        let last_accepted_id = inner_vm
            .last_accepted()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        let block = inner_vm
            .get_block(last_accepted_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::SetStateResponse {
            last_accepted_id: Bytes::from(last_accepted_id.to_vec()),
            last_accepted_parent_id: Bytes::from(block.parent().await.to_vec()),
            height: block.height().await,
            bytes: Bytes::from(block.bytes().await.to_vec()),
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(block.timestamp().await as i64, 0)
                    .unwrap(),
            )),
        }))
    }

    async fn set_preference(
        &self,
        req: Request<vm::SetPreferenceRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("set_preference called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;
        inner_vm
            .set_preference(ids::Id::from_slice(&req.id))
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn health(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::HealthResponse>, tonic::Status> {
        log::debug!("health called");

        let inner_vm = self.vm.read().await;
        let resp = inner_vm
            .health_check()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::HealthResponse {
            details: Bytes::from(resp),
        }))
    }

    async fn version(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::VersionResponse>, tonic::Status> {
        log::debug!("version called");

        let inner_vm = self.vm.read().await;
        let version = inner_vm
            .version()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::VersionResponse { version }))
    }

    async fn connected(
        &self,
        req: Request<vm::ConnectedRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("connected called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;
        let node_id = ids::node::Id::from_slice(&req.node_id);
        inner_vm
            .connected(&node_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn disconnected(
        &self,
        req: Request<vm::DisconnectedRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("disconnected called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;
        let node_id = ids::node::Id::from_slice(&req.node_id);

        inner_vm
            .disconnected(&node_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_request(
        &self,
        req: Request<vm::AppRequestMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_request called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        let inner_vm = self.vm.read().await;

        let ts = req.deadline.as_ref().expect("timestamp");
        let deadline = Utc.timestamp_opt(ts.seconds, ts.nanos as u32).unwrap();

        inner_vm
            .app_request(&node_id, req.request_id, deadline, &req.request)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_request_failed(
        &self,
        req: Request<vm::AppRequestFailedMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_request_failed called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        let inner_vm = self.vm.read().await;

        inner_vm
            .app_request_failed(&node_id, req.request_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_response(
        &self,
        req: Request<vm::AppResponseMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_response called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        let inner_vm = self.vm.read().await;

        inner_vm
            .app_response(&node_id, req.request_id, &req.response)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_gossip(
        &self,
        req: Request<vm::AppGossipMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_gossip called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        let inner_vm = self.vm.read().await;

        inner_vm
            .app_gossip(&node_id, &req.msg)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn block_verify(
        &self,
        req: Request<vm::BlockVerifyRequest>,
    ) -> std::result::Result<Response<vm::BlockVerifyResponse>, tonic::Status> {
        log::debug!("block_verify called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;

        let mut block = inner_vm
            .parse_block(&req.bytes)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        block
            .verify()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::BlockVerifyResponse {
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(block.timestamp().await as i64, 0)
                    .unwrap(),
            )),
        }))
    }

    async fn block_accept(
        &self,
        req: Request<vm::BlockAcceptRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("block_accept called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;
        let id = ids::Id::from_slice(&req.id);

        let mut block = inner_vm
            .get_block(id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        block
            .accept()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
    async fn block_reject(
        &self,
        req: Request<vm::BlockRejectRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("block_reject called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;
        let id = ids::Id::from_slice(&req.id);

        let mut block = inner_vm
            .get_block(id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        block
            .reject()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn get_ancestors(
        &self,
        req: Request<vm::GetAncestorsRequest>,
    ) -> std::result::Result<Response<vm::GetAncestorsResponse>, tonic::Status> {
        log::debug!("get_ancestors called");
        let req = req.into_inner();

        let block_id = ids::Id::from_slice(req.blk_id.as_ref());
        let max_blocks_size = usize::try_from(req.max_blocks_size).expect("cast from i32");
        let max_blocks_num = usize::try_from(req.max_blocks_num).expect("cast from i32");
        let max_blocks_retrival_time = Duration::from_secs(
            req.max_blocks_retrival_time
                .try_into()
                .expect("valid timestamp"),
        );

        let ancestors = self
            .vm_ancestors(
                req.blk_id.as_ref(),
                req.max_blocks_num,
                req.max_blocks_size,
                max_blocks_retrival_time,
            )
            .await
            .map(|blks_bytes| Response::new(vm::GetAncestorsResponse { blks_bytes }));

        let e = match ancestors {
            Ok(ancestors) => return Ok(ancestors),
            Err(e) => e,
        };

        if e.kind() != std::io::ErrorKind::Unsupported {
            return Err(tonic::Status::unknown(e.to_string()));
        }

        // not supported by underlying vm use local logic
        let start = Instant::now();
        let mut block = match self.vm.read().await.get_block(block_id).await {
            Ok(b) => b,
            Err(e) => {
                // special case ErrNotFound as an empty response: this signals
                // the client to avoid contacting this node for further ancestors
                // as they may have been pruned or unavailable due to state-sync.
                return if errors::is_not_found(&e) {
                    log::debug!("get_ancestors local get_block returned: not found");

                    Ok(Response::new(vm::GetAncestorsResponse {
                        blks_bytes: Vec::new(),
                    }))
                } else {
                    Err(e.into())
                };
            }
        };

        let mut ancestors = Vec::with_capacity(max_blocks_num);
        let block_bytes = block.bytes().await;

        // length, in bytes, of all elements of ancestors
        let mut ancestors_bytes_len = block_bytes.len() + U32_LEN;
        ancestors.push(Bytes::from(block_bytes.to_owned()));

        while ancestors.len() < max_blocks_num {
            if start.elapsed() < max_blocks_retrival_time {
                log::debug!("get_ancestors exceeded max block retrival time");
                break;
            }

            let parent_id = block.parent().await;

            block = match self.vm.read().await.get_block(parent_id).await {
                Ok(b) => b,
                Err(e) => {
                    if errors::is_not_found(&e) {
                        // after state sync we may not have the full chain
                        log::debug!("failed to get block during ancestors lookup parentId: {parent_id}: {e}");
                    }

                    break;
                }
            };

            let block_bytes = block.bytes().await;

            // Ensure response size isn't too large. Include U32_LEN because
            // the size of the message is included with each container, and the size
            // is repr. by 4 bytes.
            ancestors_bytes_len += block_bytes.len() + U32_LEN;

            if ancestors_bytes_len > max_blocks_size {
                log::debug!("get_ancestors reached maximum response size: {ancestors_bytes_len}");
                break;
            }

            ancestors.push(Bytes::from(block_bytes.to_owned()));
        }

        Ok(Response::new(vm::GetAncestorsResponse {
            blks_bytes: ancestors,
        }))
    }

    async fn batched_parse_block(
        &self,
        req: Request<vm::BatchedParseBlockRequest>,
    ) -> std::result::Result<Response<vm::BatchedParseBlockResponse>, tonic::Status> {
        log::debug!("batched_parse_block called");
        let req = req.into_inner();

        let to_parse = req
            .request
            .into_iter()
            .map(|bytes| Request::new(vm::ParseBlockRequest { bytes }))
            .map(|request| async {
                self.parse_block(request)
                    .await
                    .map(|block| block.into_inner())
            });
        let blocks = futures::future::try_join_all(to_parse).await?;

        Ok(Response::new(vm::BatchedParseBlockResponse {
            response: blocks,
        }))
    }

    #[cfg(not(feature = "subnet_metrics"))]
    async fn gather(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GatherResponse>, tonic::Status> {
        log::debug!("gather called");

        let metric_families =
            vec![crate::proto::pb::io::prometheus::client::MetricFamily::default()];

        Ok(Response::new(vm::GatherResponse { metric_families }))
    }

    #[cfg(feature = "subnet_metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "subnet_metrics")))]
    async fn gather(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GatherResponse>, tonic::Status> {
        log::debug!("gather called");

        // ref. <https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics>
        let metric_families = crate::subnet::rpc::metrics::MetricsFamilies::from(
            &self.process_metrics.read().await.gather(),
        )
        .mfs;

        Ok(Response::new(vm::GatherResponse { metric_families }))
    }

    async fn cross_chain_app_request(
        &self,
        req: Request<vm::CrossChainAppRequestMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("cross_chain_app_request called");
        let msg = req.into_inner();
        let chain_id = &ids::Id::from_slice(&msg.chain_id);

        let ts = msg.deadline.as_ref().expect("timestamp");
        let deadline = Utc.timestamp_opt(ts.seconds, ts.nanos as u32).unwrap();

        let inner_vm = self.vm.read().await;
        inner_vm
            .cross_chain_app_request(chain_id, msg.request_id, deadline, &msg.request)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn cross_chain_app_request_failed(
        &self,
        req: Request<vm::CrossChainAppRequestFailedMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("cross_chain_app_request_failed called");
        let msg = req.into_inner();
        let chain_id = &ids::Id::from_slice(&msg.chain_id);

        let inner_vm = self.vm.read().await;
        inner_vm
            .cross_chain_app_request_failed(chain_id, msg.request_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn cross_chain_app_response(
        &self,
        req: Request<vm::CrossChainAppResponseMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("cross_chain_app_response called");
        let msg = req.into_inner();
        let chain_id = &ids::Id::from_slice(&msg.chain_id);

        let inner_vm = self.vm.read().await;
        inner_vm
            .cross_chain_app_response(chain_id, msg.request_id, &msg.response)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn state_sync_enabled(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::StateSyncEnabledResponse>, tonic::Status> {
        log::debug!("state_sync_enabled called");

        // TODO: Implement state sync request/response
        Ok(Response::new(vm::StateSyncEnabledResponse {
            enabled: false,
            err: 0,
        }))
    }

    async fn get_ongoing_sync_state_summary(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GetOngoingSyncStateSummaryResponse>, tonic::Status> {
        log::debug!("get_ongoing_sync_state_summary called");

        Err(tonic::Status::unimplemented(
            "get_ongoing_sync_state_summary",
        ))
    }

    async fn parse_state_summary(
        &self,
        _req: Request<vm::ParseStateSummaryRequest>,
    ) -> std::result::Result<tonic::Response<vm::ParseStateSummaryResponse>, tonic::Status> {
        log::debug!("parse_state_summary called");

        Err(tonic::Status::unimplemented("parse_state_summary"))
    }

    async fn get_state_summary(
        &self,
        _req: Request<vm::GetStateSummaryRequest>,
    ) -> std::result::Result<Response<vm::GetStateSummaryResponse>, tonic::Status> {
        log::debug!("get_state_summary called");

        Err(tonic::Status::unimplemented("get_state_summary"))
    }

    async fn get_last_state_summary(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GetLastStateSummaryResponse>, tonic::Status> {
        log::debug!("get_last_state_summary called");

        Err(tonic::Status::unimplemented("get_last_state_summary"))
    }

    async fn state_summary_accept(
        &self,
        _req: Request<vm::StateSummaryAcceptRequest>,
    ) -> std::result::Result<tonic::Response<vm::StateSummaryAcceptResponse>, tonic::Status> {
        log::debug!("state_summary_accept called");

        Err(tonic::Status::unimplemented("state_summary_accept"))
    }

    async fn verify_height_index(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::VerifyHeightIndexResponse>, tonic::Status> {
        log::debug!("verify_height_index called");

        let inner_vm = self.vm.read().await;

        match inner_vm.verify_height_index().await {
            Ok(_) => return Ok(Response::new(vm::VerifyHeightIndexResponse { err: 0 })),
            Err(e) => {
                if error_to_error_code(&e.to_string()) != 0 {
                    return Ok(Response::new(vm::VerifyHeightIndexResponse {
                        err: error_to_error_code(&e.to_string()),
                    }));
                }
                return Err(tonic::Status::unknown(e.to_string()));
            }
        }
    }

    async fn get_block_id_at_height(
        &self,
        req: Request<vm::GetBlockIdAtHeightRequest>,
    ) -> std::result::Result<Response<vm::GetBlockIdAtHeightResponse>, tonic::Status> {
        log::debug!("get_block_id_at_height called");

        let msg = req.into_inner();
        let inner_vm = self.vm.read().await;

        match inner_vm.get_block_id_at_height(msg.height).await {
            Ok(height) => {
                return Ok(Response::new(vm::GetBlockIdAtHeightResponse {
                    blk_id: height.to_vec().into(),
                    err: 0,
                }))
            }
            Err(e) => {
                if error_to_error_code(&e.to_string()) != 0 {
                    return Ok(Response::new(vm::GetBlockIdAtHeightResponse {
                        blk_id: vec![].into(),
                        err: error_to_error_code(&e.to_string()),
                    }));
                }
                return Err(tonic::Status::unknown(e.to_string()));
            }
        }
    }
}
