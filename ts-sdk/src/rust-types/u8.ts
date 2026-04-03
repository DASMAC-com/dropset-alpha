import { RustTypeError } from "./error";

const U8_MAX = 0xff;

export { U8_MAX };

/** A `number` validated to be an 8-bit unsigned integer. */
export type U8 = number & { readonly __brand: "U8" };

/** Validates that a value is a u8 and returns it branded. */
export function ensureU8(n: number | bigint): U8 {
  const v = Number(n);
  if (!Number.isInteger(v) || v < 0 || v > U8_MAX) {
    throw new Error(RustTypeError.InvalidU8);
  }
  return v as U8;
}
