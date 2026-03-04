#!/usr/bin/env bash
source "$(dirname "$0")/common.sh"

run_bench "Pack/Unpack" "pack-orders" "bench-program-A" "pack_orders" "v2"
echo ""
run_bench "Borsh"       "pack-orders" "bench-program-B" "pack_orders" "v2"

