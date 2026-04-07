import { describe, expect, it } from "@jest/globals";
import {
  ensureU8,
  ensureU16,
  ensureU32,
  ensureU64,
  RustTypeError,
  U8_MAX,
  U16_MAX,
  U32_MAX,
  U64_MAX,
} from "@/ts-sdk/rust-types";

type EnsureFn = (n: number | bigint) => unknown;

function testUnsignedInteger(
  name: string,
  ensure: EnsureFn,
  max: number | bigint,
) {
  const errorName =
    RustTypeError[`Invalid${name}` as keyof typeof RustTypeError];
  const isBigint = typeof max === "bigint";

  describe(name, () => {
    it("accepts 0", () => {
      expect(() => ensure(0)).not.toThrow();
    });

    it("accepts 0 as bigint", () => {
      expect(() => ensure(0n)).not.toThrow();
    });

    it("accepts max", () => {
      expect(() => ensure(max)).not.toThrow();
    });

    it("accepts mid-range value", () => {
      const mid = isBigint
        ? (max as bigint) / 2n
        : Math.floor((max as number) / 2);
      expect(() => ensure(mid)).not.toThrow();
    });

    it("rejects max + 1", () => {
      const overflow = isBigint ? (max as bigint) + 1n : (max as number) + 1;
      expect(() => ensure(overflow)).toThrow(errorName);
    });

    it("rejects negative numbers", () => {
      expect(() => ensure(-1)).toThrow(errorName);
      expect(() => ensure(-1n)).toThrow(errorName);
    });

    if (!isBigint) {
      it("rejects floats", () => {
        expect(() => ensure(1.5)).toThrow(errorName);
        expect(() => ensure(0.1)).toThrow(errorName);
      });

      it("rejects NaN", () => {
        expect(() => ensure(NaN)).toThrow(errorName);
      });

      it("rejects Infinity", () => {
        expect(() => ensure(Infinity)).toThrow(errorName);
        expect(() => ensure(-Infinity)).toThrow(errorName);
      });
    }

    it("returns the correct value", () => {
      const result = ensure(1);
      expect(result).toStrictEqual(isBigint ? 1n : 1);
    });
  });
}

testUnsignedInteger("U8", ensureU8, U8_MAX);
testUnsignedInteger("U16", ensureU16, U16_MAX);
testUnsignedInteger("U32", ensureU32, U32_MAX);
testUnsignedInteger("U64", ensureU64, U64_MAX);
