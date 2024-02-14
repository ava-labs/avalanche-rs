use std::net::SocketAddr;
use std::vec;
use std::{
    io::{self, Error, ErrorKind, Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use avalanche_types::ids::node;

use log::info;
use pem::Pem;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use rustls::pki_types::{CertificateDer, UnixTime};
use rustls::{pki_types::ServerName, ClientConfig, ClientConnection};
type Certificate = rustls::pki_types::CertificateDer<'static>;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/network/peer#Start>
#[derive(std::clone::Clone)]
pub struct Connector {
    /// The client configuration of the local/source node for outbound TLS connections.
    pub client_config: Arc<ClientConfig>,
}

impl Connector {
    /// Creates a new dialer loading the PEM-encoded key and certificate pair of the local node.
    pub fn new_from_pem<S>(key_path: S, cert_path: S) -> io::Result<Self>
    where
        S: AsRef<str>,
    {
        let (private_key, certificate) =
            cert_manager::x509::load_pem_key_cert_to_der(key_path.as_ref(), cert_path.as_ref())?;

        // NOTE: AvalancheGo/* uses TLS key pair for exchanging node IDs without hostname authentication.
        // Thus, ok to skip CA verification, to be consistent with Go tls.Config.InsecureSkipVerify.
        // ref. <https://github.com/ava-labs/avalanchego/blob/master/network/peer/tls_config.go>
        // ref. <https://docs.rs/rustls/latest/rustls/struct.ConfigBuilder.html#method.with_client_auth_cert>
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
            .with_client_auth_cert(vec![certificate], private_key)
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to create TLS client config '{}'", e),
                )
            })?;

        Ok(Self {
            client_config: Arc::new(config),
        })
    }

    /// Creates a connection to the specified peer's IP and port.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/network/peer#NewTLSClientUpgrader>
    pub fn connect(&self, addr: SocketAddr, _timeout: Duration) -> io::Result<Stream> {
        info!("[rustls] connecting to {addr}");

        let server_name = ServerName::IpAddress(addr.ip().into());
        let mut conn =
            rustls::ClientConnection::new(self.client_config.clone(), server_name).unwrap();

        {
            let mut sock = TcpStream::connect(addr).unwrap();
            let mut tls = rustls::Stream::new(&mut conn, &mut sock);

            let binding = format!("GET / HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n");

            // This is a dummy write to ensure that the certificate data is transmitted.
            // Without this GET we get an error: Error: Custom { kind: NotConnected, error: "no peer certificate found" }
            tls.write_all(binding.as_bytes())?;
            info!("\n\n WROTE REQUEST\n\n");
        }

        info!("retrieving peer certificates...");

        // The certificate details are used to establish node identity.
        // See https://docs.avax.network/specs/cryptographic-primitives#tls-certificates.
        // The avalanchego certs are intentionally NOT signed by a legitimate CA.
        let (peer_certificate, total_certificates) = {
            let certs = conn.peer_certificates();

            match certs {
                Some(certs) if !certs.is_empty() => {
                    let total_certs = certs.len();
                    (certs[0].clone().into_owned(), total_certs)
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::NotConnected,
                        "no peer certificate found",
                    ))
                }
            }
        };

        let peer_node_id = node::Id::from_cert_der_bytes(&peer_certificate)?;
        info!(
            "successfully connected to {} (total {} certificates, first cert {}-byte)",
            peer_node_id,
            total_certificates,
            peer_certificate.len(),
        );

        #[cfg(feature = "pem_encoding")]
        let peer_certificate_pem = pem::encode(&Pem::new(
            "CERTIFICATE".to_string(),
            peer_certificate.as_ref(),
        ));

        Ok(Stream {
            addr,
            peer_certificate,
            peer_node_id,

            #[cfg(feature = "pem_encoding")]
            peer_certificate_pem,

            conn,
        })
    }
}

#[derive(Debug)]
pub struct NoCertificateVerification {}

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}

/// RUST_LOG=debug cargo test --package network --lib -- peer::outbound::test_connector --exact --show-output
#[test]
fn test_connector() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let key_path = random_manager::tmp_path(10, None).unwrap();
    let cert_path = random_manager::tmp_path(10, None).unwrap();
    cert_manager::x509::generate_and_write_pem(None, &key_path, &cert_path).unwrap();

    let _connector = Connector::new_from_pem(&key_path, &cert_path).unwrap();
}

/// Represents a connection to a peer.
/// ref. <https://github.com/rustls/rustls/commit/b8024301747fb0328c9493d7cf7268e0de17ffb3>
pub struct Stream {
    pub addr: SocketAddr,

    /// ref. <https://docs.rs/rustls/latest/rustls/enum.Connection.html>
    /// ref. <https://docs.rs/rustls/latest/rustls/client/struct.ClientConnection.html>
    pub conn: ClientConnection,

    pub peer_certificate: Certificate,
    pub peer_node_id: node::Id,

    #[cfg(feature = "pem")]
    pub peer_certificate_pem: String,
}

impl Stream {
    pub fn close(&mut self) -> io::Result<()> {
        self.conn.send_close_notify();
        Ok(())
    }

    /// Writes to the connection.
    pub fn write<S>(&mut self, d: S) -> io::Result<usize>
    where
        S: AsRef<[u8]>,
    {
        let mut wr = self.conn.writer();
        wr.write(d.as_ref())
    }

    /// Reads from the connection.
    pub fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut rd = self.conn.reader();
        let mut d = Vec::new();
        let _ = rd.read_to_end(&mut d)?;
        Ok(d)
    }
}
