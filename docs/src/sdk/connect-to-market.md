# Connect to a Market

This page shows how to fetch a live Dropset market account and decode its full state — seats, bids, asks, and per-user order data — using the TypeScript SDK.

## The `toMarketViewAll` function

The primary way to read a market is `toMarketViewAll`, exported from `ts-sdk/src/dropset-interface/market-view-all.ts`. It takes a decoded `MarketAccount` and returns a fully structured view of everything on the book.

```typescript
import { toMarketViewAll } from "@dropset/ts-sdk";
import type { MarketViewAll } from "@dropset/ts-sdk";
```

### What it returns

```typescript
type MarketViewAll = {
  header: MarketHeader;       // Market config: mints, tick size, lot size
  seats: MarketSeatView[];    // All registered traders on this market
  bids: OrderView[];          // All resting buy orders, linked-list order
  asks: OrderView[];          // All resting sell orders, linked-list order
  users: Map<Address, MarketUserData>; // Per-trader seat + orders
};
```

Each `OrderView` includes the order's price, size, side, and its position in the doubly-linked list:

```typescript
type OrderView = {
  prevIndex: SectorIndex;
  index: SectorIndex;
  nextIndex: SectorIndex;
  // ...all Order fields except padding
};
```

Each `MarketUserData` groups a trader's seat with their open bids and asks:

```typescript
type MarketUserData = {
  seat: MarketSeatView;
  bids: OrderView[];
  asks: OrderView[];
};
```

## Full example

```typescript
import { createSolanaRpc } from "@solana/kit";
import { address } from "@solana/kit";
import { getMarketAccountDecoder } from "@dropset/ts-sdk";
import { toMarketViewAll } from "@dropset/ts-sdk";

const rpc = createSolanaRpc("https://api.devnet.solana.com");
const marketAddress = address("YOUR_MARKET_ADDRESS_HERE");

// 1. Fetch the raw account
const { value: accountInfo } = await rpc
  .getAccountInfo(marketAddress, { encoding: "base64" })
  .send();

if (!accountInfo) throw new Error("Market account not found");

// 2. Decode the raw bytes into a MarketAccount
const marketAccount = getMarketAccountDecoder().decode(
  Buffer.from(accountInfo.data[0], "base64"),
);

// 3. Build a fully structured view of the market
const view = toMarketViewAll(marketAccount);

// 4. Inspect the book
console.log("Base mint:", view.header.baseMint);
console.log("Quote mint:", view.header.quoteMint);
console.log("Total seats:", view.seats.length);
console.log("Resting bids:", view.bids.length);
console.log("Resting asks:", view.asks.length);

// 5. Inspect a specific trader's orders
const traderAddress = address("TRADER_ADDRESS_HERE");
const userData = view.users.get(traderAddress);

if (userData) {
  console.log("Trader bids:", userData.bids);
  console.log("Trader asks:", userData.asks);
}
```

## How the sector model works

Dropset stores all on-chain state — seats, bids, and asks — in a flat byte array called `sectors`. Each sector is a fixed-size slot in that array containing a payload (either an `Order` or a `MarketSeat`) and `prev`/`next` pointers forming a doubly-linked list.

`toMarketViewAll` traverses these linked lists internally using `collectSectors`, decoding each sector's payload and attaching its index metadata. You don't need to interact with sectors directly — `toMarketViewAll` handles all of that.

::: tip
If `collectSectors` throws a `Malformed sectors bytes` error, the market account data is either corrupted or you decoded it with the wrong decoder. Make sure you're using `getMarketAccountDecoder()` from the generated layer.
:::

## Next steps

- [Post an Order](/sdk/post-order) — build and send a `post_order` transaction
- [Price Utilities](/sdk/price-utils) — convert between UI prices and on-chain encoded values
