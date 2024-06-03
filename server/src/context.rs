use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use ark_bn254::Fr;

use chain_state::transactor::Transactor;
use config::Config;
use ethers::providers::{Http, HttpRateLimitRetryPolicy, RetryClientBuilder};
use ethers::types::H160;
use ethers::{
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use storage::Storage;
use tokio::sync::{Mutex, RwLock};

mod cli {
    use clap::{arg, command, Command};

    pub fn cli_app<'a>() -> Command<'a> {
        command!()
            .arg(arg!(-c --config <FILE> "Sets a custom config file"))
            .allow_external_subcommands(true)
    }
}

pub struct Context {
    pub log_level: String,
    pub encoder_params_dir: String,
    pub grpc_listen_address: String,
    pub max_ongoing_sign_request: Option<u64>,
    pub max_verify_threads: Option<usize>,
    pub socket_address: String,
    pub eth_rpc_url: String,
    pub start_block_number: u64,
    pub da_entrance_address: H160,
    pub transactor: Arc<Mutex<Transactor>>,
    pub signer_private_key: Fr,
    pub db: Arc<RwLock<Storage>>,
}

impl Context {
    pub async fn new() -> Result<Self> {
        let matches = cli::cli_app().get_matches();
        if let Some(config_file) = matches.value_of("config") {
            let settings = Config::builder()
                .add_source(config::File::with_name(config_file))
                .build()?;
            // ethereum keys
            let eth_rpc_url = settings.get_string("eth_rpc_endpoint")?;
            let provider = Provider::new(
                RetryClientBuilder::default()
                    .rate_limit_retries(100)
                    .timeout_retries(100)
                    .initial_backoff(Duration::from_millis(500))
                    .build(
                        Http::from_str(&eth_rpc_url)?,
                        Box::new(HttpRateLimitRetryPolicy),
                    ),
            );
            let signer = (LocalWallet::from_str(&settings.get_string("validator_private_key")?))?
                .with_chain_id(provider.get_chainid().await?.as_u64());
            let transactor: Arc<Mutex<Transactor>> =
                Arc::new(Mutex::new(Transactor::new(signer.clone(), &eth_rpc_url)?));
            let da_entrance_address = H160::from_str(&settings.get_string("da_entrance_address")?)?;
            // bn254 keys
            let signer_private_key: Fr =
                Fr::from_str(&settings.get_string("signer_private_key")?).unwrap();
            // db
            let db = Arc::new(RwLock::new(Storage::new(
                settings.get_string("data_path")?,
            )?));

            Ok(Self {
                log_level: settings.get_string("log_level")?,
                grpc_listen_address: settings.get_string("grpc_listen_address")?,
                max_ongoing_sign_request: settings
                    .get_int("max_ongoing_sign_request")
                    .ok()
                    .map(|x| x as u64),
                max_verify_threads: settings
                    .get_int("max_verify_threads")
                    .ok()
                    .map(|x| x as usize),
                socket_address: settings.get_string("socket_address")?,
                eth_rpc_url,
                start_block_number: settings.get_int("start_block_number")? as u64,
                transactor,
                da_entrance_address,
                signer_private_key,
                db,
                encoder_params_dir: settings.get_string("encoder_params_dir")?,
            })
        } else {
            bail!(anyhow!("Config file missing!"));
        }
    }
}
