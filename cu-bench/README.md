# CU Benchmarks

These crates measure the Solana compute unit (CU) consumption of three DEX
programs: **Dropset**, **Manifest**, and **Phoenix Legacy**. Each crate is
independent; see its own README for setup and program-version details.

| Crate | Program | README |
|-------|---------|--------|
| [`dropset/`](dropset/) | [Dropset](https://github.com/DASMAC-com/dropset-alpha) (this repo) | [dropset/README.md](dropset/README.md) |
| [`manifest/`](manifest/) | [Manifest](https://github.com/Bonasa-Tech/manifest) | [manifest/README.md](manifest/README.md) |
| [`phoenix/`](phoenix/) | [Phoenix Legacy](https://github.com/Ellipsis-Labs/phoenix-v1) | [phoenix/README.md](phoenix/README.md) |

## Running

Each crate has a `run-bench.sh` that handles build and environment setup. Run
from the repo root:

```shell
bash cu-bench/dropset/run-bench.sh
bash cu-bench/manifest/run-bench.sh
bash cu-bench/phoenix/run-bench.sh
```

## Test categories

All three suites measure the same broad categories of operations using each
program's equivalent instructions:

- **Single-instruction**: one operation measured exactly once (deposit,
  withdraw, and placing or cancelling a single order).
- **Batched**: a single instruction that processes N items (orders placed,
  cancels, or a combination). Total CU is divided by N to get the amortized
  per-item cost. Run at several batch sizes, always including a single-item
  baseline.
- **Swap / market order**: N resting orders placed as setup (not measured),
  then a single taker instruction that crosses all of them. Total CU divided
  by N gives the amortized per-fill cost. Run at several fill counts.

Each crate's README describes which instructions map to each category and
anything unique about that program's test suite.

## Design intent

### Measuring steady-state instruction cost

The goal is the per-operation CU cost that a high-frequency market maker would
see on a running market, not first-time initialization or worst-case lookup
costs. Tests are structured to approximate that steady-state hot path.

### Pre-expanded markets (Dropset and Manifest)

Solana accounts must be reallocated when an order book grows beyond its current
allocation, and that reallocation itself costs CUs. To isolate instruction cost
from growth cost, Dropset and Manifest **pre-expand the market account** before
running any measured instruction. The reported CUs reflect the instruction
itself under steady-state conditions.

Phoenix Legacy predates Solana's support for dynamic account resizing, so it takes a
different approach: the order book is allocated at a fixed maximum capacity at
market creation time. This avoids any runtime growth but requires paying rent
on the full allocation upfront. Since there is nothing to grow at runtime,
there is nothing to pre-expand before measuring.

## Limitations

All three suites share the same fundamental constraints:

- **Nearly empty order book.** Each test starts from a fresh market with a
  single trader. Real order books have many resting orders and traders, which
  increases account sizes and traversal costs. These results are a lower-bound,
  not an average or worst-case.

- **Optimized-client hints.** Tests pass exact order hints wherever the program
  accepts them. These are not worst-case measurements; they reflect what a
  well-optimized client sees, not a naive client that scans accounts.

- **Baseline only.** Treat these numbers as a cost floor and a cross-program
  comparison baseline. Cross-reference with on-chain transaction data for
  production estimates.
