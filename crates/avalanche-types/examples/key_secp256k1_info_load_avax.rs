use std::env::args;

/// cargo run --example key_secp256k1_info_load_avax -- PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN 1
/// cargo run --example key_secp256k1_info_load_avax -- PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN 9999
/// cargo run --example key_secp256k1_info_load_avax -- PrivateKey-2kqWNDaqUKQyE4ZsV5GLCGeizE6sHAJVyjnfjXoXrtcZpK9M67 1
/// cargo run --example key_secp256k1_info_load_avax -- PrivateKey-2kqWNDaqUKQyE4ZsV5GLCGeizE6sHAJVyjnfjXoXrtcZpK9M67 9999
fn main() {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let private_key = args().nth(1).expect("no private key given");

    let network_id = args().nth(2).expect("no network ID given");
    let network_id = network_id.parse::<u32>().unwrap();

    log::info!("loading key");
    let k = avalanche_types::key::secp256k1::private_key::Key::from_cb58(&private_key).unwrap();
    let pubkey = k.to_public_key();

    let entry = k.to_info(network_id).unwrap();
    assert_eq!(private_key, entry.private_key_cb58.clone().unwrap());
    assert_eq!(
        entry.addresses.get(&network_id).unwrap().x,
        pubkey.to_hrp_address(network_id, "X").unwrap()
    );
    assert_eq!(
        entry.addresses.get(&network_id).unwrap().p,
        pubkey.to_hrp_address(network_id, "P").unwrap()
    );
    assert_eq!(entry.short_address, pubkey.to_short_id().unwrap());
    assert_eq!(entry.eth_address, pubkey.to_eth_address());

    print!("{}", entry);
}
