# On-Chain Accounts

Dropset's on-chain state is stored across several account types. Each maps to a context file in `program/src/context/`.

## Market

The central account. One per trading pair. Stores:

- The base and quote mint addresses
- Tick size and lot size configuration
- The bid and ask sides of the order book (packed arrays of resting orders)
- The event queue (a ring buffer of fill and cancel events)
- The seat registry
- Capacity metadata

The market account is mutated by nearly every instruction. Its size is fixed at creation and can be grown via `expand_market`.

## Seat

One per trader per market. A seat must exist before a trader can post or cancel orders.

- Derived as a PDA from the market and trader public keys
- Only one seat per (market, trader) pair can exist
- Closed via `close_seat` to recover rent

## Trader balance

Tracks a trader's deposited funds within a market. Separate balances for base and quote:

- **Available** — not reserved, can be used to post orders or withdrawn
- **Reserved** — locked by resting orders
- **Settled** — from filled orders, available to withdraw

Also a PDA derived from the market and trader.

## Token vaults

The program holds custody of deposited tokens in two program-owned SPL token accounts per market — one for base, one for quote. Deposits move tokens in; withdrawals move tokens out.

## Account relationships

```
Market
  ├── base_vault   (SPL token account)
  ├── quote_vault  (SPL token account)
  ├── event_queue  (ring buffer)
  └── order_book   (bid side + ask side)

Seat            (PDA: market + trader)
TraderBalance   (PDA: market + trader)
  ├── base_available / base_reserved
  └── quote_available / quote_reserved
```

## PDAs

Seats and trader balances are program-derived addresses — their addresses are computed deterministically from seeds. This means:

- Any client can compute a trader's seat address without fetching it
- There can only ever be one seat per (market, trader) pair
- The program can sign for PDA accounts without a private key

Seeds are defined in `dropset-interface`, so clients and the program always derive the same addresses.

## Fetching accounts with the TS SDK

```typescript
import { getMarketAccountDataDecoder } from "@dropset/ts-sdk";

const accountInfo = await connection.getAccountInfo(marketPubkey);
const market = getMarketAccountDataDecoder().decode(accountInfo.data);

console.log(market.baseMint);
console.log(market.quoteMint);
```

Decoders are generated from the Codama IDL and live in `ts-sdk/src/generated/`.
