use std::{
    convert::Infallible,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use futures::{TryFutureExt, TryStreamExt};
use http::{Method, Request, Response, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use log::{debug, info};
use tokio::signal;

#[derive(Debug)]
pub struct Handler {
    pub http_host: String,
    pub listener_port: u16,
    pub socket_addr: SocketAddr,
    pub request_timeout: Duration,
}

pub const DEFAULT_HTTP_HOST: &str = "0.0.0.0";
pub const DEFAULT_LISTENER_PORT: u16 = 9650;
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

impl Default for Handler {
    fn default() -> Self {
        Self {
            http_host: String::from(DEFAULT_HTTP_HOST),
            listener_port: DEFAULT_LISTENER_PORT,
            socket_addr: SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                DEFAULT_LISTENER_PORT,
            ),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        }
    }
}

impl Handler {
    pub fn new(http_host: &str, listener_port: u16, request_timeout: Duration) -> Self {
        let url = format!("{http_host}:{listener_port}");

        info!("parsing URL '{}' to socket address", url);
        let socket_addr: SocketAddr = url.parse().unwrap();
        info!(
            "handler with socket {:?} (request timeout {:?})",
            socket_addr, request_timeout,
        );

        Self {
            http_host: String::from(http_host),
            listener_port,
            socket_addr,
            request_timeout,
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        info!("starting server");

        let svc = make_service_fn(|socket: &AddrStream| {
            let remote_addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    handle_request(remote_addr, req).or_else(|(status, body)| async move {
                        println!("{}", body);
                        Ok::<_, Infallible>(
                            Response::builder()
                                .status(status)
                                .body(Body::from(body))
                                .unwrap(),
                        )
                    })
                }))
            }
        });
        let server = Server::try_bind(&self.socket_addr)?
            .serve(svc)
            .with_graceful_shutdown(handle_sigint());

        info!("listener start {}", self.socket_addr);
        server.await?;
        info!("listener done {}", self.socket_addr);

        Ok(())
    }
}

async fn handle_request(
    remote_addr: SocketAddr,
    req: Request<Body>,
) -> Result<Response<Body>, (http::StatusCode, String)> {
    let http_version = req.version();
    let method = req.method().clone();
    let uri_path = req.uri().path();
    debug!(
        "version {:?}, method {}, uri path {}, remote addr {}",
        http_version, method, uri_path, remote_addr,
    );

    let resp = match uri_path {
        "/ping" => match method {
            Method::GET => Response::new(Body::from("ping")),
            _ => Err((
                StatusCode::NOT_FOUND,
                format!("unknown method '{method}' for '{uri_path}'"),
            ))?,
        },

        "/ext/health" => match method {
            Method::GET => Response::new(Body::from("OK")),
            _ => Err((
                StatusCode::NOT_FOUND,
                format!("unknown method '{method}' for '{uri_path}'"),
            ))?,
        },

        "/ext/bc/X" => match method {
            Method::POST => {
                let body = req
                    .into_body()
                    .try_fold(Vec::new(), |mut data, chunk| async move {
                        data.extend_from_slice(&chunk);
                        Ok(data)
                    })
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("failed to read request body {}", e),
                        )
                    })?;
                debug!("read request body {}", body.len());
                Response::new(Body::from("OK"))
            }
            _ => Err((
                StatusCode::NOT_FOUND,
                format!("unknown method '{method}' for '{uri_path}'"),
            ))?,
        },

        "/ext/P" => match method {
            Method::POST => {
                let body = req
                    .into_body()
                    .try_fold(Vec::new(), |mut data, chunk| async move {
                        data.extend_from_slice(&chunk);
                        Ok(data)
                    })
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("failed to read request body {}", e),
                        )
                    })?;
                debug!("read request body {}", body.len());
                Response::new(Body::from("OK"))
            }
            _ => Err((
                StatusCode::NOT_FOUND,
                format!("unknown method '{method}' for '{uri_path}'"),
            ))?,
        },

        _ => Err((StatusCode::NOT_FOUND, format!("unknown path '{uri_path}'")))?,
    };

    Ok(resp)
}

async fn handle_sigint() {
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
