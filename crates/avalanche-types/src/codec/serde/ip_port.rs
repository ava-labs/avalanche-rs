use std::net::{IpAddr, SocketAddr};

use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use url::Url;

/// ref. <https://serde.rs/custom-date-format.html>
pub fn serialize<S>(x: &SocketAddr, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // ref. <https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html#method.to_rfc3339_opts>
    serializer.serialize_str(&x.to_string())
}

/// ref. <https://serde.rs/custom-date-format.html>
pub fn deserialize<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.parse() {
        Ok(addr) => Ok(addr),
        Err(e) => {
            log::warn!("fallback to URL parsing {:?}", e);
            let url = Url::parse(&s).map_err(serde::de::Error::custom)?;

            let host = if let Some(hs) = url.host_str() {
                hs.to_string()
            } else {
                return Err(serde::de::Error::custom("no host found"));
            };
            let ip: IpAddr = host.parse().map_err(serde::de::Error::custom)?;
            let port = if let Some(port) = url.port() {
                port
            } else {
                0 // e.g., DNS
            };
            Ok(SocketAddr::new(ip, port))
        }
    }
}

pub struct IpPort(SocketAddr);

impl SerializeAs<SocketAddr> for IpPort {
    fn serialize_as<S>(x: &SocketAddr, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&x.to_string())
    }
}

impl<'de> DeserializeAs<'de, SocketAddr> for IpPort {
    fn deserialize_as<D>(deserializer: D) -> Result<SocketAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.parse() {
            Ok(addr) => Ok(addr),
            Err(e) => {
                log::warn!("fallback to URL parsing {:?}", e);
                let url = Url::parse(&s).map_err(serde::de::Error::custom)?;

                let host = if let Some(hs) = url.host_str() {
                    hs.to_string()
                } else {
                    return Err(serde::de::Error::custom("no host found"));
                };
                let ip: IpAddr = host.parse().map_err(serde::de::Error::custom)?;
                let port = if let Some(port) = url.port() {
                    port
                } else {
                    0 // e.g., DNS
                };
                Ok(SocketAddr::new(ip, port))
            }
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- codec::serde::ip_port::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    use std::net::Ipv4Addr;

    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        #[serde_as(as = "Vec<IpPort>")]
        data: Vec<SocketAddr>,
    }

    let d = Data {
        data: vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(206, 189, 137, 87)), 9651),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(158, 255, 67, 151)), 9651),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(34, 216, 139, 126)), 9650),
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
\"data\":[\"206.189.137.87:9651\", \"158.255.67.151:9651\", \"http://34.216.139.126:9650\"]
}

",
    )
    .unwrap();
    assert_eq!(d, json_decoded_2);
}
