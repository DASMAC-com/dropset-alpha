#![allow(rustdoc::private_intra_doc_links)]

use core::mem::MaybeUninit;

use instruction_macros::{
    Pack,
    Unpack,
};
use price::{
    to_order_info,
    OrderInfo,
    OrderInfoArgs,
};
use solana_program_error::ProgramError;
use static_assertions::{
    const_assert,
    const_assert_eq,
};

use crate::{
    error::DropsetError,
    instructions::orders::private::UpToTen,
    state::user_order_sectors::{
        MAX_ORDERS,
        MAX_ORDERS_USIZE,
    },
};

/// A static array of unvalidated orders args with up to [MAX_ORDERS] valid [OrderInfoArgs].
///
/// Each element is validated in the [Self::into_valid_order_infos_iter] iterator when passed to
/// [price::to_order_info].
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnvalidatedOrders {
    /// Instruction data that isn't read is free, so it's simpler to always use [MAX_ORDERS]
    /// elements in the array and simply ignore elements at and after the first invalid element
    /// than to use a slice with a dynamic length.
    order_args: [OrderInfoArgs; MAX_ORDERS_USIZE],
}

impl UnvalidatedOrders {
    /// Create up to [MAX_ORDERS_USIZE] orders as efficiently as possible. For a less efficient,
    /// more ergonomic version of this method, see [Self::new_from_slice].
    #[inline(always)]
    pub fn new<const N: usize>(orders: [OrderInfoArgs; N]) -> Self
    where
        [OrderInfoArgs; N]: UpToTen<N>,
    {
        let mut res: [MaybeUninit<OrderInfoArgs>; MAX_ORDERS_USIZE] =
            [const { MaybeUninit::uninit() }; MAX_ORDERS_USIZE];

        unsafe {
            // Copy the orders passed in. This initializes `res[0..N]`.
            //
            // Safety:
            // - `orders` is valid for `N` reads.
            // - `res` is valid for `MAX_ORDERS` writes, and `MAX_ORDERS` >= `N`.
            // - Both pointers are aligned and do not overlap.
            core::ptr::copy_nonoverlapping(
                orders.as_ptr(),
                res.as_mut_ptr() as *mut OrderInfoArgs,
                N,
            );

            // Write zeros to the remaining elements. This initializes `res[N..MAX_ORDERS]`.
            //
            // Safety:
            // - `res.as_mut_ptr().add(N)` is valid for up to `MAX_ORDERS - N` writes and `N` is
            //   guaranteed to be <= `MAX_ORDERS`.
            // - Zero is a valid value for all field types.
            core::ptr::write_bytes(
                (res.as_mut_ptr() as *mut OrderInfoArgs).add(N),
                0u8,
                MAX_ORDERS_USIZE - N,
            );
        }

        Self {
            // Safety: The array has been fully initialized with `MAX_ORDERS` elements.
            order_args: unsafe {
                core::mem::transmute::<
                    [MaybeUninit<OrderInfoArgs>; MAX_ORDERS_USIZE],
                    [OrderInfoArgs; MAX_ORDERS_USIZE],
                >(res)
            },
        }
    }

    /// A more ergonomic version of [Self::new] that receives a slice argument instead of a
    /// statically sized array.
    pub fn new_from_slice(orders: &[OrderInfoArgs]) -> Result<Self, DropsetError> {
        fn to_array<const N: usize>(orders: &[OrderInfoArgs]) -> [OrderInfoArgs; N] {
            core::array::from_fn::<_, N, _>(|i| orders[i].clone())
        }

        Ok(match orders.len() {
            0 => Self::new(to_array::<0>(orders)),
            1 => Self::new(to_array::<1>(orders)),
            2 => Self::new(to_array::<2>(orders)),
            3 => Self::new(to_array::<3>(orders)),
            4 => Self::new(to_array::<4>(orders)),
            5 => Self::new(to_array::<5>(orders)),
            6 => Self::new(to_array::<6>(orders)),
            7 => Self::new(to_array::<7>(orders)),
            8 => Self::new(to_array::<8>(orders)),
            9 => Self::new(to_array::<9>(orders)),
            10 => Self::new(to_array::<10>(orders)),
            _ => {
                // If the max number of orders changes, the match arms above need to be updated.
                // The value checked in the const assertion below also needs to be updated.
                const_assert!(MAX_ORDERS_USIZE <= 10);
                return Err(DropsetError::InvalidInstructionData);
            }
        })
    }

    /// Converts and validates [Self::order_args] into an owned iterator of [OrderInfo].
    /// Iteration stops at the first invalid element.
    #[inline(always)]
    pub fn into_valid_order_infos_iter(self) -> impl Iterator<Item = OrderInfo> {
        self.order_args
            .into_iter()
            .map_while(|args| to_order_info(args).ok())
    }
}

unsafe impl Pack for UnvalidatedOrders {
    type Packed = [u8; OrderInfoArgs::LEN * MAX_ORDERS_USIZE];

    /// # Safety
    ///
    /// Writes [OrderInfoArgs::LEN] bytes to `dst` [MAX_ORDERS] times, with each write
    /// destination offset increased by the amount written each time.
    #[inline(always)]
    unsafe fn write_bytes(&self, dst: *mut u8) {
        // This implementation was written according to a specific max number of orders. If that
        // amount changes, this implementation needs to be updated to account for the new amount.
        const_assert_eq!(MAX_ORDERS, 10);

        self.order_args[0].write_bytes(dst);
        self.order_args[1].write_bytes(dst.add(OrderInfoArgs::LEN));
        self.order_args[2].write_bytes(dst.add(OrderInfoArgs::LEN * 2));
        self.order_args[3].write_bytes(dst.add(OrderInfoArgs::LEN * 3));
        self.order_args[4].write_bytes(dst.add(OrderInfoArgs::LEN * 4));
        self.order_args[5].write_bytes(dst.add(OrderInfoArgs::LEN * 5));
        self.order_args[6].write_bytes(dst.add(OrderInfoArgs::LEN * 6));
        self.order_args[7].write_bytes(dst.add(OrderInfoArgs::LEN * 7));
        self.order_args[8].write_bytes(dst.add(OrderInfoArgs::LEN * 8));
        self.order_args[9].write_bytes(dst.add(OrderInfoArgs::LEN * 9));
    }

    #[inline(always)]
    fn pack(&self) -> Self::Packed {
        let mut data: [MaybeUninit<u8>; Self::LEN] = [MaybeUninit::uninit(); Self::LEN];
        let dst = data.as_mut_ptr() as *mut u8;
        // Safety: `dst` points to `Self::LEN` contiguous, writable bytes.
        unsafe { self.write_bytes(dst) };

        // Safety: All bytes are initialized during the construction above.
        unsafe { *(data.as_ptr() as *const [u8; Self::LEN]) }
    }
}

unsafe impl Unpack for UnvalidatedOrders {
    /// # Safety (implementor)
    ///
    /// - Reads [OrderInfoArgs::LEN] bytes from `src` [MAX_ORDERS] times.
    /// - `src` is read from as an unaligned pointer.
    ///
    /// # Safety (caller)
    ///
    /// Caller must guarantee `src` points to at least [Self::LEN] bytes of readable memory.
    #[inline(always)]
    unsafe fn read_bytes(src: *const u8) -> Result<Self, solana_program_error::ProgramError> {
        // This implementation was written according to a specific max number of orders. If that
        // amount changes, this implementation needs to be updated to account for the new amount.
        const_assert_eq!(MAX_ORDERS, 10);

        Ok(Self {
            order_args: [
                OrderInfoArgs::read_bytes(src)?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 2))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 3))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 4))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 5))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 6))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 7))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 8))?,
                OrderInfoArgs::read_bytes(src.add(OrderInfoArgs::LEN * 9))?,
            ],
        })
    }

    #[inline(always)]
    fn unpack(data: &[u8]) -> Result<Self, solana_program_error::ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Safety: `data` has at least `Self::LEN` bytes.
        unsafe { Self::read_bytes(data.as_ptr()) }
    }
}

#[cfg(feature = "codama")]
impl instruction_macros::codama::CodamaType for UnvalidatedOrders {
    fn codama_type_node() -> instruction_macros::codama::TypeNode {
        instruction_macros::codama::array_type(
            <OrderInfoArgs as instruction_macros::codama::CodamaType>::codama_type_node(),
            MAX_ORDERS_USIZE,
        )
    }
}

mod private {
    use super::*;

    // The trait implementations below were written according to a specific max number of orders.
    // If this number changes, the trait needs to change to account for the different size.
    const_assert_eq!(MAX_ORDERS, 10);

    /// Marker trait: implemented only for arrays of length 0..=[MAX_ORDERS].
    pub trait UpToTen<const N: usize> {}

    impl UpToTen<0> for [OrderInfoArgs; 0] {}
    impl UpToTen<1> for [OrderInfoArgs; 1] {}
    impl UpToTen<2> for [OrderInfoArgs; 2] {}
    impl UpToTen<3> for [OrderInfoArgs; 3] {}
    impl UpToTen<4> for [OrderInfoArgs; 4] {}
    impl UpToTen<5> for [OrderInfoArgs; 5] {}
    impl UpToTen<6> for [OrderInfoArgs; 6] {}
    impl UpToTen<7> for [OrderInfoArgs; 7] {}
    impl UpToTen<8> for [OrderInfoArgs; 8] {}
    impl UpToTen<9> for [OrderInfoArgs; 9] {}
    impl UpToTen<10> for [OrderInfoArgs; 10] {}
}
