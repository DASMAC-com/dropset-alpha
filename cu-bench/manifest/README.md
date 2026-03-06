# Description

```shell
pnpm run bench:manifest
# or
bash run-bench.sh
```

If you run `cargo test` directly, you must set `SBF_OUT_DIR` to the directory
containing `manifest.so`. Without it, CUs will appear extraordinarily low.

See the [top-level README](../README.md) for the test categories measured and
shared limitations.

## Program version

These benchmarks use the `manifest.so` program deployed on `mainnet-beta` as of
February 16, 2026. The `manifest` program as of that same date is at tag
[program-v3.0.10]. This tag is what the `manifest-dex` dependency in
`Cargo.toml` pins to.

You can also dump the current program deployed on mainnet yourself:

```shell
solana program dump MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms \
  manifest.so --url https://api.mainnet-beta.solana.com
```

Ensure the dumped `manifest.so` file is in the directory specified in your
shell's `SBF_OUT_DIR` env var when running the tests.

## Test framework

Uses `solana-program-test`. CU is measured by simulating each transaction and
reading the consumed units from the simulation result, then the transaction is
processed again to commit state changes.

## What's unique

The batched instruction is `BatchUpdate`. Place and cancel are tested as
separate `BatchUpdate` calls rather than a combined cancel+replace, so place
and cancel costs are reported independently.

The swap instruction (`Swap`) is tested separately from the batch tests.

The market is pre-expanded before each measured instruction to isolate
instruction cost from account reallocation cost.

[program-v3.0.10]: https://github.com/Bonasa-Tech/manifest/releases/tag/program-v3.0.10
