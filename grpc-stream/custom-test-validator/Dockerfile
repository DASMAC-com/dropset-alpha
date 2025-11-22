FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive

ENV HOME="/root"
ENV PATH="${HOME}/.cargo/bin:${HOME}/.local/share/solana/install/active_release/bin:${PATH}"

RUN apt-get update -qq && apt-get upgrade -qq && apt-get install -qq --no-install-recommends \
    curl ca-certificates git jq pkg-config python3-pip \
    build-essential libclang-dev libssl-dev libudev-dev zlib1g-dev llvm clang cmake \
    libprotobuf-dev protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

RUN curl -sSfL https://sh.rustup.rs -o /tmp/rustup.sh \
 && sh /tmp/rustup.sh -y \
 && rustup component add rustfmt clippy

ENV HOME="/root"
ENV PATH="${HOME}/.cargo/bin:/opt/agave/target/release:/opt/yellowstone/target/release:${PATH}"

RUN git clone --branch "v2.3.10" --depth 1 https://github.com/anza-xyz/agave /opt/agave \
 && cd /opt/agave \
 && cargo build --release -p solana-cli -p solana-keygen \
 && cargo build --release --bin solana-test-validator

RUN git clone --branch "geyserV2-v0.1.0-rc1+solana.2.3.6" --depth 1 https://github.com/rpcpool/yellowstone-grpc /opt/yellowstone \
 && cd /opt/yellowstone \
 && cargo build --release -p yellowstone-grpc-geyser

WORKDIR /workdir
VOLUME /workdir
EXPOSE 8899 8900 10000

# Binaries now on PATH:
# - solana-test-validator, solana, solana-keygen from /opt/agave/target/release
# - libyellowstone_grpc_geyser.so from /opt/yellowstone/target/release
CMD ["/bin/bash"]
