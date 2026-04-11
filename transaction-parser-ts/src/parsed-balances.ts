import type { Address } from "@solana/kit";
import type { ResolvedAccount } from "./parsed-account";

type RpcTokenBalance = {
  accountIndex: number;
  uiTokenAmount: { amount: string };
};

export type ParsedBalances = {
  preLamportBalances: Map<Address, bigint>;
  postLamportBalances: Map<Address, bigint>;
  preTokenBalances: Map<Address, bigint>;
  postTokenBalances: Map<Address, bigint>;
};

export function resolveBalances(
  accounts: ResolvedAccount[],
  preBalances: readonly bigint[],
  postBalances: readonly bigint[],
  preTokenBalances?: readonly RpcTokenBalance[],
  postTokenBalances?: readonly RpcTokenBalance[],
): ParsedBalances {
  const preLamportBalances = new Map<Address, bigint>();
  const postLamportBalances = new Map<Address, bigint>();

  for (let i = 0; i < accounts.length; i++) {
    const addr = accounts[i].address;
    if (i < preBalances.length) preLamportBalances.set(addr, preBalances[i]);
    if (i < postBalances.length) postLamportBalances.set(addr, postBalances[i]);
  }

  const preTokens = new Map<Address, bigint>();
  if (preTokenBalances) {
    for (const b of preTokenBalances) {
      const account = accounts[b.accountIndex];
      if (account)
        preTokens.set(account.address, BigInt(b.uiTokenAmount.amount));
    }
  }

  const postTokens = new Map<Address, bigint>();
  if (postTokenBalances) {
    for (const b of postTokenBalances) {
      const account = accounts[b.accountIndex];
      if (account)
        postTokens.set(account.address, BigInt(b.uiTokenAmount.amount));
    }
  }

  return {
    preLamportBalances,
    postLamportBalances,
    preTokenBalances: preTokens,
    postTokenBalances: postTokens,
  };
}
