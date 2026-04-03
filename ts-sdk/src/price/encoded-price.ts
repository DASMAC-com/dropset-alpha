import { PRICE_MANTISSA_BITS } from "./lib";
import type { ValidatedPriceMantissa } from "./validated-mantissa";

const ENCODED_PRICE_INFINITY = 0xffff_ffff;
const ENCODED_PRICE_ZERO = 0;

export { ENCODED_PRICE_INFINITY, ENCODED_PRICE_ZERO };

/**
 * An encoded price packed into a u32: `[exponent_bits | mantissa_bits]`.
 *
 * Port of `EncodedPrice` in `price/src/encoded_price.rs`.
 */
export type EncodedPrice = number;

/** Port of `EncodedPrice::new` in `price/src/encoded_price.rs`. */
export function encodePrice(
  mantissa: ValidatedPriceMantissa,
  biasedExponent: number,
): EncodedPrice {
  return ((biasedExponent << PRICE_MANTISSA_BITS) | mantissa.value) >>> 0;
}

export function isEncodedPriceInfinity(encoded: EncodedPrice): boolean {
  return encoded === ENCODED_PRICE_INFINITY;
}

export function isEncodedPriceZero(encoded: EncodedPrice): boolean {
  return encoded === ENCODED_PRICE_ZERO;
}
