# Instruction Reference

Quick-reference cards for all 10 Dropset instructions. Click any card to expand its full account list, emitted events, and notes.

Filter by category to focus on a specific area of the protocol.

<InstructionCards />

## How to read a card

**CU** — the approximate compute unit cost of this instruction on a Solana localnet benchmark. See [Benchmarks](/benchmarks/) for full comparisons against Phoenix and Manifest.

**Accounts** — every account the instruction requires, with mutability and signer flags:
- `mut` — the instruction writes to this account
- `signer` — this account must sign the transaction
- `PDA` — this is a program-derived address, computed deterministically from seeds

**Emits** — events written to the market's event queue when this instruction executes. Read via `toMarketViewAll()` or consume with `flush_events`.

## Account conventions

All PDAs in Dropset are derived from seeds defined in `dropset-interface/`, so any client can compute them without fetching first:

| Account | Seeds |
|---|---|
| Seat | `[market, trader]` |
| Trader balance | `[market, trader]` |
| Base vault | `[market, "base"]` |
| Quote vault | `[market, "quote"]` |

## Further reading

- [Program Structure](/architecture/program-structure) — prose explanation of each instruction
- [On-Chain Accounts](/architecture/accounts) — how account data is laid out
- [Post an Order](/sdk/post-order) — SDK walkthrough using the real instruction builders
