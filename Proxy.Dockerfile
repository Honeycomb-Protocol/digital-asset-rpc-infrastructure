FROM rust:1.75-bullseye AS builder

RUN cargo install wasm-pack@0.12.1

RUN mkdir /rust
COPY Cargo.toml /rust
COPY Cargo.lock /rust
COPY .git /rust/.git
COPY core /rust/core
COPY das_api /rust/das_api
COPY digital_asset_types /rust/digital_asset_types
COPY metaplex-rpc-proxy /rust/metaplex-rpc-proxy
COPY migration /rust/migration
COPY grpc-ingest /rust/grpc-ingest
COPY ops /rust/ops
COPY program_transformers /rust/program_transformers
COPY tools /rust/tools
COPY blockbuster /rust/blockbuster

WORKDIR /rust/metaplex-rpc-proxy
RUN mkdir /rust/wasm-out/
RUN --mount=type=cache,target=/rust/target,id=das-wasm \
    wasm-pack build --release && cp /rust/target/wasm32-unknown-unknown/release/metaplex_rpc_proxy.wasm /rust/wasm-out/

FROM envoyproxy/envoy:v1.24.0
COPY --from=builder /rust/wasm-out/metaplex_rpc_proxy.wasm /etc/rpc_proxy.wasm
RUN apt-get update && apt-get install -y ca-certificates
ENTRYPOINT /usr/local/bin/envoy -c /etc/envoy.yaml -l trace --service-cluster proxy
