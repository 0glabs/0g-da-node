use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use ethers::types::U256;
use std::time::Instant;
use storage::slice_db::{BlobInfo, SliceDB, SliceIndex};

use crate::{line_candidate::LineCandidate, mine::calculate_line_quality, watcher::SampleTask};

type EpochInfo = BTreeSet<BlobInfo>;

#[derive(Default)]
pub(crate) struct LineMetadata {
    data: BTreeMap<u64, EpochInfo>,
    epoch_to_fetch: BTreeSet<u64>,
}

impl LineMetadata {
    pub fn needs_fetch(&self) -> bool {
        !self.epoch_to_fetch.is_empty()
    }

    pub fn set_epoch_range(&mut self, start_epoch: u64, end_epoch: u64) {
        // Retain keys in [start_epoch, end_epoch]
        let mut data = std::mem::take(&mut self.data);
        let mut data = data.split_off(&start_epoch);
        data.split_off(&(end_epoch + 1));
        self.data = data;

        self.epoch_to_fetch = (start_epoch..=end_epoch)
            .filter(|x| !self.data.contains_key(x))
            .collect();

        info!(
            start_epoch,
            end_epoch,
            to_fetch_epoches = self.epoch_to_fetch.len(),
            "Update metadata for epoches"
        );
    }

    pub async fn fetch_epoch(
        &mut self,
        db: &impl SliceDB,
        duration: Duration,
    ) -> Result<(), String> {
        let deadline = Instant::now() + duration;

        while Instant::now() < deadline {
            let next_epoch = if let Some(x) = self.epoch_to_fetch.pop_first() {
                x
            } else {
                break;
            };

            if self.data.contains_key(&next_epoch) {
                continue;
            }

            debug!(load_epoch = next_epoch, "Load metadata for epoch");

            let epoch_info = db
                .get_epoch_info(next_epoch)
                .await
                .map_err(|e| format!("Fail to fetch epoch {}: {:?}", next_epoch, e))?;

            self.data.insert(next_epoch, epoch_info);
        }
        Ok(())
    }

    pub fn iter_next_epoch(
        &self,
        start_epoch: u64,
        num_batch: usize,
        task: SampleTask,
    ) -> (Vec<LineCandidate>, Option<u64>) {
        if self
            .data
            .last_key_value()
            .map_or(true, |(&epoch, _)| epoch < start_epoch)
        {
            return (vec![], None);
        }

        let mut answer = vec![];
        let mut last_epoch = 0;

        let mut max_quality = [0u8; 32];
        task.podas_target.to_big_endian(&mut max_quality);

        for (&epoch, blobs) in self.data.range(start_epoch..).take(num_batch) {
            for blob in blobs.iter() {
                let quorum_id = blob.quorum_id;
                let storage_root = blob.storage_root;
                for &index in &blob.indicies {
                    let line_quality = calculate_line_quality(
                        task.sample_seed,
                        epoch,
                        quorum_id,
                        storage_root,
                        index,
                    );
                    if line_quality <= max_quality {
                        answer.push(LineCandidate::new(
                            SliceIndex {
                                epoch,
                                quorum_id,
                                storage_root,
                                index: index as u64,
                            },
                            task,
                            U256::from_big_endian(&line_quality),
                        ));
                    }
                }
            }
            last_epoch = epoch;
        }

        (answer, Some(last_epoch))
    }
}
