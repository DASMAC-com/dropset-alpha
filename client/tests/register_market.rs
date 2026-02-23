use client::{
    context::{
        market::MarketContext,
        token::TokenContext,
    },
    mollusk_helpers::{
        market_checker::MarketChecker,
        new_dropset_mollusk_context,
        utils::create_mock_user_account,
    },
    pda::find_market_address,
};
use dropset_interface::state::{
    market_header::{
        MarketHeader,
        MARKET_ACCOUNT_DISCRIMINANT,
    },
    sector::{
        Sector,
        NIL,
    },
    transmutable::Transmutable,
};
use mollusk_svm::result::Check;
use solana_address::Address;
use solana_sdk::{
    program_pack::Pack,
    rent::Rent,
};
use spl_token_interface::state::Mint;
use transaction_parser::{
    program_ids::SPL_TOKEN_ID,
    views::MarketHeaderView,
};

#[test]
fn register_market() -> anyhow::Result<()> {
    let mock_funder = create_mock_user_account(Address::new_unique(), 100_000_000_000);
    let funder = mock_funder.0;
    let mollusk = new_dropset_mollusk_context(vec![mock_funder]);
    let market_ctx = MarketContext::new(
        TokenContext::new(Some(funder), Address::new_unique(), SPL_TOKEN_ID, 8),
        TokenContext::new(Some(funder), Address::new_unique(), SPL_TOKEN_ID, 8),
    );

    let check = MarketChecker::new(&mollusk, &market_ctx);

    // Create the tokens.
    mollusk.process_instruction_chain(
        &market_ctx
            .create_tokens(funder, Rent::default().minimum_balance(Mint::LEN))
            .expect("Should create token instructions"),
    );

    // Register the market and run checks on the account post-registration.
    let num_sectors = 23;
    assert!(mollusk
        .process_and_validate_instruction(
            &market_ctx.register_market(funder, num_sectors as u16),
            &[Check::account(&market_ctx.market)
                .executable(false)
                .owner(&dropset::ID)
                .rent_exempt()
                .space(MarketHeader::LEN + Sector::LEN * num_sectors)
                .build()],
        )
        .program_result
        .is_ok());

    let (_, bump) = find_market_address(
        &market_ctx.base.mint_address,
        &market_ctx.quote.mint_address,
    );

    check.num_asks(1);
    check.num_bids(1);
    check.num_seats(1);
    check.market_header(|header| {
        assert_eq!(
            header,
            MarketHeaderView {
                discriminant: MARKET_ACCOUNT_DISCRIMINANT,
                num_seats: 0,
                num_bids: 0,
                num_asks: 0,
                num_free_sectors: num_sectors as u32,
                free_stack_top: 0,
                seats_dll_head: NIL,
                seats_dll_tail: NIL,
                bids_dll_head: NIL,
                bids_dll_tail: NIL,
                asks_dll_head: NIL,
                asks_dll_tail: NIL,
                base_mint: market_ctx.base.mint_address,
                quote_mint: market_ctx.quote.mint_address,
                market_bump: bump,
                nonce: 1, // The register market event.
                _padding: [0, 0, 0],
            }
        );
    });

    let base_mint = mollusk
        .account_store
        .borrow()
        .get(&market_ctx.base.mint_address)
        .map(|acc| Mint::unpack(&acc.data).expect("Should unpack"))
        .expect("Mint account should exist");

    assert_eq!(
        Option::from(base_mint.mint_authority),
        market_ctx.base.mint_authority
    );

    Ok(())
}
