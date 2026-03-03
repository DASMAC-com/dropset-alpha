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
    client_helpers::{
        sum_base_necessary,
        sum_quote_necessary,
    },
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

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, u64::MAX)?,
            market_ctx.quote.mint_to_owner(&user, u64::MAX)?,
        ])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, u64::MAX, NIL),
            market_ctx.deposit_quote(user, u64::MAX, 0),
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

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, u64::MAX)?,
            market_ctx.quote.mint_to_owner(&user, u64::MAX)?,
        ])
        .program_result
        .is_ok());

    let user_a_seat_index = 0; // First seat.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, u64::MAX, NIL),
            market_ctx.deposit_quote(user, u64::MAX, user_a_seat_index),
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

    let user_a_base = to_order_info(OrderInfoArgs::order_at_price(50_000_000))
        .unwrap()
        .base_atoms;

    let user_b_base = sum_base_necessary(&[
        OrderInfoArgs::order_at_price(50_000_000),
        OrderInfoArgs::order_at_price(50_000_001),
    ]);
    let user_b_quote = sum_quote_necessary(&[OrderInfoArgs::order_at_price(49_999_999)]);

    // Create the base ATA for `user_a`. Mint the intended order size to them and then have them
    // deposit it to their seat.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user_a, &user_a),
            market_ctx.base.mint_to_owner(&user_a, user_a_base)?,
            market_ctx.deposit_base(user_a, user_a_base, NIL)
        ])
        .program_result
        .is_ok());

    // Then create `user_b`'s seat by depositing. Thei amounts in `user_b`'s seat are irrelevant,
    // since ultimately they  doesn't
    // need more than a single atom per order because size is irrelevant triggering the post only
    // crossing check failure.
    let user_b_seat_index = 1; // Second seat => seat index 1.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user_b, &user_b),
            market_ctx.quote.create_ata_idempotent(&user_b, &user_b),
            market_ctx.base.mint_to_owner(&user_b, user_b_base)?,
            market_ctx.quote.mint_to_owner(&user_b, user_b_quote)?,
            market_ctx.deposit_base(user_b, user_b_base, NIL),
            market_ctx.deposit_quote(user_b, user_b_quote, user_b_seat_index),
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

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_bids(1);
    check.num_asks(3);

    let user_a_seat_index = 0;
    check.seat_index(user_a, user_a_seat_index);
    check.seat_index(user_b, user_b_seat_index);
    check.asks(|asks| {
        let user_seat_and_price_pairs = asks
            .iter()
            .map(|ask| (ask.user_seat, ask.encoded_price.as_u32()))
            .collect_vec();
        assert_eq!(
            user_seat_and_price_pairs,
            vec![
                (user_a_seat_index, 50_000_000),
                (user_b_seat_index, 50_000_000),
                (user_b_seat_index, 50_000_001),
            ]
        );
    });

    Ok(())
}

/// Verifies that canceling and re-posting an order moves it to the back of the time-priority
/// queue at its price level, even when other users' orders at the same price exist in between.
#[test]
fn price_time_priority() -> anyhow::Result<()> {
    let user_a_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user_b_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user_c_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user_a = user_a_mock.0;
    let user_b = user_b_mock.0;
    let user_c = user_c_mock.0;
    let (mollusk, market_ctx) =
        new_dropset_mollusk_context_with_default_market(&[user_a_mock, user_b_mock, user_c_mock]);

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user_a, &user_a),
            market_ctx.base.create_ata_idempotent(&user_b, &user_b),
            market_ctx.base.create_ata_idempotent(&user_c, &user_c),
            market_ctx.base.mint_to_owner(&user_a, u64::MAX / 3)?,
            market_ctx.base.mint_to_owner(&user_b, u64::MAX / 3)?,
            market_ctx.base.mint_to_owner(&user_c, u64::MAX / 3)?,
            market_ctx.deposit_base(user_a, u64::MAX / 3, NIL),
            market_ctx.deposit_base(user_b, u64::MAX / 3, NIL),
            market_ctx.deposit_base(user_c, u64::MAX / 3, NIL),
        ])
        .program_result
        .is_ok());

    let seat_a = mollusk.get_seat(market_ctx.market, user_a);
    let seat_b = mollusk.get_seat(market_ctx.market, user_b);
    let seat_c = mollusk.get_seat(market_ctx.market, user_c);

    // ---------------------------------------------------------------------------------------------
    // Create helper closures to make the test more readable.
    let post_ask = |price: u32, user: Address, seat_index: u32| {
        market_ctx.post_order(
            user,
            PostOrderInstructionData::new(OrderInfoArgs::order_at_price(price), false, seat_index),
        )
    };
    let post_ask_a = |price: u32| post_ask(price, user_a, seat_a.index);
    let post_ask_b = |price: u32| post_ask(price, user_b, seat_b.index);
    let post_ask_c = |price: u32| post_ask(price, user_c, seat_c.index);
    let cancel_ask_b = |price: u32| {
        market_ctx.cancel_order(
            user_b,
            CancelOrderInstructionData::new(price, false, seat_b.index),
        )
    };
    // ---------------------------------------------------------------------------------------------

    // Post initial asks. At 50M, time priority is: user_b, user_a, user_c.
    mollusk.process_and_validate_instruction_chain(&[
        (&post_ask_a(40_000_000), &[Check::success()]), // A, 40_000_000
        (&post_ask_b(50_000_000), &[Check::success()]), // B, 50_000_000
        (&post_ask_a(50_000_000), &[Check::success()]), // A, ..
        (&post_ask_c(50_000_000), &[Check::success()]), // C, ..
        (&post_ask_a(60_000_000), &[Check::success()]), // A, 60_000_000
    ]);

    // user_b cancels and re-posts at 50M. They move to the back of the 50M queue.
    mollusk.process_and_validate_instruction_chain(&[
        (&cancel_ask_b(50_000_000), &[Check::success()]),
        (&post_ask_b(50_000_000), &[Check::success()]),
    ]);

    // Verify that user b's order has been inserted at the end of the 50_000_000 orders.
    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_asks(5);
    check.asks(|asks| {
        let seat_and_price = asks
            .iter()
            .map(|ask| (ask.user_seat, ask.encoded_price.as_u32()))
            .collect_vec();
        assert_eq!(
            seat_and_price,
            vec![
                (seat_a.index, 40_000_000), // A, 40_000_000
                (seat_a.index, 50_000_000), // A, 50_000_000
                (seat_c.index, 50_000_000), // C, ..
                (seat_b.index, 50_000_000), // B, ..
                (seat_a.index, 60_000_000), // A, 60_000_000
            ],
        );
    });

    Ok(())
}
