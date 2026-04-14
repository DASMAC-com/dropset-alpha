# TypeScript SDK

The Dropset TypeScript SDK lives in `ts-sdk/` inside the `dropset-alpha` workspace. It provides everything you need to build clients, bots, and frontends that interact with the Dropset program.

## Structure

```
ts-sdk/src/
├── generated/          # Auto-generated from Codama IDL — do not edit
├── dropset-interface/  # Ergonomic wrappers around the generated layer
├── price/              # Price math — tick/lot conversions
├── rust-types/         # TypeScript mirrors of Rust types
├── types/              # Shared TypeScript type definitions
├── utils/              # Utility functions
├── const.ts            # Program ID and constants
└── index.ts            # Public entry point — import from here
```

## Layers explained

### `generated/`

Produced automatically by Codama from the program's IDL. Contains account decoders/encoders and instruction builders for every instruction. **Do not edit files here** — they are overwritten when the IDL is regenerated:

```bash
pnpm --filter codama-idl-gen generate
```

### `dropset-interface/`

Hand-written TypeScript that wraps the generated layer with a more ergonomic API — resolving accounts, building complete transactions, and handling common patterns.

### `price/`

Ported from the Rust `price` crate. Utilities for converting between raw token amounts and lots, and between decimal prices and ticks.

## Building and testing

```bash
# Build
pnpm --filter ts-sdk build

# Test
pnpm --filter ts-sdk test
```

## Basic usage

### Fetch and decode a market

```typescript
import { getMarketAccountDataDecoder } from "@dropset/ts-sdk";

const accountInfo = await rpc.getAccountInfo(marketAddress).send();
const market = getMarketAccountDataDecoder().decode(accountInfo.value.data);

console.log("Base mint:", market.baseMint);
console.log("Quote mint:", market.quoteMint);
```

### Build a `post_order` instruction

```typescript
import { getPostOrderInstruction } from "@dropset/ts-sdk";

const ix = getPostOrderInstruction({
  market: marketAddress,
  seat: seatAddress,
  traderBalance: traderBalanceAddress,
  trader: signer.publicKey,
  side: "bid",
  priceInTicks: 1000n,
  numBaseLots: 10n,
});
```

### Price utilities

```typescript
import { lotsToBaseAtoms, ticksToQuoteAtoms } from "@dropset/ts-sdk";

const baseAmount = lotsToBaseAtoms(numLots, market.baseLotSize);
const quoteAmount = ticksToQuoteAtoms(priceInTicks, market.tickSize, market.baseLotSize);
```
