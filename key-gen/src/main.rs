use ark_std::{rand::thread_rng, UniformRand};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = thread_rng();
    println!("{}", ark_bn254::Fr::rand(&mut rng));
    Ok(())
}
