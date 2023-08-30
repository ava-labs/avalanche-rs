pub mod inbound;
pub mod outbound;

/// Represents a remote peer from the local node.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/network/peer#Start>
pub struct Peer {
    pub stream: outbound::Stream,

    pub ready: bool,
}

impl Peer {
    pub fn new(stream: outbound::Stream) -> Self {
        Self {
            stream,
            ready: false,
        }
    }
}

/// RUST_LOG=debug cargo test --package network --lib -- peer::test::test_listener --exact --show-output
///
/// TODO: make this test work. The client and server are both initialized correctly,
/// but making a connection fails.
/// Error is Os { code: 61, kind: ConnectionRefused, message: "Connection refused" } when connecting client to server.
#[cfg(test)]
mod test {
    use rcgen::CertificateParams;
    use rustls::ServerConfig;
    use std::{
        io::{self, Error, ErrorKind},
        net::{IpAddr, SocketAddr},
        str::FromStr,
        sync::Arc,
        time::Duration,
    };
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;

    use crate::peer::outbound;

    #[tokio::test]
    #[ignore]
    async fn test_listener() -> io::Result<()> {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            // .is_test(true)
            .try_init();

        let server_key_path = random_manager::tmp_path(10, None)?;
        let server_cert_path = random_manager::tmp_path(10, None)?;
        let server_cert_sna_params = CertificateParams::new(vec!["127.0.0.1".to_string()]);
        cert_manager::x509::generate_and_write_pem(
            Some(server_cert_sna_params),
            &server_key_path,
            &server_cert_path,
        )?;

        log::info!("[rustls] loading raw PEM files for inbound listener");
        let (private_key, certificate) = cert_manager::x509::load_pem_key_cert_to_der(
            server_key_path.as_ref(),
            server_cert_path.as_ref(),
        )?;

        let ip_addr = String::from("127.0.0.1");
        let ip_port = 9649_u16;

        let join_handle = tokio::task::spawn(async move {
            let server_config = ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], private_key)
                .map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        format!("failed to create TLS server config '{}'", e),
                    )
                })
                .unwrap();

            let ip = ip_addr.clone().parse::<std::net::IpAddr>().unwrap();
            let addr = SocketAddr::new(ip, ip_port);

            let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));
            let tcp_listener = TcpListener::bind(addr).await.unwrap();

            loop {
                let (stream, _) = tcp_listener.accept().await.unwrap();
                let tls_acceptor = tls_acceptor.clone();
                log::info!("accepting TLS connection");
                let _ = tokio::spawn(async move {
                    match tls_acceptor.accept(stream).await {
                        Ok(_tls_stream) => {
                            println!("TLS connection accepted");
                            // handle(tls_stream).await
                        }
                        Err(e) => eprintln!("Error accepting TLS connection: {:?}", e),
                    }
                })
                .await;
            }
        });

        let client_key_path = random_manager::tmp_path(10, None)?;
        let client_cert_path = random_manager::tmp_path(10, None)?;
        let client_cert_sna_params = CertificateParams::new(vec!["127.0.0.1".to_string()]);
        cert_manager::x509::generate_and_write_pem(
            Some(client_cert_sna_params),
            &client_key_path,
            &client_cert_path,
        )?;
        log::info!("client cert path: {}", client_cert_path);

        let connector = outbound::Connector::new_from_pem(&client_key_path, &client_cert_path)?;
        let stream = connector.connect(
            IpAddr::from_str("127.0.0.1").unwrap(),
            ip_port,
            Duration::from_secs(5),
        )?;

        log::info!("peer certificate:\n\n{}", stream.peer_certificate_pem);

        join_handle.await?; // Hangs

        Ok(())
    }
}

// Represents an attached "test" peer to a remote peer
// with a hollow inbound handler implementation.
// Only used for testing.
// ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/network/peer#Start
// ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/network/peer#StartTestPeer
