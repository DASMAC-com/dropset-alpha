import { RustTypeError } from "./error";

const U64_MAX = 0xffff_ffff_ffff_ffffn;

export { U64_MAX };

/** A `bigint` validated to be a 64-bit unsigned integer. */
export type U64 = bigint & { readonly __brand: "U64" };

/** Validates that a value is a u64 and returns it branded. */
export function ensureU64(n: number | bigint): U64 {
  const v = BigInt(n);
  if (v < 0n || v > U64_MAX) {
    throw new Error(RustTypeError.InvalidU64);
  }
  return v as U64;
}
