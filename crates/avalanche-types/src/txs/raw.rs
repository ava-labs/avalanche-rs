use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use crate::hash;
use serde::{self, Deserialize, Serialize};

/// Represents raw transaction bytes.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/avalanche/vertex#SortHashOf>
/// ref. <https://docs.rs/zerocopy/latest/zerocopy/trait.AsBytes.html#safety>
#[derive(Debug, Clone, Deserialize, Serialize, Eq)]
#[repr(transparent)]
pub struct Data(Vec<u8>);

impl Default for Data {
    fn default() -> Self {
        Self::default()
    }
}

impl Data {
    pub fn default() -> Self {
        Data(Vec::new())
    }

    pub fn from_slice(d: &[u8]) -> Self {
        Data(Vec::from(d))
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Ord for Data {
    fn cmp(&self, other: &Data) -> Ordering {
        let h1 = hash::sha256(&self.0);
        let h2 = hash::sha256(&other.0);
        h1.cmp(&(h2))
    }
}

impl PartialOrd for Data {
    fn partial_cmp(&self, other: &Data) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// ref. <https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq>
impl Hash for Data {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Represents a slice of raw transaction bytes, to be sorted
/// in the order of SHA256 hash digests.
#[derive(Eq)]
pub struct DataSlice(Vec<Data>);

impl DataSlice {
    pub fn new(txs: &[Data]) -> Self {
        DataSlice(Vec::from(txs))
    }
}

impl Ord for DataSlice {
    fn cmp(&self, other: &DataSlice) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for DataSlice {
    fn partial_cmp(&self, other: &DataSlice) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DataSlice {
    fn eq(&self, other: &DataSlice) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- txs::raw::test_sort --exact --show-output
#[test]
fn test_sort() {
    let d1 = <Vec<u8>>::from([0x01, 0x00, 0x00, 0x00]);
    let d2 = <Vec<u8>>::from([0x01, 0x00, 0x00, 0x00]);
    assert!(d1 == d2);

    let d1 = <Vec<u8>>::from([0x01, 0x00, 0x00, 0x00]);
    let d2 = <Vec<u8>>::from([0x01, 0x00, 0x00, 0x00, 0x00]);
    assert!(d1 < d2);

    let d1 = Data(<Vec<u8>>::from([0x01, 0x00, 0x00, 0x00]));
    let d2 = Data(<Vec<u8>>::from([0x01, 0x00, 0x00, 0x00, 0x00]));
    assert!(d1 < d2);

    let d1 = Data(<Vec<u8>>::from([0x99, 0x00, 0x00, 0x00]));
    let d2 = Data(<Vec<u8>>::from([0x99, 0x00, 0x00, 0x00, 0x00]));
    assert!(d1 < d2);

    let d1 = Data(<Vec<u8>>::from([0x99, 0x00, 0x00]));
    let d2 = Data(<Vec<u8>>::from([0x99, 0x00, 0x01]));
    assert!(d1 < d2);

    let d1 = Data(<Vec<u8>>::from([0x99, 0x01, 0x01]));
    let d2 = Data(<Vec<u8>>::from([0x99, 0x20, 0x01]));
    assert!(d1 > d2);

    let d1 = Data(<Vec<u8>>::from([0x01]));
    let d2 = Data(<Vec<u8>>::from([0x02]));
    let d3 = Data(<Vec<u8>>::from([0x03]));
    assert!(d1 < d2);
    assert!(d2 > d3);
    assert!(d1 > d3);

    let mut ds1 = DataSlice(vec![
        Data(<Vec<u8>>::from([0x01])),
        Data(<Vec<u8>>::from([0x02])),
        Data(<Vec<u8>>::from([0x03])),
    ]);
    ds1.0.sort();
    let ds2 = DataSlice(vec![
        Data(<Vec<u8>>::from([0x03])),
        Data(<Vec<u8>>::from([0x01])),
        Data(<Vec<u8>>::from([0x02])),
    ]);
    assert!(ds1 == ds2);

    let mut ds1 = vec![
        Data(<Vec<u8>>::from([0x01])),
        Data(<Vec<u8>>::from([0x02])),
        Data(<Vec<u8>>::from([0x03])),
    ];
    ds1.sort();
    let ds2 = vec![
        Data(<Vec<u8>>::from([0x03])),
        Data(<Vec<u8>>::from([0x01])),
        Data(<Vec<u8>>::from([0x02])),
    ];
    assert!(ds1 == ds2);

    let mut ds1 = vec![
        <Vec<u8>>::from([0x01]),
        <Vec<u8>>::from([0x02]),
        <Vec<u8>>::from([0x03]),
    ];
    ds1.sort_by(|a, b| (Data::from_slice(a.as_ref())).cmp(&Data::from_slice(b.as_ref())));
    let ds2 = vec![
        <Vec<u8>>::from([0x03]),
        <Vec<u8>>::from([0x01]),
        <Vec<u8>>::from([0x02]),
    ];
    assert!(ds1 == ds2);
}
