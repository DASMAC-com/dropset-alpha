# Getting Started

This guide gets you from zero to running the Dropset program locally against a Solana test validator.

## Prerequisites

| Tool       | Version      | Install                                                       |
| ---------- | ------------ | ------------------------------------------------------------- |
| Rust       | stable       | [rustup.rs](https://rustup.rs)                                |
| Solana CLI | latest       | [solana.com/docs](https://solana.com/docs/intro/installation) |
| Node.js    | see `.nvmrc` | [nodejs.org](https://nodejs.org)                              |
| pnpm       | latest       | `npm install -g pnpm`                                         |

Check your versions:

```bash
rustc --version
solana --version
node --version
pnpm --version
```

::: tip Node version
The repo pins Node via `.nvmrc`. If you use `nvm`, run `nvm use` in the repo root to switch automatically.
:::

## 1. Clone the repo

```bash
git clone https://github.com/DASMAC-com/dropset-alpha.git
cd dropset-alpha
```

## 2. Install Node dependencies

```bash
pnpm install
```

Rust workspace dependencies are fetched automatically by Cargo on first build.

## 3. Build the program

```bash
cargo build
```

To build just the on-chain program:

```bash
cargo build-sbf -p dropset-program
```

## 4. Run a local validator

In a separate terminal, start a local Solana test validator and leave it running:

```bash
solana-test-validator
```

## 5. Configure Solana CLI for localhost

```bash
solana config set --url localhost
```

## 6. Deploy the program

```bash
solana program deploy target/sbf-solana-solana/release/dropset_program.so
```

Note the **program ID** printed after deployment — you'll need it to interact with the program.

## 7. Run the tests

```bash
# Rust tests
cargo test

# TypeScript SDK tests
pnpm --filter ts-sdk test
```

## 8. Run the docs locally

```bash
make docs
```

This runs `cd docs && npm install && npx vitepress dev --open` and opens the docs site in your browser.

## Next steps

- [Core Concepts](/introduction/core-concepts) — markets, seats, lots, ticks
- [Architecture Overview](/architecture/overview) — how the workspace fits together
- [Program Structure](/architecture/program-structure) — all 10 instructions explained
- [TypeScript SDK](/sdk/overview) — build a client with the TS SDK
