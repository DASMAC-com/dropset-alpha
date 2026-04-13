import {
  type Address,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  signature,
} from "@solana/kit";
import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { PUBLIC_RPC_URL, PUBLIC_WS_URL } from "@/lib/env";
import { useMarketStore } from "@/lib/stores/market-store";
import { type ParsedTransaction, parseTransaction } from "@/transaction-parser";

const MAX_TRANSACTIONS = 50;
const FETCH_DELAY_MS = 200;
const RECONNECT_BASE_MS = 3000;
const RECONNECT_MAX_MS = 30000;

export type TransactionEntry = {
  signature: string;
  slot: number;
  blockTime: number | null;
  err: boolean;
  instructionCount: number;
  parsed: ParsedTransaction;
};

type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

type PerMarketState = {
  transactions: TransactionEntry[];
  status: ConnectionStatus;
  error: string | null;
};

type TransactionLogState = {
  markets: Record<string, PerMarketState>;
  connect: (marketAddress: Address) => () => void;
};

type SubscriptionState = {
  abortController: AbortController;
  reconnectTimeout: ReturnType<typeof setTimeout> | null;
  reconnectAttempts: number;
  seenSignatures: Set<string>;
  fetchQueue: string[];
  processing: boolean;
};

const subscriptions = new Map<string, SubscriptionState>();

const rpc = createSolanaRpc(PUBLIC_RPC_URL);

function delay(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

function updateLastFillPrice(parsed: ParsedTransaction) {
  const lastFill = parsed.dropsetEvents.findLast((e) => e.kind === "fill");
  if (lastFill) {
    useMarketStore.getState().setLastEncodedPrice(lastFill.data.encodedPrice);
  }
}

const DEFAULT_MARKET_STATE = {
  transactions: [],
  status: "disconnected" as ConnectionStatus,
  error: null as string | null,
};

export const useTransactionLogStore = create<TransactionLogState>()(
  immer((set) => {
    function getOrCreateSub(key: string): SubscriptionState {
      let sub = subscriptions.get(key);
      if (!sub) {
        sub = {
          abortController: new AbortController(),
          reconnectTimeout: null,
          reconnectAttempts: 0,
          seenSignatures: new Set(),
          fetchQueue: [],
          processing: false,
        };
        subscriptions.set(key, sub);
      }
      return sub;
    }

    function teardown(key: string) {
      const sub = subscriptions.get(key);
      if (!sub) return;
      sub.abortController.abort();
      if (sub.reconnectTimeout) clearTimeout(sub.reconnectTimeout);
      subscriptions.delete(key);
    }

    async function processQueue(key: string) {
      const sub = subscriptions.get(key);
      if (!sub || sub.processing) return;
      sub.processing = true;

      while (sub.fetchQueue.length > 0) {
        if (sub.abortController.signal.aborted) break;

        const sig = sub.fetchQueue.shift();
        if (!sig) break;

        try {
          const result = await rpc
            .getTransaction(signature(sig), {
              commitment: "confirmed",
              encoding: "json",
              maxSupportedTransactionVersion: 0,
            })
            .send();

          if (sub.abortController.signal.aborted) break;

          if (result) {
            const parsed = parseTransaction(result);
            updateLastFillPrice(parsed);

            const entry: TransactionEntry = {
              signature: sig,
              slot: Number(result.slot),
              blockTime: result.blockTime ? Number(result.blockTime) : null,
              err: result.meta?.err !== null && result.meta?.err !== undefined,
              instructionCount: result.transaction.message.instructions.length,
              parsed,
            };

            set((s) => {
              const market = s.markets[key];
              if (!market) return;
              market.transactions.unshift(
                entry as (typeof market.transactions)[number],
              );
              if (market.transactions.length > MAX_TRANSACTIONS) {
                market.transactions.length = MAX_TRANSACTIONS;
              }
            });
          }
        } catch (e) {
          console.error(`Failed to fetch transaction ${sig}:`, e);
        }

        if (sub.fetchQueue.length > 0) {
          await delay(FETCH_DELAY_MS);
        }
      }

      sub.processing = false;
    }

    async function backfillRecent(marketAddress: Address) {
      const key = marketAddress as string;
      const sub = subscriptions.get(key);
      if (!sub) return;

      try {
        const sigs = await rpc
          .getSignaturesForAddress(marketAddress, { limit: 10 })
          .send();

        for (const info of sigs) {
          const sig = info.signature;
          if (sub.seenSignatures.has(sig)) continue;
          sub.seenSignatures.add(sig);
          sub.fetchQueue.push(sig);
        }
        await processQueue(key);
      } catch (e) {
        console.error("Failed to backfill recent transactions:", e);
      }
    }

    async function startSubscription(marketAddress: Address) {
      const key = marketAddress as string;
      const sub = subscriptions.get(key);
      if (!sub) return;

      const { signal } = sub.abortController;

      set((s) => {
        s.markets[key] ??= { ...DEFAULT_MARKET_STATE };
        s.markets[key].status = "connecting";
        s.markets[key].error = null;
      });

      try {
        const rpcSubs = createSolanaRpcSubscriptions(PUBLIC_WS_URL);
        const notifications = await rpcSubs
          .logsNotifications(
            { mentions: [marketAddress] },
            { commitment: "confirmed" },
          )
          .subscribe({ abortSignal: signal });

        set((s) => {
          const market = s.markets[key];
          if (market) market.status = "connected";
        });
        sub.reconnectAttempts = 0;

        for await (const notification of notifications) {
          const sig = notification.value.signature;
          if (sub.seenSignatures.has(sig)) continue;

          sub.seenSignatures.add(sig);
          sub.fetchQueue.push(sig);
          await processQueue(key);
        }
      } catch (e) {
        if (signal.aborted) return;

        const msg = e instanceof Error ? e.message : "Unknown error";
        console.error("Transaction log subscription error:", msg);

        set((s) => {
          const market = s.markets[key];
          if (market) {
            market.status = "error";
            market.error = msg;
          }
        });

        const backoff = Math.min(
          RECONNECT_BASE_MS * 2 ** sub.reconnectAttempts,
          RECONNECT_MAX_MS,
        );
        sub.reconnectAttempts++;

        sub.reconnectTimeout = setTimeout(() => {
          sub.reconnectTimeout = null;
          void startSubscription(marketAddress);
        }, backoff);
      }
    }

    return {
      markets: {},

      connect: (marketAddress: Address) => {
        const key = marketAddress as string;

        // Tear down any existing subscription for this address.
        teardown(key);
        getOrCreateSub(key);

        set((s) => {
          s.markets[key] = { ...DEFAULT_MARKET_STATE };
        });

        void backfillRecent(marketAddress).then(() =>
          startSubscription(marketAddress),
        );

        return () => {
          teardown(key);
          set((s) => {
            delete s.markets[key];
          });
        };
      },
    };
  }),
);
