//! Avalanche platformvm utilities.
pub mod txs;

use crate::ids;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants#pkg-variables>
pub fn chain_id() -> ids::Id {
    ids::Id::empty()
}
