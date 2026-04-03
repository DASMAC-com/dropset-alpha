/** Port of `OrderInfoError` in `price/src/error.rs`. */
export enum PriceError {
  ExponentUnderflow = "ExponentUnderflow",
  ArithmeticOverflow = "ArithmeticOverflow",
  InvalidPriceMantissa = "InvalidPriceMantissa",
  InvalidBiasedExponent = "InvalidBiasedExponent",
  InfinityIsNotADecimal = "InfinityIsNotADecimal",
  AmountCannotBeZero = "AmountCannotBeZero",
}
