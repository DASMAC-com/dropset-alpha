"use client";

import type { Address } from "@solana/addresses";
import { useWalletSession } from "@solana/react-hooks";
import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import { getRpcFromEnv } from "@/lib/env";
import { deriveAta } from "@/lib/solana/derive";
import { useMarketStore } from "@/lib/stores/market-store";

async function fetchAtaInfo(ata: Address | null): Promise<boolean> {
  if (!ata) return false;
  const rpc = getRpcFromEnv();
  const res = await rpc.getAccountInfo(ata, { encoding: "base64" }).send();
  return res.value !== null;
}

export function useUserAtas() {
  const market = useMarketStore((s) => s.market);
  const setUserAtas = useMarketStore((s) => s.setUserAtas);
  const setAtaExists = useMarketStore((s) => s.setAtaExists);
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

  const userBaseAta = useMarketStore((s) => s.userBaseAta);
  const userQuoteAta = useMarketStore((s) => s.userQuoteAta);

  const ataExistsQuery = useQuery({
    queryKey: ["ata-exists", userBaseAta, userQuoteAta],
    queryFn: async () => {
      const [baseExists, quoteExists] = await Promise.all([
        fetchAtaInfo(userBaseAta),
        fetchAtaInfo(userQuoteAta),
      ]);
      return { baseExists, quoteExists };
    },
    enabled: !!userBaseAta && !!userQuoteAta,
    staleTime: 10_000,
  });

  useEffect(() => {
    if (ataExistsQuery.data) {
      setAtaExists(
        ataExistsQuery.data.baseExists,
        ataExistsQuery.data.quoteExists,
      );
    }
  }, [ataExistsQuery.data, setAtaExists]);
}
