# What is Dropset?

Dropset is a fully on-chain central limit order book (CLOB) built for Solana. It lets traders post limit orders and execute market orders against a shared order book that lives entirely on-chain — no off-chain matching engine, no hybrid custody model.

## Why a CLOB on Solana?

Most DeFi exchanges rely on AMMs (automated market makers), where liquidity is pooled and prices are determined by a constant-product formula. CLOBs work differently: makers post orders at specific prices, and takers fill them. This model is closer to how traditional exchanges like NYSE or Binance operate, and it offers tighter spreads and more capital efficiency for sophisticated market participants.

Solana's combination of high throughput and low latency makes it one of the few chains where a fully on-chain CLOB is practical. Dropset is designed to take advantage of that.

## What can you build with Dropset?

- **Market makers** — post and manage resting limit orders, implement spread strategies
- **Taker clients** — send market orders that fill against the book
- **Arbitrage bots** — monitor the book and react to price dislocations
- **Composable protocols** — CPI into Dropset from other on-chain programs via the shared interface layer

## How it fits together

Dropset is structured as a Rust workspace with clearly separated concerns:

| Component            | What it does                                            |
| -------------------- | ------------------------------------------------------- |
| `dropset-program`    | The on-chain Solana program                             |
| `dropset-interface`  | Client-agnostic instruction schemas and account types   |
| `instruction-macros` | Proc macros that generate typed builders and validators |
| `dropset-client`     | Rust client for local testing and RPC integration       |
| `ts-sdk`             | TypeScript SDK for building web and bot clients         |
| `services`           | Experimental faucet, maker-bot, and taker-bot           |

The next section walks through how these pieces connect at the architecture level.
