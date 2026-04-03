import { RustTypeError } from "./error";

const U16_MAX = 0xffff;

export { U16_MAX };

/** A `number` validated to be a 16-bit unsigned integer. */
export type U16 = number & { readonly __brand: "U16" };

/** Validates that a value is a u16 and returns it branded. */
export function ensureU16(n: number | bigint): U16 {
  if (typeof n === "number" && !Number.isInteger(n))
    throw new Error(RustTypeError.InvalidU16);
  const v = BigInt(n);
  if (v < 0n || v > BigInt(U16_MAX)) throw new Error(RustTypeError.InvalidU16);
  return Number(v) as U16;
}
