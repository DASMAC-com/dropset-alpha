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
import { DROPSET_PROGRAM_ADDRESS } from "@/ts-sdk";

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

type TransactionLogState = {
  transactions: TransactionEntry[];
  status: ConnectionStatus;
  error: string | null;
};

type TransactionLogActions = {
  connect: () => () => void;
  disconnect: () => void;
  clear: () => void;
};

// Module-level subscription state (outside React).
let abortController: AbortController | null = null;
let reconnectAttempts = 0;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
const seenSignatures = new Set<string>();
const fetchQueue: string[] = [];
let processing = false;

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

export const useTransactionLogStore = create<
  TransactionLogState & TransactionLogActions
>()(
  immer((set) => {
    async function processQueue() {
      if (processing) return;
      processing = true;

      while (fetchQueue.length > 0) {
        const sig = fetchQueue.shift();
        if (!sig) break;

        try {
          const result = await rpc
            .getTransaction(signature(sig), {
              commitment: "confirmed",
              encoding: "json",
              maxSupportedTransactionVersion: 0,
            })
            .send();

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
              s.transactions.unshift(entry as (typeof s.transactions)[number]);
              if (s.transactions.length > MAX_TRANSACTIONS) {
                s.transactions.length = MAX_TRANSACTIONS;
              }
            });
          }
        } catch (e) {
          console.error(`Failed to fetch transaction ${sig}:`, e);
        }

        if (fetchQueue.length > 0) {
          await delay(FETCH_DELAY_MS);
        }
      }

      processing = false;
    }

    async function backfillRecent() {
      try {
        const sigs = await rpc
          .getSignaturesForAddress(DROPSET_PROGRAM_ADDRESS as Address, {
            limit: 10,
          })
          .send();

        for (const info of sigs) {
          const sig = info.signature;
          if (seenSignatures.has(sig)) continue;
          seenSignatures.add(sig);
          fetchQueue.push(sig);
        }
        await processQueue();
      } catch (e) {
        console.error("Failed to backfill recent transactions:", e);
      }
    }

    async function startSubscription() {
      if (abortController) {
        abortController.abort();
      }

      abortController = new AbortController();
      const { signal } = abortController;

      set((s) => {
        s.status = "connecting";
        s.error = null;
      });

      try {
        const rpcSubs = createSolanaRpcSubscriptions(PUBLIC_WS_URL);
        const notifications = await rpcSubs
          .logsNotifications(
            { mentions: [DROPSET_PROGRAM_ADDRESS] },
            { commitment: "confirmed" },
          )
          .subscribe({ abortSignal: signal });

        set((s) => {
          s.status = "connected";
        });
        reconnectAttempts = 0;

        for await (const notification of notifications) {
          const sig = notification.value.signature;
          if (seenSignatures.has(sig)) continue;

          seenSignatures.add(sig);
          fetchQueue.push(sig);
          await processQueue();
        }
      } catch (e) {
        if (signal.aborted) return;

        const msg = e instanceof Error ? e.message : "Unknown error";
        console.error("Transaction log subscription error:", msg);

        set((s) => {
          s.status = "error";
          s.error = msg;
        });

        const backoff = Math.min(
          RECONNECT_BASE_MS * 2 ** reconnectAttempts,
          RECONNECT_MAX_MS,
        );
        reconnectAttempts++;

        reconnectTimeout = setTimeout(() => {
          reconnectTimeout = null;
          void startSubscription();
        }, backoff);
      }
    }

    return {
      transactions: [],
      status: "disconnected" as ConnectionStatus,
      error: null,

      connect: () => {
        void backfillRecent().then(() => startSubscription());
        return () => {
          if (abortController) {
            abortController.abort();
            abortController = null;
          }
          if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
          }
          set((s) => {
            s.status = "disconnected";
          });
        };
      },

      disconnect: () => {
        if (abortController) {
          abortController.abort();
          abortController = null;
        }
        if (reconnectTimeout) {
          clearTimeout(reconnectTimeout);
          reconnectTimeout = null;
        }
        set((s) => {
          s.status = "disconnected";
        });
      },

      clear: () =>
        set((s) => {
          s.transactions = [];
        }),
    };
  }),
);
