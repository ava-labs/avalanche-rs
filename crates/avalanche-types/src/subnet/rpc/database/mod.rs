pub mod batch;
pub mod corruptabledb;
pub mod iterator;
pub mod manager;
pub mod memdb;
pub mod nodb;
pub mod rpcdb;
pub mod versiondb;

use std::io::Result;

use crate::subnet::rpc::health::Checkable;

use self::batch::BoxedBatch;

pub const MAX_BATCH_SIZE: usize = 128 * 1000;

#[tonic::async_trait]
pub trait Closer {
    async fn close(&self) -> Result<()>;
}

#[tonic::async_trait]
pub trait Database:
    batch::Batcher + CloneBox + KeyValueReaderWriterDeleter + Closer + Checkable + iterator::Iteratee
{
}

/// Helper type which defines a thread safe boxed Database trait.
pub type BoxedDatabase = Box<dyn Database + Send + Sync + 'static>;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#KeyValueReaderWriterDeleter>
#[tonic::async_trait]
pub trait KeyValueReaderWriterDeleter {
    async fn has(&self, key: &[u8]) -> Result<bool>;
    async fn get(&self, key: &[u8]) -> Result<Vec<u8>>;
    async fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    async fn delete(&mut self, key: &[u8]) -> Result<()>;
}

// Trait that specifies that something may be
// committed.
#[tonic::async_trait]
#[allow(dead_code)]
trait Commitable {
    /// Writes all the operations of this database to the underlying database.
    async fn commit(&mut self) -> Result<()>;
    /// Abort all changes to the underlying database.
    async fn abort(&self) -> Result<()>;
    /// Returns a batch that contains all uncommitted puts/deletes.  Calling
    /// write() on the returned batch causes the puts/deletes to be written to
    /// the underlying database. The returned batch should be written before
    /// future calls to this DB unless the batch will never be written.
    async fn commit_batch(&mut self) -> Result<BoxedBatch>;
}

pub trait CloneBox {
    /// Returns a Boxed clone of the underlying Database.
    fn clone_box(&self) -> BoxedDatabase;
}

impl<T> CloneBox for T
where
    T: 'static + Database + Clone + Send + Sync,
{
    fn clone_box(&self) -> BoxedDatabase {
        Box::new(self.clone())
    }
}

impl Clone for BoxedDatabase {
    fn clone(&self) -> BoxedDatabase {
        self.clone_box()
    }
}

#[tonic::async_trait]
pub trait VersionedDatabase {
    async fn close(&mut self) -> Result<()>;
}

#[tokio::test]
async fn clone_box_test() {
    // create box and mutate underlying hashmap
    let mut db = memdb::Database::new_boxed();
    let resp = db.put("foo".as_bytes(), "bar".as_bytes()).await;
    assert!(resp.is_ok());

    // clone and mutate
    let mut cloned_db = db.clone();
    let resp = cloned_db.delete("foo".as_bytes()).await;
    assert!(resp.is_ok());

    // verify mutation
    let resp = cloned_db.get("foo".as_bytes()).await;
    assert!(resp.is_err());
}
