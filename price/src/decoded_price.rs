use crate::{
    EncodedPrice,
    OrderInfoError,
    ValidatedPriceMantissa,
    BIAS,
    PRICE_MANTISSA_BITS,
    PRICE_MANTISSA_MASK,
};

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct DecodedPrice {
    pub price_exponent_biased: u8,
    pub price_mantissa: ValidatedPriceMantissa,
}

impl TryFrom<EncodedPrice> for DecodedPrice {
    type Error = OrderInfoError;

    fn try_from(value: EncodedPrice) -> Result<Self, Self::Error> {
        let price_exponent_biased = (value.0 >> PRICE_MANTISSA_BITS) as u8;
        let validated_mantissa = value.0 & PRICE_MANTISSA_MASK;

        Ok(Self {
            price_exponent_biased,
            price_mantissa: ValidatedPriceMantissa::new_unchecked(validated_mantissa),
        })
    }
}

impl From<DecodedPrice> for f64 {
    fn from(decoded: DecodedPrice) -> Self {
        (decoded.price_mantissa.get() as f64)
            * 10f64.powi(decoded.price_exponent_biased as i32 - BIAS as i32)
    }
}
