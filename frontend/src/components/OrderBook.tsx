"use client";

import { Decimal } from "decimal.js";
import { useMemo } from "react";
import { useMarketStore } from "@/lib/stores/market-store";
import { atomsToUiAmount, encodedU32ToDecimal } from "@/ts-sdk";

const MAX_ROWS = 10;

type OrderRow = {
  price: Decimal;
  sizeAtoms: bigint;
  sizeUi: string;
};

function decodeOrders(
  orders: { encodedPrice: number; baseRemaining: bigint }[],
  baseDecimals: number,
): OrderRow[] {
  return orders.flatMap((o) => {
    try {
      const price = encodedU32ToDecimal(o.encodedPrice);
      const sizeAtoms = o.baseRemaining;
      const sizeUi = atomsToUiAmount(sizeAtoms, baseDecimals)
        .toSignificantDigits(4)
        .toString();
      return [{ price, sizeAtoms, sizeUi }];
    } catch {
      return [];
    }
  });
}

function Row({
  row,
  side,
  barPct,
}: {
  row: OrderRow | null;
  side: "ask" | "bid";
  barPct: number;
}) {
  const isAsk = side === "ask";
  const barColor = isAsk ? "rgba(240,75,90,0.15)" : "rgba(30,135,80,0.15)";
  const flashColor = isAsk ? "rgba(240,75,90,0.4)" : "rgba(30,135,80,0.4)";
  const textColor = isAsk ? "#f04b5a" : "#1e8750";

  const flashKey = row ? row.sizeAtoms.toString() : "empty";

  return (
    <div className="relative flex items-center px-3 py-0.5 text-xs tabular-nums">
      <div
        key={flashKey}
        className="pointer-events-none absolute inset-0"
        style={{
          backgroundColor: flashColor,
          animation: row ? "ob-flash 0.6s ease-out forwards" : "none",
        }}
      />
      <div
        className="pointer-events-none absolute inset-y-0 right-0"
        style={{
          width: row ? `${barPct}%` : "0%",
          backgroundColor: barColor,
          transition: "width 300ms ease-out",
        }}
      />
      <span
        className="z-10 w-28 shrink-0 font-mono"
        style={{ color: row ? textColor : "transparent" }}
      >
        {row?.price.toDecimalPlaces(6).toString() ?? "—"}
      </span>
      <span
        className="z-10 ml-auto font-mono"
        style={{ color: row ? "inherit" : "transparent" }}
      >
        {row?.sizeUi ?? ""}
      </span>
    </div>
  );
}

export function OrderBook() {
  const view = useMarketStore((s) => s.view);
  const market = useMarketStore((s) => s.market);

  const { asks, bids, maxSizeAtoms } = useMemo(() => {
    if (!view || !market) return { asks: [], bids: [], maxSizeAtoms: 1n };

    const baseDecimals = market.base.decimals;

    const decodedAsks = decodeOrders(view.asks, baseDecimals)
      .sort((a, b) => b.price.comparedTo(a.price))
      .slice(0, MAX_ROWS);

    const decodedBids = decodeOrders(view.bids, baseDecimals)
      .sort((a, b) => b.price.comparedTo(a.price))
      .slice(0, MAX_ROWS);

    const maxSizeAtoms = [...decodedAsks, ...decodedBids].reduce(
      (m, o) => (o.sizeAtoms > m ? o.sizeAtoms : m),
      1n,
    );

    return { asks: decodedAsks, bids: decodedBids, maxSizeAtoms };
  }, [view, market]);

  if (!view) return null;

  const barPct = (sizeAtoms: bigint) =>
    Math.max(Number((sizeAtoms * 100n) / maxSizeAtoms), 2);

  const spread =
    asks.length > 0 && bids.length > 0
      ? asks[asks.length - 1].price.minus(bids[0].price)
      : null;

  const askSlots: (OrderRow | null)[] = Array.from(
    { length: MAX_ROWS },
    (_, i) => asks[i] ?? null,
  );
  const bidSlots: (OrderRow | null)[] = Array.from(
    { length: MAX_ROWS },
    (_, i) => bids[i] ?? null,
  );

  return (
    <>
      <style>{`
        @keyframes ob-flash {
          from { opacity: 1; }
          to   { opacity: 0; }
        }
      `}</style>

      <div className="rounded-lg border border-border bg-background">
        <div className="flex items-center justify-between border-border border-b px-4 py-3">
          <h3 className="font-semibold text-foreground text-sm">Order Book</h3>
          {spread && (
            <span className="font-mono text-muted-fg text-xs">
              spread {spread.toDecimalPlaces(6).toString()}
            </span>
          )}
        </div>

        <div className="flex items-center justify-between px-3 py-1 text-[10px] text-muted-fg/60 uppercase tracking-wide">
          <span>Price</span>
          <span>Size (base)</span>
        </div>

        {askSlots.map((row, i) => (
          <Row key={i} row={row} side="ask" barPct={row ? barPct(row.sizeAtoms) : 0} />
        ))}

        <div className="border-border/50 border-t border-b" />

        {bidSlots.map((row, i) => (
          <Row key={i} row={row} side="bid" barPct={row ? barPct(row.sizeAtoms) : 0} />
        ))}
      </div>
    </>
  );
}
