"use client";

import {
  useBalance,
  useSplToken,
  useWalletConnection,
} from "@solana/react-hooks";
import { MarketContext } from "@/lib/hooks/use-market";
import type { MarketInfo } from "@/lib/stores/market-store";
import { useMarketStore } from "@/lib/stores/market-store";

export function MarketProvider({ children }: { children: React.ReactNode }) {
  const market = useMarketStore((s) => s.market);
  if (!market) return null;
  return <MarketProviderInner market={market}>{children}</MarketProviderInner>;
}

function MarketProviderInner({
  market,
  children,
}: {
  market: MarketInfo;
  children: React.ReactNode;
}) {
  const baseAtaExists = useMarketStore((s) => s.baseAtaExists);
  const quoteAtaExists = useMarketStore((s) => s.quoteAtaExists);
  const userBaseAta = useMarketStore((s) => s.userBaseAta);
  const userQuoteAta = useMarketStore((s) => s.userQuoteAta);

  const { balance: baseBalance, refresh: refreshBase } = useSplToken(
    market.base.mintAddress,
  );
  const { balance: quoteBalance, refresh: refreshQuote } = useSplToken(
    market.quote.mintAddress,
  );

  const { wallet } = useWalletConnection();
  const { lamports } = useBalance(wallet?.account.address);

  return (
    <MarketContext.Provider
      value={{
        market,
        baseMint: market.base.mintAddress,
        quoteMint: market.quote.mintAddress,
        baseDecimals: market.base.decimals,
        quoteDecimals: market.quote.decimals,
        baseBalance,
        quoteBalance,
        lamports: lamports ?? null,
        refreshBaseBalance: async () => {
          await refreshBase();
        },
        refreshQuoteBalance: async () => {
          await refreshQuote();
        },
        baseAtaExists,
        quoteAtaExists,
        userBaseAta,
        userQuoteAta,
      }}
    >
      {children}
    </MarketContext.Provider>
  );
}
