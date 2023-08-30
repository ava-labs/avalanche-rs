//! Avalanche Rust SDK: Types and building blocks to assist with creating a custom `subnet` VM.
//!
//! Example VM's created with SDK:
//! * Simple Rust VM: [TimestampVM](https://github.com/ava-labs/timestampvm-rs)
//! * Complex Rust VM: [SpacesVM](https://github.com/ava-labs/spacesvm-rs)

pub mod config;
pub mod rpc;

use std::io::{self, Error, ErrorKind};

use crate::ids;

/// Convert a given Vm name to an encoded Vm Id.
pub fn vm_name_to_id(s: impl AsRef<[u8]>) -> io::Result<ids::Id> {
    let d = s.as_ref();
    if d.len() > ids::LEN {
        return Err(Error::new(
            ErrorKind::Other,
            format!("non-hashed name must be <= 32 bytes, found {}", d.len()),
        ));
    }
    Ok(ids::Id::from_slice(d))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- subnet::test_vm_name_to_id --exact --show-output
#[test]
fn test_vm_name_to_id() {
    let id = vm_name_to_id("timestampvm").unwrap();
    println!("{id}");
    assert_eq!(
        id.to_string(),
        "tGas3T58KzdjcJ2iKSyiYsWiqYctRXaPTqBCA11BqEkNg8kPc"
    );
}
