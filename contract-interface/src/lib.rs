use ethers::prelude::abigen;

// run `cargo doc --open` to read struct definition

abigen!(DAEntrance, "./abis/DAEntrance.json");
abigen!(DASigners, "./abis/IDASigners.json");
