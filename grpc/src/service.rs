#![allow(unused)]

use crate::service::signer::signer_server::{Signer, SignerServer};
use crate::service::signer::{BatchSignReply, BatchSignRequest};
use std::time::Instant;
use tonic::{Code, Request, Response, Status};
use tracing::info;

pub mod signer {
    tonic::include_proto!("signer");
}

pub struct SignerService {}

impl SignerService {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl Signer for SignerService {
    async fn batch_sign(
        &self,
        request: Request<BatchSignRequest>,
    ) -> Result<Response<BatchSignReply>, Status> {
        todo!();
        /*
        let remote_addr = request.remote_addr();
        let request_content = request.into_inner();
        info!(
            "Received request from {:?}, data length: {:?}",
            remote_addr,
        );
        */
    }
}

impl SignerService {}
