use crate::proto::google::protobuf::Timestamp;

use std::{
    convert::Infallible,
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
    time::Duration,
};

use chrono::{DateTime, Utc};
use futures::FutureExt;
use http::{Request, Response};
use hyper::Body;
use tokio::sync::broadcast::Receiver;
use tonic::{
    body::BoxBody,
    server::NamedService,
    transport::{Channel, Endpoint},
};
use tower_service::Service;

/// gRPC Defaults
///
/// Sets the [`SETTINGS_MAX_CONCURRENT_STREAMS`][spec] option for HTTP2
/// connections.
///
/// Tonic default is no limit (`None`) which is the same as u32::MAX.
///
/// [spec]: https://http2.github.io/http2-spec/#SETTINGS_MAX_CONCURRENT_STREAMS
pub const DEFAULT_MAX_CONCURRENT_STREAMS: u32 = u32::MAX;

/// Sets a timeout for receiving an acknowledgement of the keepalive ping.
///
/// If the ping is not acknowledged within the timeout, the connection will be closed.
/// Does nothing if http2_keep_alive_interval is disabled.
///
/// Tonic default is 20 seconds.
pub const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(20);

/// Set whether HTTP2 Ping frames are enabled on accepted connections.
///
/// If `None` is specified, HTTP2 keepalive is disabled, otherwise the duration
/// specified will be the time interval between HTTP2 Ping frames.
/// The timeout for receiving an acknowledgement of the keepalive ping
/// can be set with \[`Server::http2_keepalive_timeout`\].
///
/// Tonic default is no HTTP2 keepalive (`None`)
/// Avalanche default is 2 hours.
pub const DEFAULT_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(2 * 60 * 60);

/// Set whether TCP keepalive messages are enabled on accepted connections.
///
/// If `None` is specified, keepalive is disabled, otherwise the duration
/// specified will be the time to remain idle before sending TCP keepalive
/// probes.
///
/// Default is no keepalive (`None`)
/// Avalanche default is 5 seconds.
pub const DEFAULT_KEEP_ALIVE_MIN_TIME: Duration = Duration::from_secs(5);

/// Creates a tonic gRPC server with avalanche defaults.
pub fn default_server() -> tonic::transport::Server {
    tonic::transport::Server::builder()
        .max_concurrent_streams(DEFAULT_MAX_CONCURRENT_STREAMS)
        .http2_keepalive_timeout(Some(DEFAULT_KEEP_ALIVE_TIMEOUT))
        .http2_keepalive_interval(Some(DEFAULT_KEEP_ALIVE_INTERVAL))
        .tcp_keepalive(Some(DEFAULT_KEEP_ALIVE_MIN_TIME))
}

/// Creates a tonic Endpoint with avalanche defaults. The endpoint input is
/// expected in `<ip>:<port>` format.
pub fn default_client(endpoint: &str) -> Result<Endpoint> {
    let endpoint = Channel::from_shared(format!("http://{endpoint}"))
        .map_err(|e| Error::new(ErrorKind::Other, format!("invalid endpoint: {e}")))?
        .keep_alive_timeout(DEFAULT_KEEP_ALIVE_TIMEOUT)
        .http2_keep_alive_interval(DEFAULT_KEEP_ALIVE_INTERVAL)
        .tcp_keepalive(Some(DEFAULT_KEEP_ALIVE_MIN_TIME));

    Ok(endpoint)
}

/// Server is a gRPC server lifecycle manager.
pub struct Server {
    /// Waits for the broadcasted stop signal to shutdown gRPC server.
    pub stop_ch: Receiver<()>,

    /// Server address.
    pub addr: SocketAddr,
}

impl Server {
    pub fn new(addr: SocketAddr, stop_ch: Receiver<()>) -> Self {
        Self { stop_ch, addr }
    }
}

// TODO: add support for multiple services.
impl Server {
    /// Attempts to start a gRPC server for the provided service which can be
    /// shutdown by a broadcast channel.
    pub fn serve<S>(mut self, svc: S) -> Result<()>
    where
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
    {
        tokio::spawn(async move {
            default_server()
                .add_service(svc)
                .serve_with_shutdown(self.addr, self.stop_ch.recv().map(|_| ()))
                .await
                .map_err(|e| Error::new(ErrorKind::Other, format!("grpc server failed: {:?}", e)))
        });
        log::info!("gRPC server started: {}", self.addr);

        Ok(())
    }
}

/// Converts DataTime to a google::protobuf::Timestamp
pub fn timestamp_from_time(dt: &DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
