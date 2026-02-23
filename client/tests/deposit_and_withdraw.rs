use client::mollusk_helpers::{
    helper_trait::DropsetTestHelper,
    market_checker::MarketChecker,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
    MOLLUSK_DEFAULT_NUM_SECTORS,
};
use dropset_interface::state::{
    sector::{
        Sector,
        NIL,
    },
    transmutable::Transmutable,
};
use itertools::Itertools;
use solana_address::Address;
use solana_instruction::Instruction;
use transaction_parser::views::MarketSeatView;

#[test]
fn deposit_and_withdraw() -> anyhow::Result<()> {
    let user_mock = create_mock_user_account(Address::new_unique(), 100_000_000);
    let user = user_mock.0;
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(&[user_mock]);

    let initial_base: u64 = 10_000;
    let initial_quote: u64 = 20_000;

    // Create the user ATAs for base and quote and mint to them the initial amounts.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx.quote.create_ata_idempotent(&user, &user),
            market_ctx.base.mint_to_owner(&user, initial_base)?,
            market_ctx.quote.mint_to_owner(&user, initial_quote)?,
        ])
        .program_result
        .is_ok());

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.base_token_balance(user, 10_000);
    check.quote_token_balance(user, 20_000);
    check.base_token_balance(market_ctx.market, 0);
    check.quote_token_balance(market_ctx.market, 0);

    // Deposit base and quote.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.deposit_base(user, 1_000, NIL),
            market_ctx.deposit_quote(user, 1_000, 0), // The seat is the first seat on the market.
        ])
        .program_result
        .is_ok());

    check.base_token_balance(user, 9_000);
    check.quote_token_balance(user, 19_000);
    check.base_token_balance(market_ctx.market, 1_000);
    check.quote_token_balance(market_ctx.market, 1_000);

    check.has_seat(user);
    check.seat(user, |seat| {
        let expected_seat = MarketSeatView {
            base_available: 1_000,
            quote_available: 1_000,
            prev_index: NIL,
            index: 0, // The seat is the first seat on the market.
            next_index: NIL,
            user,
            user_order_sectors: Default::default(),
        };
        assert_eq!(seat, expected_seat);
    });

    // Withdraw base and quote.
    assert!(mollusk
        .process_instruction_chain(&[
            market_ctx.withdraw_base(user, 1_000, 0),
            market_ctx.withdraw_quote(user, 1_000, 0),
        ])
        .program_result
        .is_ok());

    check.base_token_balance(user, 10_000);
    check.quote_token_balance(user, 20_000);
    check.base_token_balance(market_ctx.market, 0);
    check.quote_token_balance(market_ctx.market, 0);

    Ok(())
}

#[test]
fn deposit_auto_expand() -> anyhow::Result<()> {
    let users_that_do_not_expand_market = (0..MOLLUSK_DEFAULT_NUM_SECTORS)
        .map(|_| create_mock_user_account(Address::new_unique(), 100_000_000))
        .collect_vec();
    let user_that_triggers_expansion = create_mock_user_account(Address::new_unique(), 100_000_000);
    let (mollusk, market_ctx) = new_dropset_mollusk_context_with_default_market(
        &[
            users_that_do_not_expand_market.clone(),
            vec![user_that_triggers_expansion.clone()],
        ]
        .concat(),
    );

    let create_seat_for = |user: Address| -> Vec<Instruction> {
        vec![
            market_ctx.base.create_ata_idempotent(&user, &user),
            market_ctx
                .base
                .mint_to_owner(&user, 1)
                .expect("Should mint to owner"),
            market_ctx.create_seat(user),
        ]
    };

    let check = MarketChecker::new(&mollusk, &market_ctx);
    check.num_seats(0);
    check.market_header(|header| {
        assert_eq!(header.num_free_sectors, MOLLUSK_DEFAULT_NUM_SECTORS as u32)
    });

    users_that_do_not_expand_market
        .into_iter()
        .for_each(|(user_addr, _)| {
            let data_len_before = mollusk.view_market_data(market_ctx.market).len();

            // Create the user ATAs for the base token, mint them a single atom, then create their
            // seat.
            assert!(mollusk
                .process_instruction_chain(&create_seat_for(user_addr))
                .program_result
                .is_ok());
            let data_len_after = mollusk.view_market_data(market_ctx.market).len();
            assert_eq!(data_len_before, data_len_after);
        });

    check.num_seats(MOLLUSK_DEFAULT_NUM_SECTORS as usize);
    check.market_header(|header| assert_eq!(header.num_free_sectors, 0));

    // Now that all originally free sectors are in use, the next deposit should trigger an automatic
    // market account data expansion.
    let data_len_prior_to_expansion = mollusk.view_market_data(market_ctx.market).len();
    assert!(mollusk
        .process_instruction_chain(&create_seat_for(user_that_triggers_expansion.0))
        .program_result
        .is_ok());
    let data_len_post_expansion = mollusk.view_market_data(market_ctx.market).len();

    check.num_seats(MOLLUSK_DEFAULT_NUM_SECTORS as usize + 1);
    check.market_header(|header| assert_eq!(header.num_free_sectors, 0));
    assert_eq!(
        data_len_post_expansion - data_len_prior_to_expansion,
        Sector::LEN
    );

    Ok(())
}
