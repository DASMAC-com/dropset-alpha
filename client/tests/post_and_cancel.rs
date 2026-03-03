use client::mollusk_helpers::{
    helper_trait::DropsetTestHelper,
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use dropset_interface::{
    instructions::{
        CancelOrderInstructionData,
        PostOrderInstructionData,
    },
    state::sector::{
        MAX_PERMITTED_SECTOR_INCREASE,
        NIL,
    },
};
use itertools::Itertools;
use price::{
    to_order_info,
    OrderInfoArgs,
};
use solana_address::Address;
use transaction_parser::views::OrderView;

#[test]
fn post_and_cancel() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    // Expand the market to accommodate more orders.
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.expand(user, MAX_PERMITTED_SECTOR_INCREASE as u16)])
        .program_result
        .is_ok());

    // Mint base tokens and create the user's ATA, then deposit base (and create the user's seat).
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, 10_000)?,
            market_ctx.deposit_base(user, 1_000, NIL),
        ])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.has_seat(user);
    let seat_index = mollusk.get_seat(market_ctx.market, user).index;

    let order_info_args = OrderInfoArgs::new_unscaled(10_000_000, 500);
    let order_info = to_order_info(order_info_args.clone()).expect("Should be a valid order");
    let is_bid = false;

    // Post an ask.
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.post_order(
            user,
            PostOrderInstructionData::new(order_info_args, is_bid, seat_index),
        )])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_asks(1);
    check.num_bids(0);
    check.asks(|asks| assert_eq!(asks[0].encoded_price, order_info.encoded_price));

    // Cancel the ask.
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.cancel_order(
            user,
            CancelOrderInstructionData::new(order_info.encoded_price.as_u32(), is_bid, seat_index),
        )])
        .program_result
        .is_ok());

    check.num_asks(0);
    check.num_bids(0);

    Ok(())
}

// Using order_at_price: base_atoms = 10^15 per ask, quote_atoms = price_mantissa / 10 per bid.
// Asks use high prices (60M–99M), bids use low prices (10M–50M) to avoid crossing the book.
#[test]
fn post_and_cancel_maintains_sort_order() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    // Expand the market to accommodate more orders.
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.expand(user, MAX_PERMITTED_SECTOR_INCREASE as u16)])
        .program_result
        .is_ok());

    // Create ATAs, mint to owner, then deposit it to the market.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, u64::MAX)?,
            market_ctx.quote.mint_to_owner(&user, u64::MAX)?,
            market_ctx.deposit_base(user, u64::MAX, NIL)
        ])
        .program_result
        .is_ok());

    // Get the registered seat index and deposit the remaining quote collateral.
    let seat_index = mollusk.get_seat(market_ctx.market, user).index;
    assert!(mollusk
        .process_instruction(&market_ctx.deposit_quote(user, u64::MAX, seat_index))
        .program_result
        .is_ok());

    // ---------------------------------------------------------------------------------------------
    // Create helper closures to make the test more readable.
    let post_order = |price: u32, is_bid: bool| {
        market_ctx.post_order(
            user,
            PostOrderInstructionData::new(OrderInfoArgs::order_at_price(price), is_bid, seat_index),
        )
    };
    let post_bid = |price: u32| post_order(price, true);
    let post_ask = |price: u32| post_order(price, false);
    let cancel_order = |price: u32, is_bid: bool| {
        market_ctx.cancel_order(
            user,
            CancelOrderInstructionData::new(price, is_bid, seat_index),
        )
    };
    let cancel_bid = |price: u32| cancel_order(price, true);
    let cancel_ask = |price: u32| cancel_order(price, false);

    let as_encoded_prices = |orders: Vec<OrderView>| -> Vec<u32> {
        orders
            .into_iter()
            .map(|o| o.encoded_price.as_u32())
            .collect()
    };
    // ---------------------------------------------------------------------------------------------

    // Post 5 asks and 5 bids at known prices.
    let ask_prices: [u32; 5] = [60_000_000, 70_000_000, 80_000_000, 90_000_000, 99_000_000];
    let bid_prices: [u32; 5] = [10_000_001, 20_000_000, 30_000_000, 40_000_000, 50_000_000];
    let post_asks = ask_prices.iter().map(|&p| post_ask(p));
    let post_bids = bid_prices.iter().map(|&p| post_bid(p));
    assert!(mollusk
        .process_instruction_chain(&post_asks.chain(post_bids).collect_vec())
        .program_result
        .is_ok());

    // Cancel the 2nd and 3rd asks/bids by price, leaving gaps.
    assert!(mollusk
        .process_instruction_chain(&[
            // Asks.
            cancel_ask(70_000_000),
            cancel_ask(90_000_000),
            // Bids.
            cancel_bid(20_000_000),
            cancel_bid(40_000_000),
        ])
        .program_result
        .is_ok());

    // Verify exact price sequence using the order_at_price invariant (encoded_price == mantissa).
    assert_eq!(
        as_encoded_prices(mollusk.view_market(market_ctx.market).asks),
        [60_000_000, 80_000_000, 99_000_000],
    );
    assert_eq!(
        as_encoded_prices(mollusk.view_market(market_ctx.market).bids),
        [50_000_000, 30_000_000, 10_000_001]
    );

    // Fill one gaps and add one beyond the end of each book side.
    assert!(mollusk
        .process_instruction_chain(&[
            // Asks.
            post_ask(65_000_000),
            post_ask(99_999_999),
            // Bids.
            post_bid(45_000_000),
            post_bid(10_000_000),
        ])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_asks(5);
    check.num_bids(5);

    // Verify sort order is maintained after all the insertions and removals.
    check.asks(|asks| {
        assert!(asks
            .iter()
            .tuple_windows()
            .all(|(a, b)| a.encoded_price.has_higher_ask_priority(&b.encoded_price)))
    });
    check.bids(|bids| {
        assert!(bids
            .iter()
            .tuple_windows()
            .all(|(a, b)| a.encoded_price.has_higher_bid_priority(&b.encoded_price)))
    });

    // Verify exact price sequence using the order_at_price invariant (encoded_price == mantissa).
    let asks = mollusk.view_market(market_ctx.market).asks;
    let bids = mollusk.view_market(market_ctx.market).bids;
    assert_eq!(
        as_encoded_prices(asks),
        [60_000_000, 65_000_000, 80_000_000, 99_000_000, 99_999_999]
    );
    assert_eq!(
        as_encoded_prices(bids),
        [50_000_000, 45_000_000, 30_000_000, 10_000_001, 10_000_000]
    );

    Ok(())
}
