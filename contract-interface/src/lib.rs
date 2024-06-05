use ethers::prelude::abigen;

// run `cargo doc --open` to read struct definition

abigen!(DAEntrance, "./abis/DAEntrance.json");
abigen!(DASigners, "./abis/IDASigners.json");
abigen!(DASample, "./abis/IDASample.json");

// Local reference for dev
// abigen!(
//     DAEntrance,
//     "../../0g-da-contract/build/artifacts/contracts/DAEntrance.sol/DAEntrance.json"
// );
// abigen!(
//     DASigners,
//     "../../0g-da-contract/build/artifacts/contracts/interface/IDASigners.sol/IDASigners.json"
// );
// abigen!(
//     DASample,
//     "../../0g-da-contract/build/artifacts/contracts/interface/IDASample.sol/IDASample.json"
// );
