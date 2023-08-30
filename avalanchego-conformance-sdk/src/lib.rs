use std::{
    io::{self, Error, ErrorKind},
    sync::Arc,
};

use tokio::sync::Mutex;
use tonic::transport::Channel;

pub mod rpcpb {
    tonic::include_proto!("rpcpb");
}
pub use rpcpb::{
    key_service_client::KeyServiceClient, message_service_client::MessageServiceClient,
    packer_service_client::PackerServiceClient, ping_service_client::PingServiceClient,
    AcceptedFrontierRequest, AcceptedFrontierResponse, AcceptedRequest, AcceptedResponse,
    AcceptedStateSummaryRequest, AcceptedStateSummaryResponse, AncestorsRequest, AncestorsResponse,
    AppGossipRequest, AppGossipResponse, AppRequestRequest, AppRequestResponse, AppResponseRequest,
    AppResponseResponse, BlsSignatureRequest, BlsSignatureResponse, BuildVertexRequest,
    BuildVertexResponse, CertificateToNodeIdRequest, CertificateToNodeIdResponse, ChainAddresses,
    ChitsRequest, ChitsResponse, GetAcceptedFrontierRequest, GetAcceptedFrontierResponse,
    GetAcceptedRequest, GetAcceptedResponse, GetAcceptedStateSummaryRequest,
    GetAcceptedStateSummaryResponse, GetAncestorsRequest, GetAncestorsResponse, GetRequest,
    GetResponse, GetStateSummaryFrontierRequest, GetStateSummaryFrontierResponse, Peer,
    PeerlistRequest, PeerlistResponse, PingRequest, PingResponse, PingServiceRequest,
    PingServiceResponse, PongRequest, PongResponse, PullQueryRequest, PullQueryResponse,
    PushQueryRequest, PushQueryResponse, PutRequest, PutResponse, Secp256k1Info,
    Secp256k1InfoRequest, Secp256k1InfoResponse, Secp256k1RecoverHashPublicKeyRequest,
    Secp256k1RecoverHashPublicKeyResponse, StateSummaryFrontierRequest,
    StateSummaryFrontierResponse, VersionRequest, VersionResponse,
};

pub struct Client<T> {
    pub rpc_endpoint: String,
    /// Shared gRPC client connections.
    pub grpc_client: Arc<GrpcClient<T>>,
}

pub struct GrpcClient<T> {
    pub ping_service_client: Mutex<PingServiceClient<T>>,
    pub key_service_client: Mutex<KeyServiceClient<T>>,
    pub packer_service_client: Mutex<PackerServiceClient<T>>,
    pub message_service_client: Mutex<MessageServiceClient<T>>,
}

impl Client<Channel> {
    /// Creates a new avalanchego-conformance client.
    ///
    /// # Arguments
    ///
    /// * `rpc_endpoint` - HTTP RPC endpoint to the network runner server.
    pub async fn new(rpc_endpoint: &str) -> Self {
        log::info!("creating a new client with {}", rpc_endpoint);
        let ep = String::from(rpc_endpoint);
        let ping_client = PingServiceClient::connect(ep.clone()).await.unwrap();
        let key_client = KeyServiceClient::connect(ep.clone()).await.unwrap();
        let packer_client = PackerServiceClient::connect(ep.clone()).await.unwrap();
        let message_client = MessageServiceClient::connect(ep.clone()).await.unwrap();
        let grpc_client = GrpcClient {
            ping_service_client: Mutex::new(ping_client),
            key_service_client: Mutex::new(key_client),
            packer_service_client: Mutex::new(packer_client),
            message_service_client: Mutex::new(message_client),
        };
        Self {
            rpc_endpoint: String::from(rpc_endpoint),
            grpc_client: Arc::new(grpc_client),
        }
    }

    /// Pings the avalanchego-conformance server.
    pub async fn ping_service(&self) -> io::Result<PingServiceResponse> {
        let mut ping_client = self.grpc_client.ping_service_client.lock().await;
        let req = tonic::Request::new(PingServiceRequest {});
        let resp = ping_client
            .ping_service(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed ping_service '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn certificate_to_node_id(
        &self,
        req: CertificateToNodeIdRequest,
    ) -> io::Result<CertificateToNodeIdResponse> {
        let mut cli = self.grpc_client.key_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.certificate_to_node_id(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed certificate_to_node_id '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn secp256k1_recover_hash_public_key(
        &self,
        req: Secp256k1RecoverHashPublicKeyRequest,
    ) -> io::Result<Secp256k1RecoverHashPublicKeyResponse> {
        let mut cli = self.grpc_client.key_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .secp256k1_recover_hash_public_key(req)
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed secp256k1_recover_hash_public_key '{}'", e),
                )
            })?;
        Ok(resp.into_inner())
    }

    pub async fn secp256k1_info(
        &self,
        req: Secp256k1InfoRequest,
    ) -> io::Result<Secp256k1InfoResponse> {
        let mut cli = self.grpc_client.key_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .secp256k1_info(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed secp256k1_info '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn bls_signature(
        &self,
        req: BlsSignatureRequest,
    ) -> io::Result<BlsSignatureResponse> {
        let mut cli = self.grpc_client.key_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .bls_signature(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed bls_signature '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn build_vertex(&self, req: BuildVertexRequest) -> io::Result<BuildVertexResponse> {
        let mut cli = self.grpc_client.packer_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .build_vertex(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed build_vertex '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn accepted_frontier(
        &self,
        req: AcceptedFrontierRequest,
    ) -> io::Result<AcceptedFrontierResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.accepted_frontier(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed accepted_frontier '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn accepted_state_summary(
        &self,
        req: AcceptedStateSummaryRequest,
    ) -> io::Result<AcceptedStateSummaryResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.accepted_state_summary(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed accepted_state_summary '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn accepted(&self, req: AcceptedRequest) -> io::Result<AcceptedResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .accepted(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed accepted '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn ancestors(&self, req: AncestorsRequest) -> io::Result<AncestorsResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .ancestors(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed ancestors '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn app_gossip(&self, req: AppGossipRequest) -> io::Result<AppGossipResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .app_gossip(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed app_gossip '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn app_request(&self, req: AppRequestRequest) -> io::Result<AppRequestResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .app_request(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed app_request '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn app_response(&self, req: AppResponseRequest) -> io::Result<AppResponseResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .app_response(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed app_response '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn chits(&self, req: ChitsRequest) -> io::Result<ChitsResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .chits(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed chits '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn get_accepted_frontier(
        &self,
        req: GetAcceptedFrontierRequest,
    ) -> io::Result<GetAcceptedFrontierResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.get_accepted_frontier(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed get_accepted_frontier '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn get_accepted_state_summary(
        &self,
        req: GetAcceptedStateSummaryRequest,
    ) -> io::Result<GetAcceptedStateSummaryResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.get_accepted_state_summary(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed get_accepted_state_summary '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn get_accepted(&self, req: GetAcceptedRequest) -> io::Result<GetAcceptedResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .get_accepted(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed get_accepted '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn get_ancestors(
        &self,
        req: GetAncestorsRequest,
    ) -> io::Result<GetAncestorsResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .get_ancestors(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed get_ancestors '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn get_state_summary_frontier(
        &self,
        req: GetStateSummaryFrontierRequest,
    ) -> io::Result<GetStateSummaryFrontierResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.get_state_summary_frontier(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed get_state_summary_frontier '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn get(&self, req: GetRequest) -> io::Result<GetResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .get(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed get '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn peerlist(&self, req: PeerlistRequest) -> io::Result<PeerlistResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .peerlist(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed peerlist '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn ping(&self, req: PingRequest) -> io::Result<PingResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .ping(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed ping '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn pong(&self, req: PongRequest) -> io::Result<PongResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .pong(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed pong '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn pull_query(&self, req: PullQueryRequest) -> io::Result<PullQueryResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .pull_query(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed pull_query '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn push_query(&self, req: PushQueryRequest) -> io::Result<PushQueryResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .push_query(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed push_query '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn put(&self, req: PutRequest) -> io::Result<PutResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .put(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed put '{}'", e)))?;
        Ok(resp.into_inner())
    }

    pub async fn state_summary_frontier(
        &self,
        req: StateSummaryFrontierRequest,
    ) -> io::Result<StateSummaryFrontierResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli.state_summary_frontier(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed state_summary_frontier '{}'", e),
            )
        })?;
        Ok(resp.into_inner())
    }

    pub async fn version(&self, req: VersionRequest) -> io::Result<VersionResponse> {
        let mut cli = self.grpc_client.message_service_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = cli
            .version(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed version '{}'", e)))?;
        Ok(resp.into_inner())
    }
}

pub struct CertificateToNodeIdArgs {
    pub certificate: Vec<u8>,
    pub node_id: Vec<u8>,
}

pub struct Secp256k1InfoArgs {
    pub private_key: String,
    pub private_key_hex: String,
    pub network_id: u32,
    pub xaddr: String,
    pub paddr: String,
    pub caddr: String,
    pub short_address: String,
    pub eth_address: String,
}
