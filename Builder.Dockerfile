FROM rust:1.75-bullseye AS builder
RUN apt-get update -y && \
  apt-get install -y build-essential make git

ENV GIT_VERSION 2.43.0

RUN mkdir /rust
RUN mkdir /rust/bins
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
COPY blockbuster rust/blockbuster
WORKDIR /rust
RUN git config --system --add safe.directory '*'
ENV GIT_VERSION=111
RUN --mount=type=cache,target=/rust/target,id=das-rust \
  cargo build --release --bins && cp `find /rust/target/release -maxdepth 1 -type f | sed 's/^\.\///' | grep -v "\." ` /rust/bins

FROM rust:1.75-slim-bullseye as final
COPY --from=builder /rust/bins /das/
CMD echo "Built the DAS API bins!"
