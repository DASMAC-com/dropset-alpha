import {
  ensureU8,
  ensureU64,
  type U8,
  type U32,
  type U64,
} from "../rust-types";
import { BIAS, PRICE_EXPONENT_MAX, UNBIASED_MAX } from "./const";
import { type EncodedPrice, encodePrice } from "./encoded-price";
import { PriceError } from "./error";
import {
  normalizePriceMantissa,
  validatePriceMantissa,
} from "./validated-mantissa";

/** Port of the `pow10_u64!` macro in `price/src/macros.rs`. */
export function pow10Bigint(
  value: bigint,
  biasedExponent: number | bigint,
): bigint {
  const exp = ensureU8(biasedExponent);
  if (exp === BIAS) return value;

  if (exp > PRICE_EXPONENT_MAX) {
    throw new Error(PriceError.InvalidBiasedExponent);
  }

  const unbiased = exp - BIAS;
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
  priceMantissa: number | bigint;
  baseScalar: bigint;
  baseExponentBiased: number | bigint;
  quoteExponentBiased: number | bigint;
}): {
  encodedPrice: EncodedPrice;
  baseAtoms: U64;
  quoteAtoms: U64;
} {
  const mantissa = validatePriceMantissa(args.priceMantissa);
  const baseExp = ensureU8(args.baseExponentBiased);
  const quoteExp = ensureU8(args.quoteExponentBiased);

  const baseAtoms = ensureU64(pow10Bigint(args.baseScalar, baseExp));
  const quoteAtoms = ensureU64(
    pow10Bigint(BigInt(mantissa.value) * args.baseScalar, quoteExp),
  );

  // Re-bias: price_exponent = quote_exponent_biased + BIAS - base_exponent_biased
  const rebiased = quoteExp + BIAS - baseExp;
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
export function orderAtPrice(priceMantissa: number | bigint): {
  priceMantissa: U32;
  baseScalar: bigint;
  baseExponentBiased: U8;
  quoteExponentBiased: U8;
} {
  return {
    priceMantissa: validatePriceMantissa(priceMantissa).value,
    baseScalar: 1n,
    baseExponentBiased: ensureU8(UNBIASED_MAX + BIAS),
    quoteExponentBiased: ensureU8(BIAS - 1),
  };
}

export { normalizePriceMantissa, validatePriceMantissa };
