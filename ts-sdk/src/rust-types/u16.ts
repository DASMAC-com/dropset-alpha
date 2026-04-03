import { RustTypeError } from "./error";

const U16_MAX = 0xffff;

export { U16_MAX };

/** A `number` validated to be a 16-bit unsigned integer. */
export type U16 = number & { readonly __brand: "U16" };

/** Validates that a value is a u16 and returns it branded. */
export function ensureU16(n: number | bigint): U16 {
  const v = Number(n);
  if (!Number.isInteger(v) || v < 0 || v > U16_MAX) {
    throw new Error(RustTypeError.InvalidU16);
  }
  return v as U16;
}
