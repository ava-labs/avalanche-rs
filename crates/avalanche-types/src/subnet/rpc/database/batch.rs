use std::{io::Result, sync::Arc};

use tokio::sync::Mutex;

use super::BoxedDatabase;

/// If, when a batch is reset, the cap(batch)/len(batch) > MAX_EXCESS_CAPACITY_FACTOR,
/// Higher value for MAX_EXCESS_CAPACITY_FACTOR --> less aggressive array downsizing --> less memory allocations
/// but more unnecessary data in the underlying array that can't be garbage collected.
pub const MAX_EXCESS_CAPACITY_FACTOR: usize = 4;

/// The underlying array's capacity will be reduced by a factor of CAPACITY_REDUCTION_FACTOR.
/// Higher value for CapacityReductionFactor --> more aggressive array downsizing --> more memory allocations
/// but less unnecessary data in the underlying array that can't be garbage collected.
pub const CAPACITY_REDUCTION_FACTOR: usize = 2;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Batch>
#[tonic::async_trait]
pub trait Batch: CloneBox {
    async fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    async fn delete(&mut self, key: &[u8]) -> Result<()>;

    /// Retrieves the amount of data queued up for writing, this includes
    /// the keys, values, and deleted keys.
    async fn size(&self) -> Result<usize>;

    /// Flushes any accumulated data to disk.
    async fn write(&self) -> Result<()>;

    /// Resets the batch for reuse.
    async fn reset(&mut self);

    /// Replays the batch contents in the same order they were written
    /// to the batch.
    async fn replay(&self, k: Arc<Mutex<BoxedDatabase>>) -> Result<()>;
}

/// Helper type which defines a thread safe boxed Batch trait.
pub type BoxedBatch = Box<dyn Batch + Send + Sync + 'static>;

pub trait CloneBox {
    /// Returns a Boxed clone of the underlying Batch trait implementation.
    fn clone_box(&self) -> BoxedBatch;
}

impl<T> CloneBox for T
where
    T: 'static + Batch + Clone + Send + Sync,
{
    fn clone_box(&self) -> BoxedBatch {
        Box::new(self.clone())
    }
}

impl Clone for BoxedBatch {
    fn clone(&self) -> BoxedBatch {
        self.clone_box()
    }
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Batcher>
#[tonic::async_trait]
pub trait Batcher {
    /// Creates a write-only database that buffers changes to its host db
    /// until a final write is called.
    async fn new_batch(&self) -> Result<BoxedBatch>;
}
