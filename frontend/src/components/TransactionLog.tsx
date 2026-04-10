"use client";

import { AnimatePresence, motion } from "framer-motion";
import { Activity } from "lucide-react";
import { useEffect, useState } from "react";
import { solscanTxUrl } from "@/lib/solana/explorer";
import { truncateAddress } from "@/lib/solana/format";
import {
  type TransactionEntry,
  useTransactionLogStore,
} from "@/lib/stores/transaction-log-store";

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

function fullTimestamp(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleString();
}

function TransactionRow({ tx }: { tx: TransactionEntry }) {
  return (
    <div className="flex items-center gap-3 border-border/50 border-b px-3 py-2 transition-colors last:border-b-0 hover:bg-muted/50">
      <span
        title={tx.blockTime ? fullTimestamp(tx.blockTime) : undefined}
        className="w-16 shrink-0 cursor-default font-mono text-muted-fg text-xs tabular-nums"
      >
        {tx.blockTime ? relativeTime(tx.blockTime) : "—"}
      </span>
      <a
        href={solscanTxUrl(tx.signature)}
        target="_blank"
        rel="noopener noreferrer"
        className="font-mono text-accent text-xs no-underline hover:underline"
      >
        {truncateAddress(tx.signature)}
      </a>
      <span className="ml-auto shrink-0 font-mono text-muted-fg text-xs tabular-nums">
        {tx.instructionCount} ix
      </span>
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

  return (
    <div className="rounded-lg border border-border bg-background">
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
        {transactions.length === 0 && status === "connected" && (
          <div className="flex flex-col items-center justify-center py-12 text-muted-fg">
            <Activity size={24} className="mb-2 opacity-50" />
            <p className="text-sm">Listening for transactions...</p>
          </div>
        )}

        <AnimatePresence initial={false}>
          {transactions
            .filter((tx) => !tx.err)
            .map((tx) => (
              <motion.div
                key={tx.signature}
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                transition={{ type: "spring", stiffness: 500, damping: 30 }}
              >
                <TransactionRow tx={tx} />
              </motion.div>
            ))}
        </AnimatePresence>
      </div>
    </div>
  );
}
