# Architecture Overview

Dropset is structured as a Cargo workspace. Each crate has a single responsibility, and the dependency graph flows in one direction — from interface types at the bottom, through program logic in the middle, to clients and services at the top.

## Workspace layout

```
dropset-alpha/
├── program/                  # On-chain Solana program
├── interface/                # Shared instruction schemas + account types
├── instruction-macros/       # Proc macro crate (public API)
├── instruction-macros-impl/  # Proc macro implementation
├── client/                   # Rust client for testing + RPC
├── ts-sdk/                   # TypeScript SDK
├── price/                    # Price math utilities (also mirrored in ts-sdk)
├── services/
│   ├── faucet/               # Token faucet (devnet/testnet only)
│   ├── maker-bot/            # Avellaneda-Stoikov market maker
│   └── taker-bot/            # Random market order sender
├── cu-bench/                 # Compute unit benchmarks
├── codama-idl-gen/           # Codama IDL generation tooling
└── transaction-parser/       # Transaction parsing utilities
```

## Dependency flow

```
               ┌─────────────┐
               │  interface  │  ← instruction schemas, account layouts
               └──────┬──────┘
                      │
         ┌────────────┼────────────┐
         ▼            ▼            ▼
   ┌─────────┐  ┌─────────┐  ┌────────┐
   │ program │  │ client  │  │ ts-sdk │
   └─────────┘  └─────────┘  └────────┘
         │
┌────────┴────────┐
▼                 ▼
services       cu-bench
```

The `interface` crate sits at the bottom. It has no dependency on the program itself, so it can be imported by both on-chain programs (via CPI) and off-chain clients without pulling in program logic.

## No Anchor

Dropset does **not** use Anchor. Instead it uses a custom `instruction-macros` system that generates:

- Strongly-typed account context structs
- Instruction builders and discriminators
- Validation scaffolding

This gives the team precise control over compute unit usage and avoids Anchor's runtime overhead — a meaningful difference for a high-frequency on-chain order book.

## The program layer

`program/src/` is organized into four folders:

| Folder | Purpose |
|---|---|
| `instructions/` | One file per instruction — the handler logic |
| `context/` | One file per instruction — account validation and loading |
| `shared/` | Shared utilities used across multiple instructions |
| `validation/` | Safety contracts and constraint checking |

Plus top-level files:

| File | Purpose |
|---|---|
| `entrypoint.rs` | Routes incoming instructions to handlers |
| `events.rs` | Event type definitions emitted by the program |
| `lib.rs` | Crate root |

## The interface crate

`dropset-interface` is the contract between the program and its clients. It contains instruction discriminators, parameter schemas, and on-chain account struct definitions. Because it carries no program runtime dependency, it can be imported anywhere — including from other on-chain programs via CPI.

The TypeScript SDK's `generated/` folder is produced from this crate's IDL via Codama.

## The TypeScript SDK

`ts-sdk/src/` is structured in layers:

```
ts-sdk/src/
├── generated/          # Codama IDL-generated — do not edit by hand
├── dropset-interface/  # Hand-written TS layer on top of generated
├── price/              # Price math (ported from the Rust price crate)
├── rust-types/         # TypeScript mirrors of Rust types
├── types/              # Shared TS type definitions
├── utils/              # Utility functions
├── const.ts            # Program ID and other constants
└── index.ts            # Public SDK entry point
```

The `generated/` layer is produced automatically and should never be edited by hand. The `dropset-interface/` layer wraps it with a more ergonomic API.
