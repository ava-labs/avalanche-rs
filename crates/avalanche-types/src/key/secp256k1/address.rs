use std::io::{self, Error, ErrorKind};

use crate::hash;
use primitive_types::H160;

/// ref. <https://eips.ethereum.org/EIPS/eip-55>
/// ref. <https://pkg.go.dev/github.com/ethereum/go-ethereum/crypto#PubkeyToAddress>
/// ref. <https://pkg.go.dev/github.com/ethereum/go-ethereum/common#Address.Hex>
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/utils/mod.rs> "to_checksum"
pub fn h160_to_eth_address(h160_addr: &H160, chain_id: Option<u8>) -> String {
    let prefixed_addr = match chain_id {
        Some(chain_id) => format!("{chain_id}0x{h160_addr:x}"),
        None => format!("{h160_addr:x}"),
    };

    let hex_h256 = hex::encode(hash::keccak256(prefixed_addr));
    let hex_h256 = hex_h256.as_bytes();

    let hex_addr = hex::encode(h160_addr.as_bytes());
    let hex_addr = hex_addr.as_bytes();

    hex_addr
        .iter()
        .zip(hex_h256)
        .fold("0x".to_owned(), |mut s, (addr, hash)| {
            s.push(if *hash >= 56 {
                addr.to_ascii_uppercase() as char
            } else {
                addr.to_ascii_lowercase() as char
            });
            s
        })
}

/// Converts "bech32::encode"d AVAX address to the short address bytes (20-byte) and HRP for network name.
pub fn avax_address_to_short_bytes(chain_alias: &str, addr: &str) -> io::Result<(String, Vec<u8>)> {
    let trimmed = if chain_alias.is_empty() {
        addr.trim().to_string()
    } else {
        // e.g., "P-custom12szthht8tnl455u4mz3ns3nvvkel8ezvw2n8cx".trim_start_matches("P-")
        let pfx = if chain_alias.ends_with('-') {
            chain_alias.to_string()
        } else {
            format!("{}-", chain_alias)
        };
        addr.trim_start_matches(&pfx).to_string()
    };

    let (hrp, data, _) = bech32::decode(&trimmed)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed bech32::decode '{}'", e)))?;

    let convert = bech32::convert_bits(&data, 5, 8, false).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed bech32::convert_bits '{}'", e),
        )
    })?;
    Ok((hrp, convert))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::address::test_avax_address_to_short_bytes --exact --show-output
#[test]
fn test_avax_address_to_short_bytes() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let pk = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey = pk.to_public_key();
    let short_addr = pubkey.to_short_bytes().unwrap();

    let x_avax_addr = pubkey.to_hrp_address(1, "X").unwrap();
    let p_avax_addr = pubkey.to_hrp_address(1, "P").unwrap();
    log::info!("AVAX X address: {}", x_avax_addr);
    log::info!("AVAX P address: {}", p_avax_addr);

    let (hrp, parsed_short_addr) = avax_address_to_short_bytes("X", &x_avax_addr).unwrap();
    assert_eq!(hrp, "avax");
    assert_eq!(parsed_short_addr, short_addr);

    let (hrp, parsed_short_addr) = avax_address_to_short_bytes("P", &p_avax_addr).unwrap();
    assert_eq!(hrp, "avax");
    assert_eq!(parsed_short_addr, short_addr);
}
