use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use ethers::providers::{Http, HttpRateLimitRetryPolicy, RetryClient, RetryClientBuilder};
use ethers::types::H256;
use ethers::{
    prelude::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use reqwest::Url;

pub type DefaultMiddleware = Arc<DefaultMiddlewareInner>;
pub type DefaultMiddlewareInner = SignerMiddleware<Provider<RetryClient<Http>>, LocalWallet>;

pub const DA_SIGNER_ADDRESS: &str = "0x0000000000000000000000000000000000001000";

pub async fn make_provider(eth_rpc_url: &str, eth_private_key: &H256) -> Result<DefaultMiddleware> {
    let client = reqwest::ClientBuilder::default()
        .timeout(Duration::from_secs(60))
        .build()?;
    let http_client = Http::new_with_client(Url::parse(eth_rpc_url)?, client);
    let provider = Provider::new(
        RetryClientBuilder::default().build(http_client, Box::new(HttpRateLimitRetryPolicy)),
    );

    let local_wallet = LocalWallet::from_bytes(&eth_private_key[..])
        .map_err(|e| anyhow!("Invalid validator private key: {:?}", e))?;
    let chain_id = provider
        .get_chainid()
        .await
        .map_err(|e| anyhow!("Cannot get chain id: {:?}", e))?;

    let signer = local_wallet.with_chain_id(chain_id.as_u64());

    Ok(Arc::new(SignerMiddleware::new(provider, signer)))
}
