/**
 * Flatten a type to remove any nested properties from unions and intersections.
 * {@link https://twitter.com/mattpocockuk/status/1622730173446557697}
 */
export type Flatten<T> = { [K in keyof T]: T[K] } & NonNullable<unknown>;

/**
 * Distributes a union type `T` across the given `Keys`, producing a union of
 * object types where each branch has all keys mapped to the same concrete type.
 *
 * Note that it might be useful to wrap your type with {@link Flatten} to see the
 * flattened type.
 *
 * @example
 * type MarketAddress = { market: Address } & MarketAmounts;
 * type MarketAmounts = Monomorphized<"liquidity" | "volume24h", bigint | string>;
 *
 * // MarketAddress expands to:
 *
 * type MarketAddress = {
 *     market: Address;
 *     liquidity: string;
 *     volume24h: string;
 * } | {
 *     market: Address;
 *     liquidity: bigint;
 *     volume24h: bigint;
 * }
 */
export type Monomorphized<Keys extends string, T> = T extends T
  ? { [K in Keys]: T }
  : never;
