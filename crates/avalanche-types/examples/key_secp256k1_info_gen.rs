use std::{env::args, fs::File, io::Write};

/// cargo run --example key_secp256k1_info_gen --  1 1 9999 /tmp/key.json
/// cargo run --example key_secp256k1_info_gen -- 50 1 9999 /tmp/key.json
fn main() {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let keys = args().nth(1).expect("no network ID given");
    let keys = keys.parse::<usize>().unwrap();

    let network_id = args().nth(2).expect("no network ID given");
    let network_id1 = network_id.parse::<u32>().unwrap();

    let network_id = args().nth(3).expect("no network ID given");
    let network_id2 = network_id.parse::<u32>().unwrap();

    let file_path = args().nth(4).expect("no file path given");

    let mut infos = Vec::new();
    for i in 0..keys {
        let key = if i == 0 {
            avalanche_types::key::secp256k1::private_key::Key::from_cb58(
                "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN",
            )
            .expect("unexpected key generate failure")
        } else {
            avalanche_types::key::secp256k1::private_key::Key::generate()
                .expect("unexpected key generate failure")
        };

        let mut k1 = key.to_info(network_id1).expect("failed to_info");
        let k2 = key.to_info(network_id2).expect("failed to_info");

        k1.addresses
            .insert(network_id2, k2.addresses.get(&network_id2).unwrap().clone());
        infos.push(k1);
    }

    let d = serde_json::to_vec(&infos).unwrap();
    let mut f = File::create(&file_path).unwrap();
    f.write_all(&d).unwrap();
}
