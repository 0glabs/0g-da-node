use crate::COL_SLICE;

use super::Storage;
use anyhow::Result;
use ark_serialize::CanonicalSerialize;
use async_trait::async_trait;
use zg_encoder::EncodedSlice;

struct SliceIndex {
    pub epoch: u64,
    pub quorum_id: u64,
    pub storage_root: [u8; 32],
    pub index: u64,
}

#[async_trait]
pub trait SliceDB {
    async fn put_slice(
        &self,
        epoch: u64,
        quorum_id: u64,
        storage_root: [u8; 32],
        slices: Vec<EncodedSlice>,
    ) -> Result<()>;
}

fn get_slice_key(slice_index: &SliceIndex) -> Vec<u8> {
    slice_index
        .epoch
        .to_be_bytes()
        .into_iter()
        .chain(slice_index.quorum_id.to_be_bytes())
        .chain(slice_index.storage_root)
        .chain(slice_index.index.to_be_bytes())
        .collect()
}

#[async_trait]
impl SliceDB for Storage {
    async fn put_slice(
        &self,
        epoch: u64,
        quorum_id: u64,
        storage_root: [u8; 32],
        slices: Vec<EncodedSlice>,
    ) -> Result<()> {
        let mut tx = self.db.transaction();
        for slice in slices.into_iter() {
            let key = get_slice_key(&SliceIndex {
                epoch,
                quorum_id,
                storage_root,
                index: slice.index as u64,
            });
            let mut value: Vec<u8> = Vec::new();
            slice.serialize_uncompressed(&mut value).unwrap();
            tx.put(COL_SLICE, &key, &value);
        }
        self.db.write(tx)?;
        Ok(())
    }
}
