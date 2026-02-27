use client::mollusk_helpers::{
    helper_trait::DropsetTestHelper,
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use dropset_interface::{
    error::DropsetError,
    instructions::{
        BatchReplaceInstructionData,
        UnvalidatedOrders,
    },
    state::sector::{
        MAX_PERMITTED_SECTOR_INCREASE,
        NIL,
    },
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

#[test]
fn batch_replace_add_orders_happy_path() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    let bid_order_args = [OrderInfoArgs::new_unscaled(11_000_000, 1)];
    let ask_order_args = [
        OrderInfoArgs::new_unscaled(12_000_000, 1),
        OrderInfoArgs::new_unscaled(13_000_000, 1),
    ];

    let quote_necessary = sum_quote_necessary(&bid_order_args);
    let base_necessary = sum_base_necessary(&ask_order_args);

    // Set up the user with base (for asks) and quote (for bids).
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, base_necessary)?,
            market_ctx.quote.mint_to_owner(&user, quote_necessary)?,
        ])
        .program_result
        .is_ok());

    // Deposit base and quote to create the seat and fund the orders.
    let deposit_base_res =
        mollusk.process_instruction_chain(&[market_ctx.deposit_base(user, base_necessary, NIL)]);
    assert!(deposit_base_res.program_result.is_ok());

    let seat_index = mollusk.get_seat(market_ctx.market, user).index;

    let deposit_quote_res = mollusk.process_instruction_chain(&[market_ctx.deposit_quote(
        user,
        quote_necessary,
        seat_index,
    )]);
    assert!(deposit_quote_res.program_result.is_ok());

    // BatchReplace: 1 bid at 11M and 2 asks at 12M, 13M (ascending = descending ask priority).
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.batch_replace(
            user,
            BatchReplaceInstructionData::new(
                seat_index,
                UnvalidatedOrders::new(bid_order_args.clone()),
                UnvalidatedOrders::new(ask_order_args.clone()),
            ),
        )])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_bids(1);
    check.num_asks(2);

    check.bids(|bids| {
        for (bid, order_args) in bids.iter().zip(bid_order_args) {
            let order = to_order_info(order_args).unwrap();
            assert_eq!(bid.base_remaining, order.base_atoms);
            assert_eq!(bid.quote_remaining, order.quote_atoms);
            assert_eq!(bid.encoded_price, order.encoded_price);
        }
    });

    check.asks(|asks| {
        for (ask, order_args) in asks.iter().zip(ask_order_args) {
            let order = to_order_info(order_args).unwrap();
            assert_eq!(ask.base_remaining, order.base_atoms);
            assert_eq!(ask.quote_remaining, order.quote_atoms);
            assert_eq!(ask.encoded_price, order.encoded_price);
        }
    });

    Ok(())
}

/// Setup: maker A posts bids at [90M, 70M, 50M], then maker B batch-replaces with bids at
/// [80M, 71M]. These interleave with A's orders. Correct book: [90M, 80M, 71M, 70M, 50M].
/// Buggy result: [90M, 80M, 70M, 71M, 50M].
#[test]
fn batch_replace_interleaved_bids_are_sorted() -> anyhow::Result<()> {
    let maker_a_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let maker_b_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let maker_a = maker_a_mock.0;
    let maker_b = maker_b_mock.0;
    let (mollusk, market_ctx) =
        new_dropset_mollusk_context_with_default_market(&[maker_a_mock, maker_b_mock]);

    // Bids must be in descending price priority order (highest first).
    // Maker B's prices (80M, 71M) interleave with maker A's (90M, 70M, 50M).
    // Valid mantissa range: [10_000_000, 99_999_999].
    let maker_a_bids = [
        OrderInfoArgs::order_at_price(90_000_000),
        OrderInfoArgs::order_at_price(70_000_000),
        OrderInfoArgs::order_at_price(50_000_000),
    ];
    let maker_b_bids = [
        OrderInfoArgs::order_at_price(80_000_000),
        OrderInfoArgs::order_at_price(71_000_000),
    ];

    let maker_a_quote = sum_quote_necessary(&maker_a_bids);
    let maker_b_quote = sum_quote_necessary(&maker_b_bids);

    // Set up maker A: create ATAs, mint 1 base (to create seat via deposit_base), mint quote.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&maker_a, &maker_a),
            market_ctx.quote.create_ata_idempotent(&maker_a, &maker_a),
            market_ctx.base.mint_to_owner(&maker_a, 1)?,
            market_ctx.quote.mint_to_owner(&maker_a, maker_a_quote)?,
        ])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[market_ctx.deposit_base(maker_a, 1, NIL)])
        .program_result
        .is_ok());

    let maker_a_seat = mollusk.get_seat(market_ctx.market, maker_a).index;

    assert!(mollusk
        .process_instruction_chain(&[market_ctx.deposit_quote(
            maker_a,
            maker_a_quote,
            maker_a_seat,
        )])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[market_ctx.batch_replace(
            maker_a,
            BatchReplaceInstructionData::new(
                maker_a_seat,
                UnvalidatedOrders::new(maker_a_bids),
                UnvalidatedOrders::new([]),
            ),
        )])
        .program_result
        .is_ok());

    // Set up maker B identically.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&maker_b, &maker_b),
            market_ctx.quote.create_ata_idempotent(&maker_b, &maker_b),
            market_ctx.base.mint_to_owner(&maker_b, 1)?,
            market_ctx.quote.mint_to_owner(&maker_b, maker_b_quote)?,
        ])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[market_ctx.deposit_base(maker_b, 1, NIL)])
        .program_result
        .is_ok());

    let maker_b_seat = mollusk.get_seat(market_ctx.market, maker_b).index;

    assert!(mollusk
        .process_instruction_chain(&[market_ctx.deposit_quote(
            maker_b,
            maker_b_quote,
            maker_b_seat,
        )])
        .program_result
        .is_ok());

    assert!(mollusk
        .process_instruction_chain(&[market_ctx.batch_replace(
            maker_b,
            BatchReplaceInstructionData::new(
                maker_b_seat,
                UnvalidatedOrders::new(maker_b_bids),
                UnvalidatedOrders::new([]),
            ),
        )])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_bids(5);
    check.bids(|bids| {
        let prices = bids.iter().map(|b| b.encoded_price.as_u32()).collect_vec();
        assert_eq!(
            prices,
            vec![90_000_000, 80_000_000, 71_000_000, 70_000_000, 50_000_000],
            "bid book is not sorted correctly"
        );
    });

    Ok(())
}

#[test]
fn batch_replace_unsorted_orders_failure() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);
    let check = MarketChecker::new(&mollusk, &market_ctx);

    let ascending_orders = [
        OrderInfoArgs::new_unscaled(11_000_000, 1),
        OrderInfoArgs::new_unscaled(12_000_000, 1),
        OrderInfoArgs::new_unscaled(13_000_000, 1),
    ];

    let descending_orders = [
        OrderInfoArgs::new_unscaled(13_000_000, 1),
        OrderInfoArgs::new_unscaled(12_000_000, 1),
        OrderInfoArgs::new_unscaled(11_000_000, 1),
    ];

    let equal_orders = [
        OrderInfoArgs::new_unscaled(12_000_000, 1),
        OrderInfoArgs::new_unscaled(12_000_000, 1),
        OrderInfoArgs::new_unscaled(12_000_000, 1),
    ];

    let quote_necessary = 3 * sum_quote_necessary(&ascending_orders);
    let base_necessary = 3 * sum_base_necessary(&ascending_orders);
    assert_eq!(quote_necessary, 3 * sum_quote_necessary(&descending_orders));
    assert_eq!(base_necessary, 3 * sum_base_necessary(&descending_orders));
    assert_eq!(quote_necessary, 3 * sum_quote_necessary(&equal_orders));
    assert_eq!(base_necessary, 3 * sum_base_necessary(&equal_orders));

    // Set up the user with base (for asks) and quote (for bids).
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, base_necessary)?,
            market_ctx.quote.mint_to_owner(&user, quote_necessary)?,
        ])
        .program_result
        .is_ok());

    let seat_index = 0;
    // Deposit base and quote to create the seat and fund the orders.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, base_necessary, NIL),
            market_ctx.deposit_quote(user, quote_necessary, seat_index),
            market_ctx.expand(user, MAX_PERMITTED_SECTOR_INCREASE as u16),
        ])
        .program_result
        .is_ok());

    check.seat_index(user, seat_index);

    let ascending_bids = BatchReplaceInstructionData::new(
        seat_index,
        UnvalidatedOrders::new(ascending_orders.clone()),
        UnvalidatedOrders::new([]),
    );
    let descending_bids = BatchReplaceInstructionData::new(
        seat_index,
        UnvalidatedOrders::new(descending_orders.clone()),
        UnvalidatedOrders::new([]),
    );
    let equal_bids = BatchReplaceInstructionData::new(
        seat_index,
        UnvalidatedOrders::new(equal_orders.clone()),
        UnvalidatedOrders::new([]),
    );
    let ascending_asks = BatchReplaceInstructionData::new(
        seat_index,
        UnvalidatedOrders::new([]),
        UnvalidatedOrders::new(ascending_orders.clone()),
    );
    let descending_asks = BatchReplaceInstructionData::new(
        seat_index,
        UnvalidatedOrders::new([]),
        UnvalidatedOrders::new(descending_orders.clone()),
    );
    let equal_asks = BatchReplaceInstructionData::new(
        seat_index,
        UnvalidatedOrders::new([]),
        UnvalidatedOrders::new(equal_orders.clone()),
    );

    mollusk.process_and_validate_instruction_chain(&[
        (
            &market_ctx.batch_replace(user, ascending_bids),
            &[Check::err(DropsetError::OrdersNotSorted.into())],
        ),
        (
            &market_ctx.batch_replace(user, descending_bids),
            &[Check::success()],
        ),
        (
            &market_ctx.batch_replace(user, ascending_asks),
            &[Check::success()],
        ),
        (
            &market_ctx.batch_replace(user, descending_asks),
            &[Check::err(DropsetError::OrdersNotSorted.into())],
        ),
        (
            &market_ctx.batch_replace(user, equal_bids),
            &[Check::err(DropsetError::OrdersNotSorted.into())],
        ),
        (
            &market_ctx.batch_replace(user, equal_asks),
            &[Check::err(DropsetError::OrdersNotSorted.into())],
        ),
    ]);

    Ok(())
}
