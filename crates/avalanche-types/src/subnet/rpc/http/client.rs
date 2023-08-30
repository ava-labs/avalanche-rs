use std::io::{self, Error, ErrorKind};

use crate::{proto::pb, subnet};
use prost::bytes::Bytes;
use tonic::transport::Channel;

/// Client which interacts with gRPC HTTP service
pub struct Client {
    inner: pb::http::http_client::HttpClient<Channel>,
}

impl Client {
    pub fn new(client_conn: Channel) -> Box<dyn subnet::rpc::http::Handler + Send + Sync> {
        Box::new(Client {
            inner: pb::http::http_client::HttpClient::new(client_conn),
        })
    }
}

#[tonic::async_trait]
impl subnet::rpc::http::Handler for Client {
    async fn serve_http(
        &mut self,
        _req: http::Request<Vec<u8>>,
    ) -> io::Result<http::Response<Vec<u8>>> {
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    /// http client takes an http request and sends to server.  Does not support websockets.
    async fn serve_http_simple(
        &mut self,
        req: http::Request<Vec<u8>>,
    ) -> io::Result<http::Response<Vec<u8>>> {
        let req = get_http_simple_request(req)?;

        let resp = self.inner.handle_simple(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("handle simple request failed: {:?}", e),
            )
        })?;

        get_http_response(resp.into_inner())
    }
}

/// convert from [http::Request] to [pb::http::HandleSimpleHttpRequest]
fn get_http_simple_request(
    req: http::Request<Vec<u8>>,
) -> io::Result<pb::http::HandleSimpleHttpRequest> {
    let headers = convert_to_proto_headers(req.headers())?;

    Ok(pb::http::HandleSimpleHttpRequest {
        method: req.method().to_string(),
        url: req.uri().to_string(),
        body: Bytes::from(req.body().to_owned()),
        headers,
    })
}

/// convert from [pb::http::HandleSimpleHttpResponse] to [http::Response]
fn get_http_response(
    resp: pb::http::HandleSimpleHttpResponse,
) -> io::Result<http::Response<Vec<u8>>> {
    let mut http_resp = http::Response::builder().status(resp.code as u16);

    for header in resp.headers.into_iter() {
        http_resp = http_resp.header(header.key, header.values.concat());
    }

    let http_resp = http_resp
        .body(resp.body.to_vec())
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to generate http response {:?}", e),
            )
        })
        .unwrap();
    Ok(http_resp)
}

/// converts [http::HeaderMap] to a vec of elements that avalanche proto can use
fn convert_to_proto_headers(
    headers: &http::HeaderMap<http::HeaderValue>,
) -> io::Result<Vec<pb::http::Element>> {
    let mut vec_headers: Vec<pb::http::Element> = Vec::with_capacity(headers.keys_len());
    for (key, value) in headers.into_iter() {
        let element = pb::http::Element {
            key: key.to_string(),
            values: vec![String::from_utf8_lossy(value.as_bytes()).to_string()],
        };
        vec_headers.push(element);
    }
    Ok(vec_headers)
}
