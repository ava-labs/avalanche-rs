use std::env::args;

/// m/44'/9000'/0'/0/n where n is the address index
/// (P-chain use this as an external address!)
///
/// cargo run --example key_secp256k1_mnemonic_derive_load "m/44'/9000'/0'/0/0" "vehicle arrive more spread busy regret onion fame argue nice grocery humble vocal slot quit toss learn artwork theory fault tip belt cloth disorder"
/// cargo run --example key_secp256k1_mnemonic_derive_load "m/44'/9000'/0'/0/1" "vehicle arrive more spread busy regret onion fame argue nice grocery humble vocal slot quit toss learn artwork theory fault tip belt cloth disorder"
fn main() {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let deriv_path = args().nth(1).expect("no phrase given");
    let phrase = args().nth(2).expect("no phrase given");
    let key =
        avalanche_types::key::secp256k1::private_key::Key::from_mnemonic_phrase(phrase, deriv_path)
            .unwrap();

    let entry = key.to_info(1).expect("failed to_info");
    log::info!("network ID 1:\n{}", entry);

    let entry = key.to_info(9999).expect("failed to_info");
    log::info!("network ID 9999:\n{}", entry);
}
