
#!/usr/bin/env bash

set -euo pipefail

f ! command -v pnpm &>/dev/null; then
  echo "pnpm is not installed. Please install it: https://pnpm.io/installation"
  exit 1
fi

ROOT="$(git rev-parse --show-toplevel)"
MANIFEST_PATH="$ROOT/services/Cargo.toml"

cd "$ROOT"

# Start the Solana test validator.
# Note this doesn't doesn't reset an existing localnet.
if ! solana cluster-version --url localhost &>/dev/null 2>&1; then
    echo "Localnet not running. Starting solana-test-validator..."
    nohup solana-test-validator >/tmp/test-validator.log 2>&1 &
    VALIDATOR_PID=$!
    echo "  Note: validator is running in the background (PID $VALIDATOR_PID)."
    echo "  It will survive terminal close. To stop it: kill $VALIDATOR_PID"
    echo "  Or to stop any validator: pkill -x solana-test-validator"

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

pnpm run build:dropset
pnpm run deploy

# Creates a market, writes maker-keypair.json, and patches base_mint/quote_mint
# into config.toml.
(cd "$ROOT/services" && \
    cargo run --manifest-path "$MANIFEST_PATH" --example initialization_helper)

pnpm run services:maker:docker
pnpm run services:taker:docker
pnpm run services:faucet:docker
