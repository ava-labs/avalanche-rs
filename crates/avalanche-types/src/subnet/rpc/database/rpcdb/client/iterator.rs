//! Database Iterator management implementation for rpcdb client.
use crate::{
    proto::rpcdb::{self, database_client::DatabaseClient},
    subnet::rpc::{
        database,
        errors::{self, Error},
        utils,
    },
};

use std::{
    io::Result,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::subnet::rpc::database::iterator::BoxedIterator;

/// Iterator iterates over a database's key/value pairs.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Iterator>
pub struct Iterator {
    id: u64,
    /// List of PutRequests.
    data: Vec<rpcdb::PutRequest>,
    /// Collects first error reported by iterator.
    error: Arc<RwLock<utils::Errors>>,
    db: DatabaseClient<Channel>,
    /// True if the underlying database is closed.
    closed: Arc<AtomicBool>,
}

impl Iterator {
    pub fn new(db: DatabaseClient<Channel>, id: u64, closed: Arc<AtomicBool>) -> BoxedIterator {
        Box::new(Self {
            id,
            data: vec![],
            error: Arc::new(RwLock::new(utils::Errors::new())),
            db,
            closed,
        })
    }
}

#[tonic::async_trait]
impl database::iterator::Iterator for Iterator {
    /// Implements the [`crate::subnet::rpc::database::iterator::Iterator`] trait.
    async fn next(&mut self) -> Result<bool> {
        // Short-circuit and set an error if the underlying database has been closed
        let mut db = self.db.clone();
        let mut errs = self.error.write().await;
        if self.closed.load(Ordering::Relaxed) {
            errs.add(&Error::DatabaseClosed.to_err());
            return Ok(false);
        }

        if self.data.len() > 1 {
            self.data.drain(0..1);
            return Ok(true);
        }

        match db
            .iterator_next(rpcdb::IteratorNextRequest { id: self.id })
            .await
        {
            Ok(resp) => {
                self.data = resp.into_inner().data;
                return Ok(!self.data.is_empty());
            }
            Err(s) => {
                log::error!("iterator next request failed: {:?}", s);
                errs.add(&errors::from_status(s));
                return Ok(false);
            }
        }
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iterator`] trait.
    async fn error(&mut self) -> Result<()> {
        let mut errs = self.error.write().await;
        errs.err()?;

        let mut db = self.db.clone();
        match db
            .iterator_error(rpcdb::IteratorErrorRequest { id: self.id })
            .await
        {
            Ok(resp) => {
                // check response for error
                if let Err(err) = errors::from_i32(resp.into_inner().err) {
                    errs.add(&err);
                    return Err(err);
                }
                return Ok(());
            }
            Err(s) => {
                log::error!("iterator error request failed: {:?}", s);
                let err = errors::from_status(s);
                errs.add(&err);
                return Err(err);
            }
        }
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iterator`] trait.
    async fn key(&self) -> Result<&[u8]> {
        if self.data.is_empty() {
            return Ok(&[]);
        }
        Ok(&self.data[0].key)
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iterator`] trait.
    async fn value(&self) -> Result<&[u8]> {
        if self.data.is_empty() {
            return Ok(&[]);
        }
        Ok(&self.data[0].value)
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iterator`] trait.
    async fn release(&mut self) {
        let mut errs = self.error.write().await;
        let mut db = self.db.clone();
        match db
            .iterator_release(rpcdb::IteratorReleaseRequest { id: self.id })
            .await
        {
            Ok(resp) => {
                // check response for error
                if let Err(err) = errors::from_i32(resp.into_inner().err) {
                    errs.add(&err);
                }
            }
            Err(s) => {
                log::error!("iterator release request failed: {:?}", s);
                errs.add(&errors::from_status(s));
            }
        }
    }
}
