#[macro_use]
extern crate tracing;

mod service;

use crate::service::signer::signer_server::SignerServer;
use ark_bn254::Fr;
use chain_state::ChainState;
pub use service::signer;
use service::SignerService;
use std::{net::SocketAddr, sync::Arc};
use storage::Storage;
use tokio::sync::RwLock;
use tonic::transport::Server;

const MESSAGE_SIZE_LIMIT: usize = 1024 * 1024 * 1024; // 1G

pub async fn run_server(
    db: Arc<RwLock<Storage>>,
    chain_state: Arc<ChainState>,
    signer_private_key: Fr,
    addr: SocketAddr,
    encoder_params_dir: String,
    max_ongoing_sign_request: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let signer_service = SignerService::new(
        db,
        chain_state,
        signer_private_key,
        encoder_params_dir,
        max_ongoing_sign_request,
    );
    info!("grpc server listening {:?}", addr);
    Server::builder()
        .add_service(
            SignerServer::new(signer_service)
                .max_decoding_message_size(MESSAGE_SIZE_LIMIT)
                .max_encoding_message_size(MESSAGE_SIZE_LIMIT),
        )
        .serve(addr)
        .await?;
    Ok(())
}
