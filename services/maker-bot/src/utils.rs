use std::{
    collections::HashMap,
    hash::Hash,
};

use client::context::market::MarketContext;
use dropset_services_shared::oanda_types::{
    CurrencyPair,
    OandaCandlestickResponse,
};
use price::client_helpers::ui_price_to_atoms_price;
use rust_decimal::Decimal;

pub fn get_normalized_mid_price(
    candlestick_response: OandaCandlestickResponse,
    expected_pair: &CurrencyPair,
    market_ctx: &MarketContext,
) -> anyhow::Result<Decimal> {
    let response_pair = &candlestick_response.instrument;
    if expected_pair != response_pair {
        anyhow::bail!(
            "Maker and candlestick response pair don't match. {expected_pair} != {response_pair}"
        );
    }

    let sorted_candles = {
        let mut candles = candlestick_response.candles;
        candles.sort_by_key(|c| c.time);
        candles
    };

    let latest_price = match sorted_candles.last() {
        Some(candlestick) => {
            candlestick
                .mid
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("`mid` price not found in the last candlestick."))?
                .c
        }
        None => anyhow::bail!("There are zero candlesticks in the candlestick response"),
    };

    Ok(ui_price_to_atoms_price(
        latest_price,
        market_ctx.base.mint_decimals,
        market_ctx.quote.mint_decimals,
    )?)
}

/// Returns values from each hashmap whose keys don't exist in the other.
///
/// Filtering is by key only; values are ignored when determining uniqueness.
///
/// For example, with hashmap inputs `a` and `b`:
///
/// a: (1, "a"), (2, "b"), (3, "c")]
/// b: (3, "x"), (4, "d"), (5, "e")]
///
/// This function would return two vecs: ["a", "b"] and ["d", "e"].
pub fn split_symmetric_difference<'a, K: Eq + Hash, V1, V2>(
    a: &'a HashMap<K, V1>,
    b: &'a HashMap<K, V2>,
) -> (Vec<&'a V1>, Vec<&'a V2>) {
    let a_uniques = a
        .iter()
        .filter(|(k, _)| !b.contains_key(k))
        .map(|(_, v)| v)
        .collect();
    let b_uniques = b
        .iter()
        .filter(|(k, _)| !a.contains_key(k))
        .map(|(_, v)| v)
        .collect();
    (a_uniques, b_uniques)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_symmetric_difference_doc_example() {
        // From doc comment: a = {1: "a", 2: "b", 3: "c"}, b = {3: "c", 4: "d", 5: "e"}
        // Expected: ([a, b], [d, e])
        let a: HashMap<i32, &str> = [(1, "a"), (2, "b"), (3, "c")].into();
        let b: HashMap<i32, &str> = [(3, "c"), (4, "d"), (5, "e")].into();

        let (mut a_uniques, mut b_uniques) = split_symmetric_difference(&a, &b);
        a_uniques.sort();
        b_uniques.sort();

        assert_eq!(a_uniques, vec![&"a", &"b"]);
        assert_eq!(b_uniques, vec![&"d", &"e"]);
    }
}
