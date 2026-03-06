# Description

```shell
bash cu-bench/dropset/run-bench.sh
```

The script builds the Dropset program from workspace source, then runs the
benchmark tests. See the [top-level README](../README.md) for the test
categories measured and shared limitations.

## Program version

Dropset builds the program from workspace source rather than loading a
pre-built binary. The measured version is whatever is currently checked out in
the workspace.

## Test framework

Uses [Mollusk](https://github.com/anza-xyz/mollusk), a lightweight local SVM
harness that is synchronous and does not run the full Solana runtime. Results
are deterministic and fast but slightly less representative than a full
`solana-program-test` environment.

## What's unique

**BatchReplace** is Dropset's primary batch instruction. It atomically cancels
the caller's entire resting position and places a new set in one instruction.
The batched tests exercise three scenarios with it: place only, cancel only,
and cancel-all + place-new (the true steady-state MM workload).

**Individual comparison**: the same place, cancel, and cancel+place workloads
are also run using separate `PostOrder` and `CancelOrder` instructions so you
can directly compare amortized batching cost versus individual-instruction cost.

**MarketOrder multi-maker**: the swap test automatically provisions additional
makers when the requested fill count exceeds the per-user order limit, so fill
depth is not artificially bounded.

## Limitations

See the [top-level README](../README.md) for shared limitations. Additionally:
Mollusk does not replicate every behavior of the full Solana runtime; treat
these numbers as a directional lower bound rather than a precise on-chain cost.
