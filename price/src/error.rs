#[repr(u8)]
#[derive(Debug)]
#[cfg_attr(test, derive(strum_macros::Display))]
pub enum OrderInfoError {
    InvalidBaseExponent,
    InvalidQuoteExponent,
    BaseMinusQuoteUnderflow,
    ArithmeticOverflow,
    InvalidPriceMantissa,
    InvalidBiasedExponent,
    InfinityIsNotAFloat,
}
