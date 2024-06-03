#[macro_use]
extern crate tracing;

mod context;

use std::{error::Error, net::SocketAddr, str::FromStr, sync::Arc};

use anyhow::{anyhow, Result};

use chain_state::{
    da_handler::start_da_monitor, signers_handler::start_epoch_registration, ChainState,
};
use grpc::run_server;

use tokio::signal;
use tracing::Level;

use crate::context::Context;

async fn start_grpc_server(chain_state: Arc<ChainState>, ctx: &Context) -> Result<()> {
    let db = ctx.db.clone();
    let signer_private_key = ctx.signer_private_key;
    let grpc_listen_address = ctx.grpc_listen_address.clone();
    let encoder_params_dir = ctx.encoder_params_dir.clone();
    let max_ongoing_sign_request = ctx.max_ongoing_sign_request;
    info!("starting grpc server at {:?}", grpc_listen_address);
    tokio::spawn(async move {
        run_server(
            db,
            chain_state,
            signer_private_key,
            SocketAddr::from_str(&grpc_listen_address).unwrap(),
            encoder_params_dir,
            max_ongoing_sign_request,
        )
        .await
        .map_err(|e| anyhow!(e.to_string()))
        .unwrap();
    });
    Ok(())
}

async fn setup_chain_state(ctx: &Context) -> Result<Arc<ChainState>> {
    let chain_state = Arc::new(
        ChainState::new(
            &ctx.eth_rpc_url,
            ctx.da_entrance_address,
            ctx.transactor.clone(),
            ctx.db.clone(),
        )
        .await?,
    );
    chain_state
        .check_signer_registration(ctx.signer_private_key, ctx.socket_address.clone())
        .await?;
    start_epoch_registration(chain_state.clone(), ctx.signer_private_key);
    start_da_monitor(chain_state.clone(), ctx.start_block_number).await?;
    Ok(chain_state)
}

async fn start_server(ctx: Context) -> Result<()> {
    let chain_state = setup_chain_state(&ctx).await?;
    start_grpc_server(chain_state.clone(), &ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // enable backtraces
    std::env::set_var("RUST_BACKTRACE", "1");

    // CLI, config
    let ctx = Context::new().await?;

    // rayon
    if let Some(num_threads) = ctx.max_verify_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap();
    }

    // tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(&ctx.log_level).unwrap())
        .init();

    // start server
    start_server(ctx).await?;

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("terminate signal received, stopping..");
        },
    }

    Ok(())
}
