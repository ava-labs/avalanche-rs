use std::sync::Arc;

use crate::proto::pb::{
    self,
    google::protobuf::Empty,
    http::{HandleSimpleHttpRequest, HandleSimpleHttpResponse, HttpRequest},
};

use tonic::codegen::http;

use super::handle::Handle;

#[derive(Clone)]
pub struct Server<T> {
    /// handler generated from create_handlers
    handle: Arc<T>,
}

impl<T> Server<T>
where
    T: Handle + Send + Sync + 'static,
{
    pub fn new(handler: T) -> impl pb::http::http_server::Http {
        Server {
            handle: Arc::new(handler),
        }
    }
}

#[tonic::async_trait]
impl<T> pb::http::http_server::Http for Server<T>
where
    T: Handle + Send + Sync + 'static,
{
    /// handles http requests including websockets
    async fn handle(
        &self,
        _request: tonic::Request<HttpRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        Err(tonic::Status::unimplemented("handle"))
    }

    /// handles http simple (non web-socket) requests
    async fn handle_simple(
        &self,
        request: tonic::Request<HandleSimpleHttpRequest>,
    ) -> Result<tonic::Response<HandleSimpleHttpResponse>, tonic::Status> {
        let request = request.into_inner();

        let (body, headers) = self.handle.request(&request.body, &request.headers).await?;

        Ok(tonic::Response::new(HandleSimpleHttpResponse {
            code: http::StatusCode::OK.as_u16() as i32,
            body,
            headers,
        }))
    }
}
