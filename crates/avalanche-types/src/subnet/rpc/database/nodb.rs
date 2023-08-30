use std::io::{Error, Result};

use super::iterator::BoxedIterator;

pub struct Iterator {
    err: Option<std::io::Error>,
}

/// NoDB Iterator is used in cases where condition expects an empty iterator.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Iterator>
impl Iterator {
    pub fn new(err: Option<std::io::Error>) -> BoxedIterator {
        Box::new(Iterator { err })
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::iterator::Iterator for Iterator {
    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn next(&mut self) -> Result<bool> {
        Ok(false)
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn error(&mut self) -> Result<()> {
        if let Some(err) = &self.err {
            return Err(Error::new(err.kind(), err.to_string()));
        }
        Ok(())
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn key(&self) -> Result<&[u8]> {
        Ok(&[])
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn value(&self) -> Result<&[u8]> {
        Ok(&[])
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn release(&mut self) {}
}
