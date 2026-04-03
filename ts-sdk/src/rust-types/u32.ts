import { RustTypeError } from "./error";

const U32_MAX = 0xffff_ffff;

export { U32_MAX };

/** A `number` validated to be a 32-bit unsigned integer. */
export type U32 = number & { readonly __brand: "U32" };

/** Validates that a value is a u32 and returns it branded. */
export function ensureU32(n: number | bigint): U32 {
  const v = Number(n);
  if (!Number.isInteger(v) || v < 0 || v > U32_MAX) {
    throw new Error(RustTypeError.InvalidU32);
  }
  return v as U32;
}
