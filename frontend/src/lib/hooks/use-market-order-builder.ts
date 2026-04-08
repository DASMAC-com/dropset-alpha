"use client";

import type { Address } from "@solana/addresses";
import type { TransactionSigner } from "@solana/kit";
import { useWalletSession } from "@solana/react-hooks";
import { getCreateAssociatedTokenIdempotentInstruction } from "@solana-program/token";
import { useMemo } from "react";
import { useMarketStore } from "@/lib/stores/market-store";
import { DROPSET_PROGRAM_ADDRESS, getMarketOrderInstruction } from "@/ts-sdk";

const EVENT_AUTHORITY =
  "GXuSQj95RW5HDLtYCAhFFwaqRWRXYfW3RHyfpeqSaY1i" as Address;

/**
 * Creates a TransactionSigner that marks an account as a signer in instruction
 * metadata without actually signing. Actual signing is handled by
 * `useSendTransaction` via the wallet session authority.
 */
function createPlaceholderSigner<T extends string>(
  address: Address<T>,
): TransactionSigner<T> {
  return {
    address,
    signTransactions: async (txs) => txs,
  } as TransactionSigner<T>;
}

/**
 * Builds MarketOrder instructions from current store + wallet state.
 *
 * Returns a function that, given order params, produces the full instruction
 * list (ATA creation + market order). Returns null if not ready.
 */
export function useMarketOrderBuilder() {
  const market = useMarketStore((s) => s.market);
  const userBaseAta = useMarketStore((s) => s.userBaseAta);
  const userQuoteAta = useMarketStore((s) => s.userQuoteAta);
  const session = useWalletSession();

  return useMemo(() => {
    if (!market || !userBaseAta || !userQuoteAta || !session) return null;

    const walletAddress = session.account.address as Address;
    const signer = createPlaceholderSigner(walletAddress);

    return (orderSize: bigint, isBuy: boolean, outputAtaExists: boolean) => {
      const instructions = [];

      if (!outputAtaExists) {
        const outputAta = isBuy ? userBaseAta : userQuoteAta;
        const outputMint = isBuy ? market.base : market.quote;
        instructions.push(
          getCreateAssociatedTokenIdempotentInstruction({
            payer: signer,
            ata: outputAta,
            owner: walletAddress,
            mint: outputMint.mintAddress,
            tokenProgram: outputMint.tokenProgram,
          }),
        );
      }

      instructions.push(
        getMarketOrderInstruction({
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
          isBase: true,
        }),
      );

      return instructions;
    };
  }, [market, userBaseAta, userQuoteAta, session]);
}
