use std::collections::HashMap;

use dropset_interface::{
    instructions::{
        CancelOrderInstructionData,
        PostOrderInstructionData,
    },
    state::sector::SectorIndex,
};
use itertools::Itertools;
use price::{
    client_helpers::to_order_info_args,
    to_order_info,
    OrderInfoArgs,
};
use rust_decimal::Decimal;
use transaction_parser::views::OrderView;

use crate::maker_context::{
    order_as_key::OrderAsKey,
    utils::split_symmetric_difference,
};

/// Given the collections ofbids/asks to cancel and bids/asks to post, determine which orders would
/// be redundant and then filter them out from the set of resulting instructions.
///
/// That is, if an order would be canceled and then reposted, the cancel and post instruction are
/// both redundant and should be filtered out.
///
/// The bids and asks in the latest stored state might be stale due to fills.
/// This will cause the cancel order attempts to fail and should be expected intermittently.
pub fn get_non_redundant_order_flow(
    bids_to_cancel: Vec<OrderView>,
    asks_to_cancel: Vec<OrderView>,
    bids_to_post: Vec<(Decimal, u64)>, // (price, size) tuples.
    asks_to_post: Vec<(Decimal, u64)>, // (price, size) tuples.
    maker_seat_index: SectorIndex,
) -> anyhow::Result<(
    Vec<CancelOrderInstructionData>,
    Vec<PostOrderInstructionData>,
)> {
    // Map the existing maker's key-able order infos to their respective orders.
    // These will be the orders that are canceled.
    let bid_cancels = to_order_view_map(bids_to_cancel);
    let ask_cancels = to_order_view_map(asks_to_cancel);

    // Map the incoming (to-be-posted) key-able order infos to their respective order info args.
    let bid_posts = to_order_args_map(bids_to_post)?;
    let ask_posts = to_order_args_map(asks_to_post)?;

    // Retain only the unique values in two hash maps `a` and `b`, where each item in `a` does not
    // have a corresponding matching key in `b`.
    let (c_ask, p_ask, c_bid, p_bid) = (&ask_cancels, &ask_posts, &bid_cancels, &bid_posts);
    let (unique_bid_posts, unique_bid_cancels) = split_symmetric_difference(p_bid, c_bid);
    let (unique_ask_posts, unique_ask_cancels) = split_symmetric_difference(p_ask, c_ask);

    let cancels = unique_bid_cancels
        .iter()
        .map(|c| CancelOrderInstructionData::new(c.encoded_price, true, maker_seat_index))
        .chain(
            unique_ask_cancels
                .iter()
                .map(|c| CancelOrderInstructionData::new(c.encoded_price, false, maker_seat_index)),
        )
        .collect_vec();

    let posts = unique_bid_posts
        .iter()
        .map(|p| {
            PostOrderInstructionData::new(
                p.price_mantissa,
                p.base_scalar,
                p.base_exponent_biased,
                p.quote_exponent_biased,
                true,
                maker_seat_index,
            )
        })
        .chain(unique_ask_posts.iter().map(|p| {
            PostOrderInstructionData::new(
                p.price_mantissa,
                p.base_scalar,
                p.base_exponent_biased,
                p.quote_exponent_biased,
                false,
                maker_seat_index,
            )
        }))
        .collect_vec();

    Ok((cancels, posts))
}

pub fn to_order_args_map(
    prices_and_sizes: Vec<(Decimal, u64)>,
) -> anyhow::Result<HashMap<OrderAsKey, OrderInfoArgs>> {
    prices_and_sizes
        .into_iter()
        .map(|(price, size)| {
            let args = to_order_info_args(price, size)?;
            let order_info = to_order_info(args.clone())?;
            Ok((order_info.into(), args))
        })
        .collect()
}

pub fn to_order_view_map(orders: Vec<OrderView>) -> HashMap<OrderAsKey, OrderView> {
    orders
        .into_iter()
        .map(|order| (order.clone().into(), order))
        .collect()
}
