//! CU benchmark: Pack/Unpack vs Borsh deserialization of
//! [`dropset_interface::instructions::BatchReplaceInstructionData`].

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

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

use dropset_interface::state::user_order_sectors::MAX_ORDERS_USIZE;
use price::OrderInfoArgs;

#[derive(PartialEq, Debug)]
#[cfg_attr(
    feature = "borsh-derive",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct BorshOrderInfoArgs {
    pub price_mantissa: u32,
    pub base_scalar: u64,
    pub base_exponent_biased: u8,
    pub quote_exponent_biased: u8,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(
    feature = "borsh-derive",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct BorshUnvalidatedOrders {
    pub order_args: [BorshOrderInfoArgs; MAX_ORDERS_USIZE],
}

#[derive(PartialEq, Debug)]
#[cfg_attr(
    feature = "borsh-derive",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct BorshBatchReplaceData {
    pub user_sector_index_hint: u32,
    pub new_bids: BorshUnvalidatedOrders,
    pub new_asks: BorshUnvalidatedOrders,
}

impl From<OrderInfoArgs> for BorshOrderInfoArgs {
    fn from(args: OrderInfoArgs) -> Self {
        Self {
            price_mantissa: args.price_mantissa,
            base_scalar: args.base_scalar,
            base_exponent_biased: args.base_exponent_biased,
            quote_exponent_biased: args.quote_exponent_biased,
        }
    }
}

#[inline(never)]
fn process_instruction(
    _program_id: &Address,
    _accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    // The pack version.
    #[cfg(feature = "bench-program-A")]
    {
        use dropset_interface::instructions::{
            BatchReplaceInstructionData,
            UnvalidatedOrders,
        };
        use price::OrderInfoArgs;
        use static_assertions::const_assert_eq;

        let data = BatchReplaceInstructionData::unpack_untagged(instruction_data)?;

        if data.user_sector_index_hint == u32::MAX {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Unsafely cast UnvalidatedOrders to [OrderInfoArgs; MAX_ORDERS_USIZE] to get
        // access to the underlying arrays without having to update the public API.
        // Const assertion to catch UB if the struct shape ever changes.
        const_assert_eq!(
            size_of::<UnvalidatedOrders>(),
            size_of::<[OrderInfoArgs; MAX_ORDERS_USIZE]>()
        );
        let bids_ptr = &data.new_bids as *const _ as *const [OrderInfoArgs; MAX_ORDERS_USIZE];
        let new_bids = unsafe { &*bids_ptr };
        let asks_ptr = &data.new_asks as *const _ as *const [OrderInfoArgs; MAX_ORDERS_USIZE];
        let new_asks = unsafe { &*asks_ptr };

        // Use black_box to prevent the compiler from optimizing away field accesses.
        for o in new_bids.iter() {
            black_box(o.price_mantissa);
            black_box(o.base_scalar);
            black_box(o.base_exponent_biased);
            black_box(o.quote_exponent_biased);
        }

        for o in new_asks.iter() {
            black_box(o.price_mantissa);
            black_box(o.base_scalar);
            black_box(o.base_exponent_biased);
            black_box(o.quote_exponent_biased);
        }
    }

    // The borsh version.
    #[cfg(feature = "bench-program-B")]
    {
        let data = BorshBatchReplaceData::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        if data.user_sector_index_hint == u32::MAX {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Use black_box to prevent the compiler from optimizing away field accesses.
        for o in data.new_bids.order_args.iter() {
            black_box(o.price_mantissa);
            black_box(o.base_scalar);
            black_box(o.base_exponent_biased);
            black_box(o.quote_exponent_biased);
        }

        for o in data.new_asks.order_args.iter() {
            black_box(o.price_mantissa);
            black_box(o.base_scalar);
            black_box(o.base_exponent_biased);
            black_box(o.quote_exponent_biased);
        }
    }

    Ok(())
}
