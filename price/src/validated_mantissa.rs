use pinocchio::hint;

use crate::{
    OrderInfoError,
    MANTISSA_DIGITS_LOWER_BOUND,
    MANTISSA_DIGITS_UPPER_BOUND,
};

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct ValidatedPriceMantissa(u32);

impl TryFrom<u32> for ValidatedPriceMantissa {
    type Error = OrderInfoError;

    #[inline(always)]
    fn try_from(price_mantissa: u32) -> Result<Self, Self::Error> {
        if (MANTISSA_DIGITS_LOWER_BOUND..MANTISSA_DIGITS_UPPER_BOUND).contains(&price_mantissa) {
            Ok(Self(price_mantissa))
        } else {
            hint::cold_path();
            Err(OrderInfoError::InvalidPriceMantissa)
        }
    }
}

impl ValidatedPriceMantissa {
    /// Creates a new [`ValidatedPriceMantissa`] without range checking the passed value.
    /// This should only be used when the price mantissa has definitively already been validated.
    #[inline(always)]
    pub(crate) fn new_unchecked(price_mantissa: u32) -> Self {
        Self(price_mantissa)
    }
}

impl ValidatedPriceMantissa {
    #[inline(always)]
    pub fn get(&self) -> u32 {
        self.0
    }
}
