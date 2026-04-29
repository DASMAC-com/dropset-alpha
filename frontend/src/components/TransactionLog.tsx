"use client";

import type { Address } from "@solana/addresses";
import { Decimal } from "decimal.js";
import { Activity } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useAgentRegistry } from "@/lib/hooks/use-agent-registry";
import type { AgentRegistryEntry } from "@/lib/queries/fetch-agent-registry";
import { solscanTxUrl } from "@/lib/solana/explorer";
import { truncateAddress } from "@/lib/solana/format";
import { useMarketStore } from "@/lib/stores/market-store";
import {
  type TransactionEntry,
  useTransactionLogStore,
} from "@/lib/stores/transaction-log-store";

const DEFAULT_MARKET = {
  transactions: [] as TransactionEntry[],
  status: "disconnected" as const,
  error: null,
};

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
  trader: AgentRegistryEntry | null;
  signerAddress: Address | null;
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

const TAKER_PERSONALITY_COLORS: Record<string, string> = {
  retail: "bg-sky-500/15 text-sky-400 border-sky-500/30",
  whale: "bg-violet-500/15 text-violet-400 border-violet-500/30",
  sniper: "bg-amber-500/15 text-amber-400 border-amber-500/30",
  noise: "bg-zinc-500/15 text-zinc-400 border-zinc-500/30",
  passive: "bg-emerald-500/15 text-emerald-400 border-emerald-500/30",
  aggressive: "bg-rose-500/15 text-rose-400 border-rose-500/30",
};

const MAKER_BADGE = "bg-indigo-500/15 text-indigo-400 border-indigo-500/30";
const UNKNOWN_BADGE = "bg-muted/40 text-muted-fg border-border";

function traderBadgeClass(trader: AgentRegistryEntry | null): string {
  if (!trader) return UNKNOWN_BADGE;
  if (trader.kind === "maker") return MAKER_BADGE;
  // Agent names look like `retail-1`, `whale-1`, etc. Color by the archetype prefix.
  const archetype = trader.name.split("-")[0];
  return TAKER_PERSONALITY_COLORS[archetype] ?? UNKNOWN_BADGE;
}

function traderLabel(
  trader: AgentRegistryEntry | null,
  signerAddress: Address | null,
): string {
  if (trader) return trader.name;
  if (signerAddress) return truncateAddress(signerAddress);
  return "unknown";
}

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

      <span
        className={`z-10 w-24 shrink-0 truncate rounded border px-1.5 py-0.5 text-center font-mono text-[10px] uppercase tracking-wider ${traderBadgeClass(fill.trader)}`}
        title={fill.signerAddress ?? undefined}
      >
        {traderLabel(fill.trader, fill.signerAddress)}
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

export function TransactionLog({ marketAddress }: { marketAddress: Address }) {
  const marketState = useTransactionLogStore(
    (s) => s.markets[marketAddress as string] ?? DEFAULT_MARKET,
  );
  const { transactions: allTransactions, status } = marketState;
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
    const cleanup = connect(marketAddress);
    return cleanup;
  }, [connect, marketAddress]);

  const view = useMarketStore((s) => s.view);
  const market = useMarketStore((s) => s.market);
  const totalLiquidity = useMemo(
    () => (view ? marketLiquidity(view).total : 0n),
    [view],
  );

  const baseDecimals = market?.base.decimals ?? 0;
  const quoteDecimals = market?.quote.decimals ?? 0;

  const { byPubkey: agentByPubkey } = useAgentRegistry();

  const fills: (FillData & { key: string })[] = useMemo(
    () =>
      transactions
        .filter((tx) => !tx.err)
        .flatMap((tx) => {
          // The fill-emitting transaction is a taker market order, so the
          // first writable signer (i.e. the fee payer) is the trader we want
          // to label. `parsed.accounts` from `resolveAccounts` is ordered with
          // writable signers first.
          const signer =
            tx.parsed.accounts.find((a) => a.signer && a.writable) ?? null;
          const signerAddress = signer?.address ?? null;
          const trader = signerAddress
            ? (agentByPubkey.get(signerAddress) ?? null)
            : null;

          return tx.parsed.dropsetEvents
            .filter((e) => e.kind === "fill")
            .map((fill, i) => ({
              ...fill.data,
              key: `${tx.signature}-${i}`,
              relativeTime: tx.blockTime ? relativeTime(tx.blockTime) : "—",
              signature: tx.signature,
              accounts: tx.parsed.accounts,
              trader,
              signerAddress,
              lastFillPrice: encodedU32ToDecimal(fill.data.encodedPrice),
              baseFilledUi: new Decimal(fill.data.baseFilled.toString())
                .div(new Decimal(10).pow(baseDecimals))
                .toDecimalPlaces(baseDecimals)
                .toString(),
              quoteFilledUi: new Decimal(fill.data.quoteFilled.toString())
                .div(new Decimal(10).pow(quoteDecimals))
                .toDecimalPlaces(quoteDecimals)
                .toString(),
            }));
        }),
    [transactions, baseDecimals, quoteDecimals, agentByPubkey],
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
            <span className="w-24 shrink-0 text-center text-muted-fg/60 text-xs">
              Trader
            </span>
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
                ? Math.min(
                    Number((fill.quoteFilled * 100n) / totalLiquidity),
                    100,
                  )
                : 0
            }
          />
        ))}
      </div>
    </div>
  );
}
