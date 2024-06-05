use chain_utils::{DefaultMiddleware, DefaultMiddlewareInner};
use contract_interface::{da_sample::SampleResponse, DASample};
use ethers::{abi::Address, contract::ContractCall, providers::PendingTransaction};
use task_executor::TaskExecutor;
use tokio::sync::{broadcast, mpsc};

use crate::watcher::OnChainChangeMessage;

pub struct DasSubmitter {
    da_contract: DASample<DefaultMiddlewareInner>,
    on_chain_receiver: broadcast::Receiver<OnChainChangeMessage>,
    submission_receiver: mpsc::UnboundedReceiver<SampleResponse>,
}

impl DasSubmitter {
    pub fn spawn(
        executor: TaskExecutor,
        provider: DefaultMiddleware,
        on_chain_receiver: broadcast::Receiver<OnChainChangeMessage>,
        submission_receiver: mpsc::UnboundedReceiver<SampleResponse>,
        da_address: Address,
    ) {
        let da_contract = DASample::new(da_address, provider.clone());
        let submitter = Self {
            da_contract,
            submission_receiver,
            on_chain_receiver,
        };
        executor.spawn(
            async move { Box::pin(submitter.start()).await },
            "das_submitter",
        );
    }

    async fn start(mut self) {
        use OnChainChangeMessage::*;

        let mut enabled = true;
        let mut current_task = None;

        loop {
            tokio::select! {
                biased;

                msg = self.on_chain_receiver.recv(), if enabled => {
                    match msg {
                        Ok(NewSampleTask(task)) => {
                            current_task = Some(task);
                        },
                        Ok(ClosedSampleTask(hash)) => {
                            if current_task.map_or(false, |t| t.hash == hash) {
                                current_task = None;
                            }
                        },
                        Ok(_) => {}
                        Err(broadcast::error::RecvError::Closed)=>{
                            warn!(target: "Submitter", "On-chain status channel closed.");
                            self.submission_receiver.close();
                            enabled = false;
                        }
                        Err(broadcast::error::RecvError::Lagged(n))=>{
                            warn!(target: "Submitter", number = n, "On-chain status channel lagged.");
                        }
                    }
                },

                msg = self.submission_receiver.recv(), if enabled && current_task.is_some() => {
                    if msg.is_none() {
                        warn!(target: "Submitter", "Submission channel closed.");
                    }

                    let response = msg.unwrap();
                    if response.sample_height == current_task.unwrap().height {
                        let _ = self.submit_response(response).await;
                    }
                }
            }
        }
    }

    async fn submit_response(&self, response: SampleResponse) -> Result<(), ()> {
        info_span!(target: "Submitter", "submit_response");

        let submission_call: ContractCall<_, _> =
            self.da_contract.submit_sampling_response(response).legacy();
        debug!(target: "Submitter", transaction = ?submission_call.tx, "Construct transaction");

        let estimate_gas = submission_call.estimate_gas().await;
        debug!(target: "Submitter", result = ?estimate_gas, "Estimate gas");

        let pending_transaction: PendingTransaction<'_, _> =
            submission_call.send().await.map_err(|e| {
                warn!(target: "Submitter", error = ?e, "Fail to send sample response transaction");
            })?;
        debug!(target: "Submitter", hash = ?pending_transaction.tx_hash(), "Send sample transaction");

        let receipt = pending_transaction
            .await
            .map_err(|error| {
                warn!(target: "Submitter", ?error, "Fail to execute sample transaction");
            })?
            .ok_or_else(|| {
                warn!(target: "Submitter", "Transaction not executed after 3 retires");
            })?;

        info!(target: "Submitter", "Submit Sample transaction success");
        debug!(target: "Submitter", ?receipt, "Receipt");
        Ok(())
    }
}