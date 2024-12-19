pub mod metrics;

use ark_bn254::{Fq, FqConfig, G1Affine};

use ark_ff::{BigInt, Field, MontConfig, One};

pub fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 == 0 {
        (0..s.len())
            .step_by(2)
            .map(|i| {
                s.get(i..i + 2)
                    .and_then(|sub| u8::from_str_radix(sub, 16).ok())
            })
            .collect()
    } else {
        None
    }
}

pub fn map_to_g1(digest: Vec<u8>) -> G1Affine {
    let mut x: Fq = num_bigint::BigUint::from_bytes_be(&digest).into();

    loop {
        match find_y_from_x(x) {
            Some(y) => {
                return G1Affine::new(x, y);
            }
            None => x += Fq::one(),
        }
    }
}

pub fn left_pad_zeros(x: u64, l: usize) -> Vec<u8> {
    let mut res = vec![0; l - 8];
    res.append(&mut x.to_be_bytes().to_vec());
    res
}

#[inline]
fn find_y_from_x(x: Fq) -> Option<Fq> {
    const SQRT_POW: BigInt<4> = match <FqConfig as MontConfig<4>>::MODULUS_PLUS_ONE_DIV_FOUR {
        Some(x) => x,
        None => panic!("Unsupport type"),
    };

    let beta = x * x * x + Fq::from(3);
    // y = sqrt(beta) = beta^((p+1) / 4)
    let y = beta.pow(SQRT_POW.as_ref());
    (y * y == beta).then_some(y)
}

#[test]
fn test_sqrt_consistency() {
    for i in 1u64..200000 {
        let x = Fq::from(i);
        assert_eq!((x * x * x + Fq::from(3)).sqrt(), find_y_from_x(x));
    }
}
