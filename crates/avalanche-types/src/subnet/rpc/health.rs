//! Provides health checking.
use std::io::Result;

/// Checkable can have its health checked
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/health#Checkable>
#[tonic::async_trait]
pub trait Checkable {
    async fn health_check(&self) -> Result<Vec<u8>>;
}
