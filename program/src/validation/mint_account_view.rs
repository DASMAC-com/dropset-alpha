//! See [`MintAccountView`].

use dropset_interface::{
    error::DropsetError,
    state::market::MarketRef,
};
use pinocchio::{
    account::AccountView,
    error::ProgramError,
};
use pinocchio_token_interface::state::{
    load_unchecked as pinocchio_load_unchecked,
    mint::Mint,
};
use solana_address::address_eq;

/// A validated wrapper around a raw mint [`AccountView`], exposing verified metadata such as
/// supply, decimals, and authorities.
#[derive(Clone)]
pub struct MintAccountView<'a> {
    pub account: &'a AccountView,
    /// Flag for which mint this is. Facilitates skipping several address comparisons.
    pub is_base_mint: bool,
}

impl<'a> MintAccountView<'a> {
    #[inline(always)]
    pub fn new(
        account: &'a AccountView,
        market: MarketRef,
    ) -> Result<MintAccountView<'a>, ProgramError> {
        if address_eq(account.address(), &market.header.base_mint) {
            Ok(Self {
                account,
                is_base_mint: true,
            })
        } else if address_eq(account.address(), &market.header.quote_mint) {
            Ok(Self {
                account,
                is_base_mint: false,
            })
        } else {
            Err(DropsetError::InvalidMintAccount.into())
        }
    }

    /// Verifies the `base` and `quote` accounts passed in are valid according to the addresses
    /// stored in the market header.
    #[inline(always)]
    pub fn new_base_and_quote(
        base: &'a AccountView,
        quote: &'a AccountView,
        market: MarketRef,
    ) -> Result<(MintAccountView<'a>, MintAccountView<'a>), DropsetError> {
        // The two mints in the header will never be invalid since they're checked prior to
        // initialization and never updated, so the only thing that's necessary to check is that the
        // account addresses match the ones in the header.
        if !address_eq(base.address(), &market.header.base_mint)
            || !address_eq(quote.address(), &market.header.quote_mint)
        {
            return Err(DropsetError::InvalidMintAccount);
        }

        Ok((
            Self {
                account: base,
                is_base_mint: true,
            },
            Self {
                account: quote,
                is_base_mint: false,
            },
        ))
    }

    /// Borrows the mint account's data to get the mint decimals.
    ///
    /// # Safety
    ///
    /// Caller guarantees:
    /// - WRITE accounts are not currently borrowed in *any* capacity.
    /// - READ accounts are not currently mutably borrowed.
    ///
    /// ### Accounts
    ///   0. `[READ]` Mint account
    #[inline(always)]
    pub unsafe fn get_mint_decimals(&self) -> Result<u8, ProgramError> {
        let data = unsafe { self.account.borrow_unchecked() };
        // Safety: `self` contains verifiably initialized base and quote mints, since the market
        // header stores their addresses and they are checked against the values in the header any
        // time `self` is constructed.
        Ok(unsafe { pinocchio_load_unchecked::<Mint>(data) }
            .map_err(|_| ProgramError::InvalidAccountData)?
            .decimals)
    }
}
