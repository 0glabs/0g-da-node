# 0g-DA-signer

## Download Encoding Params

Download verifier params before running server by
```sh
./dev_support/download_params.sh
```

## Configuration

Create a `config.toml` file and set the following field to proper values:
```
log_level = "info"

data_path = "./db/"

# path to downloaded params folder
encoder_params_dir = "params/" 

# grpc server listen address
grpc_listen_address = "0.0.0.0:34000"
# chain eth rpc endpoint
eth_rpc_endpoint = ""
# public grpc service socket address to register in DA contract
# ip:34000 (keep same port as the grpc listen address)
# or if you have dns, fill your dns
socket_address = "<public_ip/dns>"

# data availability contract to interact with
da_entrance_address = ""
# deployed block number of da entrance contract
start_block_number = 0 

# signer BLS private key
signer_bls_private_key = ""
# signer eth account private key
signer_eth_private_key = ""
# miner eth account private key, (could be the same as `signer_eth_private_key`, but not recommended)
miner_eth_private_key = ""

# whether to enable data availability sampling
enable_das = "true"
```

# Build from source
```
cargo build --release
./target/release/server --config config.toml
```

# Run in Docker
set following fields in your `config.toml`:
```
data_path = "/data"
encoder_params_dir = "/params"
```
build docker image and run:
```
docker build -t 0g-da-node .
docker run -d -v <YOUR_DATA_FOLDER>:/data -v <YOUR_CONFIG_TOML>:/config.toml --name <CONTAINER_NAME> --net=host 0g-da-node
```