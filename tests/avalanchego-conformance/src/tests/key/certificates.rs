use avalanche_types::ids::node;
use avalanchego_conformance_sdk::{CertificateToNodeIdRequest, Client};

#[tokio::test]
async fn generate_certificate_to_node_id() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let (key, cert) = cert_manager::x509::generate_der(None).expect("failed generate_der");
    log::debug!("key: {} bytes", key.0.len());
    log::debug!("cert: {} bytes", cert.0.len());
    let node_id = node::Id::from_cert_der_bytes(&cert.0).expect("failed from_cert_der_bytes");

    log::info!("sending node id {}", node_id);

    let resp = cli
        .certificate_to_node_id(CertificateToNodeIdRequest {
            certificate: cert.0.to_vec(),
            node_id: node_id.as_ref().to_vec(),
        })
        .await
        .expect("failed certificate_to_node_id");
    assert!(resp.success);
}

#[tokio::test]
async fn load_certificate_to_node_id() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let cert_paths = vec![
        "../../crates/avalanche-types/artifacts/staker1.insecure.crt",
        "../../crates/avalanche-types/artifacts/staker2.insecure.crt",
        "../../crates/avalanche-types/artifacts/staker3.insecure.crt",
        "../../crates/avalanche-types/artifacts/staker4.insecure.crt",
        "../../crates/avalanche-types/artifacts/staker5.insecure.crt",
        "../../crates/avalanche-types/artifacts/test.insecure.crt",
    ];
    for (i, cert_path) in cert_paths.iter().enumerate() {
        log::debug!("[{}] loading certs", i);

        let cert = cert_manager::x509::load_pem_cert_to_der(*cert_path)
            .expect("failed load_pem_cert_to_der");
        log::debug!("cert: {} bytes", cert.0.len());

        let node_id = node::Id::from_cert_pem_file(*cert_path).expect("failed from_cert_pem_file");
        log::debug!("node id: {}", node_id);

        let resp = cli
            .certificate_to_node_id(CertificateToNodeIdRequest {
                certificate: cert.0.to_vec(),
                node_id: node_id.as_ref().to_vec(),
            })
            .await
            .expect("failed certificate_to_node_id");
        assert!(resp.success);
    }
}
