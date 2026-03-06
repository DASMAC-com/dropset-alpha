#!/usr/bin/env bash

################################################################################
# To run:
#
#   bash bots/crates/market-maker/market-maker.sh
#
# Prerequisites:
#   1. Copy the config template and fill in your OANDA API token:
#
#        cp bots/crates/market-maker/config.toml.example \
#           bots/crates/market-maker/config.toml
#
#      Then edit config.toml and set oanda_auth_token.
#
#   2. That's it. The script starts localnet if it isn't already running.
################################################################################

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
MANIFEST_PATH="$ROOT/bots/crates/market-maker/Cargo.toml"
KEYPAIR_FILE="$ROOT/bots/crates/market-maker/maker-keypair.json"
CONFIG_FILE="$ROOT/bots/crates/market-maker/config.toml"
CONFIG_EXAMPLE="$ROOT/bots/crates/market-maker/config.toml.example"

cd "$ROOT"

# ── Config check ────────────────────────────────────────────────────────────

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: config.toml not found."
    echo ""
    echo "Copy the template and fill in your OANDA API token:"
    echo ""
    echo "  cp $CONFIG_EXAMPLE \\"
    echo "     $CONFIG_FILE"
    echo ""
    exit 1
fi

if grep -q 'oanda_auth_token\s*=\s*"your-token-here"' "$CONFIG_FILE"; then
    echo "Error: oanda_auth_token in config.toml is still set to the placeholder."
    echo "Edit $CONFIG_FILE and replace it with your OANDA API token."
    echo ""
    exit 1
fi

# ── Localnet ─────────────────────────────────────────────────────────────────

if ! solana cluster-version --url localhost &>/dev/null 2>&1; then
    echo "Localnet not running. Starting solana-test-validator..."
    nohup solana-test-validator -r >/tmp/test-validator.log 2>&1 &

    for i in $(seq 1 6); do
        sleep 5
        if solana cluster-version --url localhost &>/dev/null 2>&1; then
            echo "Validator is up."
            break
        fi
        if [ "$i" -eq 6 ]; then
            echo "Error: validator failed to start after 30 seconds."
            echo "Check /tmp/test-validator.log for details."
            exit 1
        fi
    done
else
    echo "Localnet already running."
fi

# ── Build and deploy ─────────────────────────────────────────────────────────

cargo build-sbf --manifest-path program/Cargo.toml
solana program deploy target/deploy/dropset.so \
    --program-id test-keypair.json \
    --url localhost

# ── Market initialization ────────────────────────────────────────────────────

# Creates a market, writes maker-keypair.json, and patches base_mint/quote_mint
# into config.toml.
(cd "$ROOT/bots/crates/market-maker" && \
    cargo run --manifest-path "$MANIFEST_PATH" --example initialization_helper)

# ── Run the bot ───────────────────────────────────────────────────────────────

cargo run --manifest-path "$MANIFEST_PATH" -- \
    --keypair "$KEYPAIR_FILE"
