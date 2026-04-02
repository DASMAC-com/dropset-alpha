/// Generates a market account fixture for the ts-sdk tests.
///
/// Run with: `cargo test -p client generate_ts_fixture -- --nocapture`
///
/// Outputs market account bytes as a JSON array to
/// `ts-sdk/src/tests/fixtures/market-account.json`. The market contains:
/// - 2 seats (maker A and maker B)
/// - 3 bids from maker A at prices [90M, 70M, 50M]
/// - 2 bids from maker B at prices [80M, 71M]
/// - Sorted book: [90M, 80M, 71M, 70M, 50M]
use std::io::Write;

use client::mollusk_helpers::{
    helper_trait::DropsetTestHelper,
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
use price::{
    client_helpers::sum_quote_necessary,
    OrderInfoArgs,
};
use solana_address::Address;

#[test]
#[ignore]
fn generate_ts_fixture() -> anyhow::Result<()> {
    let maker_a_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let maker_b_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let maker_a = maker_a_mock.0;
    let maker_b = maker_b_mock.0;
    let (mollusk, market_ctx) =
        new_dropset_mollusk_context_with_default_market(&[maker_a_mock, maker_b_mock]);

    let maker_a_bids = [
        OrderInfoArgs::order_at_price(90_000_000),
        OrderInfoArgs::order_at_price(70_000_000),
        OrderInfoArgs::order_at_price(50_000_000),
    ];
    let maker_b_bids = [
        OrderInfoArgs::order_at_price(80_000_000),
        OrderInfoArgs::order_at_price(71_000_000),
    ];

    let maker_a_quote = sum_quote_necessary(&maker_a_bids)?;
    let maker_b_quote = sum_quote_necessary(&maker_b_bids)?;

    // Setup maker A.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&maker_a, &maker_a),
            market_ctx.quote.create_ata_idempotent(&maker_a, &maker_a),
            market_ctx.base.mint_to_user(&maker_a, 1)?,
            market_ctx.quote.mint_to_user(&maker_a, maker_a_quote)?,
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

    // Setup maker B.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&maker_b, &maker_b),
            market_ctx.quote.create_ata_idempotent(&maker_b, &maker_b),
            market_ctx.base.mint_to_user(&maker_b, 1)?,
            market_ctx.quote.mint_to_user(&maker_b, maker_b_quote)?,
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

    let market_bytes = mollusk.view_market_data(market_ctx.market);
    let fixture_path = concat!(env!("CARGO_WORKSPACE_DIR"), "/ts-sdk/src/tests/fixtures");
    std::fs::create_dir_all(fixture_path).ok();

    // Write as JSON array of bytes — readable and diffable.
    let json = serde_json::to_string(&market_bytes)?;
    std::fs::write(format!("{fixture_path}/market-account.json"), &json)?;

    // Write a human-readable summary of the parsed market state.
    let market = mollusk.view_market(market_ctx.market);
    let mut summary = std::fs::File::create(format!("{fixture_path}/market-account.summary.txt"))?;
    writeln!(summary, "{market:#?}")?;

    println!(
        "Wrote fixture to {fixture_path}/market-account.json ({} bytes)",
        market_bytes.len()
    );

    Ok(())
}
