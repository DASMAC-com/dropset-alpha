import { RustTypeError } from "./error";

const U8_MAX = 0xff;

export { U8_MAX };

/** A `number` validated to be an 8-bit unsigned integer. */
export type U8 = number & { readonly __brand: "U8" };

/** Validates that a value is a u8 and returns it branded. */
export function ensureU8(n: number | bigint): U8 {
  if (typeof n === "number" && !Number.isSafeInteger(n))
    throw new Error(RustTypeError.InvalidU8);
  const v = BigInt(n);
  if (v < 0n || v > BigInt(U8_MAX)) throw new Error(RustTypeError.InvalidU8);
  return Number(v) as U8;
}
