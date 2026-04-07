import type { Address } from "@solana/addresses";

/**
 * Fetches a market's daily volume.
 */
export async function fetchDailyVolume(
  _marketAddress: Address,
): Promise<bigint | undefined> {
  if (process.env.NODE_ENV === "development") {
    // Return a non-deterministic, randomly generated value in a development environment.
    return BigInt(Math.trunc(Math.random() * 100_000_000));
  } else {
    // Stub deterministically in a non-dev environment to avoid caching random values on real infra.
    return 0n;
  }
}
