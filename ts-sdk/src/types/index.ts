import type { Address } from "@solana/kit";
import type { MarketViewAll } from "../dropset-interface";
import type { MarketAccount } from "../generated";

/**
 * Flatten a type to remove any nested properties from unions and intersections.
 * {@link https://twitter.com/mattpocockuk/status/1622730173446557697}
 */
export type Flatten<T> = { [K in keyof T]: T[K] } & NonNullable<unknown>;

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
