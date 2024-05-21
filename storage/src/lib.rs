use std::{path::Path, sync::Arc};

use anyhow::Result;
use kvdb_rocksdb::{Database, DatabaseConfig};

pub mod blob_status_db;
pub mod misc_db;
pub mod quorum_db;
pub mod slice_db;

pub const COL_NUM: u32 = 5;
pub const COL_MISC: u32 = 0;
pub const COL_SLICE: u32 = 1;
pub const COL_QUORUM: u32 = 2;
pub const COL_QUORUM_NUM: u32 = 3;
pub const COL_BLOB_STATUS: u32 = 4;

pub struct Storage {
    db: Arc<Database>,
}

impl Storage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut db_config = DatabaseConfig::with_columns(COL_NUM);
        db_config.enable_statistics = true;
        let db = Arc::new(Database::open(&db_config, path)?);
        Ok(Storage { db })
    }
}
