import type { OrderView } from "../dropset-interface";
import { encodedU32ToDecimal, quoteFromBase } from "../price";
import type { DropsetMarketView } from "../types";

/** Sum `quoteRemaining` across all bids in a market. */
function sumBidQuote(bids: OrderView[]): bigint {
  return bids.reduce((total, bid) => total + bid.quoteRemaining, 0n);
}

/** Sum asks converted to quote: `baseRemaining * decodedPrice` for each ask. */
function sumAskQuoteEquivalent(asks: OrderView[]): bigint {
  return asks.reduce((total, ask) => {
    const price = encodedU32ToDecimal(ask.encodedPrice);
    return total + quoteFromBase(ask.baseRemaining, price);
  }, 0n);
}

/** Total liquidity (in quote atoms) for a single market across both sides of the book. */
export function marketLiquidity(market: DropsetMarketView): {
  bidLiquidity: bigint;
  askLiquidity: bigint;
  total: bigint;
} {
  const bidLiquidity = sumBidQuote(market.bids);
  const askLiquidity = sumAskQuoteEquivalent(market.asks);
  return { bidLiquidity, askLiquidity, total: bidLiquidity + askLiquidity };
}

/** Aggregate liquidity (in quote atoms) across multiple markets. */
export function totalLiquidity(markets: DropsetMarketView[]): {
  bidLiquidity: bigint;
  askLiquidity: bigint;
  total: bigint;
} {
  let bidLiquidity = 0n;
  let askLiquidity = 0n;
  for (const market of markets) {
    const m = marketLiquidity(market);
    bidLiquidity += m.bidLiquidity;
    askLiquidity += m.askLiquidity;
  }
  return { bidLiquidity, askLiquidity, total: bidLiquidity + askLiquidity };
}
