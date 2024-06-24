use std::collections::VecDeque;

use ethers::types::{H256, U256};
use tiny_keccak::{Hasher, Keccak};

use crate::constants::{LINE_BYTES, NUM_SUBLINES};

pub fn calculate_line_quality(
    sample_hash: H256,
    epoch: u64,
    quorum_id: u64,
    storage_root: [u8; 32],
    index: u16,
) -> [u8; 32] {
    // Create a Keccak256 hasher instance
    let mut hasher = Keccak::v256();

    // Encode the inputs and update the hasher
    hasher.update(&sample_hash.0);

    hasher.update(&[0u8; 24]);
    hasher.update(&epoch.to_be_bytes());

    hasher.update(&[0u8; 24]);
    hasher.update(&quorum_id.to_be_bytes());

    hasher.update(&storage_root);

    hasher.update(&(index as u64).to_be_bytes());

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    output
}

pub fn calculate_data_quality(
    line_quality: U256,
    subline_index: u64,
    data: &[[u8; 32]],
) -> [u8; 32] {
    // Create a Keccak256 hasher instance
    let mut hasher = Keccak::v256();

    // Encode the inputs and update the hasher
    hasher.update(&u256_to_bytes32(line_quality));

    hasher.update(&[0u8; 24]);
    hasher.update(&subline_index.to_be_bytes());

    for item in data {
        hasher.update(item);
    }

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    output
}

pub fn build_subline_merkle(data: &[[u8; 32]]) -> Vec<Vec<[u8; 32]>> {
    assert_eq!(data.len() * 32, LINE_BYTES);

    let flow_leaves = keccak_chunked(data, 256 / 32);

    let mut last_layer = flow_leaves;
    while last_layer.len() > NUM_SUBLINES {
        last_layer = keccak_chunked(&last_layer, 2);
    }

    let mut tree = VecDeque::new();

    while last_layer.len() > 1 {
        let next_layer = keccak_chunked(&last_layer, 2);
        let mut to_push_layer = next_layer;
        // last_layer is to_push_layer
        std::mem::swap(&mut last_layer, &mut to_push_layer);
        tree.push_front(to_push_layer);
    }

    tree.into()
}

pub fn keccak_chunked(input: &[[u8; 32]], chunk_size: usize) -> Vec<[u8; 32]> {
    input
        .chunks_exact(chunk_size)
        .map(|x| {
            let mut result = [0u8; 32];
            let mut keccak256 = Keccak::v256();
            for s in x {
                keccak256.update(s.as_ref());
            }
            keccak256.finalize(&mut result);
            result
        })
        .collect()
}

pub fn u256_to_bytes32(number: U256) -> [u8; 32] {
    let mut result = [0u8; 32];
    number.to_big_endian(&mut result);
    result
}

pub fn serialize_line(line: &[[u8; 32]]) -> Vec<u8> {
    if line.is_empty() {
        return vec![];
    }

    let ptr = &line[0][0] as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, line.len() * 32).to_vec() }
}
