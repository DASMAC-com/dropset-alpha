import { describe, expect, it } from "@jest/globals";
import { NIL } from "@/ts-sdk/const";
import { toMarketViewAll } from "@/ts-sdk/dropset-interface/market-view-all";
import { getMarketAccountDecoder } from "@/ts-sdk/generated/accounts";
import fixtureBytes from "../fixtures/market-account.json";

describe("Dropset market account deserialization", () => {
  const bytes = new Uint8Array(fixtureBytes);
  const decoder = getMarketAccountDecoder();

  it("should decode raw bytes into a MarketAccount", () => {
    const market = decoder.decode(bytes);

    // Header fields from the fixture summary.
    expect(market.header.discriminant).toBe(1030976753677n);
    expect(market.header.numSeats).toBe(2);
    expect(market.header.numBids).toBe(5);
    expect(market.header.numAsks).toBe(0);
    expect(market.header.numFreeSectors).toBe(13);
    expect(market.header.freeStackTop).toBe(7);
    expect(market.header.seatsDllHead).toBe(0);
    expect(market.header.seatsDllTail).toBe(4);
    expect(market.header.bidsDllHead).toBe(1);
    expect(market.header.bidsDllTail).toBe(3);
    expect(market.header.asksDllHead).toBe(NIL);
    expect(market.header.asksDllTail).toBe(NIL);
    expect(market.header.marketBump).toBe(254);
    expect(market.header.numEvents).toBe(5n);
    expect(market.header.padding).toEqual([0, 0, 0]);
  });

  it("should interpret decoded bytes as a MarketViewAll", () => {
    const market = decoder.decode(bytes);
    const view = toMarketViewAll(market);

    // 2 seats, 5 bids, 0 asks.
    expect(view.seats).toHaveLength(2);
    expect(view.bids).toHaveLength(5);
    expect(view.asks).toHaveLength(0);

    // Bids are sorted descending by price: [90M, 80M, 71M, 70M, 50M].
    const bidPrices = view.bids.map((b) => b.encodedPrice);
    expect(bidPrices).toEqual([
      90_000_000, 80_000_000, 71_000_000, 70_000_000, 50_000_000,
    ]);

    // Each bid has expected base_remaining and quote_remaining.
    for (const bid of view.bids) {
      expect(bid.baseRemaining).toBe(1_000_000_000_000_000n);
    }
    expect(view.bids.map((b) => Number(b.quoteRemaining))).toEqual([
      9_000_000, 8_000_000, 7_100_000, 7_000_000, 5_000_000,
    ]);

    // Seat 0: maker A with 3 bids.
    const seat0 = view.seats[0];
    expect(seat0.index).toBe(0);
    expect(seat0.prevIndex).toBe(NIL);
    expect(seat0.nextIndex).toBe(4);
    expect(seat0.baseAvailable).toBe(1n);
    expect(seat0.quoteAvailable).toBe(0n);

    // Seat 4: maker B with 2 bids.
    const seat4 = view.seats[1];
    expect(seat4.index).toBe(4);
    expect(seat4.prevIndex).toBe(0);
    expect(seat4.nextIndex).toBe(NIL);
    expect(seat4.baseAvailable).toBe(1n);
    expect(seat4.quoteAvailable).toBe(0n);

    // User data: maker A has 3 bids, maker B has 2 bids.
    const userA = view.users.get(seat0.user);
    if (!userA) throw new Error("Expected user A in map");
    expect(userA.bids).toHaveLength(3);
    expect(userA.asks).toHaveLength(0);
    expect(userA.bids.map((b) => b.encodedPrice)).toEqual([
      90_000_000, 70_000_000, 50_000_000,
    ]);

    const userB = view.users.get(seat4.user);
    if (!userB) throw new Error("Expected user B in map");
    expect(userB.bids).toHaveLength(2);
    expect(userB.asks).toHaveLength(0);
    expect(userB.bids.map((b) => b.encodedPrice)).toEqual([
      80_000_000, 71_000_000,
    ]);
  });
});
