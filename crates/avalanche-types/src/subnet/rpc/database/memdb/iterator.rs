//! Database Iterator management implementation for memdb.
use std::{
    io::{Error, ErrorKind, Result},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::subnet::rpc::database::{self, iterator::BoxedIterator};

/// Iterator iterates over a membd database's key/value pairs.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Iterator>
pub struct Iterator {
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    initialized: AtomicBool,
    error: Option<Error>,
    closed: Arc<AtomicBool>,
}

impl Iterator {
    pub fn new(keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>, closed: Arc<AtomicBool>) -> BoxedIterator {
        Box::new(Self {
            keys,
            values,
            initialized: AtomicBool::new(false),
            error: None,
            closed,
        })
    }
}

#[tonic::async_trait]
impl database::iterator::Iterator for Iterator {
    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn next(&mut self) -> Result<bool> {
        // Short-circuit and set an error if the underlying database has been closed
        if self.closed.load(Ordering::Relaxed) {
            self.keys = vec![];
            self.values = vec![];
            self.error = Some(Error::new(ErrorKind::Other, "database closed"));
            return Ok(false);
        }

        // If the iterator was not yet initialized, do it now
        if !self.initialized.load(Ordering::Relaxed) {
            self.initialized.store(true, Ordering::Relaxed);
            return Ok(!self.keys.is_empty());
        }

        // Iterator already initialize, advance it
        if !self.keys.is_empty() {
            self.keys.drain(0..1);
            self.values.drain(0..1);
        }

        Ok(!self.keys.is_empty())
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn error(&mut self) -> Result<()> {
        if let Some(err) = &self.error {
            return Err(Error::new(err.kind(), err.to_string()));
        }
        Ok(())
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn key(&self) -> Result<&[u8]> {
        if self.keys.is_empty() {
            return Ok(&[]);
        }
        Ok(&self.keys[0])
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn value(&self) -> Result<&[u8]> {
        if self.values.is_empty() {
            return Ok(&[]);
        }
        Ok(&self.values[0])
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn release(&mut self) {
        self.keys = vec![];
        self.values = vec![];
    }
}
