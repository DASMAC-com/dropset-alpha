use cu_bench_dropset::{
    add_funded_maker,
    add_user,
    expand_market,
    fmt_header,
    fmt_subtable,
    measure_cu,
    new_bench_fixture,
    wc,
    ASK_PRICES,
    BASE_UNIT,
    MAX_ORDERS_USIZE,
    MAX_PERMITTED_SECTOR_INCREASE,
};
use dropset_interface::instructions::{
    BatchReplaceInstructionData,
    CancelOrderInstructionData,
    MarketOrderInstructionData,
    PostOrderInstructionData,
    UnvalidatedOrders,
};
use price::{
    client_helpers::{
        sum_base_necessary,
        sum_quote_necessary,
    },
    to_order_info,
    OrderInfoArgs,
};

/// Max orders per side in a single BatchReplace.
const BATCH_AMOUNTS: &[u64] = &[1, 5, MAX_ORDERS_USIZE as u64];
/// Number of resting asks crossed by a single MarketOrder.
const SWAP_FILL_AMOUNTS: &[u64] = &[1, 10, 50];

// ── Single-instruction benchmarks ───────────────────────────────────────────

#[test]
fn cu_deposit() -> anyhow::Result<()> {
    let mut logs = String::new();
    fmt_header(&mut logs, "Deposit");

    let f = new_bench_fixture();

    // Fixture already deposited base to create the seat; measure a subsequent
    // deposit — the hot path where the seat already exists.
    let cu = measure_cu(
        &f,
        f.market_ctx.deposit_base(f.maker, BASE_UNIT, f.seat_index),
    );

    fmt_subtable(&mut logs, "Deposits", &[(1, cu)]);
    eprintln!("{logs}");
    Ok(())
}

#[test]
fn cu_withdraw() -> anyhow::Result<()> {
    let mut logs = String::new();
    fmt_header(&mut logs, "Withdraw");

    let f = new_bench_fixture();

    let cu = measure_cu(
        &f,
        f.market_ctx.withdraw_base(f.maker, BASE_UNIT, f.seat_index),
    );

    fmt_subtable(&mut logs, "Withdrawals", &[(1, cu)]);
    eprintln!("{logs}");
    Ok(())
}

#[test]
fn cu_post_order() -> anyhow::Result<()> {
    let mut logs = String::new();

    for pre_expand in [false, true] {
        fmt_header(&mut logs, "PostOrder");
        if pre_expand {
            wc(&mut logs, "[pre-expanded]");
        }
        let f = new_bench_fixture();
        if pre_expand {
            expand_market(&f);
        }

        let cu = measure_cu(
            &f,
            f.market_ctx.post_order(
                f.maker,
                PostOrderInstructionData::new(
                    OrderInfoArgs::new_unscaled(ASK_PRICES[0], 1),
                    false,
                    f.seat_index,
                ),
            ),
        );

        fmt_subtable(&mut logs, "Orders", &[(1, cu)]);
    }

    eprintln!("{logs}");
    Ok(())
}

#[test]
fn cu_cancel_order() -> anyhow::Result<()> {
    let mut logs = String::new();
    fmt_header(&mut logs, "CancelOrder");

    let f = new_bench_fixture();

    let order_args = OrderInfoArgs::new_unscaled(ASK_PRICES[0], 1);
    let encoded_price = to_order_info(order_args.clone())
        .unwrap()
        .encoded_price
        .as_u32();

    // Setup: place the ask (not measured).
    let res = f.ctx.process_instruction_chain(&[f.market_ctx.post_order(
        f.maker,
        PostOrderInstructionData::new(order_args, false, f.seat_index),
    )]);
    assert!(res.program_result.is_ok(), "setup PostOrder failed");

    let cu = measure_cu(
        &f,
        f.market_ctx.cancel_order(
            f.maker,
            CancelOrderInstructionData::new(encoded_price, false, f.seat_index),
        ),
    );

    fmt_subtable(&mut logs, "Cancels", &[(1, cu)]);
    eprintln!("{logs}");
    Ok(())
}

// ── Batched benchmarks ───────────────────────────────────────────────────────

#[test]
fn cu_batch_replace() -> anyhow::Result<()> {
    let mut logs = String::new();

    // Place: book starts empty, BatchReplace inserts n new asks.
    for pre_expand in [false, true] {
        fmt_header(&mut logs, "BatchReplace (Place)");
        if pre_expand {
            wc(&mut logs, "[pre-expanded]");
        }
        let mut rows = Vec::new();
        for &n in BATCH_AMOUNTS {
            rows.push((n, batch_replace_place(n, pre_expand)));
        }
        fmt_subtable(&mut logs, "Orders", &rows);
    }

    // Cancel: book has n existing asks, BatchReplace replaces with empty.
    for pre_expand in [false, true] {
        fmt_header(&mut logs, "BatchReplace (Cancel)");
        if pre_expand {
            wc(&mut logs, "[pre-expanded]");
        }
        let mut rows = Vec::new();
        for &n in BATCH_AMOUNTS {
            rows.push((n, batch_replace_cancel(n, pre_expand)));
        }
        fmt_subtable(&mut logs, "Cancels", &rows);
    }

    // Replace: book has n existing asks, BatchReplace cancels them all and
    // inserts n new asks — the actual "replace" workload.
    for pre_expand in [false, true] {
        fmt_header(&mut logs, "BatchReplace (Replace)");
        if pre_expand {
            wc(&mut logs, "[pre-expanded]");
        }
        let mut rows = Vec::new();
        for &n in BATCH_AMOUNTS {
            rows.push((n, batch_replace_replace(n, pre_expand)));
        }
        // Each row is CU for 1 cancel + 1 place within a single BatchReplace, amortized over n.
        fmt_subtable(&mut logs, "Pairs (C+P)", &rows);
    }

    eprintln!("{logs}");
    Ok(())
}

/// Place `n` asks via a single BatchReplace into an empty book; return amortized CU per order.
fn batch_replace_place(n: u64, pre_expand: bool) -> u64 {
    let f = new_bench_fixture();
    if pre_expand {
        expand_market(&f);
    }

    let cu = match n as usize {
        1 => {
            let asks = [OrderInfoArgs::new_unscaled(ASK_PRICES[0], 1)];
            measure_cu(
                &f,
                f.market_ctx.batch_replace(
                    f.maker,
                    BatchReplaceInstructionData::new(
                        f.seat_index,
                        UnvalidatedOrders::new([]),
                        UnvalidatedOrders::new(asks),
                    ),
                ),
            )
        }
        5 => {
            let asks: [OrderInfoArgs; 5] =
                core::array::from_fn(|i| OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1));
            measure_cu(
                &f,
                f.market_ctx.batch_replace(
                    f.maker,
                    BatchReplaceInstructionData::new(
                        f.seat_index,
                        UnvalidatedOrders::new([]),
                        UnvalidatedOrders::new(asks),
                    ),
                ),
            )
        }
        MAX_ORDERS_USIZE => {
            let asks: [OrderInfoArgs; MAX_ORDERS_USIZE] =
                core::array::from_fn(|i| OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1));
            measure_cu(
                &f,
                f.market_ctx.batch_replace(
                    f.maker,
                    BatchReplaceInstructionData::new(
                        f.seat_index,
                        UnvalidatedOrders::new([]),
                        UnvalidatedOrders::new(asks),
                    ),
                ),
            )
        }
        _ => unreachable!(),
    };

    cu / n
}

/// Place `n` asks (setup), then BatchReplace with 0 asks; return amortized CU per cancel.
fn batch_replace_cancel(n: u64, pre_expand: bool) -> u64 {
    let f = new_bench_fixture();
    if pre_expand {
        expand_market(&f);
    }

    // Setup: place n asks via PostOrder (not measured).
    for (i, price_mantissa) in ASK_PRICES.into_iter().enumerate() {
        let res = f.ctx.process_instruction_chain(&[f.market_ctx.post_order(
            f.maker,
            PostOrderInstructionData::new(
                OrderInfoArgs::new_unscaled(price_mantissa, 1),
                false,
                f.seat_index,
            ),
        )]);
        assert!(res.program_result.is_ok(), "setup PostOrder {i} failed");
    }

    // Measure: BatchReplace with empty asks cancels all n resting asks.
    let cu = measure_cu(
        &f,
        f.market_ctx.batch_replace(
            f.maker,
            BatchReplaceInstructionData::new(
                f.seat_index,
                UnvalidatedOrders::new([]),
                UnvalidatedOrders::new([]),
            ),
        ),
    );

    cu / n
}

/// Place `n` asks (setup), then BatchReplace with `n` new asks at the same prices; return
/// amortized CU per cancel+place pair — this is the true "replace" workload.
fn batch_replace_replace(n: u64, pre_expand: bool) -> u64 {
    let f = new_bench_fixture();
    if pre_expand {
        expand_market(&f);
    }

    // Setup: place n asks via PostOrder (not measured).
    for (i, price_mantissa) in ASK_PRICES.into_iter().enumerate() {
        let res = f.ctx.process_instruction_chain(&[f.market_ctx.post_order(
            f.maker,
            PostOrderInstructionData::new(
                OrderInfoArgs::new_unscaled(price_mantissa, 1),
                false,
                f.seat_index,
            ),
        )]);
        assert!(res.program_result.is_ok(), "setup PostOrder {i} failed");
    }

    // Measure: BatchReplace cancels the n existing asks and places n new ones.
    let cu = match n as usize {
        1 => {
            let asks = [OrderInfoArgs::new_unscaled(ASK_PRICES[0], 1)];
            measure_cu(
                &f,
                f.market_ctx.batch_replace(
                    f.maker,
                    BatchReplaceInstructionData::new(
                        f.seat_index,
                        UnvalidatedOrders::new([]),
                        UnvalidatedOrders::new(asks),
                    ),
                ),
            )
        }
        5 => {
            let asks: [OrderInfoArgs; 5] =
                core::array::from_fn(|i| OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1));
            measure_cu(
                &f,
                f.market_ctx.batch_replace(
                    f.maker,
                    BatchReplaceInstructionData::new(
                        f.seat_index,
                        UnvalidatedOrders::new([]),
                        UnvalidatedOrders::new(asks),
                    ),
                ),
            )
        }
        MAX_ORDERS_USIZE => {
            let asks: [OrderInfoArgs; MAX_ORDERS_USIZE] =
                core::array::from_fn(|i| OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1));
            measure_cu(
                &f,
                f.market_ctx.batch_replace(
                    f.maker,
                    BatchReplaceInstructionData::new(
                        f.seat_index,
                        UnvalidatedOrders::new([]),
                        UnvalidatedOrders::new(asks),
                    ),
                ),
            )
        }
        _ => unreachable!(),
    };

    cu / n
}

// ── Individual-instruction comparison benchmarks ─────────────────────────────

/// Same workloads as `cu_batch_replace` but using N separate instructions instead of one
/// BatchReplace. Lets you directly compare the amortized cost of batching vs. individual calls.
#[test]
fn cu_individual_orders() -> anyhow::Result<()> {
    let mut logs = String::new();

    // N individual PostOrder calls into an empty book.
    {
        fmt_header(&mut logs, "PostOrder (Individual)");
        let mut rows = Vec::new();
        for &n in BATCH_AMOUNTS {
            rows.push((n, individual_place(n)));
        }
        fmt_subtable(&mut logs, "Orders", &rows);
    }

    // N individual CancelOrder calls (n asks placed as setup).
    {
        fmt_header(&mut logs, "CancelOrder (Individual)");
        let mut rows = Vec::new();
        for &n in BATCH_AMOUNTS {
            rows.push((n, individual_cancel(n)));
        }
        fmt_subtable(&mut logs, "Cancels", &rows);
    }

    // N individual CancelOrder calls followed by N individual PostOrder calls;
    // amortized per cancel+place pair — directly comparable to BatchReplace (Replace).
    {
        fmt_header(&mut logs, "Cancel+Post (Individual)");
        let mut rows = Vec::new();
        for &n in BATCH_AMOUNTS {
            rows.push((n, individual_cancel_and_place(n)));
        }
        // Each row is CU for 1 cancel + 1 place as separate instructions, amortized over n.
        fmt_subtable(&mut logs, "Pairs (C+P)", &rows);
    }

    eprintln!("{logs}");
    Ok(())
}

/// N separate PostOrder calls into an empty book; returns amortized CU per call.
fn individual_place(n: u64) -> u64 {
    let f = new_bench_fixture();
    expand_market(&f);

    let total_cu: u64 = (0..n as usize)
        .map(|i| {
            measure_cu(
                &f,
                f.market_ctx.post_order(
                    f.maker,
                    PostOrderInstructionData::new(
                        OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1),
                        false,
                        f.seat_index,
                    ),
                ),
            )
        })
        .sum();

    total_cu / n
}

/// Place `n` asks (setup), then cancel each one with a separate CancelOrder; returns amortized
/// CU per cancel.
fn individual_cancel(n: u64) -> u64 {
    let f = new_bench_fixture();
    expand_market(&f);

    // Setup: place n asks (not measured).
    for (i, price_mantissa) in ASK_PRICES.into_iter().enumerate() {
        let res = f.ctx.process_instruction_chain(&[f.market_ctx.post_order(
            f.maker,
            PostOrderInstructionData::new(
                OrderInfoArgs::new_unscaled(price_mantissa, 1),
                false,
                f.seat_index,
            ),
        )]);
        assert!(res.program_result.is_ok(), "setup PostOrder {i} failed");
    }

    let total_cu: u64 = (0..n as usize)
        .map(|i| {
            let encoded_price = to_order_info(OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1))
                .unwrap()
                .encoded_price
                .as_u32();
            measure_cu(
                &f,
                f.market_ctx.cancel_order(
                    f.maker,
                    CancelOrderInstructionData::new(encoded_price, false, f.seat_index),
                ),
            )
        })
        .sum();

    total_cu / n
}

/// Place `n` asks (setup), cancel each individually, then re-place each individually; returns
/// amortized CU per cancel+place pair — directly comparable to `batch_replace_replace`.
fn individual_cancel_and_place(n: u64) -> u64 {
    let f = new_bench_fixture();
    expand_market(&f);

    // Setup: place n asks (not measured).
    for (i, price_mantissa) in ASK_PRICES.into_iter().enumerate() {
        let res = f.ctx.process_instruction_chain(&[f.market_ctx.post_order(
            f.maker,
            PostOrderInstructionData::new(
                OrderInfoArgs::new_unscaled(price_mantissa, 1),
                false,
                f.seat_index,
            ),
        )]);
        assert!(res.program_result.is_ok(), "setup PostOrder {i} failed");
    }

    // Measure: n individual CancelOrder calls.
    let cancel_cu: u64 = (0..n as usize)
        .map(|i| {
            let encoded_price = to_order_info(OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1))
                .unwrap()
                .encoded_price
                .as_u32();
            measure_cu(
                &f,
                f.market_ctx.cancel_order(
                    f.maker,
                    CancelOrderInstructionData::new(encoded_price, false, f.seat_index),
                ),
            )
        })
        .sum();

    // Measure: n individual PostOrder calls (re-place the same orders).
    let place_cu: u64 = (0..n as usize)
        .map(|i| {
            measure_cu(
                &f,
                f.market_ctx.post_order(
                    f.maker,
                    PostOrderInstructionData::new(
                        OrderInfoArgs::new_unscaled(ASK_PRICES[i], 1),
                        false,
                        f.seat_index,
                    ),
                ),
            )
        })
        .sum();

    (cancel_cu + place_cu) / n
}

// ── Swap / market-order benchmarks ──────────────────────────────────────────

#[test]
fn cu_market_order() -> anyhow::Result<()> {
    let mut logs = String::new();
    fmt_header(&mut logs, "MarketOrder (Buy)");

    let mut rows = Vec::new();
    for &n in SWAP_FILL_AMOUNTS {
        rows.push((n, market_order_fill(n)));
    }

    fmt_subtable(&mut logs, "Fills", &rows);
    eprintln!("{logs}");
    Ok(())
}

/// Place `n` resting asks across one or more makers, then send a taker market buy that crosses
/// all of them. Each maker is capped at `MAX_ORDERS_USIZE` open orders; additional makers are
/// created automatically so this works for any N regardless of the per-user order limit.
/// Returns amortized CU per fill.
fn market_order_fill(n: u64) -> u64 {
    let f = new_bench_fixture();

    // Each order and each maker seat occupies one sector.
    // Expand enough times to fit all n orders plus ceil(n / MAX_ORDERS_USIZE) maker seats.
    let sectors_needed = n as usize + (n as usize).div_ceil(MAX_ORDERS_USIZE);
    for _ in 0..sectors_needed.div_ceil(MAX_PERMITTED_SECTOR_INCREASE) + 1 {
        expand_market(&f);
    }

    // Prices are distinct, ascending, and within [MANTISSA_DIGITS_LOWER_BOUND,
    // MANTISSA_DIGITS_UPPER_BOUND] (step of 1_000_000 supports up to 89 distinct prices:
    // 10M..99M). base_scalar=1 keeps base_atoms tiny so deposited balances are always
    // sufficient.
    let ask_args: Vec<OrderInfoArgs> = (0..n as usize)
        .map(|i| OrderInfoArgs::new_unscaled(10_000_000 + i as u32 * 1_000_000, 1))
        .collect();

    // Post orders in chunks of MAX_ORDERS_USIZE, one maker per chunk.
    // The first chunk reuses the fixture's pre-funded maker; subsequent chunks create new ones.
    for (chunk_idx, chunk) in ask_args.chunks(MAX_ORDERS_USIZE).enumerate() {
        let (maker, seat_index) = if chunk_idx == 0 {
            (f.maker, f.seat_index)
        } else {
            add_funded_maker(&f)
        };

        for (i, arg) in chunk.iter().enumerate() {
            let res = f.ctx.process_instruction_chain(&[f.market_ctx.post_order(
                maker,
                PostOrderInstructionData::new(arg.clone(), false, seat_index),
            )]);
            assert!(
                res.program_result.is_ok(),
                "setup PostOrder {} failed: {:?}",
                chunk_idx * MAX_ORDERS_USIZE + i,
                res.program_result
            );
        }
    }

    // Setup: add a taker with ATAs and enough quote to fill all n asks.
    let quote_needed = sum_quote_necessary(&ask_args);
    let taker = add_user(&f, 100_000_000);
    let res = f.ctx.process_instruction_chain(&[
        f.market_ctx.base.create_ata_idempotent(&taker, &taker),
        f.market_ctx.quote.create_ata_idempotent(&taker, &taker),
        f.market_ctx
            .quote
            .mint_to_owner(&taker, quote_needed * 10)
            .unwrap(),
    ]);
    assert!(res.program_result.is_ok(), "taker setup failed");

    // Measure: market buy for exactly the base that the n asks offer.
    let base_to_buy = sum_base_necessary(&ask_args);
    let cu = measure_cu(
        &f,
        f.market_ctx.market_order(
            taker,
            MarketOrderInstructionData::new(base_to_buy, true, true),
        ),
    );

    cu / n
}
