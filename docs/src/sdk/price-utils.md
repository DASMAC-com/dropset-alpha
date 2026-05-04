# Price Utilities

The `price/` module in the TypeScript SDK is a direct port of the Rust `price` crate. It handles all conversions between human-readable (UI) values and the on-chain encoded representation that the Dropset program uses.

All functions below are exported from `@dropset/ts-sdk`.

## Why price encoding exists

Dropset stores prices on-chain in a compact encoded format — a mantissa and a biased exponent — rather than as raw integers. This keeps the order book memory-efficient and avoids overflow issues with large token amounts. The `price/` utilities handle encoding and decoding so you never have to work with the raw format directly.

## Core functions

### `atomsToUiAmount`

Converts a raw token amount (in atoms — the smallest unit) to a human-readable decimal, given the mint's decimal places.

```typescript
import { atomsToUiAmount } from "@dropset/ts-sdk";
import { Decimal } from "decimal.js";

// Convert 1,000,000 atoms of a 6-decimal token → "1.0"
const uiAmount = atomsToUiAmount(1_000_000n, 6);
console.log(uiAmount.toString()); // "1"

// USDC has 6 decimals
const usdcUiAmount = atomsToUiAmount(5_500_000n, 6);
console.log(usdcUiAmount.toString()); // "5.5"
```

**Signature:**
```typescript
function atomsToUiAmount(
  atomsAmount: bigint,
  mintDecimals: number | bigint,
): Decimal
```

---

### `uiPriceToAtomsPrice`

Converts a human-readable price (quote per base, e.g. "42.50 USDC per SOL") to an atoms-denominated price, accounting for the difference in decimal places between the base and quote mints.

```typescript
import { uiPriceToAtomsPrice } from "@dropset/ts-sdk";
import { Decimal } from "decimal.js";

// SOL/USDC market: SOL has 9 decimals, USDC has 6 decimals
const uiPrice = new Decimal("42.50");
const atomsPrice = uiPriceToAtomsPrice(uiPrice, 9, 6);
console.log(atomsPrice.toString()); // "0.0000425" (adjusted for decimal difference)
```

**Signature:**
```typescript
function uiPriceToAtomsPrice(
  uiPrice: Decimal,
  baseDecimals: number | bigint,
  quoteDecimals: number | bigint,
): Decimal
```

The formula applied is: `atomsPrice = uiPrice × 10^(quoteDecimals − baseDecimals)`

---

### `toOrderInfoArgs`

The most important function for placing orders. Converts a decimal price and a base-atom order size into the four on-chain fields that `post_order` and `batch_replace` require: `priceMantissa`, `baseScalar`, `baseExponentBiased`, and `quoteExponentBiased`.

```typescript
import { toOrderInfoArgs, uiPriceToAtomsPrice } from "@dropset/ts-sdk";
import { Decimal } from "decimal.js";

// You want to post a bid: buy 2 SOL at 42.50 USDC
const uiPrice = new Decimal("42.50");
const atomsPrice = uiPriceToAtomsPrice(uiPrice, 9, 6); // SOL=9 dec, USDC=6 dec

const orderSizeBaseAtoms = 2_000_000_000n; // 2 SOL in lamports

const args = toOrderInfoArgs(atomsPrice, orderSizeBaseAtoms);

console.log(args);
// {
//   priceMantissa: ...,
//   baseScalar: ...,
//   baseExponentBiased: ...,
//   quoteExponentBiased: ...
// }

// Pass these directly into the post_order instruction builder
```

**Signature:**
```typescript
function toOrderInfoArgs(
  price: Decimal,
  orderSizeBaseAtoms: bigint,
): {
  priceMantissa: U32;
  baseScalar: U64;
  baseExponentBiased: U8;
  quoteExponentBiased: U8;
}
```

::: warning
`toOrderInfoArgs` throws `PriceError.AmountCannotBeZero` if `orderSizeBaseAtoms` is `0n`. Always validate order size before calling it.
:::

---

### `encodedU32ToDecimal`

Decodes an on-chain encoded price (a `u32`) back into a human-readable `Decimal`. Useful for displaying resting order prices from the book.

```typescript
import { encodedU32ToDecimal } from "@dropset/ts-sdk";

// Decode an encoded price from an OrderView
const order = view.bids[0];
const decodedPrice = encodedU32ToDecimal(order.price);
console.log(decodedPrice.toString()); // e.g. "42.5"
```

**Signature:**
```typescript
function encodedU32ToDecimal(encodedU32: number | bigint): Decimal
```

---

### `toBiasedExponent`

Low-level utility that converts an unbiased exponent to the biased format the program uses. You generally won't call this directly — `toOrderInfoArgs` handles it internally.

```typescript
function toBiasedExponent(unbiased: number): U8
```

Throws `PriceError.InvalidBiasedExponent` if the value is outside the supported range.

---

## Full order placement flow

```typescript
import { Decimal } from "decimal.js";
import {
  atomsToUiAmount,
  uiPriceToAtomsPrice,
  toOrderInfoArgs,
} from "@dropset/ts-sdk";

// Market config (from market header)
const BASE_DECIMALS = 9;  // e.g. SOL
const QUOTE_DECIMALS = 6; // e.g. USDC

// Step 1 — Define your order parameters in human terms
const uiPrice = new Decimal("42.50");         // 42.50 USDC per SOL
const orderSizeBaseAtoms = 1_000_000_000n;    // 1 SOL

// Step 2 — Convert UI price to atoms price
const atomsPrice = uiPriceToAtomsPrice(uiPrice, BASE_DECIMALS, QUOTE_DECIMALS);

// Step 3 — Get the on-chain instruction args
const orderArgs = toOrderInfoArgs(atomsPrice, orderSizeBaseAtoms);

// Step 4 — Pass to the instruction builder (see Post an Order)
// getPostOrderInstruction({ ...orderArgs, ... })
```

## Next steps

- [Post an Order](/sdk/post-order) — use `toOrderInfoArgs` output in a real transaction
- [Connect to a Market](/sdk/connect-to-market) — read the book and decode existing order prices
