import {
  type Address,
  getBase58Encoder,
  type ReadonlyUint8Array,
} from "@solana/kit";
import type { ResolvedAccount } from "./parsed-account";

export type ResolvedInstruction = {
  programAddress: Address;
  accounts: ResolvedAccount[];
  data: ReadonlyUint8Array;
};

export type ResolvedOuterInstruction = {
  instruction: ResolvedInstruction;
  innerInstructions: ResolvedInstruction[];
};

type RpcInstruction = {
  programIdIndex: number;
  accounts: readonly number[];
  data: string;
};

type RpcInnerInstructions = {
  index: number;
  instructions: readonly RpcInstruction[];
};

const base58 = getBase58Encoder();

function resolveInstruction(
  raw: RpcInstruction,
  resolvedAccounts: ResolvedAccount[],
): ResolvedInstruction {
  return {
    programAddress: resolvedAccounts[raw.programIdIndex].address,
    accounts: raw.accounts.map((idx) => resolvedAccounts[idx]),
    data: base58.encode(raw.data),
  };
}

/**
 * Resolves outer and inner instructions into structured form with addresses instead of indices.
 *
 * Inner instructions are grouped under their parent outer instruction via the `index` field.
 */
export function resolveInstructions(
  resolvedAccounts: ResolvedAccount[],
  instructions: readonly RpcInstruction[],
  innerInstructions?: readonly RpcInnerInstructions[] | null,
): ResolvedOuterInstruction[] {
  const outers: ResolvedOuterInstruction[] = instructions.map((raw) => ({
    instruction: resolveInstruction(raw, resolvedAccounts),
    innerInstructions: [],
  }));

  if (innerInstructions) {
    for (const group of innerInstructions) {
      const parent = outers[group.index];
      if (!parent) continue;
      for (const raw of group.instructions) {
        parent.innerInstructions.push(
          resolveInstruction(raw, resolvedAccounts),
        );
      }
    }
  }

  return outers;
}
