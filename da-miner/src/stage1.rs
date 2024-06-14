use std::{sync::Arc, time::Duration};

use storage::Storage;
use task_executor::TaskExecutor;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::{
    line_candidate::LineCandidate,
    line_metadata::LineMetadata,
    watcher::{OnChainChangeMessage, SampleTask},
};

pub struct DasStage1Miner {
    db: Arc<RwLock<Storage>>,
    on_chain_receiver: broadcast::Receiver<OnChainChangeMessage>,
    first_stage_sender: mpsc::UnboundedSender<Vec<LineCandidate>>,

    lines: LineMetadata,
}

impl DasStage1Miner {
    pub fn spawn(
        executor: TaskExecutor,
        db: Arc<RwLock<Storage>>,
        on_chain_receiver: broadcast::Receiver<OnChainChangeMessage>,
        first_stage_sender: mpsc::UnboundedSender<Vec<LineCandidate>>,
        current_epoch: u64,
    ) {
        let mut lines = LineMetadata::default();
        lines.new_epoch_range(0u64..current_epoch);

        let stage1_miner = Self {
            db,
            on_chain_receiver,
            first_stage_sender,
            lines,
        };

        executor.spawn(
            async move { Box::pin(stage1_miner.start()).await },
            "das_stage1_miner",
        );
    }

    async fn start(mut self) {
        use OnChainChangeMessage::*;
        let mut receive_channel_opened = true;
        let mut send_channel_opened = true;

        let mut current_task: Option<(SampleTask, u64)> = None;

        const MINE_EPOCH_BATCH: usize = 20;

        loop {
            tokio::select! {
                biased;

                msg = self.on_chain_receiver.recv(), if receive_channel_opened => {
                    match msg {
                        Ok(EpochUpdate(x)) => {
                            self.lines.new_epoch(x - 1);
                        },
                        Ok(NewSampleTask(task)) => {
                            current_task = Some((task, 0));
                        },
                        Ok(ClosedSampleTask(hash)) => {
                            if current_task.map_or(false, |t| t.0.hash == hash) {
                                current_task = None;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed)=>{
                            warn!(target : "Stage 1 Miner", "On-chain status channel closed.");
                            receive_channel_opened = false;
                        }
                        Err(broadcast::error::RecvError::Lagged(n))=>{
                            warn!(target : "Stage 1 Miner", number = n, "On-chain status channel lagged.");
                        }
                    }
                }

                // FIXME: the db load task may suffer a starvation because of line tasks.
                db = self.db.read(), if self.lines.needs_fetch() => {
                    if let Err(error) = self.lines.fetch_epoch(&*db, Duration::from_millis(100)).await {
                        warn!(target: "Stage 1 Miner", ?error, "DB error when fetching epochs");
                    }
                }

                _ = async {}, if current_task.is_some() && send_channel_opened => {
                    let (task, start_epoch) = current_task.unwrap();
                    let (filtered_lines, next_epoch) = self.lines.iter_next_epoch(start_epoch, MINE_EPOCH_BATCH,task);

                    current_task = next_epoch.map(|e| (task, e));
                    if !filtered_lines.is_empty() && self.first_stage_sender.send(filtered_lines).is_err() {
                        warn!(target : "Stage 1 Miner", "Two stages channel closed.");
                        send_channel_opened = false;
                    }
                }
            }
        }
    }
}
