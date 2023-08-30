use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    str,
};

use avalanche_types::{hash, key};
use avalanchego_conformance_sdk::{
    ChainAddresses, Client, Secp256k1Info, Secp256k1InfoRequest,
    Secp256k1RecoverHashPublicKeyRequest,
};

#[tokio::test]
async fn recover_hash_public_key() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let pk = key::secp256k1::private_key::Key::generate().expect("failed generate");
    let pubkey = pk.to_public_key();

    let unsigned_tx_bytes: Vec<u8> = random_manager::secure_bytes(500).unwrap();
    let tx_hash = hash::sha256(&unsigned_tx_bytes);

    let public_key_short_id_cb58 = pubkey.to_short_id().unwrap().to_string();
    log::info!(
        "testing public_key_short_id_cb58 {}",
        public_key_short_id_cb58
    );

    let sig = pk.sign_digest(&tx_hash).unwrap();
    let resp = cli
        .secp256k1_recover_hash_public_key(Secp256k1RecoverHashPublicKeyRequest {
            message: tx_hash.clone(),
            signature: Vec::from(sig.to_bytes()),
            public_key_short_id_cb58: public_key_short_id_cb58.clone(),
        })
        .await
        .expect("failed secp256k1_recover_hash_public_key");
    log::info!(
        "sign_digest secp256k1_recover_hash_public_key response: {} {}",
        resp.message,
        resp.success
    );
    assert!(resp.success);

    let pk_libsecp256k1 = pk.to_libsecp256k1().unwrap();
    let sig = pk_libsecp256k1.sign_digest(&tx_hash).unwrap();
    let resp = cli
        .secp256k1_recover_hash_public_key(Secp256k1RecoverHashPublicKeyRequest {
            message: tx_hash,
            signature: Vec::from(sig.to_bytes()),
            public_key_short_id_cb58,
        })
        .await
        .expect("failed secp256k1_recover_hash_public_key");
    log::info!(
        "to_libsecp256k1.sign_digest secp256k1_recover_hash_public_key response: {} {}",
        resp.message,
        resp.success
    );
    assert!(resp.success);
}

#[tokio::test]
async fn generate() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let k = avalanche_types::key::secp256k1::private_key::Key::generate().expect("failed generate");
    let ki = k.to_info(1).unwrap();
    let pubkey = k.to_public_key();

    let mut chain_addresses: HashMap<u32, ChainAddresses> = HashMap::new();
    chain_addresses.insert(
        1,
        ChainAddresses {
            x: pubkey.to_hrp_address(1, "X").unwrap(),
            p: pubkey.to_hrp_address(1, "P").unwrap(),
        },
    );
    let resp = cli
        .secp256k1_info(Secp256k1InfoRequest {
            secp256k1_info: Some(Secp256k1Info {
                key_type: String::from("hot"),
                private_key_cb58: ki.private_key_cb58.clone().unwrap(),
                private_key_hex: ki.private_key_hex.clone().unwrap(),
                chain_addresses,
                short_address: ki.short_address.to_string(),
                eth_address: ki.eth_address.clone(),
            }),
        })
        .await
        .expect("failed secp256k1_info");
    log::info!("secp256k1_info response: {} {}", resp.message, resp.success);
    assert!(resp.success);

    let mut chain_addresses: HashMap<u32, ChainAddresses> = HashMap::new();
    chain_addresses.insert(
        9999,
        ChainAddresses {
            x: pubkey.to_hrp_address(9999, "X").unwrap(),
            p: pubkey.to_hrp_address(9999, "P").unwrap(),
        },
    );
    let resp = cli
        .secp256k1_info(Secp256k1InfoRequest {
            secp256k1_info: Some(Secp256k1Info {
                key_type: String::from("hot"),
                private_key_cb58: ki.private_key_cb58.clone().unwrap(),
                private_key_hex: ki.private_key_hex.clone().unwrap(),
                chain_addresses,
                short_address: ki.short_address.to_string(),
                eth_address: ki.eth_address.clone(),
            }),
        })
        .await
        .expect("failed secp256k1_info");
    log::info!("secp256k1_info response: {} {}", resp.message, resp.success);
    assert!(resp.success);

    // parse from eth key
    let k = key::secp256k1::private_key::Key::from_hex(ki.private_key_hex.clone().unwrap())
        .expect("failed from_private_key_eth");
    let ki = k.to_info(9999).unwrap();
    let pubkey = k.to_public_key();

    let mut chain_addresses: HashMap<u32, ChainAddresses> = HashMap::new();
    chain_addresses.insert(
        9999,
        ChainAddresses {
            x: pubkey.to_hrp_address(9999, "X").unwrap(),
            p: pubkey.to_hrp_address(9999, "P").unwrap(),
        },
    );
    let resp = cli
        .secp256k1_info(Secp256k1InfoRequest {
            secp256k1_info: Some(Secp256k1Info {
                key_type: String::from("hot"),
                private_key_cb58: ki.private_key_cb58.clone().unwrap(),
                private_key_hex: ki.private_key_hex.clone().unwrap(),
                chain_addresses,
                short_address: ki.short_address.to_string(),
                eth_address: ki.eth_address.clone(),
            }),
        })
        .await
        .expect("failed secp256k1_info");
    log::info!("secp256k1_info response: {} {}", resp.message, resp.success);
    assert!(resp.success);
}

#[tokio::test]
async fn load() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let key_files =
        vec!["../../crates/avalanche-types/artifacts/test.insecure.secp256k1.key.infos.json"];
    for key_file in key_files.iter() {
        log::info!("reading key file {}", key_file);
        let d = read_vec(key_file).expect("failed to read key json file");
        let key_infos: Vec<key::secp256k1::Info> = serde_json::from_slice(&d).unwrap();

        for (i, ki) in key_infos.iter().enumerate() {
            log::info!("[{}] checking key infos from test key file", i);

            let addrs = ki.addresses.get(&1).unwrap();
            let mut chain_addresses: HashMap<u32, ChainAddresses> = HashMap::new();
            chain_addresses.insert(
                1,
                ChainAddresses {
                    x: addrs.x.clone(),
                    p: addrs.p.clone(),
                },
            );
            let resp = cli
                .secp256k1_info(Secp256k1InfoRequest {
                    secp256k1_info: Some(Secp256k1Info {
                        key_type: String::from("hot"),
                        private_key_cb58: ki.private_key_cb58.clone().unwrap(),
                        private_key_hex: ki.private_key_hex.clone().unwrap(),
                        chain_addresses,
                        short_address: ki.short_address.to_string(),
                        eth_address: ki.eth_address.clone(),
                    }),
                })
                .await
                .expect("failed secp256k1_info");
            assert!(resp.success);

            let addrs = ki.addresses.get(&9999).unwrap();
            let mut chain_addresses: HashMap<u32, ChainAddresses> = HashMap::new();
            chain_addresses.insert(
                9999,
                ChainAddresses {
                    x: addrs.x.clone(),
                    p: addrs.p.clone(),
                },
            );
            let resp = cli
                .secp256k1_info(Secp256k1InfoRequest {
                    secp256k1_info: Some(Secp256k1Info {
                        key_type: String::from("hot"),
                        private_key_cb58: ki.private_key_cb58.clone().unwrap(),
                        private_key_hex: ki.private_key_hex.clone().unwrap(),
                        chain_addresses,
                        short_address: ki.short_address.to_string(),
                        eth_address: ki.eth_address.clone(),
                    }),
                })
                .await
                .expect("failed secp256k1_info");
            log::info!("secp256k1_info response: {} {}", resp.message, resp.success);
            assert!(resp.success);

            // parse from eth key
            let k = key::secp256k1::private_key::Key::from_hex(ki.private_key_hex.clone().unwrap())
                .expect("failed from_private_key_eth");
            let ki = k.to_info(9999).unwrap();
            let pubkey = k.to_public_key();

            let mut chain_addresses: HashMap<u32, ChainAddresses> = HashMap::new();
            chain_addresses.insert(
                9999,
                ChainAddresses {
                    x: pubkey.to_hrp_address(9999, "X").unwrap(),
                    p: pubkey.to_hrp_address(9999, "P").unwrap(),
                },
            );
            let resp = cli
                .secp256k1_info(Secp256k1InfoRequest {
                    secp256k1_info: Some(Secp256k1Info {
                        key_type: String::from("hot"),
                        private_key_cb58: ki.private_key_cb58.clone().unwrap(),
                        private_key_hex: ki.private_key_hex.clone().unwrap(),
                        chain_addresses,
                        short_address: ki.short_address.to_string(),
                        eth_address: ki.eth_address.clone(),
                    }),
                })
                .await
                .expect("failed secp256k1_info");

            log::info!("secp256k1_info response: {} {}", resp.message, resp.success);
            assert!(resp.success);
        }
    }
}

/// ref. <https://doc.rust-lang.org/std/fs/fn.read.html>
fn read_vec(p: &str) -> io::Result<Vec<u8>> {
    let mut f = File::open(p)?;
    let metadata = fs::metadata(p)?;
    let mut buffer = vec![0; metadata.len() as usize];
    let _read_bytes = f.read(&mut buffer)?;
    Ok(buffer)
}
