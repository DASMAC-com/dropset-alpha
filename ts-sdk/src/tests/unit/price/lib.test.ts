import { describe, expect, it } from "@jest/globals";
import assert from "assert";
import { Decimal } from "decimal.js";
import {
  BIAS,
  decodedPriceToDecimal,
  decodePrice,
  MANTISSA_DIGITS_LOWER_BOUND,
  MANTISSA_DIGITS_UPPER_BOUND,
  orderAtPrice,
  PriceError,
  toBiasedExponent,
  toOrderInfo,
  UNBIASED_MAX,
} from "@/ts-sdk/price";

const biasedExponent = (unbiased: number) => toBiasedExponent(unbiased);

describe("toOrderInfo", () => {
  // Port of `happy_path_simple_price` in `price/src/lib.rs`.
  it("should compute a simple price", () => {
    const order = toOrderInfo({
      priceMantissa: 12_340_000,
      baseScalar: 1n,
      baseExponentBiased: biasedExponent(0),
      quoteExponentBiased: biasedExponent(-4),
    });
    expect(order.baseAtoms).toBe(1n);
    expect(order.quoteAtoms).toBe(1234n);

    const decoded = decodePrice(order.encodedPrice);
    const decimalPrice = decodedPriceToDecimal(decoded);
    expect(decimalPrice.eq(new Decimal("1234"))).toBe(true);
  });

  // Port of `price_with_max_sig_digits` in `price/src/lib.rs`.
  it("should handle max significant digits", () => {
    const order = toOrderInfo({
      priceMantissa: 12_345_678,
      baseScalar: 1n,
      baseExponentBiased: biasedExponent(0),
      quoteExponentBiased: biasedExponent(0),
    });
    expect(order.baseAtoms).toBe(1n);
    expect(order.quoteAtoms).toBe(12_345_678n);

    const decoded = decodePrice(order.encodedPrice);
    const decimalPrice = decodedPriceToDecimal(decoded);
    expect(decimalPrice.eq(new Decimal("12345678"))).toBe(true);
  });

  // Port of `decimal_price` in `price/src/lib.rs`.
  it("should handle a decimal price", () => {
    const mantissa = 12_345_678;
    const order = toOrderInfo({
      priceMantissa: mantissa,
      baseScalar: 1n,
      baseExponentBiased: biasedExponent(8),
      quoteExponentBiased: biasedExponent(0),
    });
    expect(order.quoteAtoms).toBe(12_345_678n);
    expect(order.baseAtoms).toBe(100_000_000n);

    const decoded = decodePrice(order.encodedPrice);
    assert(decoded.kind === "value");
    expect(decoded.mantissa.value).toBe(mantissa);
    const decimalPrice = decodedPriceToDecimal(decoded);
    expect(decimalPrice.eq(new Decimal("0.12345678"))).toBe(true);
  });

  // Port of `order_at_price_encoded_price_equals_mantissa` in `price/src/lib.rs`.
  it("should produce encoded price equal to mantissa for order_at_price args", () => {
    for (const mantissa of [
      MANTISSA_DIGITS_LOWER_BOUND,
      50_000_000,
      MANTISSA_DIGITS_UPPER_BOUND,
    ]) {
      const order = toOrderInfo({
        priceMantissa: mantissa,
        baseScalar: 1n,
        baseExponentBiased: biasedExponent(UNBIASED_MAX),
        quoteExponentBiased: biasedExponent(-1),
      });
      expect(order.encodedPrice).toBe(mantissa);
    }
  });

  // Port of `ensure_exponent_underflow` in `price/src/lib.rs`.
  it("should throw ExponentUnderflow when quote exp is too small relative to base", () => {
    expect(() =>
      toOrderInfo({
        priceMantissa: 10_000_000,
        baseScalar: 1n,
        baseExponentBiased: BIAS + 1,
        quoteExponentBiased: 0,
      }),
    ).toThrow(PriceError.ExponentUnderflow);

    expect(() =>
      toOrderInfo({
        priceMantissa: 10_000_000,
        baseScalar: 1n,
        baseExponentBiased: BIAS,
        quoteExponentBiased: 0,
      }),
    ).not.toThrow();
  });

  // Port of `order_at_price_encoded_price_equals_mantissa` in `price/src/lib.rs`.
  // (using `orderAtPrice` helper instead of inline args)
  it("should produce encoded price equal to mantissa via orderAtPrice", () => {
    for (const mantissa of [
      MANTISSA_DIGITS_LOWER_BOUND,
      50_000_000,
      MANTISSA_DIGITS_UPPER_BOUND,
    ]) {
      const order = toOrderInfo(orderAtPrice(mantissa));
      expect(order.encodedPrice).toBe(mantissa);
    }
  });
});
