import { Decimal } from "decimal.js";

/**
 * Convert a base amount to its equivalent quote amount at the given price.
 *
 * `quote = base * price`
 *
 * The output is in the same denomination as the input: pass atoms and an
 * atoms-denominated price to get quote atoms; pass UI amounts and a UI price
 * to get a UI quote amount.
 */
export function quoteFromBase(baseAmount: bigint, price: Decimal): bigint;
export function quoteFromBase(baseAmount: Decimal, price: Decimal): Decimal;
export function quoteFromBase(
  baseAmount: bigint | Decimal,
  price: Decimal,
): bigint | Decimal {
  if (typeof baseAmount === "bigint") {
    return BigInt(
      new Decimal(baseAmount.toString())
        .times(price)
        .toDP(0, Decimal.ROUND_DOWN)
        .toFixed(),
    );
  }
  return baseAmount.times(price);
}

/**
 * Convert a quote amount to its equivalent base amount at the given price.
 *
 * `base = quote / price`
 *
 * The output is in the same denomination as the input: pass atoms and an
 * atoms-denominated price to get base atoms; pass UI amounts and a UI price
 * to get a UI base amount.
 */
export function baseFromQuote(quoteAmount: bigint, price: Decimal): bigint;
export function baseFromQuote(quoteAmount: Decimal, price: Decimal): Decimal;
export function baseFromQuote(
  quoteAmount: bigint | Decimal,
  price: Decimal,
): bigint | Decimal {
  if (typeof quoteAmount === "bigint") {
    return BigInt(
      new Decimal(quoteAmount.toString())
        .div(price)
        .toDP(0, Decimal.ROUND_DOWN)
        .toFixed(),
    );
  }
  return quoteAmount.div(price);
}
