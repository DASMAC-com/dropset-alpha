#!/usr/bin/env bash
source "$(dirname "$0")/common.sh"

run_bench "to_order_info (10 calls)" "to-order-info" "bench-program-A" "to_order_info" "v2"

