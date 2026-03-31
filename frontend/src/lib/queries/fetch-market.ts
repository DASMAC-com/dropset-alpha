export type MarketDetail = {
  address: string;
  traders: number;
  liquidity: number;
  volume24h: number;
  openInterest: number;
  createdAt: string;
};

/**
 * Fetch a single market by its full address.
 * TODO: implement actual RPC / indexer call.
 */
export async function fetchMarket(address: string): Promise<MarketDetail> {
  // Stub — replace with real fetch
  return {
    address,
    traders: 142,
    liquidity: 500_000,
    volume24h: 83_200,
    openInterest: 320_000,
    createdAt: new Date().toISOString(),
  };
}
