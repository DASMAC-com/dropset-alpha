# syntax=docker/dockerfile:1
#
# Multi-stage build for dropset service binaries.
# Binary is selected via the SERVICE build arg (default: maker-bot).
# The resulting binary name is expected to be `dropset-<SERVICE>`.
#
# Build example:
#   docker build --build-arg SERVICE=maker-bot -t dropset-maker-bot .
#
# Build context must be the workspace root (the directory containing this file).

ARG SERVICE=maker-bot

# ── build ─────────────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

ARG SERVICE

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY . .

RUN cargo build --release --bin "dropset-${SERVICE}"

# ── runtime ───────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

ARG SERVICE

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/dropset-${SERVICE} /usr/local/bin/app

ENTRYPOINT ["/usr/local/bin/app"]
