use primitive_types::U256;
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

pub fn serialize<S>(x: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", *x))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");

    U256::from_str_radix(&s, 16).map_err(serde::de::Error::custom)
}

pub struct Hex0xU256(U256);

impl SerializeAs<U256> for Hex0xU256 {
    fn serialize_as<S>(x: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{:x}", *x))
    }
}

impl<'de> DeserializeAs<'de, U256> for Hex0xU256 {
    fn deserialize_as<D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches("0x");

        U256::from_str_radix(&s, 16).map_err(serde::de::Error::custom)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- codec::serde::hex_0x_primitive_types_u256::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        #[serde_as(as = "Vec<Hex0xU256>")]
        data: Vec<U256>,
    }

    let d = Data {
        data: vec![
            U256::from_dec_str("123").unwrap(),
            U256::from_dec_str("123").unwrap(),
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
