use std::str::FromStr;

use primitive_types::H256;
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

pub fn serialize<S>(x: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", *x))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");

    H256::from_str(s).map_err(serde::de::Error::custom)
}

pub struct Hex0xH256(H256);

impl SerializeAs<H256> for Hex0xH256 {
    fn serialize_as<S>(x: &H256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{:x}", *x))
    }
}

impl<'de> DeserializeAs<'de, H256> for Hex0xH256 {
    fn deserialize_as<D>(deserializer: D) -> Result<H256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches("0x");

        H256::from_str(s).map_err(serde::de::Error::custom)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- codec::serde::hex_0x_primitive_types_h256::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        #[serde_as(as = "Vec<Hex0xH256>")]
        data: Vec<H256>,
    }

    let d = Data {
        data: vec![
            H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9")
                .unwrap(),
            H256::from_str("e16906ec1c7049438bd642023ab15f8633e032940994e6940fff4ec0a2819eb6")
                .unwrap(),
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
\"data\":[\"0xff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9\", \"0xe16906ec1c7049438bd642023ab15f8633e032940994e6940fff4ec0a2819eb6\"]
}

",
    )
    .unwrap();
    assert_eq!(d, json_decoded_2);
}
