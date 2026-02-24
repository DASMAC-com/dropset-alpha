use std::{
    collections::HashMap,
    fmt::Write,
};

use client::mollusk_helpers::{
    helper_trait::DropsetTestHelper,
    new_dropset_mollusk_context_with_default_market,
    utils::create_mock_user_account,
    MOLLUSK_DEFAULT_MARKET,
};
use dropset_interface::state::sector::{
    SectorIndex,
    NIL,
};

pub use dropset_interface::state::{
    sector::MAX_PERMITTED_SECTOR_INCREASE,
    user_order_sectors::MAX_ORDERS_USIZE,
};
use mollusk_svm::MolluskContext;
use solana_account::Account;
use solana_address::Address;
use solana_instruction::Instruction;

pub use client::context::market::MarketContext;

// Token unit sizes — base and quote both use 8 decimal places.
pub const BASE_UNIT: u64 = 100_000_000;
pub const QUOTE_UNIT: u64 = 100_000_000;

/// Ask prices in ascending order (lowest ask first = highest ask priority).
/// These are kept far above bid prices to prevent any accidental crossing.
pub const ASK_PRICES: [u32; 5] = [60_000_000, 70_000_000, 80_000_000, 90_000_000, 99_000_000];

/// Bid prices in descending order (highest bid first = highest bid priority).
/// These are kept far below ask prices to prevent any accidental crossing.
pub const BID_PRICES: [u32; 5] = [50_000_000, 40_000_000, 30_000_000, 20_000_000, 10_000_000];

/// Table width for formatted output.
pub const W: usize = 40;

/// A fully initialized benchmark fixture: a market with a funded, seated maker.
pub struct BenchFixture {
    pub ctx: MolluskContext<HashMap<Address, Account>>,
    pub market_ctx: MarketContext,
    pub maker: Address,
    pub seat_index: SectorIndex,
}

/// Creates a benchmark fixture with:
/// - A default market (base + quote mints, 10 initial sectors).
/// - A maker with ATAs, minted tokens, a seat, and deposited base + quote.
///
/// The market is NOT pre-expanded; call [`expand_market`] if you want to measure instructions
/// without the cost of account reallocation.
pub fn new_bench_fixture() -> BenchFixture {
    // Give the maker enough lamports to pay for market expansions.
    let maker_mock = create_mock_user_account(Address::new_unique(), 10_000_000_000);
    let maker = maker_mock.0;
    let (ctx, market_ctx) = new_dropset_mollusk_context_with_default_market(&[maker_mock]);

    // Create ATAs and mint a large supply of both tokens.
    let res = ctx.process_instruction_chain(&[
        market_ctx.base.create_ata_idempotent(&maker, &maker),
        market_ctx.quote.create_ata_idempotent(&maker, &maker),
        market_ctx.base.mint_to_owner(&maker, 1_000 * BASE_UNIT).unwrap(),
        market_ctx.quote.mint_to_owner(&maker, 1_000_000 * QUOTE_UNIT).unwrap(),
    ]);
    assert!(res.program_result.is_ok(), "fixture ATA/mint setup failed");

    // First deposit_base with NIL creates the maker's seat.
    let res = ctx.process_instruction_chain(&[market_ctx.deposit_base(maker, 500 * BASE_UNIT, NIL)]);
    assert!(res.program_result.is_ok(), "fixture initial deposit failed");

    let seat_index = ctx.get_seat(MOLLUSK_DEFAULT_MARKET.market, maker).index;

    let res = ctx.process_instruction_chain(&[market_ctx.deposit_quote(
        maker,
        500_000 * QUOTE_UNIT,
        seat_index,
    )]);
    assert!(res.program_result.is_ok(), "fixture quote deposit failed");

    BenchFixture { ctx, market_ctx, maker, seat_index }
}

/// Expands the market by [`MAX_PERMITTED_SECTOR_INCREASE`] sectors.
///
/// Call before running a measured instruction to isolate its cost from account reallocation.
pub fn expand_market(f: &BenchFixture) {
    let res = f.ctx.process_instruction_chain(&[f
        .market_ctx
        .expand(f.maker, MAX_PERMITTED_SECTOR_INCREASE as u16)]);
    assert!(res.program_result.is_ok(), "expand_market failed");
}

/// Inserts a new system account into the fixture's account store and returns its address.
///
/// The account is funded with `lamports` so it can pay for ATA creation and other instructions.
pub fn add_user(f: &BenchFixture, lamports: u64) -> Address {
    let (addr, account) = create_mock_user_account(Address::new_unique(), lamports);
    f.ctx.account_store.borrow_mut().insert(addr, account);
    addr
}

/// Creates a new maker with ATAs, minted base tokens, and a seat deposit.
///
/// Returns the new maker's address and their assigned seat index.
pub fn add_funded_maker(f: &BenchFixture) -> (Address, SectorIndex) {
    let maker = add_user(f, 10_000_000_000);
    let res = f.ctx.process_instruction_chain(&[
        f.market_ctx.base.create_ata_idempotent(&maker, &maker),
        f.market_ctx.base.mint_to_owner(&maker, 1_000 * BASE_UNIT).unwrap(),
        f.market_ctx.deposit_base(maker, 500 * BASE_UNIT, NIL),
    ]);
    assert!(res.program_result.is_ok(), "add_funded_maker setup failed: {:?}", res.program_result);
    let seat_index = f.ctx.get_seat(MOLLUSK_DEFAULT_MARKET.market, maker).index;
    (maker, seat_index)
}

/// Processes a single instruction on the fixture's context and returns its compute units consumed.
///
/// Panics if the instruction fails.
pub fn measure_cu(f: &BenchFixture, ix: Instruction) -> u64 {
    let result = f.ctx.process_instruction_chain(&[ix]);
    assert!(
        result.program_result.is_ok(),
        "measured instruction failed: {:?}",
        result.program_result
    );
    result.compute_units_consumed
}

// ── Formatting helpers ───────────────────────────────────────────────────────

/// Write a line centered within [`W`] characters.
pub fn wc(logs: &mut String, line: &str) {
    writeln!(logs, "{:^W$}", line).unwrap();
}

/// Write a `====== title ======` header line.
pub fn fmt_header(logs: &mut String, title: &str) {
    writeln!(logs, "\n{:=^W$}", format!(" {title} ")).unwrap();
}

/// Write a centered sub-table: column header, dashes, and data rows.
pub fn fmt_subtable(logs: &mut String, col_left: &str, rows: &[(u64, u64)]) {
    logs.push('\n');
    wc(logs, &format!("{:<14}{:>9}", col_left, "Average CU"));
    wc(logs, &"-".repeat(24));
    for &(n, avg) in rows {
        let label = format!("{n:>7} ");
        wc(logs, &format!("{label:<14}  {avg:>6}  "));
    }
}
