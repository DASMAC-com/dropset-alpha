"use client";

import type { Address } from "@solana/addresses";
import { fetchMint as fetchSplMint } from "@solana-program/token";
import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import { getRpcFromEnv } from "@/lib/env";
import { deriveAta } from "@/lib/solana/derive";
import {
  type MarketInfo,
  type TokenInfo,
  useMarketStore,
} from "@/lib/stores/market-store";
import { type DropsetMarketView, fetchDropsetMarketView } from "@/ts-sdk";

type MarketViewResult = {
  view: DropsetMarketView;
  market: MarketInfo;
};

async function fetchTokenInfo(mint: Address): Promise<TokenInfo | undefined> {
  const rpc = getRpcFromEnv();
  try {
    const account = await fetchSplMint(rpc, mint);
    return {
      mintAddress: mint,
      tokenProgram: account.programAddress,
      decimals: account.data.decimals,
    };
  } catch {
    return undefined;
  }
}

async function fetchMarketViewData(
  address: Address,
): Promise<MarketViewResult | undefined> {
  const rpc = getRpcFromEnv();
  const view = await fetchDropsetMarketView(rpc, address);
  if (!view) return undefined;

  const [base, quote] = await Promise.all([
    fetchTokenInfo(view.header.baseMint),
    fetchTokenInfo(view.header.quoteMint),
  ]);
  if (!base || !quote) return undefined;

  const [baseMarketAta, quoteMarketAta] = await Promise.all([
    deriveAta(view.address, base.mintAddress, base.tokenProgram),
    deriveAta(view.address, quote.mintAddress, quote.tokenProgram),
  ]);

  return {
    view,
    market: {
      address: view.address,
      base,
      quote,
      baseMarketAta,
      quoteMarketAta,
    },
  };
}

export function useMarketView(address: Address) {
  const setView = useMarketStore((s) => s.setView);
  const setMarket = useMarketStore((s) => s.setMarket);
  const clear = useMarketStore((s) => s.clear);

  const query = useQuery({
    queryKey: ["market-view", address],
    queryFn: () => fetchMarketViewData(address),
    enabled: !!address,
  });

  useEffect(() => {
    if (query.data) {
      setView(query.data.view);
      setMarket(query.data.market);
    }
  }, [query.data, setView, setMarket]);

  useEffect(() => {
    return () => clear();
  }, [clear]);

  return query;
}
