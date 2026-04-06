import { ensureU8, type U32, U32_MAX } from "../rust-types";
import { PRICE_EXPONENT_MAX, PRICE_MANTISSA_BITS } from "./const";
import { PriceError } from "./error";
import type { ValidatedPriceMantissa } from "./validated-mantissa";

/**
 * An encoded price packed into a u32: `[exponent_bits | mantissa_bits]`.
 *
 * Port of `EncodedPrice` in `price/src/encoded_price.rs`.
 */
export type EncodedPrice = U32;

const ENCODED_PRICE_INFINITY = U32_MAX as EncodedPrice;
const ENCODED_PRICE_ZERO = 0 as EncodedPrice;

export { ENCODED_PRICE_INFINITY, ENCODED_PRICE_ZERO };

/** Port of `EncodedPrice::new` in `price/src/encoded_price.rs`. */
export function encodePrice(
  mantissa: ValidatedPriceMantissa,
  biasedExponent: number | bigint,
): EncodedPrice {
  const exp = ensureU8(biasedExponent);
  if (exp > PRICE_EXPONENT_MAX) {
    throw new Error(PriceError.InvalidBiasedExponent);
  }
  return (((exp << PRICE_MANTISSA_BITS) | mantissa.value) >>>
    0) as EncodedPrice;
}

export function isEncodedPriceInfinity(encoded: EncodedPrice): boolean {
  return encoded === ENCODED_PRICE_INFINITY;
}

export function isEncodedPriceZero(encoded: EncodedPrice): boolean {
  return encoded === ENCODED_PRICE_ZERO;
}
