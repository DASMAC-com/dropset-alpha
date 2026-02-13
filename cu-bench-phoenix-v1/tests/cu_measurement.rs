use std::fmt::Write;

use cu_bench_phoenix_v1::{
    clone_keypair,
    create_ata_pub,
    ioc_buy,
    measure_ix,
    mint_to_pub,
    new_warmed_fixture,
    send_tx_measure_cu,
    simple_post_only_ask,
    NUM_BASE_LOTS_PER_BASE_UNIT,
    QUOTE_UNIT,
};
use phoenix::program::{
    deposit::DepositParams,
    instruction_builders::*,
    new_order::{
        CondensedOrder,
        MultipleOrderPacket,
    },
};
use solana_program_test::tokio;
use solana_sdk::signature::Signer;

#[tokio::test]
async fn cu_deposit() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;
    let maker = f.maker_keypair();

    let ix = create_deposit_funds_instruction(
        &f.market,
        &maker.pubkey(),
        &f.base_mint,
        &f.quote_mint,
        &DepositParams {
            quote_lots_to_deposit: 0,
            base_lots_to_deposit: 10 * NUM_BASE_LOTS_PER_BASE_UNIT,
        },
    );

    println!("\n========== CU: Deposit ==========");
    measure_ix(&mut f, "Deposit", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_place_limit_order_1() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;
    let maker = f.maker_keypair();

    let order = simple_post_only_ask(1600, 10);
    let ix = create_new_order_instruction(
        &f.market,
        &maker.pubkey(),
        &f.base_mint,
        &f.quote_mint,
        &order,
    );

    println!("\n========== CU: PlaceLimitOrder (place 1) ==========");
    measure_ix(&mut f, "PlaceLimitOrder (place 1)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_place_multiple_4() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;
    let maker = f.maker_keypair();

    let ix = create_new_multiple_order_instruction(
        &f.market,
        &maker.pubkey(),
        &f.base_mint,
        &f.quote_mint,
        &MultipleOrderPacket {
            bids: vec![],
            asks: vec![
                CondensedOrder {
                    price_in_ticks: 2000,
                    size_in_base_lots: 10,
                },
                CondensedOrder {
                    price_in_ticks: 2100,
                    size_in_base_lots: 10,
                },
                CondensedOrder {
                    price_in_ticks: 2200,
                    size_in_base_lots: 10,
                },
                CondensedOrder {
                    price_in_ticks: 2300,
                    size_in_base_lots: 10,
                },
            ],
            client_order_id: None,
            reject_post_only: true,
        },
    );

    println!("\n========== CU: PlaceMultiplePostOnly (place 4) ==========");
    measure_ix(&mut f, "PlaceMultiplePostOnly (place 4)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_cancel_all() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;
    let maker = f.maker_keypair();

    let ix = create_cancel_all_order_with_free_funds_instruction(&f.market, &maker.pubkey());

    println!("\n========== CU: CancelAllOrders (with free funds) ==========");
    measure_ix(&mut f, "CancelAllOrders (free funds)", ix).await;
    Ok(())
}

#[tokio::test]
async fn cu_swap_fill_1() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;

    // Use the payer as the taker.
    let payer = f.payer_keypair();

    // Create payer's base ATA and fund payer's quote ATA.
    create_ata_pub(&mut f.context, &payer.pubkey(), &f.base_mint).await;
    let payer_quote_ata =
        spl_associated_token_account::get_associated_token_address(&payer.pubkey(), &f.quote_mint);
    let mint_auth = clone_keypair(&f.mint_authority);
    mint_to_pub(
        &mut f.context,
        &mint_auth,
        &f.quote_mint,
        &payer_quote_ata,
        100_000 * QUOTE_UNIT,
    )
    .await;

    // IOC buy: fill 1 resting ask. Warmup asks at ticks 1100-1500, 10 base lots each.
    // Buy 10 base lots at up to tick 1200 → fills the ask at 1100.
    let order = ioc_buy(1200, 10);
    let ix = create_new_order_instruction(
        &f.market,
        &payer.pubkey(),
        &f.base_mint,
        &f.quote_mint,
        &order,
    );

    println!("\n========== CU: Swap (fill 1 order) ==========");
    let cu = send_tx_measure_cu(&mut f.context, &[ix], &[]).await;
    println!("{:<40} {:>6} CU", "Swap (fill 1 order)", cu);
    Ok(())
}

#[tokio::test]
async fn cu_swap_fill_3() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;

    let payer = f.payer_keypair();

    // Create payer's base ATA and fund quote ATA.
    create_ata_pub(&mut f.context, &payer.pubkey(), &f.base_mint).await;
    let payer_quote_ata =
        spl_associated_token_account::get_associated_token_address(&payer.pubkey(), &f.quote_mint);
    let mint_auth = clone_keypair(&f.mint_authority);
    mint_to_pub(
        &mut f.context,
        &mint_auth,
        &f.quote_mint,
        &payer_quote_ata,
        100_000 * QUOTE_UNIT,
    )
    .await;

    // IOC buy: fill 3 resting asks (at ticks 1100, 1200, 1300).
    // Buy 30 base lots at up to tick 1400.
    let order = ioc_buy(1400, 30);
    let ix = create_new_order_instruction(
        &f.market,
        &payer.pubkey(),
        &f.base_mint,
        &f.quote_mint,
        &order,
    );

    println!("\n========== CU: Swap (fill 3 orders) ==========");
    let cu = send_tx_measure_cu(&mut f.context, &[ix], &[]).await;
    println!("{:<40} {:>6} CU", "Swap (fill 3 orders)", cu);
    Ok(())
}

#[tokio::test]
async fn cu_withdraw() -> anyhow::Result<()> {
    let mut f = new_warmed_fixture().await?;
    let maker = f.maker_keypair();

    let ix =
        create_withdraw_funds_instruction(&f.market, &maker.pubkey(), &f.base_mint, &f.quote_mint);

    println!("\n========== CU: Withdraw ==========");
    measure_ix(&mut f, "WithdrawFunds", ix).await;
    Ok(())
}

// ── Maker spam test ─────────────────────────────────────────────────────────

#[tokio::test]
async fn measure_many_maker_cancel_replace() -> anyhow::Result<()> {
    const N_ORDERS: usize = 4;
    const N_ROUNDS: usize = 5;

    let cu_logs = &mut String::new();
    let mut f = new_warmed_fixture().await?;

    writeln!(
        cu_logs,
        "\n========== Maker spam: cancel all + place {N_ORDERS}, {N_ROUNDS} times =========="
    )?;
    writeln!(cu_logs, "(Market warmed, book has 5 asks + 5 bids)\n")?;

    let mut total_cu: u64 = 0;

    for round in 0..N_ROUNDS {
        let maker = f.maker_keypair();
        let base_tick = 2000 + (round as u64) * 100;

        // Cancel all existing orders (free funds variant).
        let cancel_ix =
            create_cancel_all_order_with_free_funds_instruction(&f.market, &maker.pubkey());
        let cancel_cu = send_tx_measure_cu(&mut f.context, &[cancel_ix], &[&maker]).await;

        // Place N_ORDERS new asks.
        let asks: Vec<CondensedOrder> = (0..N_ORDERS)
            .map(|i| CondensedOrder {
                price_in_ticks: base_tick + i as u64 * 10,
                size_in_base_lots: 10,
            })
            .collect();
        let place_ix = create_new_multiple_order_instruction(
            &f.market,
            &maker.pubkey(),
            &f.base_mint,
            &f.quote_mint,
            &MultipleOrderPacket {
                bids: vec![],
                asks,
                client_order_id: None,
                reject_post_only: true,
            },
        );
        let place_cu = send_tx_measure_cu(&mut f.context, &[place_ix], &[&maker]).await;

        let round_cu = cancel_cu + place_cu;
        total_cu += round_cu;

        writeln!(
            cu_logs,
            "  Round {} (cancel all + place {N_ORDERS})   {:>6} CU  (cancel {:>5} + place {:>5})",
            round + 1,
            round_cu,
            cancel_cu,
            place_cu,
        )?;
    }

    writeln!(
        cu_logs,
        "  TOTAL  ({N_ROUNDS} rounds)                {:>6} CU",
        total_cu
    )?;
    writeln!(
        cu_logs,
        "  Average per round                {:>6} CU",
        total_cu / N_ROUNDS as u64
    )?;

    writeln!(cu_logs, "\n{}", "=".repeat(60))?;
    print!("{cu_logs}");

    Ok(())
}
