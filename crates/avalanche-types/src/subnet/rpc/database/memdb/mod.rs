//! Implements an in-memory database useful for testing.
//!
//!```rust
//! use avalanche_types::subnet::rpc::database::memdb::Database;
//!
//! let mut db = Database::new();
//! let resp = db.put("foo".as_bytes(), "bar".as_bytes()).await;
//! let resp = db.has("foo".as_bytes()).await;
//! assert_eq!(resp.unwrap(), true);
//! ```
pub mod batch;
pub mod iterator;

use std::{
    collections::HashMap,
    io,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

use crate::subnet::rpc::errors::Error;

use super::{batch::BoxedBatch, iterator::BoxedIterator, BoxedDatabase};
use tokio::sync::RwLock;

/// Database is an ephemeral key-value store that implements the Database interface.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database/memdb#Database>
#[derive(Clone)]
pub struct Database {
    /// Hashmap guarded by mutex which stores the memdb state.
    state: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    /// True if the database is closed.
    closed: Arc<AtomicBool>,
}

impl Database {
    pub fn new() -> BoxedDatabase {
        Box::new(Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            closed: Arc::new(AtomicBool::new(false)),
        })
    }
}

#[tonic::async_trait]
impl super::KeyValueReaderWriterDeleter for Database {
    /// Attempts to return if the database has a key with the provided value.
    async fn has(&self, key: &[u8]) -> io::Result<bool> {
        let db = self.state.read().await;
        match db.get(&key.to_vec()) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Attempts to return the value that was mapped to the key that was provided.
    async fn get(&self, key: &[u8]) -> io::Result<Vec<u8>> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let db = self.state.read().await;
        match db.get(&key.to_vec()) {
            Some(key) => Ok(key.to_vec()),
            None => Err(Error::NotFound.to_err()),
        }
    }

    /// Attempts to set the value this key maps to.
    async fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mut db = self.state.write().await;
        db.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    /// Attempts to remove any mapping from the key.
    async fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mut db = self.state.write().await;
        db.remove(&key.to_vec());
        Ok(())
    }
}

#[tonic::async_trait]
impl super::Closer for Database {
    /// Attempts to close the database.
    async fn close(&self) -> io::Result<()> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        self.closed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::health::Checkable for Database {
    /// Checks if the database has been closed.
    async fn health_check(&self) -> io::Result<Vec<u8>> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }
        Ok(vec![])
    }
}

#[tonic::async_trait]
impl super::iterator::Iteratee for Database {
    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator(&self) -> io::Result<BoxedIterator> {
        self.new_iterator_with_start_and_prefix(&[], &[]).await
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator_with_start(&self, start: &[u8]) -> io::Result<BoxedIterator> {
        self.new_iterator_with_start_and_prefix(start, &[]).await
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator_with_prefix(&self, prefix: &[u8]) -> io::Result<BoxedIterator> {
        self.new_iterator_with_start_and_prefix(&[], prefix).await
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator_with_start_and_prefix(
        &self,
        start: &[u8],
        prefix: &[u8],
    ) -> io::Result<BoxedIterator> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let db = self.state.read().await;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(db.len());
        for (k, _v) in db.iter() {
            if k.starts_with(prefix) && k >= &start.to_vec() {
                keys.push(k.to_owned());
            }
        }
        // keys need to be in sorted order
        keys.sort();

        let mut values: Vec<Vec<u8>> = Vec::with_capacity(keys.len());
        for key in keys.iter() {
            if let Some(v) = db.get(key) {
                values.push(v.to_owned());
            }
        }

        Ok(iterator::Iterator::new(
            keys,
            values,
            Arc::clone(&self.closed),
        ))
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::batch::Batcher for Database {
    /// Implements the [`crate::subnet::rpc::database::batch::Batcher`] trait.
    async fn new_batch(&self) -> io::Result<BoxedBatch> {
        Ok(Box::new(batch::Batch::new(
            Arc::clone(&self.state),
            Arc::clone(&self.closed),
        )))
    }
}

impl crate::subnet::rpc::database::Database for Database {}

#[tokio::test]
async fn test_memdb() {
    let mut db = Database::new();
    let _ = db.put("foo".as_bytes(), "bar".as_bytes()).await;
    let resp = db.get("notfound".as_bytes()).await;
    assert!(resp.is_err());
    assert_eq!(resp.err().unwrap().kind(), io::ErrorKind::NotFound);

    let mut db = Database::new();
    let _ = db.close().await;
    let resp = db.put("foo".as_bytes(), "bar".as_bytes()).await;
    assert!(resp.is_err());
    assert_eq!(resp.err().unwrap().to_string(), "database closed");

    let db = Database::new();
    let _ = db.close().await;
    let resp = db.get("foo".as_bytes()).await;
    print!("found {:?}", resp);
    assert!(resp.is_err());
    assert_eq!(resp.err().unwrap().to_string(), "database closed");

    let mut db = Database::new();
    let _ = db.put("foo".as_bytes(), "bar".as_bytes()).await;
    let resp = db.has("foo".as_bytes()).await;
    assert!(!resp.is_err());
    assert_eq!(resp.unwrap(), true);

    let mut db = Database::new();
    let _ = db.put("foo".as_bytes(), "bar".as_bytes()).await;
    let _ = db.delete("foo".as_bytes()).await;
    let resp = db.has("foo".as_bytes()).await;
    assert!(!resp.is_err());
    assert_eq!(resp.unwrap(), false);

    let db = Database::new();
    let resp = db.health_check().await;
    assert!(!resp.is_err());
    let _ = db.close().await;
    let resp = db.health_check().await;
    assert_eq!(resp.err().unwrap().to_string(), "database closed");
}
