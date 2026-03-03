use static_assertions::const_assert_eq;

// Static assertions for macro invariants.
static_assertions::const_assert_eq!(crate::BIAS - 16, 0);
static_assertions::const_assert_eq!(crate::MAX_BIASED_EXPONENT, 31);

/// Documentation for [`pow10_u64`] relies on [`crate::BIAS`] == 16. If that changes,
/// [`crate::BIAS`] and the [`pow10_u64`] documentation needs to be updated.
const _: () = {
    const_assert_eq!(crate::BIAS, 16);
};

/// Performs base-10 exponentiation on a value using a biased exponent.
///
/// This facilitates representing negative exponent values with unsigned integers by ensuring the
/// biased exponent is never negative. The unbiased exponent is therefore the real exponent value.
///
/// # Parameters
/// - `$value`: The `u64` to be scaled by a power of 10.
/// - `$biased_exponent`: A biased exponent in the range `0..=31`.
///
/// # Biased Exponent Concept
/// The actual (aka unbiased) exponent is:
///
/// `exponent = $biased_exponent - price::BIAS`
///
/// With the current `BIAS = 16`, this means:
/// - `0`  → exponent `-16` (division by 10^16)
/// - `16` → exponent `0`   (multiplication by 1 aka 10^0)
/// - `31` → exponent `+15` (multiplication by 10^15)
///
/// On an invalid biased exponent, the macro performs an early `return Err(OrderInfoError::InvalidBiasedExponent)`
/// from the enclosing function. Overflow in the multiply path propagates via [`checked_mul`], which
/// performs an early `return Err(OrderInfoError::ArithmeticOverflow)` from the enclosing function.
///
/// # Reasoning behind exponent range
///
/// The decision to use a larger negative range instead of a larger positive range is because
/// a larger negative range results in the price mantissa * exponent product forming in a tighter
/// range around `1`.
///
/// For example, with `[-2, 1] vs [-1, 2]`:
///
/// ```markdown
/// # With [-2, 1] as the smallest/largest exponents
/// |                      | Smallest exponent   | Largest exponent    |
/// | -------------------- | ------------------- | ------------------- |
/// | Smallest mantissa    | 1.00 * 10^-2 = 0.01 | 1.00 * 10^1 =   10  |
/// | Largest mantissa     | 9.99 * 10^-2 = ~0.1 | 9.99 * 10^1 = ~100  |
/// | -------------------- | ------------------- | ------------------- |
/// ```
///
/// Both the smallest and largest products (0.01 and 100) are 2 orders
/// of magnitude below/above `1`.
///
/// ```markdown
/// # With [-1, 2] as the smallest/largest exponents
/// |                      | Smallest exponent  | Largest exponent     |
/// | -------------------- | ------------------ | -------------------- |
/// | Smallest mantissa    | 1.00 * 10^-1 = 0.1 | 1.00 * 10^2 =   100  |
/// | Largest mantissa     | 9.99 * 10^-1 =  ~1 | 9.99 * 10^2 = ~1000  |
/// | -------------------- | ------------------ | -------------------- |
/// ```
///
/// The lower product (0.1) is 1 order of magnitude below 1 and the higher
/// product (1000) is 3 orders of magnitude above 1.
///
/// The first option is preferable because it offers a more dynamic,
/// symmetrical range in terms of orders of magnitude below/above 1.
///
/// Therefore, [-16, 15] is used as the exponent range instead of [-15, 16].
///
#[macro_export]
#[rustfmt::skip]
macro_rules! pow10_u64 {
    ($value:expr, $biased_exponent:expr) => {{
        let value = $value;
        let biased_exponent = $biased_exponent;
        if biased_exponent == 16 {
            /* BIAS + 0: identity */
            value
        } else if biased_exponent < 16 {
            /* negative unbiased exponent: divide */
            match biased_exponent {
                0  => value / 10000000000000000u64, /* BIAS - 16 */
                1  => value / 1000000000000000,     /* BIAS - 15 */
                2  => value / 100000000000000,      /* BIAS - 14 */
                3  => value / 10000000000000,       /* BIAS - 13 */
                4  => value / 1000000000000,        /* BIAS - 12 */
                5  => value / 100000000000,         /* BIAS - 11 */
                6  => value / 10000000000,          /* BIAS - 10 */
                7  => value / 1000000000,           /* BIAS - 9  */
                8  => value / 100000000,            /* BIAS - 8  */
                9  => value / 10000000,             /* BIAS - 7  */
                10 => value / 1000000,              /* BIAS - 6  */
                11 => value / 100000,               /* BIAS - 5  */
                12 => value / 10000,                /* BIAS - 4  */
                13 => value / 1000,                 /* BIAS - 3  */
                14 => value / 100,                  /* BIAS - 2  */
                15 => value / 10,                   /* BIAS - 1  */
                _  => {
                    ::pinocchio::hint::cold_path();
                    return Err($crate::OrderInfoError::InvalidBiasedExponent);
                }
            }
        } else {
            let overflow_err = $crate::OrderInfoError::ArithmeticOverflow;
            /* positive unbiased exponent: multiply */
            match biased_exponent {
                17 => $crate::checked_mul!(value, 10,                overflow_err), /* BIAS + 1  */
                18 => $crate::checked_mul!(value, 100,               overflow_err), /* BIAS + 2  */
                19 => $crate::checked_mul!(value, 1000,              overflow_err), /* BIAS + 3  */
                20 => $crate::checked_mul!(value, 10000,             overflow_err), /* BIAS + 4  */
                21 => $crate::checked_mul!(value, 100000,            overflow_err), /* BIAS + 5  */
                22 => $crate::checked_mul!(value, 1000000,           overflow_err), /* BIAS + 6  */
                23 => $crate::checked_mul!(value, 10000000,          overflow_err), /* BIAS + 7  */
                24 => $crate::checked_mul!(value, 100000000,         overflow_err), /* BIAS + 8  */
                25 => $crate::checked_mul!(value, 1000000000,        overflow_err), /* BIAS + 9  */
                26 => $crate::checked_mul!(value, 10000000000,       overflow_err), /* BIAS + 10 */
                27 => $crate::checked_mul!(value, 100000000000,      overflow_err), /* BIAS + 11 */
                28 => $crate::checked_mul!(value, 1000000000000,     overflow_err), /* BIAS + 12 */
                29 => $crate::checked_mul!(value, 10000000000000,    overflow_err), /* BIAS + 13 */
                30 => $crate::checked_mul!(value, 100000000000000,   overflow_err), /* BIAS + 14 */
                31 => $crate::checked_mul!(value, 1000000000000000,  overflow_err), /* BIAS + 15 */
                _  => {
                    ::pinocchio::hint::cold_path();
                    return Err($crate::OrderInfoError::InvalidBiasedExponent);
                }
            }
        }
    }};
}

/// A checked subtraction that performs an early `return Err($err)` from the enclosing function on
/// underflow. The error path is marked as cold.
///
/// *NOTE: This is only intended for usage with **unsigned** integer types.*
///
/// # Example
/// ```rust
/// enum MyError { BadSub1, BadSub2 }
///
/// fn do_something_with_sub() -> Result<u8, MyError> {
///     let res_1 = price::checked_sub!(5u8, 4, MyError::BadSub1); // No underflow.
///     let res_2 = price::checked_sub!(5u8, 6, MyError::BadSub2); // Underflows.
///
///     // Doesn't get here because `res_2` returns early.
///     Ok(res_1)
/// }
///
/// assert!(matches!(do_something_with_sub(), Err(MyError::BadSub2)));
/// ```
#[macro_export]
macro_rules! checked_sub {
    ($lhs:expr, $rhs:expr, $err:expr $(,)?) => {{
        let lhs = $lhs;
        let rhs = $rhs;
        if lhs >= rhs {
            // SAFETY: Just checked it will not underflow.
            unsafe { lhs.unchecked_sub(rhs) }
        } else {
            ::pinocchio::hint::cold_path();
            return Err($err);
        }
    }};
}

/// A checked multiplication that performs an early `return Err($err)` from the enclosing function
/// on overflow. The error path is marked as cold.
///
/// *NOTE: This is only intended for usage with **unsigned** integer types.*
///
/// # Example
/// ```rust
/// enum MyError { BadMul1, BadMul2 }
///
/// fn do_something_with_mul() -> Result<u8, MyError> {
///     let res_1 = price::checked_mul!(255u8, 1, MyError::BadMul1); // No overflow.
///     let res_2 = price::checked_mul!(255u8, 2, MyError::BadMul2); // Overflows.
///
///     // Doesn't get here because `res_2` returns early.
///     Ok(res_1)
/// }
///
/// assert!(matches!(do_something_with_mul(), Err(MyError::BadMul2)));
/// ```
#[macro_export]
macro_rules! checked_mul {
    ($lhs:expr, $rhs:expr, $err:expr $(,)?) => {{
        match $lhs.checked_mul($rhs) {
            Some(val) => val,
            None => {
                ::pinocchio::hint::cold_path();
                return Err($err);
            }
        }
    }};
}

/// Utility macro for infallibly converting unbiased exponents to biased exponents.
///
/// The input must be a literal or const value so that the const assertions work properly.
///
/// Requires the [`static_assertions`] crate.
#[macro_export]
macro_rules! biased_exponent {
    ($unbiased_exponent:expr) => {{
        const __UNBIASED: i16 = $unbiased_exponent as i16;
        ::static_assertions::const_assert!(__UNBIASED >= $crate::UNBIASED_MIN);
        ::static_assertions::const_assert!(__UNBIASED <= $crate::UNBIASED_MAX);
        (__UNBIASED + $crate::BIAS as i16) as u8
    }};
}

/// Utility macro for infallibly converting integer literals to [`crate::ValidatedPriceMantissa`]s.
///
/// The input must be a literal or const value so that the const assertions work properly.
///
/// Requires the [`static_assertions`] crate.
#[macro_export]
macro_rules! price_mantissa {
    ($price_mantissa:expr) => {{
        const __PRICE_MANTISSA: u32 = $price_mantissa as u32;
        ::static_assertions::const_assert!(__PRICE_MANTISSA >= $crate::MANTISSA_DIGITS_LOWER_BOUND);
        ::static_assertions::const_assert!(__PRICE_MANTISSA <= $crate::MANTISSA_DIGITS_UPPER_BOUND);
        $crate::ValidatedPriceMantissa::try_from(__PRICE_MANTISSA).unwrap()
    }};
}

/// Utility macro for infallibly creating an encoded price with two literals:
/// - A price mantissa (u32)
/// - An unbiased exponent (u8)
///
/// Requires the [`static_assertions`] crate.
#[macro_export]
macro_rules! encoded_price {
    ($price_mantissa:expr, $unbiased_exponent:expr) => {{
        $crate::EncodedPrice::new(
            $crate::price_mantissa!($price_mantissa),
            $crate::biased_exponent!($unbiased_exponent),
        )
    }};
}

#[cfg(test)]
mod tests {
    use static_assertions::const_assert_eq;

    use crate::{
        client_helpers::try_to_biased_exponent,
        EncodedPrice,
        OrderInfoError,
        ValidatedPriceMantissa,
        BIAS,
        MAX_BIASED_EXPONENT,
        UNBIASED_MAX,
        UNBIASED_MIN,
    };

    #[test]
    fn check_max_biased_exponent() -> Result<(), OrderInfoError> {
        // The max biased exponent should be valid.
        assert_eq!(
            pow10_u64!(2u64, MAX_BIASED_EXPONENT),
            2 * 10u64
                .checked_pow(MAX_BIASED_EXPONENT as u32 - BIAS as u32)
                .unwrap()
        );
        // One past the max biased exponent should result in an error.

        let get_res =
            || -> Result<_, OrderInfoError> { Ok(pow10_u64!(2u64, MAX_BIASED_EXPONENT + 1)) };
        assert!(matches!(
            get_res(),
            Err(OrderInfoError::InvalidBiasedExponent)
        ));

        Ok(())
    }

    #[test]
    fn unbiased_exponent_happy_paths() {
        let expected_min = (UNBIASED_MIN + BIAS as i16) as u8;
        assert_eq!(biased_exponent!(UNBIASED_MIN), expected_min);

        let expected_mid = BIAS;
        assert_eq!(biased_exponent!(0), expected_mid);

        let expected_max = (UNBIASED_MAX + BIAS as i16) as u8;
        assert_eq!(biased_exponent!(UNBIASED_MAX), expected_max);
    }

    #[test]
    fn biased_exponents() {
        const_assert_eq!(BIAS, biased_exponent!(0));
        const_assert_eq!(BIAS + 1, biased_exponent!(1));
        const_assert_eq!(BIAS - 1, biased_exponent!(-1));
        const_assert_eq!(BIAS + 2, biased_exponent!(2));
        const_assert_eq!(BIAS - 2, biased_exponent!(-2));
    }

    #[test]
    fn price_mantissas() {
        assert_eq!(
            ValidatedPriceMantissa::try_from(10_000_000).unwrap(),
            price_mantissa!(10_000_000)
        );
        assert_eq!(
            ValidatedPriceMantissa::try_from(99_999_999).unwrap(),
            price_mantissa!(99_999_999)
        );
    }

    /// Validate all macro output exhaustively.
    #[test]
    fn exhaustive_encoded_prices() {
        macro_rules! check_encoded_prices {
            ($price_mantissa:expr, $unbiased_exponent:expr) => {{
                let a = encoded_price!($price_mantissa, $unbiased_exponent);
                let b = EncodedPrice::new(
                    ValidatedPriceMantissa::try_from($price_mantissa).unwrap(),
                    try_to_biased_exponent($unbiased_exponent).unwrap(),
                );
                assert_eq!(a, b);
            }};
        }

        check_encoded_prices!(10_000_000, UNBIASED_MIN);
        check_encoded_prices!(10_000_001, UNBIASED_MIN);
        check_encoded_prices!(55_555_555, UNBIASED_MIN);
        check_encoded_prices!(99_999_998, UNBIASED_MIN);
        check_encoded_prices!(99_999_999, UNBIASED_MIN);
        check_encoded_prices!(10_000_000, UNBIASED_MIN + 1);
        check_encoded_prices!(10_000_001, UNBIASED_MIN + 1);
        check_encoded_prices!(55_555_555, UNBIASED_MIN + 1);
        check_encoded_prices!(99_999_998, UNBIASED_MIN + 1);
        check_encoded_prices!(99_999_999, UNBIASED_MIN + 1);
        check_encoded_prices!(10_000_000, -1);
        check_encoded_prices!(10_000_001, -1);
        check_encoded_prices!(55_555_555, -1);
        check_encoded_prices!(99_999_998, -1);
        check_encoded_prices!(99_999_999, -1);
        check_encoded_prices!(10_000_000, 0);
        check_encoded_prices!(10_000_001, 0);
        check_encoded_prices!(55_555_555, 0);
        check_encoded_prices!(99_999_998, 0);
        check_encoded_prices!(99_999_999, 0);
        check_encoded_prices!(10_000_000, 1);
        check_encoded_prices!(10_000_001, 1);
        check_encoded_prices!(55_555_555, 1);
        check_encoded_prices!(99_999_998, 1);
        check_encoded_prices!(99_999_999, 1);
        check_encoded_prices!(10_000_000, UNBIASED_MAX - 1);
        check_encoded_prices!(10_000_001, UNBIASED_MAX - 1);
        check_encoded_prices!(55_555_555, UNBIASED_MAX - 1);
        check_encoded_prices!(99_999_998, UNBIASED_MAX - 1);
        check_encoded_prices!(99_999_999, UNBIASED_MAX - 1);
        check_encoded_prices!(10_000_000, UNBIASED_MAX);
        check_encoded_prices!(10_000_001, UNBIASED_MAX);
        check_encoded_prices!(55_555_555, UNBIASED_MAX);
        check_encoded_prices!(99_999_998, UNBIASED_MAX);
        check_encoded_prices!(99_999_999, UNBIASED_MAX);
    }
}
