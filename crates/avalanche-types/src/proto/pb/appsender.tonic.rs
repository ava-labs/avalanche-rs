// @generated
/// Generated client implementations.
pub mod app_sender_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct AppSenderClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AppSenderClient<tonic::transport::Channel> {
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
    impl<T> AppSenderClient<T>
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
        ) -> AppSenderClient<InterceptedService<T, F>>
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
            AppSenderClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn send_app_request(
            &mut self,
            request: impl tonic::IntoRequest<super::SendAppRequestMsg>,
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
                "/appsender.AppSender/SendAppRequest",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn send_app_response(
            &mut self,
            request: impl tonic::IntoRequest<super::SendAppResponseMsg>,
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
                "/appsender.AppSender/SendAppResponse",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn send_app_gossip(
            &mut self,
            request: impl tonic::IntoRequest<super::SendAppGossipMsg>,
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
                "/appsender.AppSender/SendAppGossip",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn send_app_gossip_specific(
            &mut self,
            request: impl tonic::IntoRequest<super::SendAppGossipSpecificMsg>,
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
                "/appsender.AppSender/SendAppGossipSpecific",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn send_cross_chain_app_request(
            &mut self,
            request: impl tonic::IntoRequest<super::SendCrossChainAppRequestMsg>,
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
                "/appsender.AppSender/SendCrossChainAppRequest",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn send_cross_chain_app_response(
            &mut self,
            request: impl tonic::IntoRequest<super::SendCrossChainAppResponseMsg>,
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
                "/appsender.AppSender/SendCrossChainAppResponse",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod app_sender_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AppSenderServer.
    #[async_trait]
    pub trait AppSender: Send + Sync + 'static {
        async fn send_app_request(
            &self,
            request: tonic::Request<super::SendAppRequestMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn send_app_response(
            &self,
            request: tonic::Request<super::SendAppResponseMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn send_app_gossip(
            &self,
            request: tonic::Request<super::SendAppGossipMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn send_app_gossip_specific(
            &self,
            request: tonic::Request<super::SendAppGossipSpecificMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn send_cross_chain_app_request(
            &self,
            request: tonic::Request<super::SendCrossChainAppRequestMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
        async fn send_cross_chain_app_response(
            &self,
            request: tonic::Request<super::SendCrossChainAppResponseMsg>,
        ) -> Result<
            tonic::Response<super::super::google::protobuf::Empty>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct AppSenderServer<T: AppSender> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: AppSender> AppSenderServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AppSenderServer<T>
    where
        T: AppSender,
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
                "/appsender.AppSender/SendAppRequest" => {
                    #[allow(non_camel_case_types)]
                    struct SendAppRequestSvc<T: AppSender>(pub Arc<T>);
                    impl<
                        T: AppSender,
                    > tonic::server::UnaryService<super::SendAppRequestMsg>
                    for SendAppRequestSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendAppRequestMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_app_request(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendAppRequestSvc(inner);
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
                "/appsender.AppSender/SendAppResponse" => {
                    #[allow(non_camel_case_types)]
                    struct SendAppResponseSvc<T: AppSender>(pub Arc<T>);
                    impl<
                        T: AppSender,
                    > tonic::server::UnaryService<super::SendAppResponseMsg>
                    for SendAppResponseSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendAppResponseMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_app_response(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendAppResponseSvc(inner);
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
                "/appsender.AppSender/SendAppGossip" => {
                    #[allow(non_camel_case_types)]
                    struct SendAppGossipSvc<T: AppSender>(pub Arc<T>);
                    impl<
                        T: AppSender,
                    > tonic::server::UnaryService<super::SendAppGossipMsg>
                    for SendAppGossipSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendAppGossipMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_app_gossip(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendAppGossipSvc(inner);
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
                "/appsender.AppSender/SendAppGossipSpecific" => {
                    #[allow(non_camel_case_types)]
                    struct SendAppGossipSpecificSvc<T: AppSender>(pub Arc<T>);
                    impl<
                        T: AppSender,
                    > tonic::server::UnaryService<super::SendAppGossipSpecificMsg>
                    for SendAppGossipSpecificSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendAppGossipSpecificMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_app_gossip_specific(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendAppGossipSpecificSvc(inner);
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
                "/appsender.AppSender/SendCrossChainAppRequest" => {
                    #[allow(non_camel_case_types)]
                    struct SendCrossChainAppRequestSvc<T: AppSender>(pub Arc<T>);
                    impl<
                        T: AppSender,
                    > tonic::server::UnaryService<super::SendCrossChainAppRequestMsg>
                    for SendCrossChainAppRequestSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendCrossChainAppRequestMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_cross_chain_app_request(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendCrossChainAppRequestSvc(inner);
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
                "/appsender.AppSender/SendCrossChainAppResponse" => {
                    #[allow(non_camel_case_types)]
                    struct SendCrossChainAppResponseSvc<T: AppSender>(pub Arc<T>);
                    impl<
                        T: AppSender,
                    > tonic::server::UnaryService<super::SendCrossChainAppResponseMsg>
                    for SendCrossChainAppResponseSvc<T> {
                        type Response = super::super::google::protobuf::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendCrossChainAppResponseMsg>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_cross_chain_app_response(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendCrossChainAppResponseSvc(inner);
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
    impl<T: AppSender> Clone for AppSenderServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: AppSender> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: AppSender> tonic::server::NamedService for AppSenderServer<T> {
        const NAME: &'static str = "appsender.AppSender";
    }
}
