#!/usr/bin/env bash

# Tear down everything started by `run-services-on-localnet.sh`:
# - the three Dockerized services (faucet, maker-bot, taker-bot)
# - the backgrounded solana-test-validator

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

for svc in faucet maker-bot taker-bot; do
    compose_file="services/$svc/compose.yaml"
    if [ -f "$compose_file" ]; then
        echo "Stopping $svc..."
        docker compose -f "$compose_file" down --remove-orphans
    fi
done

if pgrep -x solana-test-validator >/dev/null; then
    echo "Stopping solana-test-validator..."
    pkill -x solana-test-validator
else
    echo "No solana-test-validator process running."
fi

echo "Done."
