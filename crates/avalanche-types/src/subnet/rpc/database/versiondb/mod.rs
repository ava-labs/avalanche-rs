//! An in-memory database which perists mutations to the underlying database on
//! commit().
pub mod batch;
pub mod iterator;

use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::subnet::rpc::{
    database::{self, batch::BoxedBatch, iterator::BoxedIterator, BoxedDatabase},
    errors::Error,
};

use tokio::sync::RwLock;

/// Database implements the [`crate::subnet::rpc::database::Database`] interface
/// by living on top of another database, writing changes to the underlying
/// database only when commit is called.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database/versiondb#Database>
#[derive(Clone)]
pub struct Database {
    db: BoxedDatabase,
    mem: Arc<RwLock<HashMap<Vec<u8>, iterator::ValueDelete>>>,
    batch: BoxedBatch,
    /// True if the database is closed.
    closed: Arc<AtomicBool>,
}

impl Database {
    pub fn new(db: BoxedDatabase, batch: BoxedBatch) -> Self {
        Self {
            db,
            mem: Arc::new(RwLock::new(HashMap::new())),
            batch,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[tonic::async_trait]
impl database::KeyValueReaderWriterDeleter for Database {
    /// Implements the [`crate::subnet::rpc::database::KeyValueReaderWriterDeleter`] trait.
    async fn has(&self, key: &[u8]) -> io::Result<bool> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mem = self.mem.read().await;
        if let Some(value) = mem.get(key) {
            return Ok(!value.delete);
        }

        self.db.has(key).await
    }

    /// Implements the [`crate::subnet::rpc::database::KeyValueReaderWriterDeleter`] trait.
    async fn get(&self, key: &[u8]) -> io::Result<Vec<u8>> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mem = self.mem.read().await;
        if let Some(val) = mem.get(key) {
            return Ok(val.value.clone());
        }

        self.db.get(key).await
    }

    /// Implements the [`crate::subnet::rpc::database::KeyValueReaderWriterDeleter`] trait.
    async fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mut mem = self.mem.write().await;
        mem.insert(
            key.to_vec(),
            iterator::ValueDelete {
                value: value.to_vec(),
                delete: false,
            },
        );

        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::KeyValueReaderWriterDeleter`] trait.
    async fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mut mem = self.mem.write().await;
        if let Some(val) = mem.get_mut(key) {
            val.delete = true;
        }
        mem.insert(
            key.to_vec(),
            iterator::ValueDelete {
                value: vec![],
                delete: true,
            },
        );

        Ok(())
    }
}

#[tonic::async_trait]
impl database::Closer for Database {
    /// Implements the [`crate::subnet::rpc::database::Closer`] trait.
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
    /// Implements the [`crate::subnet::rpc::health::Checkable`] trait.
    async fn health_check(&self) -> io::Result<Vec<u8>> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        self.db.health_check().await
    }
}

#[tonic::async_trait]
impl database::iterator::Iteratee for Database {
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

        let mem = self.mem.write().await;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(mem.len());
        for (k, _v) in mem.iter() {
            if k.starts_with(prefix) && k >= &start.to_vec() {
                keys.push(k.to_owned());
            }
        }

        // keys need to be in sorted order
        keys.sort();

        let mut values: Vec<iterator::ValueDelete> = Vec::with_capacity(keys.len());
        for key in keys.iter() {
            if let Some(v) = mem.get(key) {
                values.push(v.to_owned());
            }
        }

        Ok(iterator::Iterator::new(
            keys,
            values,
            Arc::clone(&self.closed),
            self.db
                .new_iterator_with_start_and_prefix(start, prefix)
                .await?,
        ))
    }
}

#[tonic::async_trait]
impl database::batch::Batcher for Database {
    /// Implements the [`crate::subnet::rpc::database::batch::Batcher`] trait.
    async fn new_batch(&self) -> io::Result<BoxedBatch> {
        Ok(Box::new(batch::Batch::new(
            Arc::clone(&self.mem),
            Arc::clone(&self.closed),
        )))
    }
}

#[tonic::async_trait]
impl database::Commitable for Database {
    /// Implements the [`crate::subnet::rpc::database::Commitable`] trait.
    async fn commit(&mut self) -> io::Result<()> {
        let mut batch = self.commit_batch().await?;
        batch.write().await?;
        batch.reset().await;
        self.abort().await?;
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::Commitable`] trait.
    async fn abort(&self) -> io::Result<()> {
        let mut mem = self.mem.write().await;
        mem.clear();
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::Commitable`] trait.
    async fn commit_batch(&mut self) -> io::Result<BoxedBatch> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let mem = self.mem.read().await;
        self.batch.reset().await;
        for (key, value) in mem.iter() {
            if value.delete {
                self.batch.delete(key).await?;
            } else {
                self.batch.put(key, &value.value).await?;
            }
        }

        Ok(self.batch.clone())
    }
}

impl database::Database for Database {}

#[tokio::test]
async fn iterate_test() {
    use crate::subnet::rpc::database::{
        iterator::Iteratee, memdb, Commitable, KeyValueReaderWriterDeleter,
    };

    let base_db = memdb::Database::new();

    let batch = base_db.new_batch().await.unwrap();
    let mut db = Database::new(base_db, batch);

    let key1 = "hello1".as_bytes();
    let value1 = "world1".as_bytes();
    let key2 = "z".as_bytes();
    let value2 = "world2".as_bytes();

    let _ = db.put(key1, value1).await.unwrap();
    let _ = db.commit().await.unwrap();

    let mut iterator = db.new_iterator().await.unwrap();
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key1);
    assert_eq!(iterator.value().await.unwrap(), value1);
    assert_eq!(iterator.next().await.unwrap(), false);
    assert!(iterator.key().await.unwrap().is_empty());
    assert!(iterator.value().await.unwrap().is_empty());
    assert!(iterator.error().await.is_ok());

    let _ = db.put(key2, value2).await.unwrap();

    let mut iterator = db.new_iterator().await.unwrap();
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key1);
    assert_eq!(iterator.value().await.unwrap(), value1);
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key2);
    assert_eq!(iterator.value().await.unwrap(), value2);
    assert_eq!(iterator.next().await.unwrap(), false);
    assert!(iterator.key().await.unwrap().is_empty());
    assert!(iterator.value().await.unwrap().is_empty());
    assert!(iterator.error().await.is_ok());

    let _ = db.delete(key1).await.unwrap();

    let mut iterator = db.new_iterator().await.unwrap();
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key2);
    assert_eq!(iterator.value().await.unwrap(), value2);
    assert_eq!(iterator.next().await.unwrap(), false);
    assert!(iterator.key().await.unwrap().is_empty());
    assert!(iterator.value().await.unwrap().is_empty());
    assert!(iterator.error().await.is_ok());

    let _ = db.commit().await.unwrap();
    let _ = db.put(key2, value1).await.unwrap();

    let mut iterator = db.new_iterator().await.unwrap();
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key2);
    assert_eq!(iterator.value().await.unwrap(), value1);
    assert_eq!(iterator.next().await.unwrap(), false);
    assert!(iterator.key().await.unwrap().is_empty());
    assert!(iterator.value().await.unwrap().is_empty());
    assert!(iterator.error().await.is_ok());

    let _ = db.commit().await.unwrap();
    let _ = db.put(key1, value2).await.unwrap();

    let mut iterator = db.new_iterator().await.unwrap();
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key1);
    assert_eq!(iterator.value().await.unwrap(), value2);
    assert_eq!(iterator.next().await.unwrap(), true);
    assert_eq!(iterator.key().await.unwrap(), key2);
    assert_eq!(iterator.value().await.unwrap(), value1);
    assert_eq!(iterator.next().await.unwrap(), false);
    assert!(iterator.key().await.unwrap().is_empty());
    assert!(iterator.value().await.unwrap().is_empty());
    assert!(iterator.error().await.is_ok());
}
