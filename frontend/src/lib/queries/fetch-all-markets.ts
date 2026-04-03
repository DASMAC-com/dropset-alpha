import { fetchDropsetMarketViews, getRpcClient } from "@/ts-sdk";

export type MarketSummary = {
  address: string;
  traders: number;
  liquidity: number;
  volume24h: number;
};

/**
 * Fetch all markets from the Dropset program.
 * TODO: implement actual RPC / indexer call.
 */
export async function fetchAllMarkets(): Promise<MarketSummary[]> {
  const rpc = getRpcClient();
  const _markets = await fetchDropsetMarketViews(rpc);

  // Stub data — replace with real fetch
  return [
    {
      address: "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
      traders: 142,
      liquidity: 500_000,
      volume24h: 83_200,
    },
    {
      address: "3Mc6vR7BFnsKgPe1j3KQCZLV9xLqqpJrKFm8W8zTMSno",
      traders: 87,
      liquidity: 210_000,
      volume24h: 41_600,
    },
    {
      address: "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
      traders: 310,
      liquidity: 1_200_000,
      volume24h: 290_000,
    },
    {
      address: "AFbX8oGjGpmVFywbVouvhQSRmiW2aR1mohfahi4Y2AdB",
      traders: 24,
      liquidity: 45_000,
      volume24h: 9_800,
    },
    {
      address: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
      traders: 56,
      liquidity: 78_000,
      volume24h: 15_400,
    },
  ];
}
