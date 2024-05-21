#[macro_use]
extern crate tracing;

mod context;

use std::{error::Error, net::SocketAddr, str::FromStr, sync::Arc};

use anyhow::{anyhow, Result};
use ark_bn254::Fr;
use chain_state::{signers_handler::start_epoch_registration, ChainState};
use grpc::run_server;
use storage::Storage;
use tokio::{signal, sync::RwLock};
use tracing::Level;

use crate::context::Context;

async fn start_grpc_server(
    db: Arc<RwLock<Storage>>,
    chain_state: Arc<ChainState>,
    signer_private_key: Fr,
    server_addr: String,
) -> Result<()> {
    info!("starting grpc server at {:?}", server_addr);
    tokio::spawn(async move {
        run_server(
            db,
            chain_state,
            signer_private_key,
            SocketAddr::from_str(&server_addr).unwrap(),
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
    Ok(chain_state)
}

async fn start_server(ctx: Context) -> Result<()> {
    let chain_state = setup_chain_state(&ctx).await?;
    start_grpc_server(
        ctx.db.clone(),
        chain_state.clone(),
        ctx.signer_private_key,
        ctx.grpc_listen_address.clone(),
    )
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // enable backtraces
    std::env::set_var("RUST_BACKTRACE", "1");

    // CLI, config
    let ctx = Context::new().await?;

    // tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(&ctx.log_level).unwrap())
        .init();

    // start server
    start_server(ctx).await?;

    tokio::select! {
        _ = signal::ctrl_c() => {},
    }

    Ok(())
}
