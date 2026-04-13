"use client";

import type { Address } from "@solana/addresses";
import type { AccountMeta, Instruction, TransactionSigner } from "@solana/kit";
import { useWalletSession } from "@solana/react-hooks";
import { getCreateAssociatedTokenIdempotentInstruction } from "@solana-program/token";
import { useMemo } from "react";
import { useMarketStore } from "@/lib/stores/market-store";
import { DROPSET_PROGRAM_ADDRESS, getMarketOrderInstruction } from "@/ts-sdk";

const EVENT_AUTHORITY =
  "GXuSQj95RW5HDLtYCAhFFwaqRWRXYfW3RHyfpeqSaY1i" as Address;

/**
 * Signer used only for codama instruction building — embeds the correct
 * signer account metadata. Actual signing is handled by `useSendTransaction`
 * via the wallet session passed as `authority`.
 */
function createMetadataSigner<T extends string>(
  address: Address<T>,
): TransactionSigner<T> {
  return {
    address,
    signTransactions: async (txs) => txs,
  } as TransactionSigner<T>;
}

/**
 * Strip embedded TransactionSigner references from instruction account metas.
 * Converts `AccountSignerMeta` → plain `AccountMeta` so `useSendTransaction`
 * doesn't see a conflicting signer. The `role` (signer flag) is preserved.
 */
function stripSignerMeta(instruction: Instruction): Instruction {
  return {
    ...instruction,
    accounts: (instruction.accounts ?? []).map((account) => {
      if ("signer" in account) {
        const { signer: _, ...rest } = account;
        return rest as AccountMeta;
      }
      return account;
    }),
  };
}

export function useMarketOrderBuilder() {
  const market = useMarketStore((s) => s.market);
  const userBaseAta = useMarketStore((s) => s.userBaseAta);
  const userQuoteAta = useMarketStore((s) => s.userQuoteAta);
  const session = useWalletSession();

  return useMemo(() => {
    if (!market || !userBaseAta || !userQuoteAta || !session) return null;

    const walletAddress = session.account.address as Address;
    const signer = createMetadataSigner(walletAddress);

    return (orderSize: bigint, isBuy: boolean, outputAtaExists: boolean) => {
      const instructions: Instruction[] = [];

      if (!outputAtaExists) {
        const outputAta = isBuy ? userBaseAta : userQuoteAta;
        const outputMint = isBuy ? market.base : market.quote;
        const createIdempotentAta =
          getCreateAssociatedTokenIdempotentInstruction({
            payer: signer,
            ata: outputAta,
            owner: walletAddress,
            mint: outputMint.mintAddress,
            tokenProgram: outputMint.tokenProgram,
          });
        instructions.push(createIdempotentAta);
      }

      const marketOrder = getMarketOrderInstruction({
        eventAuthority: EVENT_AUTHORITY,
        user: signer,
        marketAccount: market.address,
        baseUserAta: userBaseAta,
        quoteUserAta: userQuoteAta,
        baseMarketAta: market.baseMarketAta,
        quoteMarketAta: market.quoteMarketAta,
        baseMint: market.base.mintAddress,
        quoteMint: market.quote.mintAddress,
        baseTokenProgram: market.base.tokenProgram,
        quoteTokenProgram: market.quote.tokenProgram,
        dropsetProgram: DROPSET_PROGRAM_ADDRESS,
        orderSize,
        isBuy,
        isBase: !isBuy,
      });
      instructions.push(marketOrder);

      return instructions.map(stripSignerMeta);
    };
  }, [market, userBaseAta, userQuoteAta, session]);
}
