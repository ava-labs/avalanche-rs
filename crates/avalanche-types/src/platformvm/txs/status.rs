use crate::{errors::Result, packer::Packer};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Defines possible status values.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/status#Status>
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
    /// The status is unknown.
    Unknown(String),

    /// The operation has been proposed and committed (accepted).
    Committed,

    /// The operation was proposed and aborted (rejected).
    Aborted,

    /// The operation was proposed and is currently in the preferred chain.
    Processing,

    /// The operation was dropped due to failing verification.
    Dropped,
}

impl Default for Status {
    fn default() -> Self {
        Status::Unknown("default".to_owned())
    }
}

impl std::convert::From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "Committed" => Status::Committed,
            "Aborted" => Status::Aborted,
            "Processing" => Status::Processing,
            "Dropped" => Status::Dropped,
            u => Status::Unknown(u.to_owned()),
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
            Status::Unknown(s) => s.as_ref(),
            Status::Committed => "Committed",
            Status::Aborted => "Aborted",
            Status::Processing => "Processing",
            Status::Dropped => "Dropped",
        }
    }

    /// Returns all the `&str` values of the enum members.
    pub fn values() -> &'static [&'static str] {
        &["Committed", "Aborted", "Processing", "Dropped"]
    }

    /// Returns the bytes representation of this status.
    pub fn bytes(&self) -> Result<Bytes> {
        let iota = match self {
            Status::Unknown(_) => 0_u32,
            Status::Committed => 4_u32,
            Status::Aborted => 5_u32,
            Status::Processing => 6_u32,
            Status::Dropped => 8_u32,
        };

        let packer = Packer::new(4, 4);
        packer.pack_u32(iota)?;
        Ok(packer.take_bytes())
    }

    /// Returns the u32 primitive representation of this status.
    pub fn to_u32(&self) -> u32 {
        match self {
            Status::Unknown(_) => 0,
            Status::Committed => 4,
            Status::Aborted => 5,
            Status::Processing => 6,
            Status::Dropped => 8,
        }
    }

    /// Returns the i32 primitive representation of this status.
    pub fn to_i32(&self) -> i32 {
        match self {
            Status::Unknown(_) => 0,
            Status::Committed => 4,
            Status::Aborted => 5,
            Status::Processing => 6,
            Status::Dropped => 8,
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::txs::status::test_bytes --exact --show-output
#[test]
fn test_bytes() {
    let sb = Status::Unknown("()".to_string()).bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x00]));

    let sb = Status::Committed.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x04]));

    let sb = Status::Aborted.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x05]));

    let sb = Status::Processing.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x06]));

    let sb = Status::Dropped.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x08]));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- platformvm::status::test_to_u32 --exact --show-output
#[test]
fn test_to_u32() {
    assert_eq!(Status::Unknown("hello".to_string()).to_u32(), 0);
    assert_eq!(Status::Committed.to_u32(), 4);
    assert_eq!(Status::Aborted.to_u32(), 5);
    assert_eq!(Status::Processing.to_u32(), 6);
    assert_eq!(Status::Dropped.to_u32(), 8);
}
