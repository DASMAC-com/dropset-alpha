# Core Concepts

Before diving into the architecture or SDK, it helps to understand the key concepts that Dropset is built around. These terms appear throughout the codebase and docs.

## Market

A **market** is the central object in Dropset. It represents a trading pair — a `base` token and a `quote` token — and holds the order book state on-chain. A market must be registered before any trading can happen, and can be expanded to support more resting orders as needed.

Key properties of a market:

- Has a `base` mint and a `quote` mint
- Holds a **bid side** (buy orders) and an **ask side** (sell orders)
- Has a **seat** registry for authorized participants
- Emits events (fills, cancels) into an event queue

## Seat

A **seat** is a permissioned slot within a market for a specific trader. Before you can post or cancel orders on a market, you need a seat. Seats can be opened and closed — `close_seat` recovers the rent.

Think of a seat as your "account" within a specific market. You can hold one seat per market.

## Order

An **order** is a resting instruction to buy or sell at a specific price. Orders are posted with `post_order` and removed by being filled (via `market_order` or `batch_replace`) or explicitly cancelled with `cancel_order`.

Each order has:

- A **side** (bid or ask)
- A **price** (in ticks)
- A **size** (in lots)
- An **order ID** used for tracking and cancellation

## Lot and Tick

Raw token amounts are normalized into **lots** (for size) and **ticks** (for price). This keeps the order book compact on-chain and avoids floating-point math.

- A **lot** is the minimum tradeable unit of the base token
- A **tick** is the minimum price increment

Lot and tick sizes are set when a market is registered and cannot be changed.

## Events

Dropset uses an **event queue** to record fills and other state changes. Events are written on-chain but need to be flushed periodically via `flush_events` to reclaim space. Your client (or a crank service) is responsible for consuming and flushing events.

## Deposit and Withdraw

Before you can post orders, you deposit base or quote tokens into the program. The program holds custody of those tokens while your orders rest. When you cancel or your orders are filled, you can withdraw settled funds.

## Market Order vs. Post Order

|                | `post_order`                   | `market_order`            |
| -------------- | ------------------------------ | ------------------------- |
| Type           | Limit order                    | Market order              |
| Rests on book? | Yes, until filled or cancelled | No — executes immediately |
| Price control  | Exact price specified          | Fills at best available   |
| Common use     | Market making                  | Taker execution           |

## Batch Replace

`batch_replace` is an optimized instruction that cancels a set of existing orders and posts new ones in a single transaction. Market makers use this to update their quotes atomically without the overhead of separate cancel + post cycles.
