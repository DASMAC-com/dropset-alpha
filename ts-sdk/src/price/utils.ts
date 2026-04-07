import { Decimal } from "decimal.js";

/**
 * Returns a Decimal constructor with enough precision to exactly represent
 * the result of an operation on `a` and `b` (no silent rounding).
 */
function preciseDecimal(a: bigint, b: Decimal): typeof Decimal {
  const need = a.toString().length + b.precision(true) + 1;
  return need <= Decimal.precision
    ? Decimal
    : Decimal.clone({ precision: need });
}

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
    const D = preciseDecimal(baseAmount, price);
    return BigInt(
      new D(baseAmount.toString())
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
    const D = preciseDecimal(quoteAmount, price);
    return BigInt(
      new D(quoteAmount.toString())
        .div(price)
        .toDP(0, Decimal.ROUND_DOWN)
        .toFixed(),
    );
  }
  return quoteAmount.div(price);
}
