FROM rust
VOLUME ["/data"]
COPY . .
RUN apt-get update && apt-get install -y protobuf-compiler libclang-dev clang
RUN ./dev_support/download_params.sh
RUN cargo build --release
CMD ["./target/release/server", "--config", "config.toml"]