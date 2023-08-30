//! Verifiable trait.
use crate::errors::Result;

/// Verifiable can be verified.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components#Verifiable>
pub trait Verifiable {
    /// Verifies the block or vertex.
    /// The protocol must ensure that its parents has already been verified.
    fn verify(&self) -> Result<()>;
}
