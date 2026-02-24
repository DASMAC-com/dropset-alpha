use client::mollusk_helpers::{
    checks::IntoCheckFailure,
    helper_trait::DropsetTestHelper,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
};
use dropset_interface::{
    error::DropsetError,
    state::{
        market_header::MarketHeader,
        sector::Sector,
        transmutable::Transmutable,
    },
};
use mollusk_svm::result::Check;
use solana_account_view::MAX_PERMITTED_DATA_INCREASE;
use solana_address::Address;
use solana_program_error::ProgramError;

#[test]
fn expand_market() -> anyhow::Result<()> {
    let mock_funder = create_mock_user_account(Address::new_unique(), 100_000_000_000);
    let funder = mock_funder.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[mock_funder]);

    let initial_num_free_sectors = mollusk
        .view_market(market_ctx.market)
        .header
        .num_free_sectors as usize;

    let total_market_data_len = mollusk
        .account_store
        .borrow()
        .get(&market_ctx.market)
        .unwrap()
        .data
        .len();
    let market_sectors_data_len = total_market_data_len - MarketHeader::LEN;

    assert_eq!(market_sectors_data_len % Sector::LEN, 0);
    assert_eq!(
        market_sectors_data_len,
        initial_num_free_sectors * Sector::LEN
    );

    // Expand by 1, check that the account data increased by Sector::LEN.
    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, 1),
        &[
            Check::success(),
            Check::account(&market_ctx.market)
                .space(total_market_data_len + Sector::LEN)
                .build(),
        ],
    );

    // Expand by 17, check that the account data increased by (1 + 17) * Sector::LEN.
    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, 17),
        &[
            Check::success(),
            Check::account(&market_ctx.market)
                .space(total_market_data_len + ((1 + 17) * Sector::LEN))
                .build(),
        ],
    );

    // Ensure the instruction fails if the number of sectors to expand by is zero.
    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, 0),
        &[DropsetError::NumSectorsCannotBeZero.into_check_failure()],
    );

    // Expand by the max possible number of sectors according to the max permitted data increase
    // per account + instruction. Then check that the account data increased accordingly.
    let max_num_sectors_increase = MAX_PERMITTED_DATA_INCREASE / Sector::LEN;
    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, max_num_sectors_increase as u16),
        &[
            Check::success(),
            Check::account(&market_ctx.market)
                .space(total_market_data_len + ((1 + 17 + max_num_sectors_increase) * Sector::LEN))
                .build(),
        ],
    );

    // Expect an invalid reallocation error when expanding by the max number of sectors + 1.
    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, (max_num_sectors_increase + 1) as u16),
        &[Check::err(ProgramError::InvalidRealloc)],
    );
    Ok(())
}
