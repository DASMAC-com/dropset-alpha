use static_assertions::const_assert;

/// The number of significant digits in the significant; i.e., the digits represented in the price.
///
/// For example, for a price of 12.413 and [`SIGNIFICANT_DIGITS`] == 8:
///
/// THe significand/price would be `12_413_000`.
pub const SIGNIFICANT_DIGITS: u8 = 8;

#[repr(C)]
pub struct Price {
    pub price: u64,
    pub base: u64,
    pub quote: u64,
}

#[derive(Debug)]
#[cfg_attr(test, derive(strum_macros::Display))]
pub enum PriceError {
    InvalidLotExponent,
    InvalidTickExponent,
    LotMinusTickUnderflow,
}

type SignificandType = u32;
type LotsType = u16;

#[allow(clippy::absurd_extreme_comparisons)]
const _: () = {
    const_assert!((LotsType::MAX as u64 * SignificandType::MAX as u64) <= u64::MAX);
};

pub fn to_price(
    significand: u32,
    lots: u16,
    lot_exp: u8,
    tick_exp: u8,
) -> Result<Price, PriceError> {
    // This is only for compile-time type checking. It ensures that the const assertion types
    // reflect the types passed to this function.
    let _: &SignificandType = &significand;
    let _: &LotsType = &lots;

    let lots = lots as u64;
    let significand = significand as u64;

    let base = lots
        .checked_mul(pow10_u64!(lot_exp, PriceError::InvalidLotExponent))
        .ok_or(PriceError::InvalidLotExponent)?;

    // Note: significant * lots is always <= u64::MAX, checked with const asserts above.
    let quote = (significand * lots)
        .checked_mul(pow10_u64!(tick_exp, PriceError::InvalidLotExponent))
        .ok_or(PriceError::InvalidLotExponent)?;

    if lot_exp > tick_exp {
        return Err(PriceError::LotMinusTickUnderflow);
    }
    let price_exp = pow10_u64!(tick_exp - lot_exp, PriceError::InvalidLotExponent);
    let price = significand
        .checked_mul(price_exp)
        .ok_or(PriceError::InvalidLotExponent)?;

    Ok(Price { price, base, quote })
}

/// Returns `10^exp` inline using a `match` on the exponent.
///
/// Supported exponents map directly to their corresponding power-of-ten
/// value. Any unsupported exponent causes the macro to emit an early
/// `return Err($err)` from the surrounding function.
///
/// # Example
///
/// ```
/// let scale = pow10_u64!(3, MyError::InvalidExponent);
/// assert_eq!(scale, 1000); // 10^3
/// ```
#[macro_export]
macro_rules! pow10_u64 {
    ($exp:expr, $err:expr) => {{
        match $exp {
            0 => 1u64,
            1 => 10,
            2 => 100,
            3 => 1_000,
            4 => 10_000,
            5 => 100_000,
            6 => 1_000_000,
            7 => 10_000_000,
            8 => 100_000_000,
            9 => 1_000_000_000,
            10 => 10_000_000_000,
            11 => 100_000_000_000,
            12 => 1_000_000_000_000,
            13 => 10_000_000_000_000,
            14 => 100_000_000_000_000,
            15 => 1_000_000_000_000_000,
            _ => return Err($err),
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _result = to_price(2, 2, 3, 4).expect("Should calculate price");
    }
}
