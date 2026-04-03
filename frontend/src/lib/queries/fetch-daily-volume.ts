import type { Address } from "@solana/addresses";

/**
 * Fetches a market's daily volume.
 */
export async function fetchDailyVolume(
  _marketAddress: Address,
): Promise<bigint | undefined> {
  // Stub for now until there's a clear way to get the daily volume.
  return BigInt(Math.trunc(Math.random() * 100_000_000));
}
