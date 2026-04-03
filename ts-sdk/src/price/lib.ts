import { type EncodedPrice, encodePrice } from "./encoded-price";
import { PriceError } from "./error";
import {
  normalizePriceMantissa,
  validatePriceMantissa,
} from "./validated-mantissa";

/** Port of `MANTISSA_DIGITS_LOWER_BOUND` in `price/src/lib.rs`. */
export const MANTISSA_DIGITS_LOWER_BOUND = 10_000_000;
/** Port of `MANTISSA_DIGITS_UPPER_BOUND` in `price/src/lib.rs`. */
export const MANTISSA_DIGITS_UPPER_BOUND = 99_999_999;

export const PRICE_MANTISSA_BITS = 27;
export const PRICE_MANTISSA_MASK = 0xffff_ffff >>> (32 - PRICE_MANTISSA_BITS);

export const BIAS = 16;
export const UNBIASED_MIN = -BIAS;
export const UNBIASED_MAX = (1 << (32 - PRICE_MANTISSA_BITS)) - 1 - BIAS;

/** Port of the `pow10_u64!` macro in `price/src/macros.rs`. */
export function pow10Bigint(value: bigint, biasedExponent: number): bigint {
  if (biasedExponent === BIAS) return value;

  if (biasedExponent < 0 || biasedExponent > 31) {
    throw new Error(PriceError.InvalidBiasedExponent);
  }

  const unbiased = biasedExponent - BIAS;
  if (unbiased < 0) {
    return value / 10n ** BigInt(-unbiased);
  }
  return value * 10n ** BigInt(unbiased);
}

/**
 * Compute full order info (encoded price, base atoms, quote atoms) from order args.
 *
 * Port of `to_order_info` in `price/src/lib.rs`.
 */
export function toOrderInfo(args: {
  priceMantissa: number;
  baseScalar: bigint;
  baseExponentBiased: number;
  quoteExponentBiased: number;
}): {
  encodedPrice: EncodedPrice;
  baseAtoms: bigint;
  quoteAtoms: bigint;
} {
  const mantissa = validatePriceMantissa(args.priceMantissa);

  const baseAtoms = pow10Bigint(args.baseScalar, args.baseExponentBiased);
  const quoteAtoms = pow10Bigint(
    BigInt(mantissa.value) * args.baseScalar,
    args.quoteExponentBiased,
  );

  // Re-bias: price_exponent = quote_exponent_biased + BIAS - base_exponent_biased
  const rebiased = args.quoteExponentBiased + BIAS - args.baseExponentBiased;
  if (rebiased < 0) throw new Error(PriceError.ExponentUnderflow);

  return {
    encodedPrice: encodePrice(mantissa, rebiased),
    baseAtoms,
    quoteAtoms,
  };
}

/**
 * Creates order args such that the output `EncodedPrice` from `toOrderInfo`
 * equals the input `priceMantissa` exactly.
 *
 * Port of `OrderInfoArgs::order_at_price` in `price/src/lib.rs`.
 */
export function orderAtPrice(priceMantissa: number): {
  priceMantissa: number;
  baseScalar: bigint;
  baseExponentBiased: number;
  quoteExponentBiased: number;
} {
  return {
    priceMantissa,
    baseScalar: 1n,
    baseExponentBiased: UNBIASED_MAX + BIAS,
    quoteExponentBiased: BIAS - 1,
  };
}

export { validatePriceMantissa, normalizePriceMantissa };
