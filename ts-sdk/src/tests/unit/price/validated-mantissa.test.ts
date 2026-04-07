import { describe, expect, it } from "@jest/globals";
import { Decimal } from "decimal.js";
import {
  MANTISSA_DIGITS_LOWER_BOUND,
  MANTISSA_DIGITS_UPPER_BOUND,
  normalizePriceMantissa,
  PriceError,
  validatePriceMantissa,
} from "@/ts-sdk/price";

describe("ValidatedPriceMantissa", () => {
  // Port of `valid_mantissas` in `price/src/validated_mantissa.rs`.
  it("should accept valid mantissas", () => {
    for (const m of [
      MANTISSA_DIGITS_LOWER_BOUND,
      MANTISSA_DIGITS_LOWER_BOUND + 1,
      MANTISSA_DIGITS_UPPER_BOUND,
      MANTISSA_DIGITS_UPPER_BOUND - 1,
    ]) {
      const v = validatePriceMantissa(m);
      expect(v.value).toBe(m);
    }
  });

  // Port of `invalid_mantissas` in `price/src/validated_mantissa.rs`.
  it("should reject invalid mantissas", () => {
    expect(() =>
      validatePriceMantissa(MANTISSA_DIGITS_LOWER_BOUND - 1),
    ).toThrow(PriceError.InvalidPriceMantissa);
    expect(() =>
      validatePriceMantissa(MANTISSA_DIGITS_UPPER_BOUND + 1),
    ).toThrow(PriceError.InvalidPriceMantissa);
  });

  // Port of `test_normalize_values` in `price/src/validated_mantissa.rs`.
  it("should normalize prices into mantissa + scale", () => {
    const check = (
      price: string,
      expectedMantissa: number,
      expectedScale: number,
    ) => {
      const { mantissa, scale } = normalizePriceMantissa(new Decimal(price));
      expect(mantissa.value).toBe(expectedMantissa);
      expect(scale).toBe(expectedScale);
    };

    check("1.32", 13_200_000, -7);
    check("0.95123", 95_123_000, -8);
    check("123456789", 12_345_678, 1);
    check("78.12300001", 78_123_000, -6);
    check("0.000000000000012345678", 12_345_678, -21);
    check("0.000000000001", 10_000_000, -19);
  });

  it("should reject zero and negative prices", () => {
    expect(() => normalizePriceMantissa(new Decimal("0"))).toThrow(
      PriceError.InvalidPriceMantissa,
    );
    expect(() => normalizePriceMantissa(new Decimal("0.000000"))).toThrow(
      PriceError.InvalidPriceMantissa,
    );
    expect(() => normalizePriceMantissa(new Decimal("-1"))).toThrow(
      PriceError.InvalidPriceMantissa,
    );
    expect(() =>
      normalizePriceMantissa(new Decimal("-0.0000000000001")),
    ).toThrow(PriceError.InvalidPriceMantissa);
  });
});
