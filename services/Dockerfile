# Multi-stage build for dropset service binaries.
# Binary is selected via the SERVICE build arg (required).
# The resulting binary name is expected to be `dropset-<SERVICE>`.
#
# Build example:
#   docker build --build-arg SERVICE=maker-bot -t dropset-maker-bot .
#
# Build context must be the workspace root since the services have dependencies
# spread across the entire `dropset` workspace.

# ── cook/build ────────────────────────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-slim-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Declare the SERVICE ARG here (instead of above) so the cargo chef cook and
# apt-get layers are cached regardless of which SERVICE is being built.
ARG SERVICE
RUN test -n "$SERVICE" || { echo "ERROR: --build-arg SERVICE=<name> is required"; exit 1; }
RUN cargo build --release --bin "dropset-${SERVICE}"

# ── runtime ───────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
ARG SERVICE

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/dropset-${SERVICE} /app/service

ENTRYPOINT ["/app/service"]
