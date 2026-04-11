"use client";

import type { Decimal } from "decimal.js";
import { Activity } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { solscanTxUrl } from "@/lib/solana/explorer";
import { truncateAddress } from "@/lib/solana/format";
import { useMarketStore } from "@/lib/stores/market-store";
import {
  type TransactionEntry,
  useTransactionLogStore,
} from "@/lib/stores/transaction-log-store";
import type { ResolvedAccount } from "@/transaction-parser";
import { encodedU32ToDecimal, marketLiquidity } from "@/ts-sdk";

function relativeTime(unixSeconds: number): string {
  const diff = Math.floor(Date.now() / 1000 - unixSeconds);
  if (diff < 5) return "just now";
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
}

function LiveIndicator() {
  return (
    <span className="flex items-center gap-1.5 text-green-500 text-xs">
      <span className="relative flex h-2 w-2">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
        <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500" />
      </span>
      Live
    </span>
  );
}

type FillData = {
  relativeTime: string;
  signature: string;
  accounts: ResolvedAccount[];
  lastFillPrice: Decimal;
  discriminator: number;
  isBuy: boolean;
  isBase: boolean;
  baseFilled: bigint;
  quoteFilled: bigint;
  baseFilledUi: string;
  quoteFilledUi: string;
  encodedPrice: number;
};

function FillEntry({
  fill,
  barPercent,
}: {
  fill: FillData;
  barPercent: number;
}) {
  const color = fill.isBuy ? "text-green-500" : "text-red-500";
  const barColor = fill.isBuy
    ? "rgba(34, 197, 94, 0.12)"
    : "rgba(239, 68, 68, 0.12)";
  const flashColor = fill.isBuy
    ? "rgba(34, 197, 94, 0.25)"
    : "rgba(239, 68, 68, 0.25)";

  return (
    <div
      className="relative flex items-center gap-3 border-border/50 border-b px-3 py-2 last:border-b-0 hover:bg-muted/50"
      style={
        {
          "--flash-color": flashColor,
          animation: "fill-flash 1.5s ease-out forwards",
        } as React.CSSProperties
      }
    >
      <div
        className="pointer-events-none absolute inset-y-0 left-0"
        style={{ width: `${barPercent}%`, backgroundColor: barColor }}
      />
      <span className="w-16 shrink-0 font-mono text-muted-fg text-xs tabular-nums">
        {fill.relativeTime}
      </span>

      <span className={`w-20 shrink-0 font-mono text-xs tabular-nums ${color}`}>
        {fill.lastFillPrice.toDecimalPlaces(6).toString()}
      </span>

      <span className="z-10 w-24 shrink-0 text-right font-mono text-foreground text-xs tabular-nums">
        {fill.baseFilledUi}
      </span>

      <span className="z-10 w-24 shrink-0 text-right font-mono text-muted-fg text-xs tabular-nums">
        {fill.quoteFilledUi}
      </span>

      <a
        href={solscanTxUrl(fill.signature)}
        target="_blank"
        rel="noopener noreferrer"
        className="ml-auto font-mono text-accent text-xs no-underline hover:underline"
      >
        {truncateAddress(fill.signature)}
      </a>
    </div>
  );
}

export function TransactionLog() {
  const allTransactions = useTransactionLogStore((s) => s.transactions);
  const status = useTransactionLogStore((s) => s.status);
  const connect = useTransactionLogStore((s) => s.connect);
  const [hovered, setHovered] = useState(false);
  const [frozen, setFrozen] = useState<TransactionEntry[]>([]);

  const transactions = hovered ? frozen : allTransactions;

  // Force re-render every 10s to update relative timestamps.
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 10_000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    const cleanup = connect();
    return cleanup;
  }, [connect]);

  const view = useMarketStore((s) => s.view);
  const market = useMarketStore((s) => s.market);
  const totalLiquidity = useMemo(
    () => (view ? marketLiquidity(view).total : 0n),
    [view],
  );

  const baseDecimals = market?.base.decimals ?? 0;
  const quoteDecimals = market?.quote.decimals ?? 0;

  const fills: (FillData & { key: string })[] = useMemo(
    () =>
      transactions
        .filter((tx) => !tx.err)
        .flatMap((tx) =>
          tx.parsed.dropsetEvents
            .filter((e) => e.kind === "fill")
            .map((fill, i) => ({
              ...fill.data,
              key: `${tx.signature}-${i}`,
              relativeTime: tx.blockTime ? relativeTime(tx.blockTime) : "—",
              signature: tx.signature,
              accounts: tx.parsed.accounts,
              lastFillPrice: encodedU32ToDecimal(fill.data.encodedPrice),
              baseFilledUi: (
                Number(fill.data.baseFilled) /
                10 ** baseDecimals
              ).toLocaleString(undefined, {
                maximumFractionDigits: baseDecimals,
              }),
              quoteFilledUi: (
                Number(fill.data.quoteFilled) /
                10 ** quoteDecimals
              ).toLocaleString(undefined, {
                maximumFractionDigits: quoteDecimals,
              }),
            })),
        ),
    [transactions, baseDecimals, quoteDecimals],
  );

  return (
    <div className="rounded-lg border border-border bg-background">
      <style>{`
        @keyframes fill-flash {
          from { background-color: var(--flash-color); }
          to { background-color: transparent; }
        }
      `}</style>

      <div className="flex items-center justify-between border-border border-b px-4 py-3">
        <h3 className="font-semibold text-foreground text-sm">
          Transaction Log
        </h3>
        {status === "connected" && <LiveIndicator />}
        {status === "connecting" && (
          <span className="text-muted-fg text-xs">Connecting...</span>
        )}
        {status === "error" && (
          <span className="text-red-500 text-xs">Reconnecting...</span>
        )}
      </div>

      <div
        role="log"
        className="max-h-100 overflow-y-auto"
        onMouseEnter={() => {
          setHovered(true);
          setFrozen(allTransactions);
        }}
        onMouseLeave={() => setHovered(false)}
      >
        {fills.length > 0 && (
          <div className="flex items-center gap-3 border-border/50 border-b px-3 py-1.5">
            <span className="w-16 shrink-0 text-muted-fg/60 text-xs">Time</span>
            <span className="w-20 shrink-0 text-muted-fg/60 text-xs">
              Price
            </span>
            <span className="w-24 shrink-0 text-right text-muted-fg/60 text-xs">
              Base
            </span>
            <span className="w-24 shrink-0 text-right text-muted-fg/60 text-xs">
              Quote
            </span>
            <span className="ml-auto text-muted-fg/60 text-xs">Tx</span>
          </div>
        )}

        {fills.length === 0 && status === "connected" && (
          <div className="flex flex-col items-center justify-center py-12 text-muted-fg">
            <Activity size={24} className="mb-2 opacity-50" />
            <p className="text-sm">Listening for transactions...</p>
          </div>
        )}

        {fills.map((fill) => (
          <FillEntry
            key={fill.key}
            fill={fill}
            barPercent={
              totalLiquidity > 0n
                ? Number((fill.quoteFilled * 100n) / totalLiquidity)
                : 0
            }
          />
        ))}
      </div>
    </div>
  );
}
