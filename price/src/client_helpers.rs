//! Utility functions to assist in calculating prices with decimals client-side. Not intended
//! to be used in smart contracts.

use core::num::NonZeroU64;

use rust_decimal::{
    dec,
    Decimal,
    MathematicalOps,
};

use crate::{
    to_order_info,
    DecodedPrice,
    EncodedPrice,
    OrderInfoArgs,
    OrderInfoError,
    ValidatedPriceMantissa,
    BIAS,
    UNBIASED_MAX,
    UNBIASED_MIN,
};

/// Try converting an unbiased exponent to a biased one.
pub fn try_to_biased_exponent(unbiased_exponent: i16) -> Result<u8, OrderInfoError> {
    if !(UNBIASED_MIN..=UNBIASED_MAX).contains(&unbiased_exponent) {
        return Err(OrderInfoError::InvalidBiasedExponent);
    }
    Ok((unbiased_exponent + BIAS as i16) as u8)
}

/// Returns the significant figures/digits in a u64 and the power of 10 to which that number must
/// be multiplied by to achieve the original input value.
fn get_sig_figs(value: NonZeroU64) -> (u64, i16) {
    let mut x = value.into();
    let mut pow: i16 = 0;
    while x % 10 == 0 {
        x /= 10;
        pow += 1;
    }

    (x, pow)
}

/// A helper function to convert a price ratio and order size (in base atoms) to order info args.
///
/// NOTE: Make sure `price` here equals `quote_atoms / base_atoms`. That is, the price ratio in
/// atoms doesn't equal the price ratio in UI-based (aka human-readable) units if the base and quote
/// tokens don't use the same amount of decimals.
///
/// To convert a price in human-readable units to atoms, use [ui_price_to_atoms_price].
pub fn to_order_info_args(
    price: Decimal,
    order_size_base_atoms: u64,
) -> Result<OrderInfoArgs, OrderInfoError> {
    let (validated_mantissa, price_exponent) = ValidatedPriceMantissa::try_into_with_scale(price)?;

    let order_size_non_zero =
        NonZeroU64::try_from(order_size_base_atoms).or(Err(OrderInfoError::AmountCannotBeZero))?;
    let (base_scalar, base_exponent_unbiased) = get_sig_figs(order_size_non_zero);

    // price_exponent == quote_exponent - base_exponent.
    // quote_exponent == price_exponent + base_exponent.
    let quote_exponent_unbiased = price_exponent
        .checked_add(base_exponent_unbiased)
        .ok_or(OrderInfoError::InvalidBiasedExponent)?;

    let quote_exponent_biased = try_to_biased_exponent(quote_exponent_unbiased)?;
    let base_exponent_biased = try_to_biased_exponent(base_exponent_unbiased)?;

    Ok(OrderInfoArgs::new(
        validated_mantissa.as_u32(),
        base_scalar,
        base_exponent_biased,
        quote_exponent_biased,
    ))
}

/// Multiplies a `value` by 10 to the power of `pow`.
pub fn decimal_pow10(value: Decimal, pow: i64) -> Result<Decimal, OrderInfoError> {
    dec!(10)
        .checked_powi(pow)
        .and_then(|scale| value.checked_mul(scale))
        .ok_or(OrderInfoError::ArithmeticOverflow)
}

/// Converts an amount denominated in atoms to an amount denominated with a token's decimals.
pub fn atoms_to_ui_amount(atoms_amount: u64, mint_decimals: u8) -> Result<Decimal, OrderInfoError> {
    decimal_pow10(Decimal::from(atoms_amount), -(mint_decimals as i64))
}

/// Converts a token price not denominated in atoms to a token price denominated in atoms using
/// exponentiation based on the base and quote token's decimals.
pub fn ui_price_to_atoms_price(
    ui_price: Decimal,
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<Decimal, OrderInfoError> {
    decimal_pow10(ui_price, quote_decimals as i64 - base_decimals as i64)
}

/// Converts a u32 encoded price to a decoded decimal price. Typical usage would be converting the
/// on-chain u32 in an order to the decoded decimal price.
pub fn try_encoded_u32_to_decoded_decimal(encoded_u32: u32) -> Result<Decimal, OrderInfoError> {
    let encoded_price: EncodedPrice = encoded_u32.try_into()?;
    let decoded_price: DecodedPrice = encoded_price.try_into()?;
    let decimal_price: Decimal = decoded_price.try_into()?;

    Ok(decimal_price)
}

/// Sum the total base necessary to post every order in the passed order slice.
///
/// Typically used for summing ask collateral.
pub fn sum_base_necessary(orders: &[OrderInfoArgs]) -> Result<u64, OrderInfoError> {
    orders
        .iter()
        .map(|o| to_order_info(o.clone()).map(|info| info.base_atoms))
        .sum()
}

/// Sum the total quote necessary to post every order in the passed order slice.
///
/// Typically used for summing bid collateral.
pub fn sum_quote_necessary(orders: &[OrderInfoArgs]) -> Result<u64, OrderInfoError> {
    orders
        .iter()
        .map(|o| to_order_info(o.clone()).map(|info| info.quote_atoms))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biased_exponent;

    #[test]
    fn test_sig_figs() {
        assert_eq!(get_sig_figs(NonZeroU64::new(16801).unwrap()), (16801, 0));
        assert_eq!(get_sig_figs(NonZeroU64::new(168010).unwrap()), (16801, 1));
        assert_eq!(
            get_sig_figs(NonZeroU64::new(100_000_000_000).unwrap()),
            (1, 11)
        );
        assert_eq!(
            get_sig_figs(NonZeroU64::new(909_512_730_220).unwrap()),
            (90_951_273_022, 1)
        );
        assert_eq!(get_sig_figs(NonZeroU64::new(99).unwrap()), (99, 0));
        assert_eq!(get_sig_figs(NonZeroU64::new(909).unwrap()), (909, 0));
        assert_eq!(get_sig_figs(NonZeroU64::new(9090).unwrap()), (909, 1));
        assert_eq!(get_sig_figs(NonZeroU64::new(404_000).unwrap()), (404, 3));

        // Check that the values returned actually do equal the sig figs and power of 10.
        let n = NonZeroU64::new(4_125_900).unwrap();
        let expected_num: u64 = 41_259;
        let expected_pow_10: i16 = 2;
        assert_eq!(get_sig_figs(n), (expected_num, expected_pow_10));
        assert_eq!(n.get(), expected_num * 10u64.pow(expected_pow_10 as u32));
    }

    #[test]
    fn test_try_biased_exponents() {
        let expected_min = (UNBIASED_MIN + BIAS as i16) as u8;
        let expected_mid = BIAS;
        let expected_max = (UNBIASED_MAX + BIAS as i16) as u8;

        assert_eq!(try_to_biased_exponent(UNBIASED_MIN).unwrap(), expected_min);
        assert_eq!(try_to_biased_exponent(0).unwrap(), expected_mid);
        assert_eq!(try_to_biased_exponent(UNBIASED_MAX).unwrap(), expected_max);

        assert!(try_to_biased_exponent(UNBIASED_MIN - 1).is_err());
        assert!(try_to_biased_exponent(UNBIASED_MAX + 1).is_err());
    }

    #[test]
    fn test_to_order_info_args() {
        assert!(to_order_info_args(rust_decimal::dec!(1.5123), 500_000).is_ok());

        // Test the example in the doctest for the main to order info function.
        let base_atoms = 500 * 10u64.pow(6);
        let res = to_order_info_args(rust_decimal::dec!(1.25), base_atoms);
        let expected = OrderInfoArgs::new(12_500_000, 5, biased_exponent!(8), biased_exponent!(1));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
    }

    #[test]
    fn test_decimal_pow10() -> Result<(), OrderInfoError> {
        assert_eq!(decimal_pow10(dec!(1.23), 2)?, dec!(123));
        assert_eq!(decimal_pow10(dec!(1.6923), 3)?, dec!(1692.3));
        assert_eq!(decimal_pow10(dec!(1.000333), 4)?, dec!(10003.33));
        assert_eq!(decimal_pow10(dec!(1.23), -1)?, dec!(0.123));
        assert_eq!(decimal_pow10(dec!(1.23), -2)?, dec!(0.0123));
        assert_eq!(decimal_pow10(dec!(0.05123), -9)?, dec!(0.00000000005123));

        Ok(())
    }

    #[test]
    fn varying_decimal_pair() -> Result<(), OrderInfoError> {
        // Equal decimals => do nothing.
        assert_eq!(ui_price_to_atoms_price(dec!(1.27), 6, 6)?, dec!(1.27));

        // 10 ^ (quote - base) == 10 ^ 1 == multiply by 10
        assert_eq!(ui_price_to_atoms_price(dec!(1.27), 5, 6)?, dec!(12.7));

        // 10 ^ (quote - base) == 10 ^ -1 == divide by 10
        assert_eq!(ui_price_to_atoms_price(dec!(1.27), 6, 5)?, dec!(0.127));

        // 10 ^ (quote - base) == 10 ^ (19 - 11) == multiply by 10 ^ 8
        assert_eq!(
            ui_price_to_atoms_price(dec!(1.27), 11, 19)?,
            dec!(127_000_000)
        );

        // 10 ^ (quote - base) == 10 ^ (11 - 19) = divide by 10 ^ 8
        assert_eq!(
            ui_price_to_atoms_price(dec!(1.27), 19, 11)?,
            dec!(0.0000000127)
        );

        Ok(())
    }
}
