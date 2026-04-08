"use client";

import Link from "next/link";
import { useMemo } from "react";
import { useAllMarkets } from "@/lib/hooks/use-all-markets";
import { buildPrefixMap } from "@/lib/slug";

function formatNumber(n: bigint): string {
  if (n <= BigInt(Number.MAX_SAFE_INTEGER)) {
    const val = Number(n);
    if (val >= 1_000_000) return `${(val / 1_000_000).toFixed(1)}M`;
    if (val >= 1_000) return `${(val / 1_000).toFixed(1)}K`;
  } else {
    if (n >= 1_000_000n) return `${(Number(n / 1_000_000n)).toFixed(1)}M`;
    if (n >= 1_000n) return `${(Number(n / 1_000n)).toFixed(1)}K`;
  }
  return n.toString();
}

export default function Home() {
  const { data: markets, isLoading, error } = useAllMarkets();

  const addresses = useMemo(
    () => markets?.map((m) => m.address) ?? [],
    [markets],
  );

  const prefixMap = useMemo(() => buildPrefixMap(addresses), [addresses]);

  return (
    <div className="mx-auto max-w-6xl px-6 py-8">
      <div className="mb-8">
        <h1 className="font-semibold text-2xl tracking-tight">Markets</h1>
        <p className="mt-1 text-muted-fg">Browse active Dropset markets</p>
      </div>

      {isLoading && <p className="text-zinc-500">Loading markets…</p>}
      {error && <p className="text-red-500">Failed to load markets.</p>}

      {markets && (
        <div className="overflow-x-auto rounded-lg border border-border">
          <table className="w-full border-collapse text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="px-4 py-2.5 text-left font-medium text-muted-fg text-xs uppercase tracking-[0.05em]">
                  Address
                </th>
                <th className="px-4 py-2.5 text-right font-medium font-mono text-muted-fg text-xs uppercase tabular-nums tracking-[0.05em]">
                  Traders
                </th>
                <th className="px-4 py-2.5 text-right font-medium font-mono text-muted-fg text-xs uppercase tabular-nums tracking-[0.05em]">
                  Liquidity
                </th>
                <th className="px-4 py-2.5 text-right font-medium font-mono text-muted-fg text-xs uppercase tabular-nums tracking-[0.05em]">
                  24h Volume
                </th>
              </tr>
            </thead>
            <tbody>
              {markets.map((market) => {
                const shortSlug =
                  prefixMap.get(market.address) ?? market.address;
                return (
                  <tr key={market.address} className="hover:bg-muted">
                    <td className="border-border border-t px-4 py-3">
                      <Link
                        href={`/market/${shortSlug}`}
                        className="text-accent no-underline transition-colors hover:text-accent-hover hover:underline"
                        title={market.address}
                      >
                        <code className="font-mono text-[0.8125rem]">
                          {market.address}
                        </code>
                      </Link>
                    </td>
                    <td className="border-border border-t px-4 py-3 text-right font-mono tabular-nums">
                      {market.traders}
                    </td>
                    <td className="border-border border-t px-4 py-3 text-right font-mono tabular-nums">
                      ${formatNumber(market.liquidity)}
                    </td>
                    <td className="border-border border-t px-4 py-3 text-right font-mono tabular-nums">
                      ${formatNumber(market.volume24h)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
