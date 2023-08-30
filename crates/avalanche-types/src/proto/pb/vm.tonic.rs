// @generated
/// Generated client implementations.
pub mod vm_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct VmClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl VmClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> VmClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> VmClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            VmClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn initialize(
            &mut self,
            request: impl tonic::IntoRequest<super::InitializeRequest>,
        ) -> Result<tonic::Response<super::InitializeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Initialize");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn set_state(
            &mut self,
            request: impl tonic::IntoRequest<super::SetStateRequest>,
        ) -> Result<tonic::Response<super::SetStateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/SetState");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn shutdown(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Shutdown");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_handlers(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::CreateHandlersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/CreateHandlers");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_static_handlers(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<
            tonic::Response<super::CreateStaticHandlersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/vm.VM/CreateStaticHandlers",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn connected(
            &mut self,
            request: impl tonic::IntoRequest<super::ConnectedRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Connected");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn disconnected(
            &mut self,
            request: impl tonic::IntoRequest<super::DisconnectedRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Disconnected");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn build_block(
            &mut self,
            request: impl tonic::IntoRequest<super::BuildBlockRequest>,
        ) -> Result<tonic::Response<super::BuildBlockResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/BuildBlock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn parse_block(
            &mut self,
            request: impl tonic::IntoRequest<super::ParseBlockRequest>,
        ) -> Result<tonic::Response<super::ParseBlockResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/ParseBlock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_block(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockRequest>,
        ) -> Result<tonic::Response<super::GetBlockResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/GetBlock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn set_preference(
            &mut self,
            request: impl tonic::IntoRequest<super::SetPreferenceRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/SetPreference");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn health(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::HealthResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Health");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn version(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::VersionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Version");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn app_request(
            &mut self,
            request: impl tonic::IntoRequest<super::AppRequestMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/AppRequest");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn app_request_failed(
            &mut self,
            request: impl tonic::IntoRequest<super::AppRequestFailedMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/AppRequestFailed");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn app_response(
            &mut self,
            request: impl tonic::IntoRequest<super::AppResponseMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/AppResponse");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn app_gossip(
            &mut self,
            request: impl tonic::IntoRequest<super::AppGossipMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/AppGossip");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn gather(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::GatherResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/Gather");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn cross_chain_app_request(
            &mut self,
            request: impl tonic::IntoRequest<super::CrossChainAppRequestMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/vm.VM/CrossChainAppRequest",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn cross_chain_app_request_failed(
            &mut self,
            request: impl tonic::IntoRequest<super::CrossChainAppRequestFailedMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/vm.VM/CrossChainAppRequestFailed",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn cross_chain_app_response(
            &mut self,
            request: impl tonic::IntoRequest<super::CrossChainAppResponseMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/vm.VM/CrossChainAppResponse",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_ancestors(
            &mut self,
            request: impl tonic::IntoRequest<super::GetAncestorsRequest>,
        ) -> Result<tonic::Response<super::GetAncestorsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/GetAncestors");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn batched_parse_block(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchedParseBlockRequest>,
        ) -> Result<tonic::Response<super::BatchedParseBlockResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/BatchedParseBlock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn verify_height_index(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::VerifyHeightIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/VerifyHeightIndex");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_block_id_at_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockIdAtHeightRequest>,
        ) -> Result<tonic::Response<super::GetBlockIdAtHeightResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/GetBlockIDAtHeight");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn state_sync_enabled(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::StateSyncEnabledResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/StateSyncEnabled");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_ongoing_sync_state_summary(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<
            tonic::Response<super::GetOngoingSyncStateSummaryResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/vm.VM/GetOngoingSyncStateSummary",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_last_state_summary(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::GetLastStateSummaryResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/vm.VM/GetLastStateSummary",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn parse_state_summary(
            &mut self,
            request: impl tonic::IntoRequest<super::ParseStateSummaryRequest>,
        ) -> Result<tonic::Response<super::ParseStateSummaryResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/ParseStateSummary");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_state_summary(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStateSummaryRequest>,
        ) -> Result<tonic::Response<super::GetStateSummaryResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/GetStateSummary");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn block_verify(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockVerifyRequest>,
        ) -> Result<tonic::Response<super::BlockVerifyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/BlockVerify");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn block_accept(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockAcceptRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/BlockAccept");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn block_reject(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockRejectRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/BlockReject");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn state_summary_accept(
            &mut self,
            request: impl tonic::IntoRequest<super::StateSummaryAcceptRequest>,
        ) -> Result<tonic::Response<super::StateSummaryAcceptResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/vm.VM/StateSummaryAccept");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod vm_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with VmServer.
    #[async_trait]
    pub trait Vm: Send + Sync + 'static {
        async fn initialize(
            &self,
            request: tonic::Request<super::InitializeRequest>,
        ) -> Result<tonic::Response<super::InitializeResponse>, tonic::Status>;
        async fn set_state(
            &self,
            request: tonic::Request<super::SetStateRequest>,
        ) -> Result<tonic::Response<super::SetStateResponse>, tonic::Status>;
        async fn shutdown(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn create_handlers(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::CreateHandlersResponse>, tonic::Status>;
        async fn create_static_handlers(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::CreateStaticHandlersResponse>, tonic::Status>;
        async fn connected(
            &self,
            request: tonic::Request<super::ConnectedRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn disconnected(
            &self,
            request: tonic::Request<super::DisconnectedRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn build_block(
            &self,
            request: tonic::Request<super::BuildBlockRequest>,
        ) -> Result<tonic::Response<super::BuildBlockResponse>, tonic::Status>;
        async fn parse_block(
            &self,
            request: tonic::Request<super::ParseBlockRequest>,
        ) -> Result<tonic::Response<super::ParseBlockResponse>, tonic::Status>;
        async fn get_block(
            &self,
            request: tonic::Request<super::GetBlockRequest>,
        ) -> Result<tonic::Response<super::GetBlockResponse>, tonic::Status>;
        async fn set_preference(
            &self,
            request: tonic::Request<super::SetPreferenceRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn health(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::HealthResponse>, tonic::Status>;
        async fn version(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::VersionResponse>, tonic::Status>;
        async fn app_request(
            &self,
            request: tonic::Request<super::AppRequestMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn app_request_failed(
            &self,
            request: tonic::Request<super::AppRequestFailedMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn app_response(
            &self,
            request: tonic::Request<super::AppResponseMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn app_gossip(
            &self,
            request: tonic::Request<super::AppGossipMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn gather(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::GatherResponse>, tonic::Status>;
        async fn cross_chain_app_request(
            &self,
            request: tonic::Request<super::CrossChainAppRequestMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn cross_chain_app_request_failed(
            &self,
            request: tonic::Request<super::CrossChainAppRequestFailedMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn cross_chain_app_response(
            &self,
            request: tonic::Request<super::CrossChainAppResponseMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn get_ancestors(
            &self,
            request: tonic::Request<super::GetAncestorsRequest>,
        ) -> Result<tonic::Response<super::GetAncestorsResponse>, tonic::Status>;
        async fn batched_parse_block(
            &self,
            request: tonic::Request<super::BatchedParseBlockRequest>,
        ) -> Result<tonic::Response<super::BatchedParseBlockResponse>, tonic::Status>;
        async fn verify_height_index(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::VerifyHeightIndexResponse>, tonic::Status>;
        async fn get_block_id_at_height(
            &self,
            request: tonic::Request<super::GetBlockIdAtHeightRequest>,
        ) -> Result<tonic::Response<super::GetBlockIdAtHeightResponse>, tonic::Status>;
        async fn state_sync_enabled(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::StateSyncEnabledResponse>, tonic::Status>;
        async fn get_ongoing_sync_state_summary(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<
            tonic::Response<super::GetOngoingSyncStateSummaryResponse>,
            tonic::Status,
        >;
        async fn get_last_state_summary(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<super::GetLastStateSummaryResponse>, tonic::Status>;
        async fn parse_state_summary(
            &self,
            request: tonic::Request<super::ParseStateSummaryRequest>,
        ) -> Result<tonic::Response<super::ParseStateSummaryResponse>, tonic::Status>;
        async fn get_state_summary(
            &self,
            request: tonic::Request<super::GetStateSummaryRequest>,
        ) -> Result<tonic::Response<super::GetStateSummaryResponse>, tonic::Status>;
        async fn block_verify(
            &self,
            request: tonic::Request<super::BlockVerifyRequest>,
        ) -> Result<tonic::Response<super::BlockVerifyResponse>, tonic::Status>;
        async fn block_accept(
            &self,
            request: tonic::Request<super::BlockAcceptRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn block_reject(
            &self,
            request: tonic::Request<super::BlockRejectRequest>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn state_summary_accept(
            &self,
            request: tonic::Request<super::StateSummaryAcceptRequest>,
        ) -> Result<tonic::Response<super::StateSummaryAcceptResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct VmServer<T: Vm> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Vm> VmServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for VmServer<T>
    where
        T: Vm,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/vm.VM/Initialize" => {
                    #[allow(non_camel_case_types)]
                    struct InitializeSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::InitializeRequest>
                    for InitializeSvc<T> {
                        type Response = super::InitializeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InitializeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).initialize(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InitializeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/SetState" => {
                    #[allow(non_camel_case_types)]
                    struct SetStateSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::SetStateRequest>
                    for SetStateSvc<T> {
                        type Response = super::SetStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetStateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).set_state(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SetStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/Shutdown" => {
                    #[allow(non_camel_case_types)]
                    struct ShutdownSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for ShutdownSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).shutdown(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ShutdownSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/CreateHandlers" => {
                    #[allow(non_camel_case_types)]
                    struct CreateHandlersSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for CreateHandlersSvc<T> {
                        type Response = super::CreateHandlersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_handlers(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateHandlersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/CreateStaticHandlers" => {
                    #[allow(non_camel_case_types)]
                    struct CreateStaticHandlersSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for CreateStaticHandlersSvc<T> {
                        type Response = super::CreateStaticHandlersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_static_handlers(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateStaticHandlersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/Connected" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectedSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::ConnectedRequest>
                    for ConnectedSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ConnectedRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).connected(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/Disconnected" => {
                    #[allow(non_camel_case_types)]
                    struct DisconnectedSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::DisconnectedRequest>
                    for DisconnectedSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DisconnectedRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).disconnected(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DisconnectedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/BuildBlock" => {
                    #[allow(non_camel_case_types)]
                    struct BuildBlockSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::BuildBlockRequest>
                    for BuildBlockSvc<T> {
                        type Response = super::BuildBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BuildBlockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).build_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BuildBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/ParseBlock" => {
                    #[allow(non_camel_case_types)]
                    struct ParseBlockSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::ParseBlockRequest>
                    for ParseBlockSvc<T> {
                        type Response = super::ParseBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ParseBlockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).parse_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ParseBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/GetBlock" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::GetBlockRequest>
                    for GetBlockSvc<T> {
                        type Response = super::GetBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/SetPreference" => {
                    #[allow(non_camel_case_types)]
                    struct SetPreferenceSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::SetPreferenceRequest>
                    for SetPreferenceSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetPreferenceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).set_preference(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SetPreferenceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/Health" => {
                    #[allow(non_camel_case_types)]
                    struct HealthSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for HealthSvc<T> {
                        type Response = super::HealthResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).health(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HealthSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/Version" => {
                    #[allow(non_camel_case_types)]
                    struct VersionSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for VersionSvc<T> {
                        type Response = super::VersionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).version(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VersionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/AppRequest" => {
                    #[allow(non_camel_case_types)]
                    struct AppRequestSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::AppRequestMsg>
                    for AppRequestSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppRequestMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).app_request(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppRequestSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/AppRequestFailed" => {
                    #[allow(non_camel_case_types)]
                    struct AppRequestFailedSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::AppRequestFailedMsg>
                    for AppRequestFailedSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppRequestFailedMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).app_request_failed(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppRequestFailedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/AppResponse" => {
                    #[allow(non_camel_case_types)]
                    struct AppResponseSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::AppResponseMsg>
                    for AppResponseSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppResponseMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).app_response(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppResponseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/AppGossip" => {
                    #[allow(non_camel_case_types)]
                    struct AppGossipSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::AppGossipMsg>
                    for AppGossipSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppGossipMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).app_gossip(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppGossipSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/Gather" => {
                    #[allow(non_camel_case_types)]
                    struct GatherSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for GatherSvc<T> {
                        type Response = super::GatherResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).gather(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GatherSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/CrossChainAppRequest" => {
                    #[allow(non_camel_case_types)]
                    struct CrossChainAppRequestSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::CrossChainAppRequestMsg>
                    for CrossChainAppRequestSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CrossChainAppRequestMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).cross_chain_app_request(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CrossChainAppRequestSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/CrossChainAppRequestFailed" => {
                    #[allow(non_camel_case_types)]
                    struct CrossChainAppRequestFailedSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::CrossChainAppRequestFailedMsg>
                    for CrossChainAppRequestFailedSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CrossChainAppRequestFailedMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).cross_chain_app_request_failed(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CrossChainAppRequestFailedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/CrossChainAppResponse" => {
                    #[allow(non_camel_case_types)]
                    struct CrossChainAppResponseSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::CrossChainAppResponseMsg>
                    for CrossChainAppResponseSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CrossChainAppResponseMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).cross_chain_app_response(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CrossChainAppResponseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/GetAncestors" => {
                    #[allow(non_camel_case_types)]
                    struct GetAncestorsSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::GetAncestorsRequest>
                    for GetAncestorsSvc<T> {
                        type Response = super::GetAncestorsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetAncestorsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_ancestors(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAncestorsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/BatchedParseBlock" => {
                    #[allow(non_camel_case_types)]
                    struct BatchedParseBlockSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::BatchedParseBlockRequest>
                    for BatchedParseBlockSvc<T> {
                        type Response = super::BatchedParseBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchedParseBlockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).batched_parse_block(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BatchedParseBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/VerifyHeightIndex" => {
                    #[allow(non_camel_case_types)]
                    struct VerifyHeightIndexSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for VerifyHeightIndexSvc<T> {
                        type Response = super::VerifyHeightIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).verify_height_index(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VerifyHeightIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/GetBlockIDAtHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockIDAtHeightSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::GetBlockIdAtHeightRequest>
                    for GetBlockIDAtHeightSvc<T> {
                        type Response = super::GetBlockIdAtHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockIdAtHeightRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_block_id_at_height(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockIDAtHeightSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/StateSyncEnabled" => {
                    #[allow(non_camel_case_types)]
                    struct StateSyncEnabledSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for StateSyncEnabledSvc<T> {
                        type Response = super::StateSyncEnabledResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).state_sync_enabled(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StateSyncEnabledSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/GetOngoingSyncStateSummary" => {
                    #[allow(non_camel_case_types)]
                    struct GetOngoingSyncStateSummarySvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for GetOngoingSyncStateSummarySvc<T> {
                        type Response = super::GetOngoingSyncStateSummaryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_ongoing_sync_state_summary(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetOngoingSyncStateSummarySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/GetLastStateSummary" => {
                    #[allow(non_camel_case_types)]
                    struct GetLastStateSummarySvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::super::google::protobuf::Empty>
                    for GetLastStateSummarySvc<T> {
                        type Response = super::GetLastStateSummaryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_last_state_summary(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetLastStateSummarySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/ParseStateSummary" => {
                    #[allow(non_camel_case_types)]
                    struct ParseStateSummarySvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::ParseStateSummaryRequest>
                    for ParseStateSummarySvc<T> {
                        type Response = super::ParseStateSummaryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ParseStateSummaryRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).parse_state_summary(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ParseStateSummarySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/GetStateSummary" => {
                    #[allow(non_camel_case_types)]
                    struct GetStateSummarySvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::GetStateSummaryRequest>
                    for GetStateSummarySvc<T> {
                        type Response = super::GetStateSummaryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetStateSummaryRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_state_summary(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetStateSummarySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/BlockVerify" => {
                    #[allow(non_camel_case_types)]
                    struct BlockVerifySvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::BlockVerifyRequest>
                    for BlockVerifySvc<T> {
                        type Response = super::BlockVerifyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BlockVerifyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).block_verify(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BlockVerifySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/BlockAccept" => {
                    #[allow(non_camel_case_types)]
                    struct BlockAcceptSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::BlockAcceptRequest>
                    for BlockAcceptSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BlockAcceptRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).block_accept(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BlockAcceptSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/BlockReject" => {
                    #[allow(non_camel_case_types)]
                    struct BlockRejectSvc<T: Vm>(pub Arc<T>);
                    impl<T: Vm> tonic::server::UnaryService<super::BlockRejectRequest>
                    for BlockRejectSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BlockRejectRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).block_reject(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BlockRejectSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/vm.VM/StateSummaryAccept" => {
                    #[allow(non_camel_case_types)]
                    struct StateSummaryAcceptSvc<T: Vm>(pub Arc<T>);
                    impl<
                        T: Vm,
                    > tonic::server::UnaryService<super::StateSummaryAcceptRequest>
                    for StateSummaryAcceptSvc<T> {
                        type Response = super::StateSummaryAcceptResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StateSummaryAcceptRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).state_summary_accept(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StateSummaryAcceptSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Vm> Clone for VmServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Vm> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Vm> tonic::server::NamedService for VmServer<T> {
        const NAME: &'static str = "vm.VM";
    }
}
