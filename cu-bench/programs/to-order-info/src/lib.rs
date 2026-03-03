//! CU benchmark: `to_order_info` — 10 varied inputs, measure total / 10.

#![no_std]

use core::hint::black_box;

use pinocchio::{
    account::AccountView,
    error::ProgramError,
    no_allocator,
    nostd_panic_handler,
    program_entrypoint,
    Address,
    ProgramResult,
};
use price::{
    biased_exponent,
    to_order_info,
    OrderInfoArgs,
};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

#[inline(never)]
fn process_instruction(
    _program_id: &Address,
    _accounts: &[AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Ten varied inputs covering positive, zero, and negative unbiased exponents.
    // Each field is black_box'd to prevent const-folding.
    macro_rules! call {
        ($m:expr, $s:expr, $b:expr, $q:expr) => {{
            let args = OrderInfoArgs::new(
                black_box($m),
                black_box($s),
                black_box(biased_exponent!($b)),
                black_box(biased_exponent!($q)),
            );
            black_box(to_order_info(args).map_err(|_| ProgramError::InvalidInstructionData)?);
        }};
    }

    call!(12_500_000u32, 5u64, 8, 1);
    call!(50_000_000u32, 1u64, 0, 0);
    call!(10_000_000u32, 1u64, 6, -1);
    call!(99_999_999u32, 1u64, -3, -6);
    call!(20_000_000u32, 10u64, 5, 2);
    call!(75_000_000u32, 1u64, 0, -7);
    call!(33_333_333u32, 3u64, 4, 0);
    call!(15_000_000u32, 100u64, 2, -1);
    call!(88_000_000u32, 7u64, 3, 1);
    call!(42_000_000u32, 1_000u64, 0, -2);

    Ok(())
}
