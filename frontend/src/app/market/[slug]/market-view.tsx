"use client";

import { useMarket } from "@/lib/hooks/use-market";

export function MarketView({ address }: { address: string }) {
  const { data: market, isLoading } = useMarket(address);

  return (
    <div className="page-container">
      <div className="page-header">
        <h1 className="page-title">Market</h1>
        <p className="page-subtitle font-mono text-sm break-all">{address}</p>
      </div>

      {isLoading && <p className="text-zinc-500">Loading…</p>}

      {market && (
        <div className="market-detail-grid">
          <div className="market-detail-card">
            <span className="market-detail-label">Traders</span>
            <span className="market-detail-value">{market.traders}</span>
          </div>
          <div className="market-detail-card">
            <span className="market-detail-label">Liquidity</span>
            <span className="market-detail-value">
              ${market.liquidity.toLocaleString()}
            </span>
          </div>
          <div className="market-detail-card">
            <span className="market-detail-label">24h Volume</span>
            <span className="market-detail-value">
              ${market.volume24h.toLocaleString()}
            </span>
          </div>
          <div className="market-detail-card">
            <span className="market-detail-label">Open Interest</span>
            <span className="market-detail-value">
              ${market.openInterest.toLocaleString()}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
