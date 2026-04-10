import type { Address } from "@solana/kit";
import type { MarketViewAll } from "../dropset-interface";
import type { MarketAccount } from "../generated";
import type { Flatten } from "./utility-types";

export type { Flatten } from "./utility-types";

export type SectorIndex = number;

export type DropsetMarketAccount = Flatten<
  {
    address: Address<string>;
  } & MarketAccount
>;

export type DropsetMarketView = Flatten<
  {
    address: Address<string>;
  } & MarketViewAll
>;
