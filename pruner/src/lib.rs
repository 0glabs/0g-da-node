#[macro_use]
extern crate tracing;

use anyhow::Result;
use chain_state::ChainState;
use std::{sync::Arc, time::Duration};
use storage::misc_db::MiscDB;
use storage::slice_db::SliceDB;
use storage::Storage;
use tokio::{sync::RwLock, time::sleep};

pub async fn run_pruner(db: Arc<RwLock<Storage>>, chain_state: Arc<ChainState>) -> Result<()> {
    let maybe_progress = db.read().await.get_prune_progress().await?;
    match maybe_progress {
        Some(_) => {}
        None => {
            db.write().await.put_prune_progress(0).await?;
        }
    }
    loop {
        match prune(db.clone(), chain_state.clone()).await {
            Ok(pruned) => {
                info!("database pruned to epoch {:?}.", pruned);
            }
            Err(e) => {
                error!("failed to prune data, e={:?}", e);
            }
        }
        sleep(Duration::from_secs(600)).await;
    }
}

async fn prune(db: Arc<RwLock<Storage>>, chain_state: Arc<ChainState>) -> Result<u64> {
    let epoch = chain_state.da_signers.epoch_number().call().await?.as_u64();
    let epoch_window_size = chain_state
        .da_entrance
        .epoch_window_size()
        .call()
        .await?
        .as_u64();
    let mut pruned = db.read().await.get_prune_progress().await?.unwrap();
    while pruned + 1 + epoch_window_size < epoch {
        db.write().await.prune(pruned + 1).await?;
        db.write().await.put_prune_progress(pruned + 1).await?;
        pruned += 1;
    }

    Ok(pruned)
}
