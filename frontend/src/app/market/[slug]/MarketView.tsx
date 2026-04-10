"use client";

import type { Address } from "@solana/addresses";
import { SwapPanel } from "@/components/swap/SwapPanel";
import { useMarketView } from "@/lib/hooks/use-market-view";
import { useUserAtas } from "@/lib/hooks/use-user-atas";
import { MarketProvider } from "@/lib/providers/market-provider";
import { marketLiquidity } from "@/ts-sdk";

export function MarketView({ address }: { address: Address }) {
  const { data, isLoading } = useMarketView(address);
  useUserAtas();

  const view = data?.view;
  const liquidity = view ? marketLiquidity(view).total : undefined;

  return (
    <div className="mx-auto max-w-6xl px-6 py-8">
      <div className="mb-8">
        <h1 className="font-semibold text-2xl tracking-tight">Market</h1>
        <p className="mt-1 break-all font-mono text-muted-fg text-sm">
          {address}
        </p>
      </div>

      {isLoading && <p className="text-zinc-500">Loading…</p>}
      {!isLoading && !view && (
        <p className="text-red-500">Couldn't load market</p>
      )}

      {view && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4">
          <div className="flex flex-col gap-1 rounded-lg border border-border p-4">
            <span className="text-muted-fg text-xs uppercase tracking-[0.05em]">
              Traders
            </span>
            <span className="font-semibold text-xl tabular-nums">
              {view.users.size}
            </span>
          </div>
          <div className="flex flex-col gap-1 rounded-lg border border-border p-4">
            <span className="text-muted-fg text-xs uppercase tracking-[0.05em]">
              Liquidity
            </span>
            <span className="font-semibold text-xl tabular-nums">
              ${liquidity?.toLocaleString()}
            </span>
          </div>
        </div>
      )}

      <MarketProvider>
        <div className="mx-auto mt-8 max-w-sm">
          <SwapPanel />
        </div>
      </MarketProvider>
    </div>
  );
}
