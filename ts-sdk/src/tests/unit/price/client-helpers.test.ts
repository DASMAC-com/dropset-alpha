import { describe, expect, it } from "@jest/globals";
import { Decimal } from "decimal.js";
import {
  atomsToUiAmount,
  BIAS,
  PriceError,
  toBiasedExponent,
  toOrderInfoArgs,
  UNBIASED_MAX,
  UNBIASED_MIN,
  uiPriceToAtomsPrice,
} from "@/ts-sdk/price";
import { decimalPow10 } from "@/ts-sdk/price/client-helpers-decimal-pow10";

const biasedExponent = (unbiased: number) => toBiasedExponent(unbiased);

describe("client helpers", () => {
  // Port of `test_try_biased_exponents` in `price/src/client_helpers.rs`.
  describe("toBiasedExponent", () => {
    it("should convert valid unbiased exponents", () => {
      expect(toBiasedExponent(UNBIASED_MIN)).toBe(UNBIASED_MIN + BIAS);
      expect(toBiasedExponent(0)).toBe(BIAS);
      expect(toBiasedExponent(UNBIASED_MAX)).toBe(UNBIASED_MAX + BIAS);
    });

    it("should reject out-of-range exponents", () => {
      expect(() => toBiasedExponent(UNBIASED_MIN - 1)).toThrow(
        PriceError.InvalidBiasedExponent,
      );
      expect(() => toBiasedExponent(UNBIASED_MAX + 1)).toThrow(
        PriceError.InvalidBiasedExponent,
      );
    });
  });

  // Port of `test_to_order_info_args` in `price/src/client_helpers.rs`.
  describe("toOrderInfoArgs", () => {
    it("should produce valid args", () => {
      expect(() =>
        toOrderInfoArgs(new Decimal("1.5123"), 500_000n),
      ).not.toThrow();
    });

    it("should match the EUR/USD example from the Rust doctest", () => {
      const baseAtoms = 500n * 10n ** 6n;
      const result = toOrderInfoArgs(new Decimal("1.25"), baseAtoms);
      expect(result.priceMantissa).toBe(12_500_000);
      expect(result.baseScalar).toBe(5n);
      expect(result.baseExponentBiased).toBe(biasedExponent(8));
      expect(result.quoteExponentBiased).toBe(biasedExponent(1));
    });
  });

  // Port of `test_decimal_pow10` in `price/src/client_helpers.rs`.
  describe("decimalPow10", () => {
    it("should scale decimals by powers of 10", () => {
      const check = (value: string, pow: number, expected: string) => {
        expect(
          decimalPow10(new Decimal(value), pow).eq(new Decimal(expected)),
        ).toBe(true);
      };

      check("1.23", 2, "123");
      check("1.6923", 3, "1692.3");
      check("1.000333", 4, "10003.33");
      check("1.23", -1, "0.123");
      check("1.23", -2, "0.0123");
      check("0.05123", -9, "0.00000000005123");
    });
  });

  // Port of `varying_decimal_pair` in `price/src/client_helpers.rs`.
  describe("uiPriceToAtomsPrice", () => {
    it("should scale price by 10^(quoteDecimals - baseDecimals)", () => {
      const check = (
        price: string,
        base: number,
        quote: number,
        expected: string,
      ) => {
        expect(
          uiPriceToAtomsPrice(new Decimal(price), base, quote).eq(
            new Decimal(expected),
          ),
        ).toBe(true);
      };

      check("1.27", 6, 6, "1.27");
      check("1.27", 5, 6, "12.7");
      check("1.27", 6, 5, "0.127");
      check("1.27", 11, 19, "127000000");
      check("1.27", 19, 11, "0.0000000127");
    });
  });

  describe("atomsToUiAmount", () => {
    it("should convert atoms to UI amount", () => {
      expect(atomsToUiAmount(1_000_000n, 6).eq(new Decimal("1"))).toBe(true);
      expect(atomsToUiAmount(500_000_000n, 9).eq(new Decimal("0.5"))).toBe(
        true,
      );
    });
  });
});
