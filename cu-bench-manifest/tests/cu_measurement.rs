// Clippy: we intentionally hold a `RefCell` borrow across `.await`.
// This is safe only because this `Rc<RefCell<ProgramTestContext>>` is never used concurrently.
// Do not call helpers with the same `context` in parallel (e.g. `join!`, `spawn_local`).
#![allow(clippy::await_holding_refcell_ref)]

use std::{
    fmt::Write,
    rc::Rc,
};

use cu_bench_manifest::{
    batch_update_ix,
    collect_order_indices,
    expand_market,
    measure_ix,
    new_fixture,
    send_tx_measure_cu,
    simple_ask,
    ONE_SOL,
    SOL_UNIT_SIZE,
    USDC_UNIT_SIZE,
};
use manifest::program::{
    batch_update::{
        CancelOrderParams,
        PlaceOrderParams,
    },
    deposit_instruction,
    swap_instruction,
    withdraw_instruction,
};
use solana_program_test::tokio;

const START_INDEX: u64 = 0;

#[tokio::test]
async fn cu_deposit() -> anyhow::Result<()> {
    let (mut test_fixture, _trader_index) = new_fixture().await?;

    test_fixture
        .sol_mint_fixture
        .mint_to(&test_fixture.payer_sol_fixture.key, 10 * SOL_UNIT_SIZE)
        .await;

    let payer = test_fixture.payer();
    let deposit_ix = deposit_instruction(
        &test_fixture.market_fixture.key,
        &payer,
        &test_fixture.sol_mint_fixture.key,
        10 * SOL_UNIT_SIZE,
        &test_fixture.payer_sol_fixture.key,
        spl_token::id(),
        None,
    );

    println!("\n========== CU: Deposit ==========");
    measure_ix(&test_fixture, "Deposit", deposit_ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_batch_update_place_1() -> anyhow::Result<()> {
    let (test_fixture, trader_index) = new_fixture().await?;

    let payer = test_fixture.payer();
    let ix = batch_update_ix(
        &test_fixture,
        &payer,
        Some(trader_index),
        vec![],
        vec![simple_ask(ONE_SOL, 15, 0)],
    );

    println!("\n========== CU: BatchUpdate (place 1) ==========");
    measure_ix(&test_fixture, "BatchUpdate (place 1)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_batch_update_cancel_1() -> anyhow::Result<()> {
    let (mut test_fixture, trader_index) = new_fixture().await?;

    let payer_keypair = test_fixture.payer_keypair();

    // Place one order so we have something to cancel.
    test_fixture
        .batch_update_for_keypair(
            Some(trader_index),
            vec![],
            vec![simple_ask(ONE_SOL, 15, 0)],
            &payer_keypair,
        )
        .await?;

    let order_indices = collect_order_indices(&mut test_fixture).await;
    let payer = test_fixture.payer();
    let ix = batch_update_ix(
        &test_fixture,
        &payer,
        Some(trader_index),
        vec![CancelOrderParams::new_with_hint(
            START_INDEX,
            Some(order_indices[&START_INDEX]),
        )],
        vec![],
    );

    println!("\n========== CU: BatchUpdate (cancel 1) ==========");
    measure_ix(&test_fixture, "BatchUpdate (cancel 1)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_batch_update_cancel_1_place_1() -> anyhow::Result<()> {
    let (mut test_fixture, trader_index) = new_fixture().await?;

    let payer_keypair = test_fixture.payer_keypair();

    // Place one order to cancel.
    test_fixture
        .batch_update_for_keypair(
            Some(trader_index),
            vec![],
            vec![simple_ask(ONE_SOL, 15, 0)],
            &payer_keypair,
        )
        .await?;

    let order_indices = collect_order_indices(&mut test_fixture).await;
    let payer = test_fixture.payer();
    let ix = batch_update_ix(
        &test_fixture,
        &payer,
        Some(trader_index),
        vec![CancelOrderParams::new_with_hint(
            START_INDEX,
            Some(order_indices[&START_INDEX]),
        )],
        vec![simple_ask(ONE_SOL, 16, 0)],
    );

    println!("\n========== CU: BatchUpdate (cancel 1 + place 1) ==========");
    measure_ix(&test_fixture, "BatchUpdate (cancel 1 + place 1)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_batch_update_cancel_4_place_4() -> anyhow::Result<()> {
    let (mut test_fixture, trader_index) = new_fixture().await?;

    let payer_keypair = test_fixture.payer_keypair();

    // Place 4 orders so we have known seq nums to cancel.
    test_fixture
        .batch_update_for_keypair(
            Some(trader_index),
            vec![],
            vec![
                simple_ask(ONE_SOL, 20, 0),
                simple_ask(ONE_SOL, 21, 0),
                simple_ask(ONE_SOL, 22, 0),
                simple_ask(ONE_SOL, 23, 0),
            ],
            &payer_keypair,
        )
        .await?;

    let order_indices = collect_order_indices(&mut test_fixture).await;
    let payer = test_fixture.payer();
    let ix = batch_update_ix(
        &test_fixture,
        &payer,
        Some(trader_index),
        // The canceled orders' sequence numbers and indices are the same (for each order).
        (START_INDEX..=START_INDEX + 3)
            .map(|i| CancelOrderParams::new_with_hint(i, Some(order_indices[&i])))
            .collect(),
        vec![
            simple_ask(ONE_SOL, 30, 0),
            simple_ask(ONE_SOL, 31, 0),
            simple_ask(ONE_SOL, 32, 0),
            simple_ask(ONE_SOL, 33, 0),
        ],
    );

    println!("\n========== CU: BatchUpdate (cancel 4 + place 4) ==========");
    measure_ix(&test_fixture, "BatchUpdate (cancel 4 + place 4)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_swap_fill_1() -> anyhow::Result<()> {
    let (mut test_fixture, _trader_index) = new_fixture().await?;

    println!("\n========== CU: Swap (fill 1 order) ==========");

    // Ensure payer has plenty of USDC.
    test_fixture
        .usdc_mint_fixture
        .mint_to(
            &test_fixture.payer_usdc_fixture.key,
            100_000 * USDC_UNIT_SIZE,
        )
        .await;

    let payer = test_fixture.payer();
    let ix = swap_instruction(
        &test_fixture.market_fixture.key,
        &payer,
        &test_fixture.sol_mint_fixture.key,
        &test_fixture.usdc_mint_fixture.key,
        &test_fixture.payer_sol_fixture.key,
        &test_fixture.payer_usdc_fixture.key,
        ONE_SOL,
        0,
        false, // quote (USDC) is input
        true,  // is_exact_in
        spl_token::id(),
        spl_token::id(),
        false,
    );

    measure_ix(&test_fixture, "Swap (fill 1 order)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_swap_fill_3() -> anyhow::Result<()> {
    let (mut test_fixture, trader_index) = new_fixture().await?;

    println!("\n========== CU: Swap (fill 3 orders) ==========");

    let payer_keypair = test_fixture.payer_keypair();

    // Add 3 more asks at the same price so the swap walks through several.
    test_fixture
        .batch_update_for_keypair(
            Some(trader_index),
            vec![],
            vec![
                simple_ask(ONE_SOL, 10, 0),
                simple_ask(ONE_SOL, 10, 0),
                simple_ask(ONE_SOL, 10, 0),
            ],
            &payer_keypair,
        )
        .await?;

    // Ensure payer has plenty of USDC.
    test_fixture
        .usdc_mint_fixture
        .mint_to(
            &test_fixture.payer_usdc_fixture.key,
            100_000 * USDC_UNIT_SIZE,
        )
        .await;

    let payer = test_fixture.payer();
    let ix = swap_instruction(
        &test_fixture.market_fixture.key,
        &payer,
        &test_fixture.sol_mint_fixture.key,
        &test_fixture.usdc_mint_fixture.key,
        &test_fixture.payer_sol_fixture.key,
        &test_fixture.payer_usdc_fixture.key,
        3 * SOL_UNIT_SIZE,
        0,
        false,
        true,
        spl_token::id(),
        spl_token::id(),
        false,
    );

    measure_ix(&test_fixture, "Swap (fill 3 orders)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_withdraw() -> anyhow::Result<()> {
    let (test_fixture, _trader_index) = new_fixture().await?;

    println!("\n========== CU: Withdraw ==========");

    let payer = test_fixture.payer();
    let ix = withdraw_instruction(
        &test_fixture.market_fixture.key,
        &payer,
        &test_fixture.sol_mint_fixture.key,
        ONE_SOL,
        &test_fixture.payer_sol_fixture.key,
        spl_token::id(),
        None,
    );

    measure_ix(&test_fixture, "Withdraw", ix).await;
    Ok(())
}

// ── Multiple maker orders ──────────────────────────────────────────────────

#[tokio::test]
async fn measure_several_maker_cancel_replace() -> anyhow::Result<()> {
    maker_cancel_replace(20, 5).await
}

#[tokio::test]
async fn measure_many_maker_cancel_replace_1() -> anyhow::Result<()> {
    maker_cancel_replace(50, 5).await
}

#[tokio::test]
async fn measure_many_maker_cancel_replace() -> anyhow::Result<()> {
    maker_cancel_replace(100, 5).await
}

async fn maker_cancel_replace(n_orders: u64, n_rounds: u64) -> anyhow::Result<()> {
    let (mut test_fixture, trader_index) = new_fixture().await?;

    let payer = test_fixture.payer();
    let payer_keypair = test_fixture.payer_keypair();

    writeln!(
        &mut test_fixture.logs,
        "\n========== Multiple maker orders: cancel {n_orders} + place {n_orders}, {n_rounds} times =========="
    )?;

    // Ensure the market has enough free blocks for large batches.
    expand_market(
        Rc::clone(&test_fixture.context),
        &test_fixture.market_fixture.key,
        n_orders as u32 * 2,
    )
    .await?;

    // Place n_orders initial asks to kick things off.
    let initial_places: Vec<PlaceOrderParams> = (0..n_orders)
        .map(|i| simple_ask(ONE_SOL, 20 + i as u32, 0))
        .collect();
    test_fixture
        .batch_update_for_keypair(Some(trader_index), vec![], initial_places, &payer_keypair)
        .await?;

    let mut prev_seq_nums: Vec<u64> = (START_INDEX..START_INDEX + n_orders).collect();
    let mut total_cancel_cu: u64 = 0;
    let mut total_place_cu: u64 = 0;
    let mut total_num_cancels: u64 = 0;
    let mut total_num_places: u64 = 0;

    for round in 0..n_rounds {
        let n_cancels = prev_seq_nums.len() as u64;

        // Look up data indices for the orders we're about to cancel.
        let order_indices = collect_order_indices(&mut test_fixture).await;

        let cancels: Vec<CancelOrderParams> = prev_seq_nums
            .iter()
            .map(|&seq| CancelOrderParams::new_with_hint(seq, Some(order_indices[&seq])))
            .collect();

        let cancel_ix = batch_update_ix(&test_fixture, &payer, Some(trader_index), cancels, vec![]);
        let cancel_cu = send_tx_measure_cu(
            Rc::clone(&test_fixture.context),
            &[cancel_ix],
            Some(&payer),
            &[&payer_keypair],
        )
        .await;

        // Place n_orders new asks.
        let base_price = 100 + round * (n_orders + 10);
        let places: Vec<PlaceOrderParams> = (0..n_orders)
            .map(|i| simple_ask(ONE_SOL, (base_price + i) as u32, 0))
            .collect();

        let place_ix = batch_update_ix(&test_fixture, &payer, Some(trader_index), vec![], places);
        let place_cu = send_tx_measure_cu(
            Rc::clone(&test_fixture.context),
            &[place_ix],
            Some(&payer),
            &[&payer_keypair],
        )
        .await;

        let round_cu = cancel_cu + place_cu;
        let cancel_per = cancel_cu / n_cancels;
        let place_per = place_cu / n_orders;

        total_cancel_cu += cancel_cu;
        total_place_cu += place_cu;
        total_num_cancels += n_cancels;
        total_num_places += n_orders;

        // Each round places n_orders new orders whose seq nums follow sequentially.
        let first_seq = START_INDEX + (round + 1) * n_orders;
        prev_seq_nums = (first_seq..first_seq + n_orders).collect();

        writeln!(
            &mut test_fixture.logs,
            "  Round {} (cancel {:>3} + place {:>3})   {:>6} CU  (avg cancel: {:>4}, avg place: {:>4})",
            round + 1,
            n_cancels,
            n_orders,
            round_cu,
            cancel_per,
            place_per,
        )?;
    }

    writeln!(
        &mut test_fixture.logs,
        "  Average cancel  ({n_rounds} rounds)       {:>6} CU",
        total_cancel_cu / total_num_cancels,
    )?;
    writeln!(
        &mut test_fixture.logs,
        "  Average place                    {:>6} CU",
        total_place_cu / total_num_places,
    )?;

    writeln!(&mut test_fixture.logs, "\n{}", "=".repeat(60))?;

    Ok(())
}
