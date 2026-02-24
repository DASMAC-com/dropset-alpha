#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"

export RUST_LOG=warn

cd "$ROOT"
cargo build-sbf --manifest-path program/Cargo.toml
cargo test --quiet -p cu-bench-dropset -- --nocapture --test-threads=1 --format=terse 2>&1
