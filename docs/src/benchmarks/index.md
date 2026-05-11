# Compute Unit Benchmarks

Dropset is built without Anchor and uses hand-written SBF assembly for critical paths. The result is dramatically lower compute unit consumption compared to other on-chain order books on Solana.

**Lower compute units = lower transaction fees and higher throughput.**

## Interactive comparison

Filter by instruction category and see the CU cost for each protocol side by side. The "Dropset advantage" column shows how many times more efficient Dropset is versus the best alternative.

<CuExplorer />

## How to run the benchmarks yourself

The benchmark suite lives in `cu-bench/` in the `dropset-alpha` repo. Each protocol has its own benchmark script:

```bash
# Dropset
bash cu-bench/dropset/run-bench.sh

# Phoenix
bash cu-bench/phoenix/run-bench.sh

# Manifest
bash cu-bench/manifest/run-bench.sh
```

Results are printed to stdout. See `cu-bench/README.md` for details on methodology and what each benchmark measures.

## Why compute units matter

On Solana, every transaction consumes compute units up to a per-transaction limit. Lower CU usage means:

- **Lower fees** — users pay less per trade
- **More instructions per transaction** — batch operations fit in a single tx
- **Higher theoretical throughput** — more transactions fit per block

For a high-frequency on-chain order book, CU efficiency is not a nice-to-have — it is the core engineering constraint.
