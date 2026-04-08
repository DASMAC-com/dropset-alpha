import type { Address } from "@solana/addresses";
import { enableMapSet } from "immer";
import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { DropsetMarketView } from "@/ts-sdk";

enableMapSet();

/** Mirrors the Rust `TokenContext` in `client/src/context/token.rs`. */
export type TokenInfo = {
  mintAddress: Address;
  tokenProgram: Address;
  decimals: number;
};

/** Mirrors the Rust `MarketContext` in `client/src/context/market.rs`. */
export type MarketInfo = {
  address: Address;
  base: TokenInfo;
  quote: TokenInfo;
  baseMarketAta: Address;
  quoteMarketAta: Address;
};

type MarketState = {
  view: DropsetMarketView | null;
  market: MarketInfo | null;
  userBaseAta: Address | null;
  userQuoteAta: Address | null;
  baseAtaExists: boolean;
  quoteAtaExists: boolean;
};

type MarketActions = {
  setView: (view: DropsetMarketView) => void;
  setMarket: (info: MarketInfo) => void;
  setUserAtas: (baseAta: Address, quoteAta: Address) => void;
  setAtaExists: (base: boolean, quote: boolean) => void;
  clear: () => void;
};

export const useMarketStore = create<MarketState & MarketActions>()(
  immer((set) => ({
    view: null,
    market: null,
    userBaseAta: null,
    userQuoteAta: null,
    baseAtaExists: false,
    quoteAtaExists: false,

    setView: (view) =>
      set((s) => {
        s.view = view;
      }),

    setMarket: (info) =>
      set((s) => {
        s.market = info;
      }),

    setUserAtas: (baseAta, quoteAta) =>
      set((s) => {
        s.userBaseAta = baseAta;
        s.userQuoteAta = quoteAta;
      }),

    setAtaExists: (base, quote) =>
      set((s) => {
        s.baseAtaExists = base;
        s.quoteAtaExists = quote;
      }),

    clear: () =>
      set((s) => {
        s.view = null;
        s.market = null;
        s.userBaseAta = null;
        s.userQuoteAta = null;
        s.baseAtaExists = false;
        s.quoteAtaExists = false;
      }),
  })),
);
