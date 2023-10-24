//! Node ID utilities.
use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
    io::{self, Error, ErrorKind},
    path::Path,
    str::FromStr,
};

use lazy_static::lazy_static;
use serde::{self, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use zerocopy::{AsBytes, FromBytes, FromZeroes, Unaligned};

use crate::{formatting, hash, ids::short};

pub const LEN: usize = 20;
pub const ENCODE_PREFIX: &str = "NodeID-";

lazy_static! {
    static ref EMPTY: Vec<u8> = vec![0; LEN];
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ShortID>
/// ref. <https://docs.rs/zerocopy/latest/zerocopy/trait.AsBytes.html#safety>
#[derive(Debug, Copy, Clone, Eq, AsBytes, FromZeroes, FromBytes, Unaligned)]
#[repr(transparent)]
pub struct Id([u8; LEN]);

impl Default for Id {
    fn default() -> Self {
        Self::empty()
    }
}

impl Id {
    pub fn empty() -> Self {
        Id([0; LEN])
    }

    pub fn is_empty(&self) -> bool {
        (*self) == Self::empty()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_slice(d: &[u8]) -> Self {
        assert_eq!(d.len(), LEN);
        let d: [u8; LEN] = d.try_into().unwrap();
        Id(d)
    }

    /// Loads a node ID from the PEM-encoded X509 certificate.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/node#Node.Initialize>
    pub fn from_cert_pem_file(cert_file_path: &str) -> io::Result<Self> {
        log::info!("loading node ID from certificate {}", cert_file_path);
        let pub_key_der = cert_manager::x509::load_pem_cert_to_der(cert_file_path)?;

        // "ids.ToShortID(hashing.PubkeyBytesToAddress(StakingTLSCert.Leaf.Raw))"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/node#Node.Initialize
        Self::from_cert_der_bytes(pub_key_der)
    }

    /// Encodes the DER-encoded certificate bytes to a node ID.
    /// It applies "sha256" and "ripemd160" on "Certificate.Leaf.Raw".
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ToShortID>
    pub fn from_cert_der_bytes<S>(cert_bytes: S) -> io::Result<Self>
    where
        S: AsRef<[u8]>,
    {
        let short_address = hash::sha256_ripemd160(cert_bytes)?;
        let node_id = Self::from_slice(&short_address);
        Ok(node_id)
    }

    /// Loads the existing staking certificates if exists,
    /// and returns the loaded or generated node Id.
    /// Returns "true" if generated.
    pub fn load_or_generate_pem(key_path: &str, cert_path: &str) -> io::Result<(Self, bool)> {
        let tls_key_exists = Path::new(&key_path).exists();
        log::info!("staking TLS key {} exists? {}", key_path, tls_key_exists);

        let tls_cert_exists = Path::new(&cert_path).exists();
        log::info!("staking TLS cert {} exists? {}", cert_path, tls_cert_exists);

        let mut generated = false;
        if !tls_key_exists || !tls_cert_exists {
            log::info!(
                "generating staking TLS certs (key exists {}, cert exists {})",
                tls_key_exists,
                tls_cert_exists
            );
            cert_manager::x509::generate_and_write_pem(None, key_path, cert_path)?;
            generated = true;
        } else {
            log::info!(
                "loading existing staking TLS certificates from '{}' and '{}'",
                key_path,
                cert_path
            );
        }

        let node_id = Self::from_cert_pem_file(cert_path)?;
        Ok((node_id, generated))
    }

    pub fn short_id(&self) -> short::Id {
        short::Id::from_slice(&self.0)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut node_id = String::from(ENCODE_PREFIX);
        let short_id = formatting::encode_cb58_with_checksum_string(&self.0);
        node_id.push_str(&short_id);
        write!(f, "{}", node_id)
    }
}

/// ref. <https://doc.rust-lang.org/std/str/trait.FromStr.html>
impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // trim in case it's parsed from list
        let processed = s.trim().trim_start_matches(ENCODE_PREFIX);
        let decoded = formatting::decode_cb58_with_checksum(processed).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed decode_cb58_with_checksum '{}'", e),
            )
        })?;
        Ok(Self::from_slice(&decoded))
    }
}

/// Custom serializer.
/// ref. <https://serde.rs/impl-serialize.html>
impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Id, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a base-58 encoded ID-string with checksum")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Id::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::node::test_custom_de_serializer --exact --show-output
#[test]
fn test_custom_de_serializer() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        node_id: Id,
    }

    let d = Data {
        node_id: Id::from_str("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx").unwrap(),
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    println!("yaml_encoded:\n{}", yaml_encoded);
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{}", json_encoded);
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);

    let json_decoded_2: Data =
        serde_json::from_str(r#"{ "node_id":"NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx" }"#)
            .unwrap();
    assert_eq!(d, json_decoded_2);

    let json_encoded_3 = serde_json::json!(
        {
            "node_id": "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"
        }
    );
    let json_decoded_3: Data = serde_json::from_value(json_encoded_3).unwrap();
    assert_eq!(d, json_decoded_3);
}

fn fmt_id<'de, D>(deserializer: D) -> Result<Id, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Id::from_str(&s).map_err(serde::de::Error::custom)
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
pub fn deserialize_id<'de, D>(deserializer: D) -> Result<Option<Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_id")] Id);
    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

/// Custom deserializer.
/// Use #[serde(deserialize_with = "ids::must_deserialize_id")] to serde without derive.
/// ref. <https://serde.rs/impl-deserialize.html>
pub fn must_deserialize_id<'de, D>(deserializer: D) -> Result<Id, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_id")] Id);
    let v = Option::deserialize(deserializer)?;
    match v.map(|Wrapper(a)| a) {
        Some(unwrapped) => Ok(unwrapped),
        None => Err(serde::de::Error::custom(
            "empty node::Id from deserialization",
        )),
    }
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
pub fn deserialize_ids<'de, D>(deserializer: D) -> Result<Option<Vec<Id>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_ids")] Vec<Id>);
    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

/// Custom deserializer.
/// Use #[serde(deserialize_with = "short::must_deserialize_ids")] to serde with derive.
/// ref. <https://serde.rs/impl-deserialize.html>
pub fn must_deserialize_ids<'de, D>(deserializer: D) -> Result<Vec<Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_ids")] Vec<Id>);
    let v = Option::deserialize(deserializer)?;
    match v.map(|Wrapper(a)| a) {
        Some(unwrapped) => Ok(unwrapped),
        None => Err(serde::de::Error::custom("empty Ids from deserialization")),
    }
}

fn fmt_ids<'de, D>(deserializer: D) -> Result<Vec<Id>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    type Strings = Vec<String>;
    let ss = Strings::deserialize(deserializer)?;
    match ss
        .iter()
        .map(|x| x.parse::<Id>())
        .collect::<Result<Vec<Id>, Error>>()
    {
        Ok(x) => Ok(x),
        Err(e) => Err(serde::de::Error::custom(format!(
            "failed to deserialize Ids {}",
            e
        ))),
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::node::test_serialize --exact --show-output
#[test]
fn test_serialize() {
    let id = Id::from_slice(&<Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
    ]));
    assert_eq!(id.to_string(), "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx");

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        id: Id,
        id2: Option<Id>,
        ids: Vec<Id>,
    }
    let d = Data {
        id,
        id2: Some(id),
        ids: vec![id, id, id, id, id],
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    assert!(yaml_encoded.contains("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"));
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    assert!(json_encoded.contains("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"));
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);
}

/// Set is a set of NodeIds
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#NewNodeIDSet>
pub type Set = HashSet<Id>;

/// Return a new NodeIdSet with initial capacity \[size\].
/// More or less than \[size\] elements can be added to this set.
/// Using NewNodeIDSet() rather than ids.NodeIDSet{} is just an optimization that can
/// be used if you know how many elements will be put in this set.
pub fn new_set(size: usize) -> Set {
    let set: HashSet<Id> = HashSet::with_capacity(size);
    set
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::node::test_from_cert_file --exact --show-output
#[test]
fn test_from_cert_file() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let node_id = Id::from_slice(&<Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
    ]));
    assert_eq!(
        format!("{}", node_id),
        "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"
    );
    assert_eq!(
        node_id.short_id().to_string(),
        "6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"
    );
    assert_eq!(
        node_id,
        Id::from_str("6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx").unwrap()
    );

    // copied from "avalanchego/staking/local/staking1.key,crt"
    // verified by "avalanchego-compatibility/node-id" for compatibility with Go
    let node_id = Id::from_cert_pem_file("./artifacts/staker1.insecure.crt").unwrap();
    assert_eq!(
        format!("{}", node_id),
        "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg"
    );
    assert_eq!(
        node_id,
        Id::from_str("7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg").unwrap()
    );

    let node_id = Id::from_cert_pem_file("./artifacts/staker2.insecure.crt").unwrap();
    assert_eq!(
        format!("{}", node_id),
        "NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ"
    );
    assert_eq!(
        node_id,
        Id::from_str("MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ").unwrap()
    );

    let node_id = Id::from_cert_pem_file("./artifacts/staker3.insecure.crt").unwrap();
    assert_eq!(
        format!("{}", node_id),
        "NodeID-NFBbbJ4qCmNaCzeW7sxErhvWqvEQMnYcN"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-NFBbbJ4qCmNaCzeW7sxErhvWqvEQMnYcN"
    );
    assert_eq!(
        node_id,
        Id::from_str("NFBbbJ4qCmNaCzeW7sxErhvWqvEQMnYcN").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-NFBbbJ4qCmNaCzeW7sxErhvWqvEQMnYcN").unwrap()
    );

    let node_id = Id::from_cert_pem_file("./artifacts/staker4.insecure.crt").unwrap();
    assert_eq!(
        format!("{}", node_id),
        "NodeID-GWPcbFJZFfZreETSoWjPimr846mXEKCtu"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-GWPcbFJZFfZreETSoWjPimr846mXEKCtu"
    );
    assert_eq!(
        node_id,
        Id::from_str("GWPcbFJZFfZreETSoWjPimr846mXEKCtu").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-GWPcbFJZFfZreETSoWjPimr846mXEKCtu").unwrap()
    );

    let node_id = Id::from_cert_pem_file("./artifacts/staker5.insecure.crt").unwrap();
    assert_eq!(
        format!("{}", node_id),
        "NodeID-P7oB2McjBGgW2NXXWVYjV8JEDFoW9xDE5"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-P7oB2McjBGgW2NXXWVYjV8JEDFoW9xDE5"
    );
    assert_eq!(
        node_id,
        Id::from_str("P7oB2McjBGgW2NXXWVYjV8JEDFoW9xDE5").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-P7oB2McjBGgW2NXXWVYjV8JEDFoW9xDE5").unwrap()
    );

    let node_id = Id::from_cert_pem_file("./artifacts/test.insecure.crt").unwrap();
    assert_eq!(
        format!("{}", node_id),
        "NodeID-29HTAG5cfN2fw79A67Jd5zY9drcT51EBG"
    );
    assert_eq!(
        node_id.to_string(),
        "NodeID-29HTAG5cfN2fw79A67Jd5zY9drcT51EBG"
    );
    assert_eq!(
        node_id,
        Id::from_str("29HTAG5cfN2fw79A67Jd5zY9drcT51EBG").unwrap()
    );
    assert_eq!(
        node_id,
        Id::from_str("NodeID-29HTAG5cfN2fw79A67Jd5zY9drcT51EBG").unwrap()
    );
}

impl Ord for Id {
    fn cmp(&self, other: &Id) -> Ordering {
        self.0.cmp(&(other.0))
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// ref. <https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq>
impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Ids(Vec<Id>);

impl Ids {
    pub fn new(ids: &[Id]) -> Self {
        Ids(Vec::from(ids))
    }
}

impl From<Vec<Id>> for Ids {
    fn from(ids: Vec<Id>) -> Self {
        Self::new(&ids)
    }
}

impl Ord for Ids {
    fn cmp(&self, other: &Ids) -> Ordering {
        // packer encodes the array length first
        // so if the lengths differ, the ordering is decided
        let l1 = self.0.len();
        let l2 = other.0.len();
        l1.cmp(&l2) // returns when lengths are not Equal
            .then_with(
                || self.0.cmp(&other.0), // if lengths are Equal, compare the ids
            )
    }
}

impl PartialOrd for Ids {
    fn partial_cmp(&self, other: &Ids) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Ids {
    fn eq(&self, other: &Ids) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::node::test_sort --exact --show-output
#[test]
fn test_sort() {
    // lengths of individual ids do not matter since all are fixed-sized
    let id1 = Id::from_slice(&<Vec<u8>>::from([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]));
    let id2 = Id::from_slice(&<Vec<u8>>::from([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]));
    assert!(id1 == id2);

    // lengths of individual ids do not matter since all are fixed-sized
    let id1 = Id::from_slice(&<Vec<u8>>::from([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]));
    let id2 = Id::from_slice(&<Vec<u8>>::from([
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]));
    assert!(id1 < id2);

    // lengths of individual ids do not matter since all are fixed-sized
    let id1 = Id::from_slice(&<Vec<u8>>::from([
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]));
    let id2 = Id::from_slice(&<Vec<u8>>::from([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]));
    assert!(id1 > id2);

    // lengths of NodeIds matter
    let ids1 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    let ids2 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    assert!(ids1 == ids2);

    // lengths of NodeIds matter
    let ids1 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    let ids2 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    assert!(ids1 < ids2);

    // lengths of NodeIds matter
    let ids1 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    let ids2 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    assert!(ids1 > ids2);

    // lengths of NodeIds matter
    let ids1 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    let ids2 = Ids(vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ]);
    assert!(ids1 < ids2);

    let mut ids1 = vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ];
    ids1.sort();
    let ids2 = vec![
        Id::from_slice(&<Vec<u8>>::from([
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
        Id::from_slice(&<Vec<u8>>::from([
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])),
    ];
    assert!(ids1 == ids2);
}
