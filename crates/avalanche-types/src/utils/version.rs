use std::{
    clone::Clone,
    cmp::Ordering,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, Eq)]
pub struct ApplicationVersion {
    pub app: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Ord for ApplicationVersion {
    fn cmp(&self, other: &ApplicationVersion) -> Ordering {
        self.major
            .cmp(&other.major) // returns when major versions are not equal
            .then_with(
                || self.minor.cmp(&other.minor), // if major versions are Equal, compare the minor
            )
            .then_with(
                || self.patch.cmp(&other.patch), // if major/minor versions are Equal, compare the patch
            )
    }
}

impl PartialOrd for ApplicationVersion {
    fn partial_cmp(&self, other: &ApplicationVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ApplicationVersion {
    fn eq(&self, other: &ApplicationVersion) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// ref. <https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq>
impl Hash for ApplicationVersion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app.hash(state);
        self.major.hash(state);
        self.minor.hash(state);
        self.patch.hash(state);
    }
}

impl ApplicationVersion {
    pub fn before(&self, other: &ApplicationVersion) -> bool {
        if self.app != other.app {
            return false;
        }
        self.cmp(other) == Ordering::Less
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- utils::version::test_version --exact --show-output
#[test]
fn test_version() {
    // lengths of individual ids do not matter since all are fixed-sized
    let v1 = ApplicationVersion {
        app: String::from("hello"),
        major: 1,
        minor: 7,
        patch: 15,
    };
    let v2 = ApplicationVersion {
        app: String::from("hello"),
        major: 1,
        minor: 7,
        patch: 15,
    };
    let v3 = ApplicationVersion {
        app: String::from("hello"),
        major: 1,
        minor: 7,
        patch: 17,
    };
    let v4 = ApplicationVersion {
        app: String::from("hello2"),
        major: 1,
        minor: 7,
        patch: 17,
    };
    assert!(v1 == v2);
    assert!(v1 < v3 && v2 < v3);
    assert!(v1.before(&v3) && v2.before(&v3));
    assert!(!v1.before(&v4) && !v2.before(&v4) && !v3.before(&v4));
}
