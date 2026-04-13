"use client";

import { lamports as getLamports } from "@solana/kit";
import { useWalletConnection } from "@solana/react-hooks";
import { AnimatePresence, motion } from "framer-motion";
import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { useMarket } from "@/lib/hooks/use-market";
import { getTestRpcClient } from "@/lib/rpc";
import { requestFaucetTransaction } from "@/lib/solana/faucet";
import { truncateAddress } from "@/lib/solana/format";

type FaucetStatus = "idle" | "requesting" | "signing" | "success" | "error";

const ONE_SOL = getLamports(1_000_000_000n);

function BalanceLabel({ value }: { value?: string }) {
  const prev = useRef(value);
  const [flash, setFlash] = useState(false);

  useEffect(() => {
    if (prev.current !== value && prev.current !== undefined) {
      setFlash(true);
    }
    prev.current = value;
  }, [value]);

  return (
    <span className="font-mono text-muted-fg text-xs">
      Balance:{" "}
      <span className="relative">
        {value ?? "0"}
        <AnimatePresence>
          {flash && (
            <motion.span
              className="absolute inset-0 text-amber-400"
              initial={{ opacity: 1 }}
              animate={{ opacity: 0 }}
              transition={{ duration: 1.5, ease: "easeOut" }}
              onAnimationComplete={() => setFlash(false)}
            >
              {value}
            </motion.span>
          )}
        </AnimatePresence>
      </span>
    </span>
  );
}

export default function FaucetPage() {
  const { connected, wallet } = useWalletConnection();
  const {
    market,
    baseMint,
    quoteMint,
    baseBalance,
    quoteBalance,
    lamports,
    refreshBaseBalance,
    refreshQuoteBalance,
  } = useMarket();

  const solUiBalance =
    lamports != null
      ? (Number(lamports) / 1e9).toLocaleString(undefined, {
          maximumFractionDigits: 4,
        })
      : undefined;

  const [amount, setAmount] = useState("1");
  const [status, setStatus] = useState<FaucetStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [lastTx, setLastTx] = useState<string | null>(null);

  const [airdropStatus, setAirdropStatus] = useState<
    "idle" | "requesting" | "success" | "error"
  >("idle");

  const handleAirdrop = async () => {
    if (!wallet) return;
    setAirdropStatus("requesting");
    try {
      await getTestRpcClient()
        .requestAirdrop(wallet.account.address, ONE_SOL)
        .send();
      setAirdropStatus("success");
      setTimeout(() => setAirdropStatus("idle"), 3000);
    } catch (e) {
      console.error("Airdrop failed:", e);
      setAirdropStatus("error");
      setTimeout(() => setAirdropStatus("idle"), 3000);
    }
  };

  const handleRequest = async (isBase: boolean) => {
    if (!wallet?.signTransaction || !wallet.sendTransaction) return;

    setStatus("requesting");
    setError(null);
    setLastTx(null);

    try {
      const tx = await requestFaucetTransaction({
        address: wallet.account.address,
        is_base: isBase,
        amount: Number(amount) || 1,
      });

      setStatus("signing");

      const sig = await wallet.sendTransaction(
        tx as Parameters<typeof wallet.signTransaction>[0],
      );

      setLastTx(sig);
      setStatus("success");
      // Wait for confirmation before refreshing the balance.
      setTimeout(() => {
        void (isBase ? refreshBaseBalance() : refreshQuoteBalance());
      }, 2000);
    } catch (e) {
      console.error(e);
      setStatus("error");
      setError(e instanceof Error ? e.message : "Unknown error");
    }
  };

  const busy = status === "requesting" || status === "signing";

  return (
    <div className="mx-auto max-w-lg px-6 py-8">
      <div className="mb-8">
        <h1 className="font-semibold text-2xl tracking-tight">Faucet</h1>
        <p className="mt-1 text-muted-fg">Request test tokens for market</p>
        <Link
          href={`/market/${market.address}`}
          className="mt-1 block break-all font-mono text-accent text-sm no-underline hover:underline"
        >
          {market.address}
        </Link>
      </div>

      {!connected && (
        <p className="text-muted-fg">Connect your wallet to use the faucet.</p>
      )}

      {connected && (
        <div className="flex flex-col gap-4">
          <div>
            <label
              htmlFor="faucet-amount"
              className="mb-1 block text-muted-fg text-xs uppercase tracking-wide"
            >
              Amount (whole tokens)
            </label>
            <input
              id="faucet-amount"
              type="number"
              min="1"
              value={amount}
              onChange={(e) => {
                const n = Math.trunc(Number.parseFloat(e.target.value));
                setAmount(Number.isFinite(n) && n >= 1 ? String(n) : "");
              }}
              className="w-full rounded-lg border border-border bg-background px-3 py-2 font-mono text-foreground outline-none"
            />
          </div>

          <div className="flex gap-3">
            <div className="flex flex-1 flex-col items-center gap-1.5">
              <button
                type="button"
                disabled={busy}
                onClick={() => handleRequest(true)}
                className="w-full rounded-lg bg-accent py-2.5 font-medium text-sm text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
              >
                {busy
                  ? "Processing…"
                  : `Mint Base${baseMint ? ` (${truncateAddress(baseMint)})` : ""}`}
              </button>
              <BalanceLabel value={baseBalance?.uiAmount} />
            </div>
            <div className="flex flex-1 flex-col items-center gap-1.5">
              <button
                type="button"
                disabled={busy}
                onClick={() => handleRequest(false)}
                className="w-full rounded-lg bg-accent py-2.5 font-medium text-sm text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
              >
                {busy
                  ? "Processing…"
                  : `Mint Quote${quoteMint ? ` (${truncateAddress(quoteMint)})` : ""}`}
              </button>
              <BalanceLabel value={quoteBalance?.uiAmount} />
            </div>
          </div>

          <div className="flex flex-col items-center gap-1.5">
            <button
              type="button"
              disabled={airdropStatus === "requesting"}
              onClick={handleAirdrop}
              className="w-full rounded-lg border border-border bg-muted py-2.5 font-medium text-foreground text-sm transition-colors hover:bg-border disabled:opacity-50"
            >
              {airdropStatus === "requesting"
                ? "Requesting…"
                : airdropStatus === "success"
                  ? "Airdropped 1 SOL!"
                  : airdropStatus === "error"
                    ? "Airdrop failed"
                    : "Airdrop 1 SOL"}
            </button>
            <BalanceLabel value={solUiBalance} />
          </div>

          {status === "success" && lastTx && (
            <p className="text-center text-green-500 text-sm">
              Success! Tx: {truncateAddress(lastTx)}
            </p>
          )}

          {status === "error" && error && (
            <p className="text-center text-red-500 text-sm">{error}</p>
          )}
        </div>
      )}
    </div>
  );
}
