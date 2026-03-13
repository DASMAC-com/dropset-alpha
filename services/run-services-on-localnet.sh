#!/usr/bin/env bash

set -euo pipefail

if ! command -v pnpm &>/dev/null; then
  echo "pnpm is not installed. Please install it: https://pnpm.io/installation"
  exit 1
fi

# Check if `--force` was passed to overwrite existing keypair files.
FORCE_FLAG=""
if [[ "${1:-}" == "--force" ]]; then
  FORCE_FLAG="--force"
fi

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Start the Solana test validator.
# Note that this doesn't reset an existing localnet.
if ! solana cluster-version --url localhost &>/dev/null 2>&1; then
    echo "Localnet not running. Starting solana-test-validator..."
    nohup solana-test-validator >/tmp/test-validator.log 2>&1 &
    VALIDATOR_PID=$!
    echo "  A validator is now running in the background (PID $VALIDATOR_PID)."
    echo "  It will survive terminal close. To stop it: kill $VALIDATOR_PID"
    echo "  Or to stop any validator: pkill -x solana-test-validator"

    for i in $(seq 1 6); do
        sleep 5
        if solana cluster-version --url localhost &>/dev/null 2>&1; then
            break
        fi
        if [ "$i" -eq 6 ]; then
            echo "Error: validator failed to start after 30 seconds."
            echo "Check /tmp/test-validator.log for details."
            exit 1
        fi
    done
fi

# Creates a market, writes to the faucet, taker, and maker keypair files, and
# patches the base and quote mints into their respective toml config files.
cargo run -p dropset-services-shared --example initialization_helper -- $FORCE_FLAG

pnpm run services:maker:docker
pnpm run services:taker:docker
pnpm run services:faucet:docker

echo "Waiting to see if services are healthy..."
sleep 3

for service in maker-bot taker-bot faucet; do
    status=$(docker inspect --format='{{.State.Status}}' $service 2>/dev/null)
    if [ "$status" = "exited" ] || [ "$status" = "restarting" ]; then
        echo "Error: $service failed to start (status: $status):"
        docker logs --tail 5 $service 2>&1
        exit 1
    fi
done
