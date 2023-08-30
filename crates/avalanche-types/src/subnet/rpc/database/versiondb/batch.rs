//! Database Batch management implementation for versiondb.
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{Mutex, RwLock};

use crate::subnet::rpc::{
    database::{
        batch::{CAPACITY_REDUCTION_FACTOR, MAX_EXCESS_CAPACITY_FACTOR},
        BoxedDatabase,
    },
    errors::Error,
};

use super::iterator::ValueDelete;

struct KeyValue {
    key: Vec<u8>,
    value: Vec<u8>,
    delete: bool,
}

/// Batch is a write-only database that commits changes to its host database
/// when Write is called. Although batch is currently async and thread safe it
/// should not be used concurrently at this time.
#[derive(Clone)]
pub struct Batch {
    writes: Arc<RwLock<Vec<KeyValue>>>,
    size: usize,

    db_mem: Arc<RwLock<HashMap<Vec<u8>, ValueDelete>>>,
    db_closed: Arc<AtomicBool>,
}

impl Batch {
    pub fn new(
        db_mem: Arc<RwLock<HashMap<Vec<u8>, ValueDelete>>>,
        db_closed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            writes: Arc::new(RwLock::new(Vec::new())),
            size: 0,
            db_mem,
            db_closed,
        }
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::batch::Batch for Batch {
    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        let mut writes = self.writes.write().await;
        writes.push(KeyValue {
            key: key.to_owned(),
            value: value.to_owned(),
            delete: false,
        });
        self.size += key.len() + value.len();
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        let mut writes = self.writes.write().await;
        writes.push(KeyValue {
            key: key.to_owned(),
            value: vec![],
            delete: true,
        });
        self.size += key.len();
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn size(&self) -> io::Result<usize> {
        Ok(self.size)
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn write(&self) -> io::Result<()> {
        if self.db_closed.load(Ordering::Relaxed) {
            return Err(Error::DatabaseClosed.to_err());
        }

        let writes = self.writes.write().await;
        let mut mem = self.db_mem.write().await;
        for kv in writes.iter() {
            mem.insert(
                kv.key.clone(),
                ValueDelete {
                    value: kv.value.clone(),
                    delete: kv.delete,
                },
            );
        }
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn reset(&mut self) {
        let mut writes = self.writes.write().await;
        if writes.capacity() > writes.len() * MAX_EXCESS_CAPACITY_FACTOR {
            let kv: Vec<KeyValue> =
                Vec::with_capacity(writes.capacity() / CAPACITY_REDUCTION_FACTOR);
            writes.clear();
            *writes = kv;
        } else {
            writes.clear()
        }
        self.size = 0;
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn replay(&self, k: Arc<Mutex<BoxedDatabase>>) -> io::Result<()> {
        let writes = self.writes.write().await;
        let mut db = k.lock().await;
        for kv in writes.iter() {
            if kv.delete {
                db.delete(&kv.key).await.map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("replay delete failed: {:?}", e),
                    )
                })?;
            } else {
                db.put(&kv.key, &kv.value).await.map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("replay put failed: {:?}", e))
                })?;
            }
        }

        Ok(())
    }
}
