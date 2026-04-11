"use client";

import type { Address } from "@solana/addresses";
import type { useSplToken } from "@solana/react-hooks";
import { createContext, useContext } from "react";
import type { MarketInfo } from "@/lib/stores/market-store";

export type SplTokenBalance = ReturnType<typeof useSplToken>["balance"];

export type MarketContextValue = {
  market: MarketInfo;
  baseMint: Address;
  quoteMint: Address;
  baseDecimals: number;
  quoteDecimals: number;
  baseBalance: SplTokenBalance | null;
  quoteBalance: SplTokenBalance | null;
  lamports: bigint | null;
  refreshBaseBalance: () => Promise<void>;
  refreshQuoteBalance: () => Promise<void>;
  baseAtaExists: boolean;
  quoteAtaExists: boolean;
  userBaseAta: Address | null;
  userQuoteAta: Address | null;
};

export const MarketContext = createContext<MarketContextValue | null>(null);

export function useMarket(): MarketContextValue {
  const ctx = useContext(MarketContext);
  if (!ctx) throw new Error("useMarket must be used inside MarketProvider");
  return ctx;
}
