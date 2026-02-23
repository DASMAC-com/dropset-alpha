use client::mollusk_helpers::{
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use dropset_interface::{
    instructions::{
        MarketOrderInstructionData,
        PostOrderInstructionData,
    },
    state::sector::NIL,
};
use price::{
    to_order_info,
    OrderInfoArgs,
};
use solana_address::Address;

#[test]
fn market_order() -> anyhow::Result<()> {
    let maker_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let taker_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let maker = maker_mock.0;
    let taker = taker_mock.0;
    let (mollusk, market_ctx) =
        new_dropset_mollusk_context_with_default_market(&[maker_mock, taker_mock]);

    let order_info_args = OrderInfoArgs::new_unscaled(10_000_000, 500);
    let market_order = to_order_info(order_info_args.clone()).expect("Should be a valid order");

    // Mint base for maker, deposit it, then post the maker ask with seat idx 0 (first market seat).
    let create_maker_base_ata = market_ctx.base.create_ata_idempotent(&maker, &maker);
    let mint_base_to_maker = market_ctx
        .base
        .mint_to_owner(&maker, market_order.base_atoms)?;
    let maker_deposit_base = market_ctx.deposit_base(maker, market_order.base_atoms, NIL);
    let maker_post_ask = market_ctx.post_order(
        maker,
        PostOrderInstructionData::new(order_info_args, false, 0),
    );
    // Set up taker: mint quote for the fill, create both ATAs (base to receive, quote to spend).
    let create_taker_base_ata = market_ctx.base.create_ata_idempotent(&taker, &taker);
    let create_taker_quote_ata = market_ctx.quote.create_ata_idempotent(&taker, &taker);
    let mint_quote_to_taker = market_ctx
        .quote
        .mint_to_owner(&taker, market_order.quote_atoms)?;
    assert!(mollusk
        .process_instruction_chain(&[
            create_maker_base_ata,
            mint_base_to_maker,
            maker_deposit_base,
            maker_post_ask,
            create_taker_base_ata,
            create_taker_quote_ata,
            mint_quote_to_taker,
        ])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_asks(1);
    check.num_bids(0);

    // Market buy: taker buys base_atoms worth of base, spending quote.
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.market_order(
            taker,
            MarketOrderInstructionData::new(market_order.base_atoms, true, true),
        )])
        .program_result
        .is_ok());

    // Taker should have sent quote and received base.

    // Taker should have received all the base.
    check.base_token_balance(taker, market_order.base_atoms);

    // Maker's seat should have received the quote.
    check.seat_quote_available(maker, market_order.quote_atoms);

    // The ask should be fully filled and removed.
    check.num_asks(0);

    Ok(())
}
