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
fn batch_replace() -> anyhow::Result<()> {
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
