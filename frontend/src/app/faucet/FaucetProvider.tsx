"use client";

import type { Address } from "@solana/addresses";
import { useEffect, useState } from "react";
import { MarketProvider } from "@/lib/providers/market-provider";
import { useMarketStore } from "@/lib/stores/market-store";
import { deriveMarketAddress } from "@/ts-sdk";

type FaucetInfo = {
  base_mint: string;
  quote_mint: string;
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

    deriveMarketAddress(baseMint, quoteMint).then(([marketAddress]) => {
      setMarket({
        address: marketAddress,
        base: {
          mintAddress: baseMint,
          tokenProgram:
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Address,
          decimals: info.base_decimals,
        },
        quote: {
          mintAddress: quoteMint,
          tokenProgram:
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Address,
          decimals: info.quote_decimals,
        },
        baseMarketAta: baseMint,
        quoteMarketAta: quoteMint,
      });
      setReady(true);
    });
  }, [info, market, setMarket]);

  if (!ready) return null;
  return <MarketProvider>{children}</MarketProvider>;
}
