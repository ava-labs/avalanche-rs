use std::io::Result;

use semver::Version;

use crate::subnet::rpc::database::{self, BoxedDatabase};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database/manager#VersionedDatabase>
#[derive(Clone)]
pub struct VersionedDatabase {
    pub db: BoxedDatabase,
    pub version: Version,
}

impl VersionedDatabase {
    pub fn new(db: BoxedDatabase, version: Version) -> Self {
        Self { db, version }
    }
}

#[tonic::async_trait]
impl database::VersionedDatabase for VersionedDatabase {
    async fn close(&mut self) -> Result<()> {
        let db = &self.db;
        match db.close().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
