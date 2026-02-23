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
use solana_address::Address;

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

    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, 1),
        &[
            Check::success(),
            Check::account(&market_ctx.market)
                .space(total_market_data_len + Sector::LEN)
                .build(),
        ],
    );

    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, 17),
        &[
            Check::success(),
            Check::account(&market_ctx.market)
                .space(total_market_data_len + ((1 + 17) * Sector::LEN))
                .build(),
        ],
    );

    mollusk.process_and_validate_instruction(
        &market_ctx.expand(funder, 0),
        &[DropsetError::NumSectorsCannotBeZero.into_check_failure()],
    );

    Ok(())
}
