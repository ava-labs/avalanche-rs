//! Status enum that represents the possible statuses of an consensus operation.
use crate::errors;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Defines possible status values.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/choices#Status>
#[derive(
    Deserialize,
    Serialize,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub enum Status {
    /// The operation is known but has not been decided yet.
    Processing,

    /// The operation is already rejected and will never be accepted.
    Rejected,

    /// The operation has been accepted.
    Accepted,

    /// The status is unknown.
    Unknown(String),
}

impl Default for Status {
    fn default() -> Self {
        Status::Unknown("default".to_owned())
    }
}

impl std::convert::From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "Processing" => Status::Processing,
            "Rejected" => Status::Rejected,
            "Accepted" => Status::Accepted,
            other => Status::Unknown(other.to_owned()),
        }
    }
}

impl std::str::FromStr for Status {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Status::from(s))
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::Processing => "Processing",
            Status::Rejected => "Rejected",
            Status::Accepted => "Accepted",
            Status::Unknown(s) => s.as_ref(),
        }
    }

    /// Returns all the `&str` values of the enum members.
    pub fn values() -> &'static [&'static str] {
        &["Processing", "Rejected", "Accepted"]
    }

    /// Returns "true" if the status has been decided.
    pub fn decided(&self) -> bool {
        matches!(self, Status::Rejected | Status::Accepted)
    }

    /// Returns "true" if the status has been set.
    pub fn fetched(&self) -> bool {
        match self {
            Status::Processing => true,
            _ => self.decided(),
        }
    }

    /// Returns the bytes representation of this status.
    pub fn bytes(&self) -> errors::Result<Bytes> {
        let iota = match self {
            Status::Unknown(_) => 0_u32,
            Status::Processing => 1_u32,
            Status::Rejected => 2_u32,
            Status::Accepted => 3_u32,
        }
        .to_be_bytes();

        let boxed: Box<[u8]> = Box::new(iota);

        Ok(Bytes::from(boxed))
    }

    /// Returns the u32 primitive representation of this status.
    pub fn to_u32(&self) -> u32 {
        match self {
            Status::Processing => 1,
            Status::Rejected => 2,
            Status::Accepted => 3,
            Status::Unknown(_) => 0,
        }
    }

    /// Returns the i32 primitive representation of this status.
    pub fn to_i32(&self) -> i32 {
        match self {
            Status::Processing => 1,
            Status::Rejected => 2,
            Status::Accepted => 3,
            Status::Unknown(_) => 0,
        }
    }

    /// Returns native endian value from a slice if u8s.
    pub fn u32_from_slice(bytes: &[u8]) -> u32 {
        assert!(bytes.len() <= 4);
        let d: [u8; 4] = bytes.try_into().unwrap();
        u32::from_ne_bytes(d)
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- choices::status::test_bytes --exact --show-output
#[test]
fn test_bytes() {
    let sb = Status::Processing.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x01]));

    let sb = Status::Rejected.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x02]));

    let sb = Status::Accepted.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x03]));

    let sb = Status::Unknown("()".to_string()).bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x00]));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- choices::status::test_to_u32 --exact --show-output
#[test]
fn test_to_u32() {
    assert_eq!(Status::Unknown("hello".to_string()).to_u32(), 0);
    assert_eq!(Status::Processing.to_u32(), 1);
    assert_eq!(Status::Rejected.to_u32(), 2);
    assert_eq!(Status::Accepted.to_u32(), 3);
}
