use std::borrow::Cow;
#[cfg(any(test, feature = "proto"))]
use std::io::{self, Error, ErrorKind};

use avalanche_types::{
    proto::{
        http::Element,
        pb::{
            self,
            http::http_server::{Http, HttpServer},
            rpcdb::database_server::{Database, DatabaseServer},
        },
    },
    subnet::rpc::{http::handle::Handle, utils::grpc::default_server},
};
use bytes::Bytes;
use jsonrpsee_core::server::RpcModule;
use jsonrpsee_types::{Id, Request, Response, ResponsePayload, TwoPointZero};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

pub async fn serve_test_database<D>(database: D, listener: TcpListener) -> io::Result<()>
where
    D: Database,
{
    default_server()
        .add_service(DatabaseServer::new(database))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to serve test database service: {}", e),
            )
        })
}

pub async fn serve_test_http_server<H: Http + 'static>(
    http: H,
    listener: TcpListener,
) -> std::io::Result<()>
where
    H: pb::http::http_server::Http,
{
    default_server()
        .add_service(HttpServer::new(http))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to serve test http service: {}", e),
            )
        })
}

pub fn generate_http_request(
    method_name: &str,
    address: &str,
    param: &[&str],
) -> http::Request<Vec<u8>> {
    let raw_params = serde_json::value::to_raw_value(param).unwrap();

    let m = Request {
        jsonrpc: TwoPointZero,
        method: Cow::Borrowed(method_name).into(),
        params: Some(&*raw_params),
        id: Id::Number(1),
    };

    let body = serde_json::to_vec(&m).unwrap();

    http::Request::builder()
        .method("POST")
        .uri(address)
        .header("Content-type", "application/json")
        .header("Content-length", body.len().to_string().as_str())
        .body(body)
        .unwrap()
}

#[derive(Clone)]
pub struct TestHandler {
    pub module: RpcModule<()>,
}

impl TestHandler {
    pub fn new() -> Self {
        let mut module = RpcModule::new(());

        module
            .register_blocking_method("foo", |_, _| {
                serde_json::Value::String("Hello, from foo".to_string())
            })
            .unwrap();

        module
            .register_blocking_method("bar", |params, _| {
                let params: Option<[[String; 1]; 1]> = params.parse().unwrap();

                serde_json::Value::String(format!("Hello, {}, from bar", params.unwrap()[0][0]))
            })
            .unwrap();

        Self { module }
    }
}

#[tonic::async_trait]
impl Handle for TestHandler {
    async fn request(
        &self,
        req: &Bytes,
        _headers: &[Element],
    ) -> std::io::Result<(Bytes, Vec<Element>)> {
        let request: Request = serde_json::from_slice(req).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to deserialize request: {e}"),
            )
        })?;

        // module.call has a trait bound of ToRpcParams for params
        // The trait is not implemented for `T: Serialize`, but is for the tuple `(T0,): Serialize`
        // This means we have to wrap request.params as a tuple (which serde will also turn into an array)
        match self
            .module
            .call::<_, Value>(&request.method, (request.params,))
            .await
        {
            Ok(resp) => {
                let owned = Cow::<'static, Value>::Owned(resp);
                let payload = ResponsePayload::Result(owned);
                let resp = Response::new(payload, request.id);

                Ok((
                    Bytes::from(serde_json::to_string(&resp).unwrap()),
                    Vec::new(),
                ))
            }
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }
}
