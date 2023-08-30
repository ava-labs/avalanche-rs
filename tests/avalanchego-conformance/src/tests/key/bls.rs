use avalanche_types::key::bls;
use avalanchego_conformance_sdk::{BlsSignatureRequest, Client};

#[tokio::test]
async fn generate_bls_signature() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let msg = random_manager::secure_bytes(100).unwrap();

    let sk = bls::private_key::Key::generate().unwrap();
    log::debug!("key: {} bytes", sk.to_bytes().len());

    let sig = sk.sign(&msg);
    let sig_pop = sk.sign_proof_of_possession(&msg);

    let resp = cli
        .bls_signature(BlsSignatureRequest {
            private_key: sk.to_bytes().to_vec(),
            public_key: sk.to_public_key().to_compressed_bytes().to_vec(),
            message: msg,
            signature: sig.to_compressed_bytes().to_vec(),
            signature_proof_of_possession: sig_pop.to_compressed_bytes().to_vec(),
        })
        .await
        .expect("failed bls_signature");
    assert!(resp.success);
}
