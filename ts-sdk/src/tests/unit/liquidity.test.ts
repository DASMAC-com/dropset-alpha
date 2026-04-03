import { describe, expect, it } from "@jest/globals";
import type { Address } from "@solana/kit";
import type { OrderView } from "@/ts-sdk/dropset-interface";
import {
  encodePrice,
  toBiasedExponent,
  validatePriceMantissa,
} from "@/ts-sdk/price";
import type { DropsetMarketView } from "@/ts-sdk/types";
import { marketLiquidity, totalLiquidity } from "@/ts-sdk/utils/liquidity";

// Encode a price of exactly 2.0:
//   mantissa = 20_000_000, unbiased exponent = -7
//   price = 20_000_000 * 10^-7 = 2.0
const PRICE_2 = encodePrice(
  validatePriceMantissa(20_000_000),
  toBiasedExponent(-7),
);

// Encode a price of exactly 0.5:
//   mantissa = 50_000_000, unbiased exponent = -8
//   price = 50_000_000 * 10^-8 = 0.5
const PRICE_0_5 = encodePrice(
  validatePriceMantissa(50_000_000),
  toBiasedExponent(-8),
);

function makeOrder(
  overrides: Partial<OrderView> & Pick<OrderView, "encodedPrice">,
): OrderView {
  return {
    prevIndex: 0,
    index: 0,
    nextIndex: 0,
    userSeatIndex: 0,
    baseRemaining: 0n,
    quoteRemaining: 0n,
    ...overrides,
  };
}

function makeMarket({
  bids,
  asks,
}: {
  bids: OrderView[];
  asks: OrderView[];
}): DropsetMarketView {
  return {
    header: {} as DropsetMarketView["header"],
    seats: [],
    bids,
    asks,
    users: new Map(),
    address: "11111111111111111111111111111111" as Address,
  };
}

describe("marketLiquidity", () => {
  it("should sum bid quoteRemaining directly", () => {
    const market = makeMarket({
      bids: [
        makeOrder({ encodedPrice: PRICE_2, quoteRemaining: 100n }),
        makeOrder({ encodedPrice: PRICE_2, quoteRemaining: 200n }),
      ],
      asks: [],
    });
    const result = marketLiquidity(market);
    expect(result.bidLiquidity).toBe(300n);
    expect(result.askLiquidity).toBe(0n);
    expect(result.total).toBe(300n);
  });

  it("should convert ask baseRemaining to quote via encoded price", () => {
    // 2 asks each with 1000 base atoms at price 2.0 → 2000 quote atoms each.
    const market = makeMarket({
      bids: [],
      asks: [
        makeOrder({ encodedPrice: PRICE_2, baseRemaining: 1000n }),
        makeOrder({ encodedPrice: PRICE_2, baseRemaining: 1000n }),
      ],
    });
    const result = marketLiquidity(market);
    expect(result.bidLiquidity).toBe(0n);
    expect(result.askLiquidity).toBe(4000n);
    expect(result.total).toBe(4000n);
  });

  it("should combine bids and asks with different prices", () => {
    const market = makeMarket({
      bids: [makeOrder({ encodedPrice: PRICE_2, quoteRemaining: 500n })],
      asks: [
        // 200 base at price 2.0 → 400 quote
        makeOrder({ encodedPrice: PRICE_2, baseRemaining: 200n }),
        // 600 base at price 0.5 → 300 quote
        makeOrder({ encodedPrice: PRICE_0_5, baseRemaining: 600n }),
      ],
    });
    const result = marketLiquidity(market);
    expect(result.bidLiquidity).toBe(500n);
    expect(result.askLiquidity).toBe(700n);
    expect(result.total).toBe(1200n);
  });

  it("should return zero for an empty book", () => {
    const market = makeMarket({ bids: [], asks: [] });
    const result = marketLiquidity(market);
    expect(result.bidLiquidity).toBe(0n);
    expect(result.askLiquidity).toBe(0n);
    expect(result.total).toBe(0n);
  });
});

describe("totalLiquidity", () => {
  it("should aggregate across multiple markets", () => {
    const market1 = makeMarket({
      bids: [makeOrder({ encodedPrice: PRICE_2, quoteRemaining: 100n })],
      asks: [makeOrder({ encodedPrice: PRICE_2, baseRemaining: 100n })], // → 200 quote
    });
    const market2 = makeMarket({
      bids: [makeOrder({ encodedPrice: PRICE_0_5, quoteRemaining: 50n })],
      asks: [makeOrder({ encodedPrice: PRICE_0_5, baseRemaining: 400n })], // → 200 quote
    });
    const result = totalLiquidity([market1, market2]);
    expect(result.bidLiquidity).toBe(150n);
    expect(result.askLiquidity).toBe(400n);
    expect(result.total).toBe(550n);
  });
});
