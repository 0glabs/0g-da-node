#[macro_use]
extern crate tracing;

pub mod da_handler;
pub mod signers_handler;
pub mod transactor;

use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::Result;

use chain_utils::DA_SIGNER_ADDRESS;
use contract_interface::{DAEntrance, DASigners};
use ethers::{
    providers::{Http, HttpRateLimitRetryPolicy, Provider, RetryClient, RetryClientBuilder},
    types::H160,
};
use storage::Storage;
use tokio::sync::{Mutex, RwLock};
use transactor::Transactor;

pub struct ChainState {
    provider: Arc<Provider<RetryClient<Http>>>,
    pub da_entrance: Arc<DAEntrance<Provider<RetryClient<Http>>>>,
    da_signers: Arc<DASigners<Provider<RetryClient<Http>>>>,
    transactor: Arc<Mutex<Transactor>>,
    signer_address: H160,
    db: Arc<RwLock<Storage>>,
}

impl ChainState {
    pub async fn new(
        eth_rpc_url: &str,
        da_entrance_address: H160,
        transactor: Arc<Mutex<Transactor>>,
        db: Arc<RwLock<Storage>>,
    ) -> Result<Self> {
        let provider = Arc::new(Provider::new(
            RetryClientBuilder::default()
                .rate_limit_retries(100)
                .timeout_retries(100)
                .initial_backoff(Duration::from_millis(500))
                .build(
                    Http::from_str(eth_rpc_url)?,
                    Box::new(HttpRateLimitRetryPolicy),
                ),
        ));
        let da_entrance = Arc::new(DAEntrance::new(da_entrance_address, provider.clone()));
        let da_signers = Arc::new(DASigners::new(
            H160::from_str(DA_SIGNER_ADDRESS).unwrap(),
            provider.clone(),
        ));
        let signer_address = transactor.lock().await.signer_address();
        Ok(Self {
            provider,
            da_entrance,
            da_signers,
            transactor,
            signer_address,
            db,
        })
    }
}
