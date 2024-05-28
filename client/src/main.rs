use std::error::Error;

use ark_bn254::{G1Affine, G1Projective};
use ark_ff::Fp;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use grpc::signer::{signer_client::SignerClient, BatchSignRequest, SignRequest};
use utils::hex_to_bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // enable backtraces
    std::env::set_var("RUST_BACKTRACE", "1");
    let data_root =
        &hex_to_bytes("1111111111111111111111111111111111111111111111111111111111111111").unwrap();
    let erasure_commitment = &G1Projective::new(Fp::from(1), Fp::from(2), Fp::from(1));
    let mut serialized_erasure_commitment: Vec<u8> = Vec::new();
    erasure_commitment
        .serialize_uncompressed(&mut serialized_erasure_commitment)
        .unwrap();
    let mut client = SignerClient::connect("http://0.0.0.0:34000").await.unwrap();
    let reply = client
        .batch_sign(BatchSignRequest {
            requests: vec![SignRequest {
                epoch: 1,
                quorum_id: 0,
                erasure_commitment: serialized_erasure_commitment,
                storage_root: data_root.clone(),
                encoded_slice: vec![],
            }],
        })
        .await
        .unwrap();
    println!("reply: {:?}", reply);
    let signature = G1Affine::deserialize_uncompressed(&*reply.into_inner().signatures[0]).unwrap();
    println!("signature: {:?}", signature);
    Ok(())
}
