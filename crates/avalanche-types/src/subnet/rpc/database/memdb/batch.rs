//! Database Batch management implementation for memdb.
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{Mutex, RwLock};

use crate::subnet::rpc::{database::BoxedDatabase, errors::Error};

struct KeyValue {
    key: Vec<u8>,
    value: Vec<u8>,
    delete: bool,
}

/// Batch is a write-only database that commits changes to its host database
/// when Write is called. Although batch is currently async and thread safe it
/// should not be used concurrently.
#[derive(Clone)]
pub struct Batch {
    writes: Arc<Mutex<Vec<KeyValue>>>,
    size: usize,

    db_state: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    db_closed: Arc<AtomicBool>,
}

impl Batch {
    pub fn new(
        db_state: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
        db_closed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            writes: Arc::new(Mutex::new(Vec::new())),
            size: 0,
            db_state,
            db_closed,
        }
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::batch::Batch for Batch {
    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        let mut writes = self.writes.lock().await;
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
        let mut writes = self.writes.lock().await;
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

        let writes = self.writes.lock().await;
        let mut db = self.db_state.write().await;
        for write in writes.iter() {
            if write.delete {
                db.remove(&write.key);
            } else {
                db.insert(write.key.clone(), write.value.clone());
            }
        }
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn reset(&mut self) {
        let mut writes = self.writes.lock().await;
        if writes.capacity()
            > writes.len() * crate::subnet::rpc::database::batch::MAX_EXCESS_CAPACITY_FACTOR
        {
            let kv: Vec<KeyValue> = Vec::with_capacity(
                writes.capacity() / crate::subnet::rpc::database::batch::CAPACITY_REDUCTION_FACTOR,
            );
            writes.clear();
            *writes = kv;
        } else {
            writes.clear()
        }
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn replay(&self, k: Arc<Mutex<BoxedDatabase>>) -> io::Result<()> {
        let writes = self.writes.lock().await;
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
