#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
MANIFEST_PATH="$ROOT/bots/crates/market-maker/Cargo.toml"
KEYPAIR_FILE="$ROOT/bots/crates/market-maker/maker-keypair.json"

cd "$ROOT"

cargo build-sbf --manifest-path program/Cargo.toml
solana program deploy target/deploy/dropset.so --program-id test-keypair.json

# The example file outputs local market-info.json and maker-keypair.json files.
# They will be stored in the root lest the shell `cd`s to the market-maker crate
# prior to running the example.
(cd "$ROOT/bots/crates/market-maker" && cargo run --manifest-path "$MANIFEST_PATH" --example initialization_helper)

BASE_MINT=$(jq -r '.base_mint' "$ROOT/bots/crates/market-maker/market-info.json")
QUOTE_MINT=$(jq -r '.quote_mint' "$ROOT/bots/crates/market-maker/market-info.json")

cargo run --manifest-path "$MANIFEST_PATH" -- \
  --base-mint "$BASE_MINT" \
  --quote-mint "$QUOTE_MINT" \
  --pair EUR_USD \
  --target-base 8000 \
  --keypair "$KEYPAIR_FILE"
