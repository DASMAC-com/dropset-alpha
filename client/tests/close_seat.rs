use client::mollusk_helpers::{
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use solana_address::Address;

#[test]
fn close_seat() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
        ])
        .program_result
        .is_ok());

    // Mint 1 base and create the seat via create_seat (which deposits 1 base).
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.mint_to_owner(&user, 1)?,
            market_ctx.create_seat(user),
        ])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    let seat_index = 0; // User is the first registered seat.
    check.num_seats(1);
    check.has_seat(&user);
    check.seat_index(&user, seat_index);
    check.seat_base_available(&user, 1);
    check.seat_quote_available(&user, 0);

    // Close the seat. This returns the 1 base of collateral back to the user's ATA.
    assert!(mollusk
        .process_instruction_chain(&[market_ctx.close_seat(user, seat_index)])
        .program_result
        .is_ok());

    check.num_seats(0);
    check.base_token_balance(&user, 1);
    check.quote_token_balance(&user, 0);

    Ok(())
}
