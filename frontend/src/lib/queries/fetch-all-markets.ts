import type { Address } from "@solana/addresses";
import { unstable_cache } from "next/cache";
import { getRpcFromEnv } from "@/lib/env";
import { fetchDropsetMarketViews, marketLiquidity } from "@/ts-sdk";
import { fetchDailyVolume } from "./fetch-daily-volume";

export type MarketSummary<T extends bigint | string> = {
  address: Address;
  traders: number;
  liquidity: T;
  volume24h: T;
};

/**
 * The JSON-serializable version of {@link fetchAllMarkets}.
 */
export async function fetchAllMarketsJson(): Promise<MarketSummary<string>[]> {
  const client = getRpcFromEnv();
  const markets = await fetchDropsetMarketViews(client);

  const marketData = await Promise.all(
    markets.map(async (m) => ({
      address: m.address,
      traders: m.users.size,
      liquidity: marketLiquidity(m).total.toString(),
      volume24h: ((await fetchDailyVolume(m.address)) ?? 0n).toString(),
    })),
  );

  return marketData;
}

function convertStringToBigInt(
  m: MarketSummary<string>,
): MarketSummary<bigint> {
  return {
    ...m,
    liquidity: BigInt(m.liquidity),
    volume24h: BigInt(m.volume24h),
  };
}

/**
 * Fetch all markets from the Dropset program.
 */
export async function fetchAllMarkets(): Promise<MarketSummary<bigint>[]> {
  return (await fetchAllMarketsJson()).map(convertStringToBigInt);
}

const cachedFunction = unstable_cache(fetchAllMarketsJson, [], {
  revalidate: 10,
});

/**
 * Cached {@link fetchAllMarkets}.
 */
export const fetchAllMarketsCached = () =>
  cachedFunction().then((r) => r.map(convertStringToBigInt));
