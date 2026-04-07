import type { Address } from "@solana/addresses";
import { getRpcFromEnv } from "@/lib/env";
import {
  fetchDropsetMarketView,
  marketLiquidity,
  type RpcClient,
} from "@/ts-sdk";
import { fetchDailyVolume } from "./fetch-daily-volume";

export type MarketDetail = {
  address: string;
  traders: number;
  liquidity: bigint;
  volume24h: bigint;
};

/**
 * Fetch a single market by its full address.
 */
export async function fetchMarket(
  address: Address,
  rpc?: RpcClient,
): Promise<MarketDetail | undefined> {
  const rpcClient = getRpcFromEnv(rpc);
  const market = await fetchDropsetMarketView(rpcClient, address);
  if (!market) return undefined;

  const volume24h = (await fetchDailyVolume(address)) ?? 0n;

  return {
    address,
    traders: market.users.size,
    liquidity: marketLiquidity(market).total,
    volume24h,
  };
}
