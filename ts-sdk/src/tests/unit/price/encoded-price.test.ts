import { describe, expect, it } from "@jest/globals";
import assert from "assert";
import {
  decodePrice,
  type EncodedPrice,
  encodePrice,
  toBiasedExponent,
  validatePriceMantissa,
} from "@/ts-sdk/price";

describe("EncodedPrice", () => {
  // Port of `test_zero_and_infinity` in `price/src/encoded_price.rs`.
  it("should recognize zero and infinity", () => {
    const zero = decodePrice(0);
    const infinity = decodePrice(0xffff_ffff);
    expect(zero.kind).toBe("zero");
    expect(infinity.kind).toBe("infinity");
  });

  // Port of `round_trip_encoded_to_le_encoded` in `price/src/encoded_price.rs`.
  it("should round-trip through encode and decode", () => {
    const mantissa = validatePriceMantissa(12_345_678);
    const biased = toBiasedExponent(1);
    const encoded = encodePrice(mantissa, biased);

    const decoded = decodePrice(encoded);
    assert(decoded.kind === "value");
    expect(decoded.mantissa.value).toBe(12_345_678);
    expect(decoded.biasedExponent).toBe(biased);
  });

  // Port of `price_priority` in `price/src/encoded_price.rs`.
  it("should maintain price priority ordering", () => {
    const prices: EncodedPrice[] = [
      10_000_000, 20_000_000, 30_000_000, 40_000_000,
    ].map((m) => encodePrice(validatePriceMantissa(m), toBiasedExponent(0)));
    const [p1, p2, p3, p4] = prices;

    // Bids: higher price = higher priority.
    expect(p4).toBeGreaterThan(p3);
    expect(p3).toBeGreaterThan(p2);
    expect(p2).toBeGreaterThan(p1);

    // Asks: lower price = higher priority.
    expect(p1).toBeLessThan(p2);
    expect(p2).toBeLessThan(p3);
    expect(p3).toBeLessThan(p4);
  });
});
