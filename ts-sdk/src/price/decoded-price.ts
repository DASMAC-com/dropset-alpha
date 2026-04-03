import { Decimal } from "decimal.js";
import { ensureU8, ensureU32, type U8 } from "../rust-types";
import { decimalPow10 } from "./client-helpers";
import { ENCODED_PRICE_INFINITY, ENCODED_PRICE_ZERO } from "./encoded-price";
import { PriceError } from "./error";
import { BIAS, PRICE_MANTISSA_BITS, PRICE_MANTISSA_MASK } from "./lib";
import {
  type ValidatedPriceMantissa,
  validatePriceMantissa,
} from "./validated-mantissa";

/**
 * An enum representing a decoded `EncodedPrice`.
 *
 * Port of `DecodedPrice` in `price/src/decoded_price.rs`.
 */
export type DecodedPrice =
  | { kind: "zero" }
  | { kind: "infinity" }
  | {
      kind: "value";
      biasedExponent: U8;
      mantissa: ValidatedPriceMantissa;
    };

/** Port of `DecodedPrice::try_from(EncodedPrice)` in `price/src/decoded_price.rs`. */
export function decodePrice(encoded: number | bigint): DecodedPrice {
  const v = ensureU32(encoded);
  if (v === ENCODED_PRICE_ZERO) return { kind: "zero" };
  if (v === ENCODED_PRICE_INFINITY) return { kind: "infinity" };

  const biasedExponent = ensureU8(v >>> PRICE_MANTISSA_BITS);
  const mantissa = validatePriceMantissa(v & PRICE_MANTISSA_MASK);
  return { kind: "value", biasedExponent, mantissa };
}

/** Port of `Decimal::try_from(DecodedPrice)` in `price/src/decoded_price.rs`. */
export function decodedPriceToDecimal(decoded: DecodedPrice): Decimal {
  switch (decoded.kind) {
    case "zero":
      return new Decimal(0);
    case "infinity":
      throw new Error(PriceError.InfinityIsNotADecimal);
    case "value":
      return decimalPow10(
        new Decimal(decoded.mantissa.value),
        decoded.biasedExponent - BIAS,
      );
  }
}
