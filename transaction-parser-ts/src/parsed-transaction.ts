import type { Address } from "@solana/kit";
import { DROPSET_PROGRAM_ADDRESS } from "@/ts-sdk";
import { type DropsetEvent, parseDropsetEvents } from "./events/index";
import type { ResolvedAccount } from "./parsed-account";
import { type LoadedAddresses, resolveAccounts } from "./parsed-account";
import { type ParsedBalances, resolveBalances } from "./parsed-balances";
import {
  type ResolvedOuterInstruction,
  resolveInstructions,
} from "./parsed-instruction";

export type ParsedTransaction = {
  signature: string;
  slot: bigint;
  blockTime: bigint | null;
  err: unknown;
  fee: bigint;
  accounts: ResolvedAccount[];
  instructions: ResolvedOuterInstruction[];
  balances: ParsedBalances;
  logMessages: readonly string[] | null;
  computeUnitsConsumed: bigint | null;
  dropsetEvents: DropsetEvent[];
};

type RpcTransactionResult = {
  slot: bigint;
  blockTime: bigint | null;
  transaction: {
    signatures: readonly string[];
    message: {
      accountKeys: readonly Address[];
      header: {
        numRequiredSignatures: number;
        numReadonlySignedAccounts: number;
        numReadonlyUnsignedAccounts: number;
      };
      instructions: readonly {
        programIdIndex: number;
        accounts: readonly number[];
        data: string;
      }[];
    };
  };
  meta: {
    err: unknown;
    fee: bigint;
    preBalances: readonly bigint[];
    postBalances: readonly bigint[];
    preTokenBalances?: readonly {
      accountIndex: number;
      uiTokenAmount: { amount: string };
    }[];
    postTokenBalances?: readonly {
      accountIndex: number;
      uiTokenAmount: { amount: string };
    }[];
    innerInstructions?:
      | readonly {
          index: number;
          instructions: readonly {
            programIdIndex: number;
            accounts: readonly number[];
            data: string;
          }[];
        }[]
      | null;
    loadedAddresses?: LoadedAddresses;
    logMessages: readonly string[] | null;
    computeUnitsConsumed?: bigint;
  } | null;
};

export function parseTransaction(
  result: RpcTransactionResult,
): ParsedTransaction {
  const { transaction, meta } = result;

  const accounts = resolveAccounts(
    transaction.message.accountKeys,
    transaction.message.header,
    meta?.loadedAddresses,
  );

  const instructions = resolveInstructions(
    accounts,
    transaction.message.instructions,
    meta?.innerInstructions,
  );

  const balances = resolveBalances(
    accounts,
    meta?.preBalances ?? [],
    meta?.postBalances ?? [],
    meta?.preTokenBalances,
    meta?.postTokenBalances,
  );

  const dropsetEvents: DropsetEvent[] = [];
  for (const outer of instructions) {
    for (const inner of outer.innerInstructions) {
      if (inner.programAddress === DROPSET_PROGRAM_ADDRESS) {
        const events = parseDropsetEvents(inner.data);
        dropsetEvents.push(...events);
      }
    }
  }

  return {
    signature: transaction.signatures[0],
    slot: result.slot,
    blockTime: result.blockTime,
    err: meta?.err ?? null,
    fee: meta?.fee ?? 0n,
    accounts,
    instructions,
    balances,
    logMessages: meta?.logMessages ?? null,
    computeUnitsConsumed: meta?.computeUnitsConsumed ?? null,
    dropsetEvents,
  };
}
