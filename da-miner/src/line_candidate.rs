use contract_interface::da_sample::SampleResponse;
use ethers::types::U256;
use once_cell::unsync::Lazy;
use storage::slice_db::{SliceDB, SliceIndex};

use crate::{
    constants::{LINE_BYTES, NUM_SUBLINES, SUBLINE_BYTES},
    mine::{build_subline_merkle, calculate_data_quality, serialize_line},
    watcher::SampleTask,
};

#[derive(Debug)]
pub struct LineCandidate {
    index: SliceIndex,
    task: SampleTask,
    line_quality: U256,
}

impl LineCandidate {
    pub fn new(index: SliceIndex, task: SampleTask, line_quality: U256) -> Self {
        Self {
            index,
            task,
            line_quality,
        }
    }

    pub async fn mine(&self, db: &impl SliceDB) -> Result<Vec<SampleResponse>, String> {
        let line_hits = self.find_valid_answer(db).await?;
        self.make_sample_response(db, line_hits).await
    }

    async fn find_valid_answer(&self, db: &impl SliceDB) -> Result<Vec<LineHit>, String> {
        let SliceIndex {
            epoch,
            quorum_id,
            storage_root,
            index,
        } = self.index;

        let line = if let Some(data) = db
            .get_slice_data(epoch, quorum_id, storage_root, index as usize)
            .await
            .map_err(|e| {
                format!(
                    "Cannot load slice data, slice index {:?}, error {:?}",
                    index, e
                )
            })? {
            data
        } else {
            return Ok(vec![]);
        };

        if line.len() * 32 != LINE_BYTES {
            return Err(format!("Incorrect slice length {}", line.len()));
        }

        const SUBLINE_ITEMS: usize = SUBLINE_BYTES / 32;
        let mut found = vec![];

        let lazy_subline_merkle = Lazy::new(|| build_subline_merkle(&line));

        for (subline_index, subline) in line.chunks_exact(SUBLINE_ITEMS).enumerate() {
            let data_quality = U256::from_big_endian(&calculate_data_quality(
                self.line_quality,
                subline_index as u64,
                subline,
            ));

            let final_quality = self.line_quality.checked_add(data_quality);
            if final_quality.map_or(true, |x| x > self.task.podas_target) {
                continue;
            }

            const DEPTH: usize = NUM_SUBLINES.trailing_zeros() as usize;
            let subline_merkle = &*lazy_subline_merkle;
            let proof = (1..=DEPTH)
                .rev()
                .map(|d| {
                    let height = DEPTH - d;
                    let idx = subline_index >> height;
                    subline_merkle[d - 1][idx ^ 1]
                })
                .collect();

            found.push(LineHit {
                subline_index,
                proof,
                data: subline.to_vec(),
                quality: final_quality.unwrap(),
            });
        }

        Ok(found)
    }

    async fn make_sample_response(
        &self,
        db: &impl SliceDB,
        line_hits: Vec<LineHit>,
    ) -> Result<Vec<SampleResponse>, String> {
        if line_hits.is_empty() {
            return Ok(vec![]);
        }

        let SliceIndex {
            epoch,
            quorum_id,
            storage_root,
            index,
        } = self.index;

        let maybe_slice = db
            .get_slice(epoch, quorum_id, storage_root, index as usize)
            .await
            .map_err(|e| {
                format!(
                    "Cannot load encoded slice, slice index {:?}, error {:?}",
                    index, e
                )
            })?;

        let light_slice = if let Some(x) = maybe_slice {
            x
        } else {
            warn!(index = ?self.index, "Encoded slice doesn't exist");
            return Ok(vec![]);
        };

        let answer: Vec<_> = line_hits
            .into_iter()
            .map(|hit| {
                let mut proof = hit.proof.clone();
                proof.extend(&light_slice.merkle_proof);

                SampleResponse {
                    epoch,
                    quorum_id,
                    data_root: storage_root,
                    quality: hit.quality,
                    line_index: index as u32,
                    subline_index: hit.subline_index as u32,
                    data: serialize_line(&hit.data).into(),
                    blob_roots: light_slice.merkle_root,
                    proof,
                    sample_seed: self.task.sample_seed.0,
                }
            })
            .collect();

        Ok(answer)
    }
}

impl PartialEq for LineCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.line_quality == other.line_quality
    }
}

impl Eq for LineCandidate {}

impl PartialOrd for LineCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(&self, &other))
    }
}

impl Ord for LineCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.line_quality.cmp(&self.line_quality)
    }
}

#[derive(Debug)]
pub struct LineHit {
    subline_index: usize,
    proof: Vec<[u8; 32]>,
    data: Vec<[u8; 32]>,
    quality: U256,
}
