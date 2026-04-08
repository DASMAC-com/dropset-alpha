"use client";

import * as Tooltip from "@radix-ui/react-tooltip";
import {
  useSendTransaction,
  useSplToken,
  useWalletConnection,
} from "@solana/react-hooks";
import { Decimal } from "decimal.js";
import { motion } from "framer-motion";
import { AlertTriangle, ArrowUpDown, Wallet } from "lucide-react";
import {
  type ChangeEvent,
  useCallback,
  useMemo,
  useRef,
  useState,
} from "react";
import { useMarketOrderBuilder } from "@/lib/hooks/use-market-order-builder";
import { solscanTokenUrl } from "@/lib/solana/explorer";
import {
  formatBalance,
  sanitizeDecimalInput,
  truncateAddress,
  uiToAtoms,
} from "@/lib/solana/format";
import { type MarketInfo, useMarketStore } from "@/lib/stores/market-store";

// Stub price until we implement the tx events parser.
const STUB_PRICE = new Decimal("123.456789");

type Side = "buy" | "sell";

function InsufficientBalanceWarning() {
  return (
    <Tooltip.Provider delayDuration={0}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <button type="button" className="text-amber-500/80">
            <AlertTriangle size={11} />
          </button>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            side="top"
            sideOffset={4}
            className="z-50 rounded-md bg-foreground px-2.5 py-1.5 text-background text-xs shadow-md"
          >
            Insufficient balance
            <Tooltip.Arrow className="fill-foreground" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}

function TokenRow({
  label,
  mintAddress,
  amount,
  onAmountChange,
  highlighted,
  decimals,
  readOnly,
  connected,
}: {
  label: string;
  mintAddress: string;
  amount: string;
  onAmountChange?: (value: string) => void;
  highlighted: boolean;
  decimals: number;
  readOnly?: boolean;
  connected: boolean;
}) {
  const { balance } = useSplToken(mintAddress);
  const uiBalance = balance?.uiAmount;

  const insufficientBalance =
    connected &&
    !readOnly &&
    !!amount &&
    !!uiBalance &&
    new Decimal(amount).gt(new Decimal(uiBalance));

  const handleChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      if (readOnly || !onAmountChange) return;
      const sanitized = sanitizeDecimalInput(e.target.value);
      const dotIndex = sanitized.indexOf(".");
      if (dotIndex !== -1 && sanitized.length - dotIndex - 1 > decimals) {
        return;
      }
      onAmountChange(sanitized);
    },
    [onAmountChange, decimals, readOnly],
  );

  return (
    <div
      className={`flex items-center justify-between rounded-lg border px-4 py-3 transition-colors ${
        highlighted
          ? "border-accent/80 bg-accent/5"
          : "border-border bg-background"
      }`}
    >
      <div className="flex flex-col gap-0.5">
        <span className="text-muted-fg text-xs uppercase tracking-wide">
          {label}
        </span>
        <a
          href={solscanTokenUrl(mintAddress)}
          target="_blank"
          rel="noopener noreferrer"
          className="font-mono text-foreground text-sm no-underline hover:underline"
        >
          {truncateAddress(mintAddress)}
        </a>
      </div>
      <div className="flex flex-col items-end gap-0.5">
        <div className="flex items-center gap-1 text-muted-fg text-xs">
          {insufficientBalance && <InsufficientBalanceWarning />}
          <Wallet size={11} />
          <span className="font-mono tabular-nums">
            {connected && uiBalance ? formatBalance(uiBalance) : "—"}
          </span>
        </div>
        <div className="flex items-center justify-end">
          <input
            type="text"
            inputMode="decimal"
            placeholder="0.00"
            value={amount}
            onChange={handleChange}
            readOnly={readOnly}
            className={`bg-transparent text-right font-mono text-foreground text-lg outline-none placeholder:text-muted-fg/50 ${
              readOnly ? "cursor-default" : ""
            }`}
            style={{ width: `${(amount || "0.00").length}ch` }}
          />
        </div>
      </div>
    </div>
  );
}

export function SwapPanel() {
  const market = useMarketStore((s) => s.market);
  if (!market) return null;
  return <SwapPanelInner market={market} />;
}

function SwapPanelInner({ market }: { market: MarketInfo }) {
  const {
    connected,
    connect,
    connectors,
    status,
    wallet: session,
  } = useWalletConnection();
  const { send, isSending } = useSendTransaction();
  const buildOrder = useMarketOrderBuilder();
  const baseAtaExists = useMarketStore((s) => s.baseAtaExists);
  const quoteAtaExists = useMarketStore((s) => s.quoteAtaExists);
  const [side, setSide] = useState<Side>("buy");
  const [amount, setAmount] = useState("");
  const [hovering, setHovering] = useState(false);
  const [priceInverted, setPriceInverted] = useState(false);
  const outputRef = useRef("");

  const handleSwapSide = useCallback(() => {
    setAmount(outputRef.current);
    setSide((s) => (s === "buy" ? "sell" : "buy"));
  }, []);

  const handleConnect = useCallback(() => {
    const first = connectors[0];
    if (first) connect(first.id);
  }, [connectors, connect]);

  const atoms = useMemo(() => {
    if (!amount) return 0n;
    const decimals =
      side === "buy" ? market.quote.decimals : market.base.decimals;
    return uiToAtoms(amount, decimals);
  }, [amount, market, side]);

  const isBuy = side === "buy";
  const topLabel = isBuy ? "You pay" : "You sell";
  const topMint = isBuy ? market.quote.mintAddress : market.base.mintAddress;
  const bottomMint = isBuy ? market.base.mintAddress : market.quote.mintAddress;
  const topDecimals = isBuy ? market.quote.decimals : market.base.decimals;
  const bottomDecimals = isBuy ? market.base.decimals : market.quote.decimals;
  // Price is quote-per-base (how much quote for 1 base).
  // Buy: user pays quote, receives base → output = input / price
  // Sell: user pays base, receives quote → output = input * price
  const outputAmount = amount
    ? (isBuy
        ? new Decimal(amount).div(STUB_PRICE)
        : new Decimal(amount).mul(STUB_PRICE)
      )
        .toDecimalPlaces(bottomDecimals)
        .toString()
    : "";
  outputRef.current = outputAmount;

  return (
    <fieldset className="relative rounded-xl border border-border p-4">
      <legend className="px-2 font-semibold text-foreground text-sm">
        Swap
      </legend>

      {/* Greyed-out overlay when wallet is not connected */}
      {!connected && (
        <div className="absolute inset-0 z-10 flex items-center justify-center rounded-xl bg-background/80">
          <button
            type="button"
            onClick={handleConnect}
            disabled={status === "connecting"}
            className="w-[calc(100%-2rem)] rounded-lg bg-accent py-3 font-medium text-base text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
          >
            {status === "connecting" ? "Connecting…" : "Connect Wallet"}
          </button>
        </div>
      )}

      <div className="relative flex flex-col gap-2">
        <TokenRow
          label={topLabel}
          mintAddress={topMint}
          amount={amount}
          onAmountChange={setAmount}
          highlighted
          decimals={topDecimals}
          connected={connected}
        />

        <TokenRow
          label="You receive"
          mintAddress={bottomMint}
          amount={outputAmount}
          highlighted={false}
          decimals={bottomDecimals}
          readOnly
          connected={connected}
        />

        {/* Superimposed swap button centered between the two rows */}
        <div className="absolute inset-x-0 top-1/2 z-10 flex -translate-y-1/2 items-center justify-center">
          <motion.button
            type="button"
            onClick={handleSwapSide}
            onHoverStart={() => setHovering(true)}
            onHoverEnd={() => setHovering(false)}
            animate={hovering ? { rotate: 360 * 1.5 } : { rotate: 0 }}
            transition={{ type: "spring", stiffness: 800, damping: 70 }}
            className="flex h-8 w-8 items-center justify-center rounded-full border border-border bg-background text-muted-fg shadow-sm transition-colors hover:border-accent hover:text-accent"
          >
            <div className="flex items-center">
              <ArrowUpDown size={15} strokeWidth={2} />
            </div>
          </motion.button>
        </div>
      </div>

      <button
        type="button"
        onClick={() => setPriceInverted((v) => !v)}
        className="mt-3 w-full cursor-pointer text-center font-mono text-xs transition-colors hover:text-foreground"
      >
        {priceInverted ? (
          <>
            <span className="font-medium text-foreground">1</span>{" "}
            <span className="text-muted-fg/70">
              {truncateAddress(market.quote.mintAddress)}
            </span>{" "}
            <span className="font-medium text-foreground">
              ≈{" "}
              {new Decimal(1)
                .div(STUB_PRICE)
                .toDecimalPlaces(market.base.decimals)
                .toString()}
            </span>{" "}
            <span className="text-muted-fg/70">
              {truncateAddress(market.base.mintAddress)}
            </span>
          </>
        ) : (
          <>
            <span className="font-medium text-foreground">1</span>{" "}
            <span className="text-muted-fg/70">
              {truncateAddress(market.base.mintAddress)}
            </span>{" "}
            <span className="font-medium text-foreground">
              ≈ {STUB_PRICE.toString()}
            </span>{" "}
            <span className="text-muted-fg/70">
              {truncateAddress(market.quote.mintAddress)}
            </span>
          </>
        )}
      </button>

      <button
        type="button"
        disabled={
          !connected || !buildOrder || !amount || atoms === 0n || isSending
        }
        onClick={async () => {
          if (!buildOrder || atoms === 0n) return;
          const outputAtaExists = isBuy ? baseAtaExists : quoteAtaExists;
          const instructions = buildOrder(atoms, isBuy, outputAtaExists);
          await send({ instructions, authority: session });
        }}
        className="mt-3 w-full rounded-lg bg-accent py-2.5 font-medium text-sm text-white transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
      >
        {isSending
          ? "Submitting…"
          : !amount
            ? "Enter amount"
            : isBuy
              ? "Buy"
              : "Sell"}
      </button>
    </fieldset>
  );
}
