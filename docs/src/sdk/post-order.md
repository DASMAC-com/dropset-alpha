# Post an Order

This page walks through building and sending a `post_order` transaction using the TypeScript SDK — from converting a human-readable price to submitting the signed transaction.

## Prerequisites

Before you can post an order you need:

- A registered market (see `register_market`)
- A seat on that market
- A deposited balance of base or quote tokens (see `deposit`)

## The full flow

```typescript
import { createSolanaRpc, createSolanaRpcSubscriptions } from "@solana/kit";
import { address } from "@solana/kit";
import { Decimal } from "decimal.js";
import {
  getPostOrderInstruction,
  toOrderInfoArgs,
  uiPriceToAtomsPrice,
} from "@dropset/ts-sdk";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

// Your addresses
const marketAddress = address("MARKET_ADDRESS");
const traderAddress = address("TRADER_ADDRESS");
const seatAddress = address("SEAT_ADDRESS");       // PDA: market + trader
const balanceAddress = address("BALANCE_ADDRESS"); // PDA: market + trader

// Market decimal configuration
const BASE_DECIMALS = 9;
const QUOTE_DECIMALS = 6;

// Step 1 — Convert your human price to on-chain args
const uiPrice = new Decimal("42.50");
const orderSizeBaseAtoms = 1_000_000_000n; // 1 base token

const atomsPrice = uiPriceToAtomsPrice(uiPrice, BASE_DECIMALS, QUOTE_DECIMALS);
const orderArgs = toOrderInfoArgs(atomsPrice, orderSizeBaseAtoms);

// Step 2 — Build the instruction
const ix = getPostOrderInstruction({
  market: marketAddress,
  trader: traderAddress,
  seat: seatAddress,
  traderBalance: balanceAddress,
  side: 0,  // 0 = bid, 1 = ask
  ...orderArgs,
});

// Step 3 — Build and send the transaction
const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

// Sign and send using your wallet/keypair
// (exact signing depends on your client setup)
```

## Order sides

| Value | Side | Meaning |
|---|---|---|
| `0` | Bid | Buy order — you're offering quote tokens to buy base tokens |
| `1` | Ask | Sell order — you're offering base tokens to receive quote tokens |

## The `post_order` instruction accounts

| Account | Description |
|---|---|
| `market` | The market account (writable) |
| `trader` | The trader's public key (signer) |
| `seat` | The trader's seat PDA on this market |
| `traderBalance` | The trader's balance PDA on this market (writable) |

## What happens on-chain

When `post_order` executes:

1. The program validates the trader's seat and balance accounts
2. It reserves the required funds from the trader's available balance
3. It inserts the order into the correct side of the book (bid or ask) in price-time priority order
4. The order rests until it is filled by a `market_order` or `batch_replace`, or cancelled via `cancel_order`

## Cancelling an order

To cancel a resting order you need its `index` from the `OrderView`. Get this by reading the market first:

```typescript
import { toMarketViewAll, getMarketAccountDecoder } from "@dropset/ts-sdk";
import { getCancelOrderInstruction } from "@dropset/ts-sdk";

// Fetch and decode the market
const view = toMarketViewAll(marketAccount);

// Find your orders
const myOrders = view.users.get(traderAddress);
const orderToCancel = myOrders?.bids[0];

if (orderToCancel) {
  const cancelIx = getCancelOrderInstruction({
    market: marketAddress,
    trader: traderAddress,
    seat: seatAddress,
    traderBalance: balanceAddress,
    orderIndex: orderToCancel.index,
  });
}
```

## Next steps

- [Price Utilities](/sdk/price-utils) — understand how `toOrderInfoArgs` works
- [Connect to a Market](/sdk/connect-to-market) — read the book before and after posting
