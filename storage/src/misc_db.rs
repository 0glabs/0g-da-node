use crate::COL_MISC;

use super::Storage;
use anyhow::Result;
use async_trait::async_trait;

const PROGRESS_KEY: &[u8] = &[0];

#[async_trait]
pub trait MiscDB {
    async fn put_progress(&self, block_number: u64) -> Result<()>;

    async fn get_progress(&self) -> Result<Option<u64>>;
}

#[async_trait]
impl MiscDB for Storage {
    async fn put_progress(&self, block_number: u64) -> Result<()> {
        let mut tx = self.db.transaction();
        tx.put(COL_MISC, PROGRESS_KEY, &block_number.to_be_bytes());
        self.db.write(tx)?;
        Ok(())
    }

    async fn get_progress(&self) -> Result<Option<u64>> {
        if let Some(raw_data) = self.db.get(COL_MISC, PROGRESS_KEY)? {
            return Ok(Some(u64::from_be_bytes(raw_data.try_into().unwrap())));
        }
        Ok(None)
    }
}
