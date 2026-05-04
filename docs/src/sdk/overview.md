# TypeScript SDK

The Dropset TypeScript SDK lives in `ts-sdk/` inside the `dropset-alpha` workspace. It provides everything you need to build clients, bots, and frontends that interact with the Dropset program.

## Structure

```
ts-sdk/src/
├── generated/          # Auto-generated from Codama IDL — do not edit
├── dropset-interface/  # Hand-written wrappers around the generated layer
│   └── market-view-all.ts  # toMarketViewAll — read the full book state
├── price/              # Price math — UI ↔ atoms conversions
│   ├── client-helpers.ts   # toOrderInfoArgs, atomsToUiAmount, uiPriceToAtomsPrice
│   ├── decoded-price.ts
│   ├── encoded-price.ts
│   └── ...
├── rust-types/         # TypeScript mirrors of Rust types (U8, U32, U64)
├── types/              # Shared TypeScript type definitions
├── utils/              # Utility functions
├── const.ts            # Program ID, NIL sentinel, SECTOR_SIZE
└── index.ts            # Public entry point — import from here
```

## Two layers

The SDK is intentionally split into two layers:

**Generated layer** (`generated/`) — produced automatically by Codama from the program's IDL. Contains raw instruction builders and account decoders. Do not edit these files — they are overwritten when the IDL is regenerated:

```bash
pnpm --filter codama-idl-gen generate
```

**Interface layer** (`dropset-interface/`) — hand-written TypeScript that wraps the generated layer with ergonomic, higher-level APIs. This is where `toMarketViewAll` lives, which gives you a fully structured view of the order book in one call.

## Building and testing

```bash
# Build
pnpm --filter ts-sdk build

# Test
pnpm --filter ts-sdk test
```

## Guides

- [Connect to a Market](/sdk/connect-to-market) — fetch and decode a live market, read seats, bids, and asks
- [Post an Order](/sdk/post-order) — build and send a `post_order` transaction end to end
- [Price Utilities](/sdk/price-utils) — convert between UI prices and on-chain encoded values
