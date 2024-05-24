use ark_bn254::{Fq, G1Affine};

use ark_ff::Field;

pub fn map_to_g1(digest: Vec<u8>) -> G1Affine {
    let one = Fq::from(1);
    let three = Fq::from(3);
    let mut x: Fq = num_bigint::BigUint::from_bytes_be(&digest).into();
    loop {
        match (x * x * x + three).sqrt() {
            Some(y) => {
                return G1Affine::new(x, y);
            }
            None => x += one,
        }
    }
}

pub fn left_pad_zeros(x: u64, l: usize) -> Vec<u8> {
    let mut res = vec![0; l - 8];
    res.append(&mut x.to_be_bytes().to_vec());
    res
}
