#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"

run_bench() {
    local label="$1"
    local program="$2"
    local features="$3"
    local test_name="$4"

    echo "=== $label ==="
    cd "$ROOT_DIR/cu-bench/programs/$program"
    build_output=$(cargo build-sbf --features "$features" --no-default-features 2>&1) || { echo "$build_output"; exit 1; }
    cd "$ROOT_DIR"

    output=$(cargo test -p cu-bench-tests --test "$test_name" --quiet -- --nocapture 2>&1) || { echo "$output"; exit 1; }
    echo "$output" | grep "Compute units consumed"
}
