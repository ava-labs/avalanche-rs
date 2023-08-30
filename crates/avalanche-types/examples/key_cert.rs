use std::{env::args, fs, path::Path};

use avalanche_types::ids::node;

/// cargo run --example key_cert -- /tmp/test.insecure.key /tmp/test.insecure.cert
fn main() {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let key_path = args().nth(1).expect("no key path given");
    let cert_path = args().nth(2).expect("no cert path given");

    if Path::new(&key_path).exists() {
        fs::remove_file(&key_path).expect("failed remove_file");
    }
    if Path::new(&cert_path).exists() {
        fs::remove_file(&cert_path).expect("failed remove_file");
    }

    cert_manager::x509::generate_and_write_pem(None, key_path.as_str(), cert_path.as_str())
        .expect("failed to generate certs");
    // openssl x509 -in /tmp/test.insecure.cert -text -noout
    // openssl x509 -in artifacts/staker1.insecure.crt -text -noout

    let node_id = node::Id::from_cert_pem_file(cert_path.as_str()).unwrap();
    println!("Node ID: {}", node_id);

    let (loaded_node_id, generated) =
        node::Id::load_or_generate_pem(&key_path, &cert_path).unwrap();
    assert!(!generated);
    assert_eq!(node_id, loaded_node_id);
}
