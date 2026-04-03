import { RustTypeError } from "./error";

const U32_MAX = 0xffff_ffff;

export { U32_MAX };

/** A `number` validated to be a 32-bit unsigned integer. */
export type U32 = number & { readonly __brand: "U32" };

/** Validates that a value is a u32 and returns it branded. */
export function ensureU32(n: number | bigint): U32 {
  if (typeof n === "number" && !Number.isSafeInteger(n))
    throw new Error(RustTypeError.InvalidU32);
  const v = BigInt(n);
  if (v < 0n || v > BigInt(U32_MAX)) throw new Error(RustTypeError.InvalidU32);
  return Number(v) as U32;
}
