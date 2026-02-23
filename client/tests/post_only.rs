use client::mollusk_helpers::{
    checks::IntoCheckFailure,
    helper_trait::DropsetTestHelper,
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use dropset_interface::{
    error::DropsetError,
    instructions::{
        CancelOrderInstructionData,
        PostOrderInstructionData,
    },
    state::sector::NIL,
};
use itertools::Itertools;
use mollusk_svm::result::Check;
use price::{
    to_order_info,
    OrderInfoArgs,
};
use solana_address::Address;

/// Verifies that post-only crossing checks fire in both directions, including at equal price:
/// - A bid whose price is at or above the best ask is rejected.
/// - An ask whose price is at or below the best bid is rejected.
#[test]
fn post_only_crossing_check() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    let ask = to_order_info(OrderInfoArgs::order_at_price(50_000_000)).unwrap();
    let bid = to_order_info(OrderInfoArgs::order_at_price(40_000_000)).unwrap();

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, ask.base_atoms)?,
            market_ctx.quote.mint_to_owner(&user, bid.quote_atoms)?,
        ])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, ask.base_atoms, NIL),
            market_ctx.deposit_quote(user, bid.quote_atoms, 0),
        ])
        .program_result
        .is_ok());

    let seat = mollusk.get_seat(market_ctx.market, user);

    let post = |price, is_bid| {
        market_ctx.post_order(
            user,
            PostOrderInstructionData::new(OrderInfoArgs::order_at_price(price), is_bid, seat.index),
        )
    };
    let fail = || [DropsetError::PostOnlyWouldImmediatelyFill.into_check_failure()];

    let chain = [
        (post(50_000_000, false), [Check::success()]), // posting ask succeeds
        (post(50_000_001, true), fail()),              // bid above ask crosses
        (post(50_000_000, true), fail()),              // bid equal to ask crosses
        (post(40_000_000, true), [Check::success()]),  // posting bid succeeds
        (post(39_999_999, false), fail()),             // ask below bid crosses
        (post(40_000_000, false), fail()),             // ask equal to bid crosses
    ];
    let chain_refs: Vec<_> = chain.iter().map(|(i, c)| (i, c.as_slice())).collect();
    mollusk.process_and_validate_instruction_chain(&chain_refs);

    Ok(())
}

/// Verifies that a crossing failure clears after canceling the blocking order, but that the next
/// level of the book still blocks a more aggressive order.
#[test]
fn crossing_check_clears_with_cancel() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    let ask_50 = to_order_info(OrderInfoArgs::order_at_price(50_000_000)).unwrap();
    let ask_60 = to_order_info(OrderInfoArgs::order_at_price(60_000_000)).unwrap();
    let bid_55 = to_order_info(OrderInfoArgs::order_at_price(55_000_000)).unwrap();

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx
                .base
                .mint_to_owner(&user, ask_50.base_atoms + ask_60.base_atoms)?,
            market_ctx.quote.mint_to_owner(&user, bid_55.quote_atoms)?,
        ])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, ask_50.base_atoms + ask_60.base_atoms, NIL),
            market_ctx.deposit_quote(user, bid_55.quote_atoms, 0),
        ])
        .program_result
        .is_ok());

    let seat = mollusk.get_seat(market_ctx.market, user);

    let post = |price, is_bid| {
        let data =
            PostOrderInstructionData::new(OrderInfoArgs::order_at_price(price), is_bid, seat.index);
        market_ctx.post_order(user, data)
    };
    let post_bid = |price| post(price, true);
    let post_ask = |price| post(price, false);
    let cancel = |price| {
        let data = CancelOrderInstructionData::new(price, false, seat.index);
        market_ctx.cancel_order(user, data)
    };
    let cross_failure = || DropsetError::PostOnlyWouldImmediatelyFill.into_check_failure();

    let chain = [
        (post_ask(50_000_000), [Check::success()]), // Ask at 50M
        (post_ask(60_000_000), [Check::success()]), // Ask at 60M
        (post_bid(55_000_000), [cross_failure()]),  // Bid at 55M fails because 50M ask exists
        (cancel(50_000_000), [Check::success()]),   // Cancel the 50M ask
        (post_bid(55_000_000), [Check::success()]), // Bid at 55M now clears
        (post_bid(65_000_000), [cross_failure()]),  // Bid at 65M still crosses 60M ask
    ];
    let chain_refs: Vec<_> = chain.iter().map(|(i, c)| (i, c.as_slice())).collect();
    mollusk.process_and_validate_instruction_chain(&chain_refs);

    Ok(())
}

/// Verifies that the crossing check is market-wide and not scoped to a single user's orders.
#[test]
fn crossing_check_across_users() -> anyhow::Result<()> {
    let user_a_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user_b_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user_a = user_a_mock.0;
    let user_b = user_b_mock.0;
    let (mollusk, market_ctx) =
        new_dropset_mollusk_context_with_default_market(&[user_a_mock, user_b_mock]);

    let ask = to_order_info(OrderInfoArgs::order_at_price(50_000_000)).unwrap();

    // Create the base ATA for `user_a`. Mint the intended order size to them and then have them
    // deposit it to their seat.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user_a, &user_a),
            market_ctx.base.mint_to_owner(&user_a, ask.base_atoms)?,
            market_ctx.deposit_base(user_a, ask.base_atoms, NIL)
        ])
        .program_result
        .is_ok());

    // Then create `user_b`'s seat by depositing. Thei amounts in `user_b`'s seat are irrelevant,
    // since ultimately they  doesn't
    // need more than a single atom per order because size is irrelevant triggering the post only
    // crossing check failure.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user_b, &user_b),
            market_ctx.quote.create_ata_idempotent(&user_b, &user_b),
            market_ctx.base.mint_to_owner(&user_b, 2)?,
            market_ctx.quote.mint_to_owner(&user_b, 1)?,
            market_ctx.deposit_base(user_b, 2, NIL),
            market_ctx.deposit_quote(user_b, 1, 1), // Seat index 1 since it's the second seat.
        ])
        .program_result
        .is_ok());

    let seat_a = mollusk.get_seat(market_ctx.market, user_a);
    let seat_b = mollusk.get_seat(market_ctx.market, user_b);

    // Have `user_a` post the ask.
    mollusk.process_and_validate_instruction(
        &market_ctx.post_order(
            user_a,
            PostOrderInstructionData::new(
                OrderInfoArgs::order_at_price(50_000_000),
                false,
                seat_a.index,
            ),
        ),
        &[Check::success()],
    );

    let user_b_post = |is_bid: bool, price: u32| {
        market_ctx.post_order(
            user_b,
            PostOrderInstructionData::new(
                OrderInfoArgs::order_at_price(price),
                is_bid,
                seat_b.index,
            ),
        )
    };

    let user_b_post_ask = |price: u32| user_b_post(false, price);
    let user_b_post_bid = |price: u32| user_b_post(true, price);

    let cross_check_fail = || [DropsetError::PostOnlyWouldImmediatelyFill.into_check_failure()];

    mollusk.process_and_validate_instruction_chain(&[
        (&user_b_post_bid(50_000_000), &cross_check_fail()),
        (&user_b_post_bid(50_000_001), &cross_check_fail()),
    ]);
    mollusk.process_and_validate_instruction_chain(&[
        (&user_b_post_bid(49_999_999), &[Check::success()]),
        (&user_b_post_ask(50_000_000), &[Check::success()]),
        (&user_b_post_ask(50_000_001), &[Check::success()]),
    ]);

    let market = mollusk.view_market(market_ctx.market);
    println!("{market:?}");

    let check = MarketChecker::new(&mollusk, &market_ctx);

    // check.num_bids(1);
    check.num_asks(3);
    check.asks(|asks| {
        println!("{asks:?}");
        assert_eq!(
            asks.iter()
                .map(|ask| ask.encoded_price.as_u32())
                .collect_vec(),
            vec![50_000_000, 50_000_000, 50_000_001]
        );
    });

    Ok(())
}
