import { describe, expect, it } from "@jest/globals";
import { Decimal } from "decimal.js";
import { baseFromQuote, quoteFromBase } from "@/ts-sdk/price";

describe("token conversion", () => {
  describe("quoteFromBase / baseFromQuote with bigint (atoms)", () => {
    it("should convert at price = 2 (simple)", () => {
      const base = 100_000n;
      const price = new Decimal("2");
      const quote = quoteFromBase(base, price);
      expect(quote).toBe(200_000n);
      expect(baseFromQuote(quote, price)).toBe(base);
    });

    it("should convert at price = 0.5 (fractional)", () => {
      const base = 200_000n;
      const price = new Decimal("0.5");
      const quote = quoteFromBase(base, price);
      expect(quote).toBe(100_000n);
      expect(baseFromQuote(quote, price)).toBe(base);
    });

    it("should truncate toward zero on indivisible conversions", () => {
      expect(quoteFromBase(7n, new Decimal("3"))).toBe(21n);
      expect(baseFromQuote(30n, new Decimal("3"))).toBe(10n);
      // 10 base at price 0.33 = 3.3 → truncated to 3.
      expect(quoteFromBase(10n, new Decimal("0.33"))).toBe(3n);
      // 10 quote at price 3 = 3.333... → truncated to 3.
      expect(baseFromQuote(10n, new Decimal("3"))).toBe(3n);
    });

    it("should handle SOL/USDC-like atoms (9 vs 6 decimals, price ~150)", () => {
      // 1 SOL = 1_000_000_000 lamports, 1 USDC = 1_000_000 micro-USDC.
      // UI price = 150 USDC/SOL.
      // Atoms price = 150 * 10^(6-9) = 0.15.
      const atomsPrice = new Decimal("0.15");
      const baseLamports = 1_000_000_000n; // 1 SOL
      const expectedQuoteMicro = 150_000_000n; // 150 USDC

      expect(quoteFromBase(baseLamports, atomsPrice)).toBe(expectedQuoteMicro);
      expect(baseFromQuote(expectedQuoteMicro, atomsPrice)).toBe(baseLamports);
    });

    it("should handle BTC/USDC-like atoms (8 vs 6 decimals, price ~60000)", () => {
      // 1 BTC = 100_000_000 sats, 1 USDC = 1_000_000 micro-USDC.
      // UI price = 60000 USDC/BTC.
      // Atoms price = 60000 * 10^(6-8) = 600.
      const atomsPrice = new Decimal("600");
      const baseSats = 100_000_000n; // 1 BTC
      const expectedQuoteMicro = 60_000_000_000n; // 60000 USDC

      expect(quoteFromBase(baseSats, atomsPrice)).toBe(expectedQuoteMicro);
      expect(baseFromQuote(expectedQuoteMicro, atomsPrice)).toBe(baseSats);
    });

    it("should handle tiny fractional price (sub-penny token)", () => {
      // Token at $0.000001 with 9 decimals each.
      const atomsPrice = new Decimal("0.000001");
      const base = 1_000_000_000n;
      const expectedQuote = 1000n;

      expect(quoteFromBase(base, atomsPrice)).toBe(expectedQuote);
      expect(baseFromQuote(expectedQuote, atomsPrice)).toBe(base);
    });
  });

  describe("quoteFromBase / baseFromQuote with Decimal (UI amounts)", () => {
    it("should convert UI amounts at price = 150", () => {
      const base = new Decimal("2.5"); // 2.5 SOL
      const price = new Decimal("150"); // 150 USDC/SOL
      const quote = quoteFromBase(base, price);
      expect(quote.eq(new Decimal("375"))).toBe(true);
      expect(baseFromQuote(quote, price).eq(base)).toBe(true);
    });

    it("should preserve full decimal precision", () => {
      const base = new Decimal("0.123456789");
      const price = new Decimal("60000.50");
      const quote = quoteFromBase(base, price);
      expect(baseFromQuote(quote, price).eq(base)).toBe(true);
    });
  });
});
