"use client";

import Link from "next/link";
import { useAllMarkets } from "@/lib/hooks/use-all-markets";
import { buildPrefixMap } from "@/lib/slug";

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

export default function Home() {
  const { data: markets, isLoading, error } = useAllMarkets();

  const prefixMap = markets ? buildPrefixMap(markets.map((m) => m.address)) : new Map();

  return (
    <div className="page-container">
      <div className="page-header">
        <h1 className="page-title">Markets</h1>
        <p className="page-subtitle">Browse active Dropset markets</p>
      </div>

      {isLoading && <p className="text-zinc-500">Loading markets…</p>}
      {error && <p className="text-red-500">Failed to load markets.</p>}

      {markets && (
        <div className="market-table-wrapper">
          <table className="market-table">
            <thead>
              <tr>
                <th>Address</th>
                <th className="numeric">Traders</th>
                <th className="numeric">Liquidity</th>
                <th className="numeric">24h Volume</th>
              </tr>
            </thead>
            <tbody>
              {markets.map((market) => {
                const shortSlug = prefixMap.get(market.address) ?? market.address;
                return (
                  <tr key={market.address}>
                    <td>
                      <Link
                        href={`/market/${shortSlug}`}
                        className="market-link"
                        title={market.address}
                      >
                        <code>{market.address}</code>
                      </Link>
                    </td>
                    <td className="numeric">{market.traders}</td>
                    <td className="numeric">${formatNumber(market.liquidity)}</td>
                    <td className="numeric">${formatNumber(market.volume24h)}</td>
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
