use std::collections::BinaryHeap;
use std::sync::Arc;

use contract_interface::da_sample::SampleResponse;
use storage::slice_db::SliceDB;
use storage::Storage;
use task_executor::TaskExecutor;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::line_candidate::LineCandidate;

pub struct DasStage2Miner {
    db: Arc<RwLock<Storage>>,
    first_stage_receiver: mpsc::UnboundedReceiver<Vec<LineCandidate>>,
    submission_sender: mpsc::UnboundedSender<SampleResponse>,
}

impl DasStage2Miner {
    pub fn spawn(
        executor: TaskExecutor,
        db: Arc<RwLock<Storage>>,
        first_stage_receiver: mpsc::UnboundedReceiver<Vec<LineCandidate>>,
        submission_sender: mpsc::UnboundedSender<SampleResponse>,
    ) {
        let stage2_miner = Self {
            db,
            first_stage_receiver,
            submission_sender,
        };
        executor.spawn(
            async move { Box::pin(stage2_miner.start()).await },
            "stage2_miner",
        );
    }

    pub async fn start(mut self) {
        let mut receiver_channel_openned = true;
        let mut miner_enabled = true;

        let mut line_candidates = BinaryHeap::new();

        loop {
            tokio::select! {
                biased;

                msg = self.first_stage_receiver.recv(), if receiver_channel_openned && miner_enabled => {
                    match msg {
                        Some(lines) => {
                            debug!(number = lines.len(), "Receive first stage lines");
                            line_candidates.extend(lines);
                        },
                        None => {
                            warn!(target : "Stage 2 Miner", "Two stage channel closed");
                            receiver_channel_openned = false;
                        }
                    }
                },

                db = self.db.read(), if !line_candidates.is_empty() && miner_enabled => {
                    if let Err(e) = self.mine(&*db, &mut line_candidates).await {
                        warn!(error = e, "Unexpected error, mine service stopped");
                        miner_enabled = false;
                        self.first_stage_receiver.close();
                    }
                }

                else => {
                    warn!("all channel has been closed, return.");
                    break;
                }
            }
        }
    }

    async fn mine(
        &self,
        db: &impl SliceDB,
        line_candidates: &mut BinaryHeap<LineCandidate>,
    ) -> Result<(), String> {
        while let Some(candidate) = line_candidates.pop() {
            for sample_response in candidate.mine(db).await? {
                info!("Hit a valid answer");
                if self.submission_sender.send(sample_response).is_err() {
                    warn!("Submission channel closed.");
                    return Err("Submission channel closed".to_string());
                }
            }
        }
        Ok(())
    }
}
