use serde::{Deserialize, Deserializer, Serializer};
use serde_with::{formats, DeserializeAs, SerializeAs};

/// ref. "serde_with::hex::Hex"
pub struct Hex0xBytes<FORMAT: formats::Format = formats::Lowercase>(
    std::marker::PhantomData<FORMAT>,
);

impl<T> SerializeAs<T> for Hex0xBytes<formats::Lowercase>
where
    T: AsRef<[u8]>,
{
    fn serialize_as<S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = hex::encode(x);
        serializer.serialize_str(&format!("0x{}", s))
    }
}

impl<T> SerializeAs<T> for Hex0xBytes<formats::Uppercase>
where
    T: AsRef<[u8]>,
{
    fn serialize_as<S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = hex::encode_upper(x);
        serializer.serialize_str(&format!("0x{}", s))
    }
}

impl<'de, T, FORMAT> DeserializeAs<'de, T> for Hex0xBytes<FORMAT>
where
    T: TryFrom<Vec<u8>>,
    FORMAT: formats::Format,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        <std::borrow::Cow<'de, str> as Deserialize<'de>>::deserialize(deserializer)
            .and_then(|s| {
                hex::decode(&*s.trim_start_matches("0x")).map_err(serde::de::Error::custom)
            })
            .and_then(|vec: Vec<u8>| {
                let length = vec.len();
                vec.try_into().map_err(|_e: T::Error| {
                    serde::de::Error::custom(format_args!(
                        "Can't convert a Byte Vector of length {} to the output type.",
                        length
                    ))
                })
            })
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- codec::serde::hex_0x_bytes::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        #[serde_as(as = "Vec<Hex0xBytes>")]
        data: Vec<Vec<u8>>,
    }

    let d = Data {
        data: vec![vec![123], vec![123]],
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
