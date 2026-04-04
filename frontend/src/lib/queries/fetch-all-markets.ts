import type { Address } from "@solana/addresses";
import { getRpcFromEnv } from "@/lib/env";
import {
  fetchDropsetMarketViews,
  marketLiquidity,
  type RpcClient,
} from "@/ts-sdk";
import { fetchDailyVolume } from "./fetch-daily-volume";

export type MarketSummary = {
  address: Address;
  traders: number;
  liquidity: bigint;
  volume24h: bigint;
};

/**
 * Fetch all markets from the Dropset program.
 */
export async function fetchAllMarkets(
  rpc?: RpcClient,
): Promise<MarketSummary[]> {
  const client = getRpcFromEnv(rpc);
  const markets = await fetchDropsetMarketViews(client);

  const marketData = await Promise.all(
    markets.map(async (m) => ({
      address: m.address,
      traders: m.users.size,
      liquidity: marketLiquidity(m).total,
      volume24h: (await fetchDailyVolume(m.address)) ?? 0n,
    })),
  );

  return marketData;
}
