"use client";

import { useMarket } from "@/lib/hooks/use-market";

export function SolBalance() {
  const { lamports } = useMarket();
  if (lamports == null) return null;

  const sol = (Number(lamports) / 1e9).toLocaleString(undefined, {
    maximumFractionDigits: 4,
  });

  return (
    <p className="mt-2 px-4 text-right font-mono text-muted-fg text-xs tabular-nums">
      SOL: {sol}
    </p>
  );
}
