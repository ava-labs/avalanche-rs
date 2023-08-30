use std::io::Result;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Iterator>
#[tonic::async_trait]
pub trait Iterator {
    /// Attempts to move the iterator to the next key/value pair. It returns whether
    /// the iterator successfully moved to a new key/value pair.
    /// The iterator may return false if the underlying database has been closed
    /// before the iteration has completed, in which case future calls to Error()
    /// must return ErrorKind::Other, "database closed")
    async fn next(&mut self) -> Result<bool>;

    /// Returns any accumulated error. Exhausting all the key/value pairs
    /// is not considered to be an error.
    /// Error should be called after all key/value pairs have been exhausted ie.
    /// after Next() has returned false.
    async fn error(&mut self) -> Result<()>;

    /// Returns the key of the current key/value pair, or empty slice if done.
    async fn key(&self) -> Result<&[u8]>;

    /// Returns the key of the current k&ey/value pair, or empty slice if done.
    async fn value(&self) -> Result<&[u8]>;

    /// Releases associated resources. Release should always succeed and
    /// can be called multiple times without causing error.
    async fn release(&mut self);
}

/// Helper type which defines a thread safe boxed Iterator interface.
pub type BoxedIterator = Box<dyn Iterator + Send + Sync + 'static>;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Iteratee>
#[tonic::async_trait]
pub trait Iteratee {
    /// Creates an iterator over the entire keyspace contained within
    /// the key-value database.
    async fn new_iterator(&self) -> Result<BoxedIterator>;

    /// Creates an iterator over a subset of database content starting
    /// at a particular initial key.
    async fn new_iterator_with_start(&self, start: &[u8]) -> Result<BoxedIterator>;

    /// Creates an iterator over a subset of database content with a
    /// particular key prefix.
    async fn new_iterator_with_prefix(&self, prefix: &[u8]) -> Result<BoxedIterator>;

    /// Creates an iterator over a subset of database content with a
    /// particular key prefix starting at a specified key.
    async fn new_iterator_with_start_and_prefix(
        &self,
        start: &[u8],
        prefix: &[u8],
    ) -> Result<BoxedIterator>;
}
