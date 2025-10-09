use dropset_interface::{
    error::DropsetError,
    program,
    state::{
        market::{Market, MarketRef, MarketRefMut},
        market_header::MarketHeader,
        sector::SECTOR_SIZE,
        transmutable::Transmutable,
    },
    utils::owned_by,
};
use pinocchio::{account_info::AccountInfo, ProgramResult};

use crate::shared::account_resize::fund_then_resize_unchecked;

#[derive(Clone)]
pub struct MarketAccountInfo<'a> {
    /// The account info as a private field. This disallows manual construction, guaranteeing an
    /// extra level of safety and simplifying the safety contracts for the unsafe internal methods.
    info: &'a AccountInfo,
}

impl<'a> MarketAccountInfo<'a> {
    #[inline(always)]
    pub fn info(&self) -> &'a AccountInfo {
        self.info
    }

    /// Checks that the account is owned by this program and is a properly initialized `Market`.
    ///
    /// # Safety
    ///
    /// Caller guarantees the market account info's data isn't actively being borrowed.
    ///
    /// ## NOTE
    ///
    /// The safety contract is only guaranteed if market accounts are never resized below the
    /// header size after initialization. If this invariant isn't always upheld, the validation
    /// performed by this method isn't guaranteed permanently.
    #[inline(always)]
    pub unsafe fn new(info: &'a AccountInfo) -> Result<MarketAccountInfo<'a>, DropsetError> {
        if !owned_by(info, &program::ID) {
            return Err(DropsetError::InvalidMarketAccountOwner);
        }

        // Safety: Caller guarantees aliasing contract.
        let data = unsafe { info.borrow_data_unchecked() };
        if data.len() < MarketHeader::LEN {
            return Err(DropsetError::AccountNotInitialized);
        }

        if !(Market::from_bytes(data).is_initialized()) {
            return Err(DropsetError::AccountNotInitialized);
        }

        Ok(Self { info })
    }

    /// Helper function to load market data given the owner-validated and initialized account.
    ///
    /// # Safety
    ///
    /// Caller guarantees the market account info's data isn't actively being borrowed.
    #[inline(always)]
    pub unsafe fn load_unchecked(&self) -> MarketRef {
        let data = unsafe { self.info.borrow_data_unchecked() };
        // Safety: `Self::new` guarantees the account info is program-owned and initialized.
        unsafe { Market::from_bytes(data) }
    }

    /// Helper function to load market data given the owner-validated and initialized account.
    ///
    /// # Safety
    ///
    /// Caller guarantees the market account info's data isn't actively being borrowed.
    #[inline(always)]
    pub unsafe fn load_unchecked_mut(&self) -> MarketRefMut {
        let data = unsafe { self.info.borrow_mut_data_unchecked() };
        // Safety: `Self::new` guarantees the account info is program-owned and initialized.
        unsafe { Market::from_bytes_mut(data) }
    }

    #[inline(always)]
    /// Resizes the market account data and then initializes free nodes onto the free stack by
    /// calculating the available space as a factor of SECTOR_SIZE.
    ///
    /// # Safety
    ///
    /// Caller guarantees the market account info's data isn't actively being borrowed.
    pub unsafe fn resize(&self, payer: &AccountInfo, num_sectors: u16) -> ProgramResult {
        if num_sectors == 0 {
            return Err(DropsetError::InvalidNonZeroInteger.into());
        }

        let curr_n_sectors = (self.info.data_len() - MarketHeader::LEN) / SECTOR_SIZE;
        let new_n_sectors = curr_n_sectors + (num_sectors as usize);
        let additional_space = (num_sectors as usize) * SECTOR_SIZE;

        // Safety: Caller guarantees no active borrows on the market account data.
        let mut market = unsafe {
            fund_then_resize_unchecked(payer, self.info, additional_space)?;
            self.load_unchecked_mut()
        };

        let mut stack = market.free_stack();

        // Safety: Account data just zero-initialized new account space, and both indices are in
        // bounds and non-NIL.
        unsafe {
            stack.convert_zeroed_bytes_to_free_nodes(curr_n_sectors as u32, new_n_sectors as u32)
        }?;

        Ok(())
    }
}
