# Multi-stage build for dropset service binaries.
# Binary is selected via the SERVICE build arg (default: maker-bot).
# The resulting binary name is expected to be `dropset-<SERVICE>`.
#
# Build example:
#   docker build --build-arg SERVICE=maker-bot -t dropset-maker-bot .
#
# Build context must be the workspace root since the services have dependencies
# spread across the entire `dropset` workspace.

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

WORKDIR /app
COPY --from=builder /build/target/release/dropset-${SERVICE} /app/service

ENTRYPOINT ["/app/service"]
