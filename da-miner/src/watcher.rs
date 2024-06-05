use chain_utils::{DefaultMiddleware, DefaultMiddlewareInner, DA_SIGNER_ADDRESS};
use contract_interface::{
    da_sample::{self},
    DASample, DASigners,
};
use std::str::FromStr;
use task_executor::TaskExecutor;

use ethers::types::{Address, H256, U256};

use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, Instant};

const TARGET_SUBMISSIONS: usize = 20;

#[derive(Debug, Clone, Copy)]
pub struct SampleTask {
    pub hash: H256,
    pub height: u64,
    pub quality: U256,
}

#[derive(Debug, Clone, Copy)]
pub enum OnChainChangeMessage {
    EpochUpdate(u64),
    NewSampleTask(SampleTask),
    ClosedSampleTask(H256),
}

pub struct OnChainStatus {
    current_epoch: u64,
    sample_hash: H256,
}

pub struct DasWatcher {
    da_contract: DASample<DefaultMiddlewareInner>,
    da_signer: DASigners<DefaultMiddlewareInner>,

    sender: broadcast::Sender<OnChainChangeMessage>,
    last_status: Option<OnChainStatus>,
}

impl DasWatcher {
    pub async fn spawn(
        executor: TaskExecutor,
        provider: DefaultMiddleware,
        sender: broadcast::Sender<OnChainChangeMessage>,
        da_address: Address,
    ) -> Result<u64, String> {
        let da_contract = DASample::new(da_address, provider.clone());
        let da_signer = DASigners::new(
            Address::from_str(DA_SIGNER_ADDRESS).unwrap(),
            provider.clone(),
        );
        let epoch_number = da_signer
            .epoch_number()
            .call()
            .await
            .map_err(|e| format!("Failed to query sample context: {:?}", e))?
            .as_u64();
        info!("Epoch number at building stage {}", epoch_number);

        let das_watcher = Self {
            da_contract,
            da_signer,
            sender,
            last_status: None,
        };
        executor.spawn(
            async move { Box::pin(das_watcher.start()).await },
            "das_watcher",
        );
        Ok(epoch_number)
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
        let epoch_call = self.da_signer.epoch_number();
        let (sample_context_res, epoch_call_res) =
            tokio::join!(sample_call.call(), epoch_call.call());

        let sample_context: da_sample::SampleTask =
            sample_context_res.map_err(|e| format!("Failed to query sample task: {:?}", e))?;
        let epoch_number = epoch_call_res
            .map_err(|e| format!("Failed to query sample context: {:?}", e))?
            .as_u64();

        let last_status = self.last_status.as_ref();

        if last_status.map_or(true, |x| x.current_epoch != epoch_number) {
            self.sender
                .send(EpochUpdate(epoch_number))
                .map_err(|e| format!("Broadcast error: {:?}", e))?;
        }

        let sample_hash = H256(sample_context.sample_hash);
        if last_status.map_or(true, |x| x.sample_hash != sample_hash) {
            self.sender
                .send(NewSampleTask(SampleTask {
                    hash: sample_hash,
                    quality: sample_context.quality,
                    height: 0,
                }))
                .map_err(|e| format!("Broadcast error: {:?}", e))?;
        }

        if sample_context.num_submissions > TARGET_SUBMISSIONS as u64 * 2 {
            self.sender
                .send(ClosedSampleTask(sample_hash))
                .map_err(|e| format!("Broadcast error: {:?}", e))?;
        }

        Ok(OnChainStatus {
            current_epoch: epoch_number,
            sample_hash,
        })
    }
}
