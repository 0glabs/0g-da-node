mod service;

use crate::service::signer::signer_server::SignerServer;
use service::SignerService;
use std::net::SocketAddr;
use tonic::transport::Server;

const MESSAGE_SIZE_LIMIT: usize = 1024 * 1024 * 1024; // 1G

pub async fn run_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let signer_service = SignerService::new();
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
