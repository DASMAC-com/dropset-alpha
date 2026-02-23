use client::mollusk_helpers::{
    helper_trait::DropsetTestHelper,
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use dropset_interface::{
    instructions::{
        BatchReplaceInstructionData,
        UnvalidatedOrders,
    },
    state::sector::NIL,
};
use price::OrderInfoArgs;
use solana_address::Address;

#[test]
fn batch_replace() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    // Set up the user with base (for asks) and quote (for bids).
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, 100_000)?,
            market_ctx.quote.mint_to_owner(&user, 100_000)?,
        ])
        .program_result
        .is_ok());

    // Deposit base and quote to create the seat and fund the orders.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, 10_000, NIL),
            market_ctx.deposit_quote(user, 10_000, 0),
        ])
        .program_result
        .is_ok());

    let market = mollusk.view_market(market_ctx.market);
    let seat = market_ctx
        .find_seat(&market.seats, &user)
        .expect("User should have a seat after deposit");

    // BatchReplace: 1 bid at 11M and 2 asks at 12M, 13M (ascending = descending ask priority).
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.batch_replace(
            user,
            BatchReplaceInstructionData::new(
                seat.index,
                UnvalidatedOrders::new([OrderInfoArgs::new_unscaled(11_000_000, 1)]),
                UnvalidatedOrders::new([
                    OrderInfoArgs::new_unscaled(12_000_000, 1),
                    OrderInfoArgs::new_unscaled(13_000_000, 1),
                ]),
            ),
        )])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_bids(1);
    check.num_asks(2);

    Ok(())
}
