# Description

```shell
pnpm run bench:phoenix
# or
bash run-bench.sh
```

If you run `cargo test` directly, you must set `SBF_OUT_DIR` to the directory
containing `phoenix.so`. Without it, CUs will appear extraordinarily low.

See the [top-level README](../README.md) for the test categories measured and
shared limitations.

## Program version

These benchmarks use the `phoenix.so` program deployed on `mainnet-beta` as of
February 16, 2026. The `master` branch for the `phoenix-v1` program as of
that same date is at commit [1820ad9]. This commit is the `rev` the
`phoenix-v1` dependency in `Cargo.toml` pins to.

You can also dump the current program deployed on mainnet yourself:

```shell
solana program dump PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY \
  phoenix.so --url https://api.mainnet-beta.solana.com
```

Ensure the dumped `phoenix.so` file is in the directory specified in your
shell's `SBF_OUT_DIR` env var when running the tests.

## Test framework

Uses `solana-program-test`. CU is measured by simulating each transaction and
reading the consumed units from the simulation result, then the transaction is
processed again to commit state changes.

## What's unique

**PlaceLimitOrder** (single order): Phoenix Legacy has a separate single-order
instruction distinct from its batch-order instruction, so both are measured.

The batched place instruction is `MultipleOrderPacket`. The batched cancel
instruction is `CancelAllOrdersWithFreeFunds`, which cancels all resting orders
for the caller in one call.

A combined **place-then-cancel** test measures both operations back-to-back
within the same test, so place and cancel CU are reported from the same
initial book state.

The swap instruction is an IOC (immediate-or-cancel) buy via the standard
`NewOrder` instruction.

Phoenix Legacy allocates its order book at market creation with a fixed capacity;
there is no runtime account growth to separate out, so no pre-expansion is
needed before measurements.

[1820ad9]: https://github.com/Ellipsis-Labs/phoenix-v1/commit/1820ad9208c0546be1e93b3adb534c46598e02cb
