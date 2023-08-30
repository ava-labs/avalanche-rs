//! Database manager.
pub mod versioned_database;

use std::{
    io::{self, Error, ErrorKind},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::subnet::rpc::database::manager::versioned_database::VersionedDatabase;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database/manager#Manager>
#[tonic::async_trait]
pub trait Manager {
    async fn current(&self) -> io::Result<VersionedDatabase>;
    async fn previous(&self) -> Option<VersionedDatabase>;
    async fn close(&self) -> io::Result<()>;
}

#[derive(Clone)]
pub struct DatabaseManager {
    inner: Arc<RwLock<Vec<VersionedDatabase>>>,
}

impl DatabaseManager {
    /// Returns a database manager from a Vec of versioned database.
    pub fn from_databases(dbs: Vec<VersionedDatabase>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(dbs)),
        }
    }
}

#[tonic::async_trait]
impl Manager for DatabaseManager {
    /// Returns the database with the current database version.
    async fn current(&self) -> io::Result<VersionedDatabase> {
        let dbs = self.inner.read().await;
        Ok(dbs[0].clone())
    }

    /// Returns the database prior to the current database and true if a
    // previous database exists.
    async fn previous(&self) -> Option<VersionedDatabase> {
        let dbs = self.inner.read().await;

        if dbs.len() < 2 {
            return None;
        }
        Some(dbs[1].clone())
    }

    /// Close all of the databases controlled by the manager.
    async fn close(&self) -> io::Result<()> {
        let dbs = self.inner.read().await;

        let mut errors = Vec::with_capacity(dbs.len());
        for db in dbs.iter() {
            let db = &db.db;
            match db.close().await {
                Ok(_) => continue,
                Err(e) => errors.push(e.to_string()),
            }
        }

        if !errors.is_empty() {
            return Err(Error::new(
                ErrorKind::Other,
                errors.first().unwrap().to_string(),
            ));
        }
        Ok(())
    }
}
