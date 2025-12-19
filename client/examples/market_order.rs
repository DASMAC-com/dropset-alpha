use std::collections::HashSet;

use client::{
    context::market::MarketContext,
    transactions::{
        CustomRpcClient,
        SendTransactionConfig,
    },
};
use dropset_interface::{
    instructions::{
        MarketOrderInstructionData,
        PostOrderInstructionData,
    },
    state::sector::NIL,
};
use price::to_biased_exponent;
use solana_sdk::signer::Signer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc = &CustomRpcClient::new(
        None,
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(true),
            program_id_filter: HashSet::from([dropset_interface::program::ID.into()]),
        }),
    );
    let payer = rpc.fund_new_account().await?;

    let market_ctx = MarketContext::new_market(rpc).await?;
    let register = market_ctx.register_market(payer.pubkey(), 10);

    market_ctx.base.create_ata_for(rpc, &payer).await?;
    market_ctx.quote.create_ata_for(rpc, &payer).await?;

    market_ctx.base.mint_to(rpc, &payer, 1_000_000_000).await?;
    market_ctx.quote.mint_to(rpc, &payer, 1_000_000_000).await?;

    let deposit = market_ctx.deposit_base(payer.pubkey(), 1_000_000_000, NIL);

    rpc.send_and_confirm_txn(&payer, &[&payer], &[register.into(), deposit.into()])
        .await?;

    let market = market_ctx.view_market(rpc)?;
    println!("Market after user deposit\n{:#?}", market);

    let user_seat = market_ctx
        .find_seat(rpc, &payer.pubkey())?
        .expect("User should have been registered on deposit");

    let (price_mantissa, base_scalar, base_exponent, quote_exponent) = (
        11_000_000,
        5,
        to_biased_exponent!(8),
        to_biased_exponent!(0),
    );

    // Post an ask so the user puts up quote as collateral with base to get filled.
    let is_bid = false;
    let post_ask = market_ctx.post_order(
        payer.pubkey(),
        PostOrderInstructionData::new(
            price_mantissa,
            base_scalar,
            base_exponent,
            quote_exponent,
            is_bid,
            user_seat.index,
        ),
    );

    let res = rpc
        .send_and_confirm_txn(&payer, &[&payer], &[post_ask.into()])
        .await?;

    println!(
        "Post ask transaction signature: {}",
        res.parsed_transaction.signature
    );

    let market = market_ctx.view_market(rpc)?;
    println!("Market after posting user ask:\n{:#?}", market);

    let user_seat = market_ctx.find_seat(rpc, &payer.pubkey())?.unwrap();
    println!("User seat after posting ask: {user_seat:#?}");

    let market_buy = market_ctx.market_order(
        payer.pubkey(),
        MarketOrderInstructionData::new(500000000 / 10, true),
    );

    let market_buy_res = rpc
        .send_and_confirm_txn(&payer, &[&payer], &[market_buy.into()])
        .await?;

    println!(
        "Market buy transaction signature: {}",
        market_buy_res.parsed_transaction.signature
    );

    let market = market_ctx.view_market(rpc)?;
    println!("Market after market buy:\n{:#?}", market);

    let user_seat = market_ctx.find_seat(rpc, &payer.pubkey())?.unwrap();
    println!("User seat after market buy: {user_seat:#?}");

    Ok(())
}
