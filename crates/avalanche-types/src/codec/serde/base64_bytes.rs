use base64::Engine;
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

pub fn serialize<S>(x: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(x))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD
        .decode(&s)
        .map_err(serde::de::Error::custom)
}

pub struct Base64Bytes(Vec<u8>);

impl SerializeAs<Vec<u8>> for Base64Bytes {
    fn serialize_as<S>(x: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(x))
    }
}

impl<'de> DeserializeAs<'de, Vec<u8>> for Base64Bytes {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- codec::serde::base64_bytes::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        #[serde(with = "crate::codec::serde::base64_bytes")]
        msg: Vec<u8>,
        #[serde_as(as = "Vec<Base64Bytes>")]
        data: Vec<Vec<u8>>,
    }

    let d = Data {
        msg: String::from("hello").as_bytes().to_vec(),
        data: vec![vec![123_u8], vec![123_u8]],
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
    \"msg\":\"aGVsbG8=\",
\"data\":[\"ew==\", \"ew==\"]
}

",
    )
    .unwrap();
    assert_eq!(d, json_decoded_2);
}
