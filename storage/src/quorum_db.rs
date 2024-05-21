use crate::{COL_QUORUM, COL_QUORUM_NUM};

use super::Storage;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignedSlices(pub Vec<u64>);

#[async_trait]
pub trait QuorumDB {
    async fn put_quorums(&self, epoch: u64, quorums: Vec<AssignedSlices>) -> Result<()>;
    async fn get_quorum_num(&self, epoch: u64) -> Result<Option<u64>>;
    async fn get_assgined_slices(
        &self,
        epoch: u64,
        quorum_id: u64,
    ) -> Result<Option<AssignedSlices>>;
}

fn get_quorum_key(epoch: u64, idx: u64) -> Vec<u8> {
    [epoch.to_be_bytes(), idx.to_be_bytes()].concat()
}

#[async_trait]
impl QuorumDB for Storage {
    async fn put_quorums(&self, epoch: u64, assgined: Vec<AssignedSlices>) -> Result<()> {
        let mut tx = self.db.transaction();
        for (idx, assigned) in assgined.iter().enumerate() {
            let key = get_quorum_key(epoch, idx as u64);
            let value = bincode::serialize(assigned).unwrap();
            tx.put(COL_QUORUM, &key, &value);
        }
        tx.put(
            COL_QUORUM_NUM,
            &epoch.to_be_bytes(),
            &(assgined.len() as u64).to_be_bytes(),
        );
        self.db.write(tx)?;
        Ok(())
    }

    async fn get_quorum_num(&self, epoch: u64) -> Result<Option<u64>> {
        if let Some(raw_data) = self.db.get(COL_QUORUM_NUM, &epoch.to_be_bytes())? {
            return Ok(Some(u64::from_be_bytes(raw_data.try_into().unwrap())));
        }
        Ok(None)
    }

    async fn get_assgined_slices(
        &self,
        epoch: u64,
        quorum_id: u64,
    ) -> Result<Option<AssignedSlices>> {
        if let Some(raw_data) = self.db.get(COL_QUORUM, &get_quorum_key(epoch, quorum_id))? {
            return Ok(Some(bincode::deserialize(&raw_data).unwrap()));
        }
        Ok(None)
    }
}
