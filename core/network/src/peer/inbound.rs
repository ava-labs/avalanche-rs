use std::{
    io::{self, Error, ErrorKind},
    net::SocketAddr,
    sync::Arc,
};

use hyper::server::conn::AddrIncoming;
use rustls::server::NoClientAuth;
use tokio_rustls::rustls::ServerConfig;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/network#Network> "Dispatch"
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.7.11/network/network.go>
#[derive(std::clone::Clone)]
pub struct Listener {
    /// The server configuration of the local/source node for outbound TLS connections.
    pub server_config: Arc<ServerConfig>,
}

impl Listener {
    /// Creates a new dialer loading the PEM-encoded key and certificate pair of the local node.
    pub fn new_from_pem<S>(key_path: S, cert_path: S) -> io::Result<Self>
    where
        S: AsRef<str>,
    {
        log::info!("[rustls] loading raw PEM files for inbound listener");
        let (private_key, certificate) =
            cert_manager::x509::load_pem_key_cert_to_der(key_path.as_ref(), cert_path.as_ref())?;

        // ref. https://docs.rs/rustls/latest/rustls/struct.ConfigBuilder.html#method.with_single_cert
        // ref. https://github.com/rustls/hyper-rustls/blob/main/examples/server.rs
        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(NoClientAuth))
            .with_single_cert(vec![certificate], private_key)
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to create TLS server config '{}'", e),
                )
            })?;

        Ok(Self {
            server_config: Arc::new(server_config),
        })
    }

    /// Creates a listening stream for the specified IP and port.
    /// ref. <https://github.com/rustls/hyper-rustls/blob/main/examples/server.rs>
    pub fn listen(&self, addr: SocketAddr) -> io::Result<Stream> {
        log::info!("[rustls] listening on {addr}");

        let incoming = AddrIncoming::bind(&addr)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to bind '{}'", e)))?;
        let local_addr = incoming.local_addr();

        Ok(Stream {
            addr,
            local_addr,
            incoming,
        })
    }
}

/// RUST_LOG=debug cargo test --package network --lib -- peer::inbound::test_listener --exact --show-output
#[test]
fn test_listener() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let key_path = random_manager::tmp_path(10, None).unwrap();
    let cert_path = random_manager::tmp_path(10, None).unwrap();
    cert_manager::x509::generate_and_write_pem(None, &key_path, &cert_path).unwrap();

    let _listener = Listener::new_from_pem(&key_path, &cert_path).unwrap();
}

/// Represents a connection to a peer.
/// ref. <https://github.com/rustls/rustls/commit/b8024301747fb0328c9493d7cf7268e0de17ffb3>
pub struct Stream {
    pub addr: SocketAddr,
    pub local_addr: SocketAddr,

    pub incoming: AddrIncoming,
}
