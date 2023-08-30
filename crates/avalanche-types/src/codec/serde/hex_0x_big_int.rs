use std::io::{self, Error, ErrorKind};

use num_bigint::BigInt;
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

pub fn serialize<S>(x: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&big_int_to_lower_hex(x))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    from_hex_to_big_int(&s).map_err(serde::de::Error::custom)
}

pub struct Hex0xBigInt(BigInt);

impl SerializeAs<BigInt> for Hex0xBigInt {
    fn serialize_as<S>(x: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&big_int_to_lower_hex(x))
    }
}

impl<'de> DeserializeAs<'de, BigInt> for Hex0xBigInt {
    fn deserialize_as<D>(deserializer: D) -> Result<BigInt, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        from_hex_to_big_int(&s).map_err(serde::de::Error::custom)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- codec::serde::hex_0x_big_int::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        #[serde_as(as = "Vec<Hex0xBigInt>")]
        data: Vec<BigInt>,
    }

    let d = Data {
        data: vec![
            from_hex_to_big_int("0x7b").unwrap(),
            from_hex_to_big_int("0x7b").unwrap(),
        ],
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    println!("yaml_encoded:\n{}", yaml_encoded);
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{}", json_encoded);
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);

    let json_decoded_2: Data = serde_json::from_str(
        "

{
\"data\":[\"0x7b\", \"0x7b\"]
}

",
    )
    .unwrap();
    assert_eq!(d, json_decoded_2);
}

/// Parses the big.Int encoded in hex.
/// "0x52B7D2DCC80CD2E4000000" is "100000000000000000000000000" (100,000,000 AVAX).
/// "0x5f5e100" or "0x5F5E100" is "100000000".
/// "0x1312D00" is "20000000".
/// NOTE: copied from "big-num-manager".
/// ref. <https://www.rapidtables.com/convert/number/hex-to-decimal.html>
fn from_hex_to_big_int(s: &str) -> io::Result<BigInt> {
    let sb = s.trim_start_matches("0x").as_bytes();

    // ref. https://docs.rs/num-bigint/latest/num_bigint/struct.BigInt.html
    let b = match BigInt::parse_bytes(sb, 16) {
        Some(v) => v,
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to parse hex big int {} (parse returned None)", s),
            ));
        }
    };
    Ok(b)
}

/// #, adds a 0x in front of the output.
/// NOTE: copied from "big-num-manager".
/// ref. <https://doc.rust-lang.org/nightly/core/fmt/trait.LowerHex.html>
fn big_int_to_lower_hex(v: &BigInt) -> String {
    format!("{:#x}", v)
}
