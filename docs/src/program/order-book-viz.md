# Order Book Visualizer

This interactive diagram walks through how each Dropset instruction affects the bid/ask ladder. Click any instruction button or use Prev/Next to step through the sequence.

<OrderBookViz />

## Reading the diagram

- **Green rows** — bid (buy) orders, sorted highest price first
- **Red rows** — ask (sell) orders, sorted lowest price first
- **Purple highlight** — newly added order in this step
- **Strikethrough** — order that was filled or cancelled in this step
- **Spread** — the gap between the best bid and best ask

## Key takeaways

**`post_order`** inserts a resting order at a specific price. It does not execute immediately — it waits for a counterparty.

**`market_order`** crosses the spread and fills against the best resting price. It leaves no order on the book.

**`cancel_order`** removes a resting order and returns the reserved funds to available balance. The position in the book is freed.

**`batch_replace`** is the most important instruction for market makers. It cancels and re-quotes atomically — there is no window where a maker has no orders on the book between a cancel and a new post.

## The event queue

Every fill and cancellation writes an event to the market's event queue on-chain. These events need to be periodically consumed via `flush_events` to keep the queue from filling up. See [Services](/services/overview) for how the crank bots handle this in practice.
