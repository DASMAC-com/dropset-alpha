import { Decimal } from "decimal.js";
import { PriceError } from "./error";
import {
  MANTISSA_DIGITS_LOWER_BOUND,
  MANTISSA_DIGITS_UPPER_BOUND,
} from "./lib";

/**
 * A price mantissa validated to be within `[MANTISSA_DIGITS_LOWER_BOUND, MANTISSA_DIGITS_UPPER_BOUND]`.
 *
 * Port of `ValidatedPriceMantissa` in `price/src/validated_mantissa.rs`.
 */
export type ValidatedPriceMantissa = {
  readonly __brand: "ValidatedPriceMantissa";
  readonly value: number;
};

/** Port of `ValidatedPriceMantissa::try_from` in `price/src/validated_mantissa.rs`. */
export function validatePriceMantissa(
  mantissa: number,
): ValidatedPriceMantissa {
  if (
    mantissa < MANTISSA_DIGITS_LOWER_BOUND ||
    mantissa > MANTISSA_DIGITS_UPPER_BOUND
  ) {
    throw new Error(PriceError.InvalidPriceMantissa);
  }
  return {
    __brand: "ValidatedPriceMantissa",
    value: mantissa,
  } as ValidatedPriceMantissa;
}

/**
 * Normalize a decimal price into a validated mantissa and scale, where
 * `price = mantissa * 10^scale`.
 *
 * Port of `ValidatedPriceMantissa::try_into_with_scale` in `price/src/validated_mantissa.rs`.
 */
export function normalizePriceMantissa(price: Decimal): {
  mantissa: ValidatedPriceMantissa;
  scale: number;
} {
  if (price.lte(0)) {
    throw new Error(PriceError.InvalidPriceMantissa);
  }

  const MAX_ITERS = 100;
  let res = price;
  let pow = 0;

  const lower = new Decimal(MANTISSA_DIGITS_LOWER_BOUND);
  const upperPlusOne = new Decimal(MANTISSA_DIGITS_UPPER_BOUND + 1);

  while (res.lt(lower)) {
    res = res.times(10);
    pow -= 1;
    if (pow < -MAX_ITERS) throw new Error(PriceError.InvalidPriceMantissa);
  }

  while (res.gte(upperPlusOne)) {
    res = res.div(10);
    pow += 1;
    if (pow > MAX_ITERS) throw new Error(PriceError.InvalidPriceMantissa);
  }

  return {
    mantissa: validatePriceMantissa(res.toDP(0, Decimal.ROUND_DOWN).toNumber()),
    scale: pow,
  };
}
