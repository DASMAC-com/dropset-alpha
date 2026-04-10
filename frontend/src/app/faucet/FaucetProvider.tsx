"use client";

import type { Address } from "@solana/addresses";
import { useEffect, useState } from "react";
import { MarketProvider } from "@/lib/providers/market-provider";
import { useMarketStore } from "@/lib/stores/market-store";

type FaucetInfo = {
  market: Address;
  base_mint: Address;
  quote_mint: Address;
  base_token_program: Address;
  quote_token_program: Address;
  base_decimals: number;
  quote_decimals: number;
};

export function FaucetProvider({
  info,
  children,
}: {
  info: FaucetInfo;
  children: React.ReactNode;
}) {
  const setMarket = useMarketStore((s) => s.setMarket);
  const market = useMarketStore((s) => s.market);
  const [ready, setReady] = useState(!!market);

  useEffect(() => {
    if (market) {
      setReady(true);
      return;
    }

    const baseMint = info.base_mint as Address;
    const quoteMint = info.quote_mint as Address;

    setMarket({
      address: info.market,
      base: {
        mintAddress: baseMint,
        tokenProgram: info.base_token_program,
        decimals: info.base_decimals,
      },
      quote: {
        mintAddress: quoteMint,
        tokenProgram: info.quote_token_program,
        decimals: info.quote_decimals,
      },
      baseMarketAta: baseMint,
      quoteMarketAta: quoteMint,
    });
    setReady(true);
  }, [info, market, setMarket]);

  if (!ready) return null;
  return <MarketProvider>{children}</MarketProvider>;
}
