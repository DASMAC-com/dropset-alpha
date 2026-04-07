"use client";

import type { Address } from "@solana/addresses";
import { useWalletSession } from "@solana/react-hooks";
import { useEffect } from "react";
import { deriveAta } from "@/lib/solana/derive";
import { useMarketStore } from "@/lib/stores/market-store";

export function useUserAtas() {
  const market = useMarketStore((s) => s.market);
  const setUserAtas = useMarketStore((s) => s.setUserAtas);
  const session = useWalletSession();
  const walletAddress = session?.account.address as Address | undefined;

  // ATAs are derived locally (no RPC call), but the derivation is async.
  useEffect(() => {
    if (!walletAddress || !market) return;

    let canceled = false;

    Promise.all([
      deriveAta(
        walletAddress,
        market.base.mintAddress,
        market.base.tokenProgram,
      ),
      deriveAta(
        walletAddress,
        market.quote.mintAddress,
        market.quote.tokenProgram,
      ),
    ])
      .then(([baseAta, quoteAta]) => {
        if (!canceled) setUserAtas(baseAta, quoteAta);
      })
      .catch(() => {
        canceled = true;
      });

    return () => {
      canceled = true;
    };
  }, [walletAddress, market, setUserAtas]);
}
