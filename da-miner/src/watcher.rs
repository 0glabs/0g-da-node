use chain_utils::{DefaultMiddleware, DefaultMiddlewareInner};
use contract_interface::{
    da_sample::{self},
    DASample,
};
use task_executor::TaskExecutor;

use ethers::types::{Address, H256, U256};

use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct SampleTask {
    pub sample_seed: H256,
    pub podas_target: U256,
}

#[derive(Debug, Clone, Copy)]
pub enum OnChainChangeMessage {
    UpdateSampleRange(u64, u64),
    NewSampleTask(SampleTask),
    ClosedSampleTask(H256),
}

#[derive(Debug, Clone, Copy)]
pub struct OnChainStatus {
    sample_range: (u64, u64),
    sample_hash: H256,
}

pub struct DasWatcher {
    da_contract: DASample<DefaultMiddlewareInner>,

    sender: broadcast::Sender<OnChainChangeMessage>,
    last_status: Option<OnChainStatus>,
}

impl DasWatcher {
    pub async fn spawn(
        executor: TaskExecutor,
        provider: DefaultMiddleware,
        sender: broadcast::Sender<OnChainChangeMessage>,
        da_address: Address,
    ) -> Result<(), String> {
        let da_contract = DASample::new(da_address, provider.clone());

        let das_watcher = Self {
            da_contract,
            sender,
            last_status: None,
        };
        executor.spawn(
            async move { Box::pin(das_watcher.start()).await },
            "das_watcher",
        );

        Ok(())
    }

    async fn start(mut self) {
        let throttle = sleep(Duration::from_secs(0));
        tokio::pin!(throttle);

        loop {
            tokio::select! {
                biased;

                () = &mut throttle, if !throttle.is_elapsed() => {
                }

                _ = async {}, if throttle.is_elapsed() => {
                    throttle.as_mut().reset(Instant::now() + Duration::from_secs(1));
                    match self.fetch_on_chain_status().await {
                        Ok(status) => {
                            self.last_status = Some(status);
                            trace!(?status, "Update on chain status");
                        }
                        Err(err) => {
                            warn!(error = ?err, "Cannot fetch on chain status");
                        }

                    }
                }
            }
        }
    }

    async fn fetch_on_chain_status(&mut self) -> Result<OnChainStatus, String> {
        use OnChainChangeMessage::*;

        let sample_call = self.da_contract.sample_task();
        let range_call = self.da_contract.sample_range();
        let (sample_context_res, range_res) = tokio::join!(sample_call.call(), range_call.call());

        let sample_context: da_sample::SampleTask =
            sample_context_res.map_err(|e| format!("Failed to query sample task: {:?}", e))?;
        let da_sample::SampleRange {
            start_epoch,
            end_epoch,
        } = range_res.map_err(|e| format!("Failed to query sample range: {:?}", e))?;

        let last_status = self.last_status.as_ref();

        if last_status.map_or(true, |x| x.sample_range != (start_epoch, end_epoch)) {
            self.sender
                .send(UpdateSampleRange(start_epoch, end_epoch))
                .map_err(|e| format!("Broadcast error: {:?}", e))?;
        }

        let sample_hash = H256(sample_context.sample_hash);
        if sample_hash != H256::zero() && last_status.map_or(true, |x| x.sample_hash != sample_hash)
        {
            self.sender
                .send(NewSampleTask(SampleTask {
                    sample_seed: sample_hash,
                    podas_target: sample_context.podas_target,
                }))
                .map_err(|e| format!("Broadcast error: {:?}", e))?;
        }

        if sample_hash != H256::zero() && sample_context.rest_submissions == 0 {
            self.sender
                .send(ClosedSampleTask(sample_hash))
                .map_err(|e| format!("Broadcast error: {:?}", e))?;
        }

        Ok(OnChainStatus {
            sample_range: (start_epoch, end_epoch),
            sample_hash,
        })
    }
}
