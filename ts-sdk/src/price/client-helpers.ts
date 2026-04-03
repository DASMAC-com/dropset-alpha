import { Decimal } from "decimal.js";
import {
  ensureU8,
  ensureU64,
  type U8,
  type U32,
  type U64,
} from "../rust-types";
import { decodedPriceToDecimal, decodePrice } from "./decoded-price";
import { PriceError } from "./error";
import {
  BIAS,
  normalizePriceMantissa,
  UNBIASED_MAX,
  UNBIASED_MIN,
} from "./lib";

/** Multiply `value` by `10^pow`. Port of `decimal_pow10` in `price/src/client_helpers.rs`. */
export function decimalPow10(value: Decimal, pow: number): Decimal {
  if (pow === 0) return value;
  return value.times(new Decimal(10).pow(pow));
}

/** Port of `try_to_biased_exponent` in `price/src/client_helpers.rs`. */
export function toBiasedExponent(unbiased: number): U8 {
  if (unbiased < UNBIASED_MIN || unbiased > UNBIASED_MAX) {
    throw new Error(PriceError.InvalidBiasedExponent);
  }
  return ensureU8(unbiased + BIAS);
}

/** Port of `atoms_to_ui_amount` in `price/src/client_helpers.rs`. */
export function atomsToUiAmount(
  atomsAmount: bigint,
  mintDecimals: number | bigint,
): Decimal {
  const dec = ensureU8(mintDecimals);
  return decimalPow10(new Decimal(atomsAmount.toString()), -dec);
}

/**
 * Convert a UI price (human-readable quote/base) to an atoms-denominated price,
 * accounting for differing base/quote decimals.
 *
 * `atomsPrice = uiPrice * 10^(quoteDecimals - baseDecimals)`
 *
 * Port of `ui_price_to_atoms_price` in `price/src/client_helpers.rs`.
 */
export function uiPriceToAtomsPrice(
  uiPrice: Decimal,
  baseDecimals: number | bigint,
  quoteDecimals: number | bigint,
): Decimal {
  const base = ensureU8(baseDecimals);
  const quote = ensureU8(quoteDecimals);
  return decimalPow10(uiPrice, quote - base);
}

/** Port of `try_encoded_u32_to_decoded_decimal` in `price/src/client_helpers.rs`. */
export function encodedU32ToDecimal(encodedU32: number | bigint): Decimal {
  return decodedPriceToDecimal(decodePrice(encodedU32));
}

/** Port of `get_sig_figs` in `price/src/client_helpers.rs`. */
function getSigFigs(value: bigint): { scalar: bigint; pow: number } {
  if (value === 0n) throw new Error(PriceError.AmountCannotBeZero);
  let x = value;
  let pow = 0;
  while (x % 10n === 0n) {
    x /= 10n;
    pow += 1;
  }
  return { scalar: x, pow };
}

/**
 * Convert a decimal price and base-atoms order size into `OrderInfoArgs`-equivalent values.
 *
 * Port of `to_order_info_args` in `price/src/client_helpers.rs`.
 */
export function toOrderInfoArgs(
  price: Decimal,
  orderSizeBaseAtoms: bigint,
): {
  priceMantissa: U32;
  baseScalar: U64;
  baseExponentBiased: U8;
  quoteExponentBiased: U8;
} {
  const { mantissa, scale: priceExponent } = normalizePriceMantissa(price);

  if (orderSizeBaseAtoms === 0n) throw new Error(PriceError.AmountCannotBeZero);
  ensureU64(orderSizeBaseAtoms);

  const { scalar: baseScalar, pow: baseExponentUnbiased } =
    getSigFigs(orderSizeBaseAtoms);
  const quoteExponentUnbiased = priceExponent + baseExponentUnbiased;

  return {
    priceMantissa: mantissa.value,
    baseScalar: ensureU64(baseScalar),
    baseExponentBiased: toBiasedExponent(baseExponentUnbiased),
    quoteExponentBiased: toBiasedExponent(quoteExponentUnbiased),
  };
}
