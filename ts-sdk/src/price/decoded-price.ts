import { Decimal } from "decimal.js";
import { decimalPow10 } from "./client-helpers";
import {
  ENCODED_PRICE_INFINITY,
  ENCODED_PRICE_ZERO,
  type EncodedPrice,
} from "./encoded-price";
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
      biasedExponent: number;
      mantissa: ValidatedPriceMantissa;
    };

/** Port of `DecodedPrice::try_from(EncodedPrice)` in `price/src/decoded_price.rs`. */
export function decodePrice(encoded: EncodedPrice): DecodedPrice {
  if (encoded === ENCODED_PRICE_ZERO) return { kind: "zero" };
  if (encoded === ENCODED_PRICE_INFINITY) return { kind: "infinity" };

  const biasedExponent = encoded >>> PRICE_MANTISSA_BITS;
  const mantissa = validatePriceMantissa(encoded & PRICE_MANTISSA_MASK);
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
