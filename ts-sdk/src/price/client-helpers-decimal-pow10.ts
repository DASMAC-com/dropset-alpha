import Decimal from "decimal.js";

/** Multiply `value` by `10^pow`. Port of `decimal_pow10` in `price/src/client_helpers.rs`. */
export function decimalPow10(value: Decimal, pow: number): Decimal {
  if (pow === 0) return value;
  return value.times(new Decimal(10).pow(pow));
}
