// Constants ported from `price/src/lib.rs`.

export const MANTISSA_DIGITS_LOWER_BOUND = 10_000_000;
export const MANTISSA_DIGITS_UPPER_BOUND = 99_999_999;

export const PRICE_MANTISSA_BITS = 27;
export const PRICE_MANTISSA_MASK = 0xffff_ffff >>> (32 - PRICE_MANTISSA_BITS);
export const PRICE_EXPONENT_MAX = (1 << (32 - PRICE_MANTISSA_BITS)) - 1;

export const BIAS = 16;
export const UNBIASED_MIN = -BIAS;
export const UNBIASED_MAX = (1 << (32 - PRICE_MANTISSA_BITS)) - 1 - BIAS;
