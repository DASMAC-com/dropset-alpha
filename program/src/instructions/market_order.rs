//! See [`process_market_order`].

use core::num::NonZeroU128;

// #[cfg(feature = "debug")]
use dropset_interface::events::MarketOrderEventInstructionData;
use dropset_interface::{
    error::DropsetError,
    instructions::MarketOrderInstructionData,
    state::{
        asks_dll::AskOrders,
        bids_dll::BidOrders,
        linked_list::LinkedListHeaderOperations,
        market_seat::MarketSeat,
        node::Node,
        sector::NIL,
    },
};
use pinocchio::{
    account_info::AccountInfo,
    hint::unlikely,
    program_error::ProgramError,
};

use crate::{
    context::{
        market_order_context::MarketOrderContext,
        EventBufferContext,
    },
    events::EventBuffer,
    shared::order_operations::{
        load_mut_order_from_sector_index,
        load_order_from_sector_index,
    },
};

/// Instruction handler logic for processing a market order.
///
/// # Safety
///
/// Caller guarantees the safety contract detailed in
/// [`dropset_interface::instructions::generated_pinocchio::MarketOrder`].
#[inline(never)]
pub unsafe fn process_market_order<'a>(
    accounts: &'a [AccountInfo],
    instruction_data: &[u8],
    _event_buffer: &mut EventBuffer,
) -> Result<EventBufferContext<'a>, ProgramError> {
    let MarketOrderInstructionData {
        order_size_base,
        is_buy,
    } = MarketOrderInstructionData::unpack_pinocchio(instruction_data)?;
    let mut ctx = MarketOrderContext::load(accounts)?;

    if order_size_base == 0 {
        return Err(DropsetError::AmountCannotBeZero.into());
    }

    let mut base_not_filled_yet = order_size_base;
    let mut total_quote_filled: u64 = 0;

    // Iterate over orders on the book, matching the market order to each posted order as long as
    // the market order is not fully filled.
    // Skip expensive muldiv operations by filling posted orders in whole. That is, as long as the
    // base atoms remaining in the market order exceed the amount in the next posted order, simply
    // close the order and decrement the remaining market order amount by the amount that was filled
    // in the order.
    while let Some((
        base_in_top_order,
        quote_in_top_order,
        encoded_price,
        maker_seat_sector_index,
        top_order_sector_index,
    )) = top_of_book_order_info(&ctx, is_buy)
    {
        // If there's no base left to fill, break from the loop. There's no last partial
        // order to fill.
        if unlikely(base_not_filled_yet == 0) {
            break;
        }
        // The order will only partially fill- perform the partial fill and break the loop.
        if base_not_filled_yet < base_in_top_order {
            // Safety: Orders with a base remaining of size zero can't exist.
            let base_in_top_order = NonZeroU128::new_unchecked(base_in_top_order as u128);
            let partial_quote_fill_amount = quote_filled_mul_div(
                base_not_filled_yet as u128,
                base_in_top_order,
                quote_in_top_order as u128,
            )?;
            total_quote_filled = total_quote_filled
                .checked_add(partial_quote_fill_amount)
                .ok_or(DropsetError::ArithmeticOverflow)?;

            // Update the partially filled order to reflect the new remaining base and
            // quote amounts after the partial fill.
            {
                // Safety: Scoped mutable borrow of the market account data.
                let market = unsafe { ctx.market_account.load_unchecked_mut() };
                // Safety: The order sector index is non-NIL and pointing to a valid order
                // node.
                let order =
                    unsafe { load_mut_order_from_sector_index(market, top_order_sector_index) };
                // Safety: The base not filled yet is less than the base in the top order.
                let new_base_remaining =
                    (base_in_top_order.get() as u64).unchecked_sub(base_not_filled_yet);
                // Safety: The partial quote fill amount can only be <= the total amount.
                let new_quote_remaining =
                    quote_in_top_order.unchecked_sub(partial_quote_fill_amount);
                order.set_base_remaining(new_base_remaining);
                order.set_quote_remaining(new_quote_remaining);
            }

            // Set the remaining base to zero; the market order was totally filled.
            base_not_filled_yet = 0;

            break;
        }

        // Fully fill the order by:
        // 1. Closing/removing it from the orders collection and
        // 2. Updating the filled maker seat's balance.
        // 3. Updating the local base not filled yet and the quote filled for the taker.

        // 1. Close/remove the order from the orders collection.
        // Safety: Singular, scoped, mutable borrows of market account data.
        unsafe {
            if is_buy {
                ctx.market_account
                    .load_unchecked_mut()
                    .asks()
                    .remove_at(top_order_sector_index);
            } else {
                ctx.market_account
                    .load_unchecked_mut()
                    .bids()
                    .remove_at(top_order_sector_index);
            }
        };

        // 2. Update the filled maker seat's balance.
        {
            // Safety: Single, scoped mutable borrow of the market account data.
            let market = ctx.market_account.load_unchecked_mut();
            // Safety: The user seat sector index is in-bounds, as it came from the
            // order.
            let node =
                unsafe { Node::from_sector_index_mut(market.sectors, maker_seat_sector_index) };
            let maker_seat = node.load_payload_mut::<MarketSeat>();
            if is_buy {
                // Market buy means a maker's ask got filled, so they receive quote.
                maker_seat.try_increment_quote_available(quote_in_top_order)?;
                maker_seat.user_order_sectors.asks.remove(encoded_price)?;
            } else {
                // Market sell means a maker's bid got filled, so they receive base.
                maker_seat.try_increment_base_available(base_in_top_order)?;
                maker_seat.user_order_sectors.bids.remove(encoded_price)?;
            }
        }

        // 3. Update the base not filled yet and the quote filled for the taker.
        // Safety: The base not yet filled is greater than base amount in the top order.
        base_not_filled_yet = base_not_filled_yet.unchecked_sub(base_in_top_order);
        total_quote_filled = total_quote_filled
            .checked_add(quote_in_top_order)
            .ok_or(DropsetError::ArithmeticOverflow)?;
    }

    let total_base_filled = order_size_base - base_not_filled_yet;

    /////////////// TODO:
    // transfer the funds from the market account to the taker! right now it just emits amounts.

    // #[cfg(feature = "debug")]
    _event_buffer.add_to_buffer(
        MarketOrderEventInstructionData::new(
            order_size_base,
            is_buy,
            total_base_filled,
            total_quote_filled,
        ),
        ctx.event_authority,
        ctx.market_account.clone(),
    )?;

    Ok(EventBufferContext {
        event_authority: ctx.event_authority,
        market_account: ctx.market_account,
    })
}

/// Returns an optional tuple of
/// (`base_remaining, quote_remaining, encoded_price, user_seat_sector_index, order_sector_index`)
/// as (u64, u32, u32, u32) for the top of book order.
///
/// Returns `None` if there is no order at the top of the book for the respective side.
#[inline(always)]
fn top_of_book_order_info(
    ctx: &'_ MarketOrderContext,
    is_buy: bool,
) -> Option<(u64, u64, u32, u32, u32)> {
    // Safety: Scoped borrow of the market account data to check the top of book.
    let market = unsafe { ctx.market_account.load_unchecked() };
    let head_index = if is_buy {
        AskOrders::head(market.header)
    } else {
        BidOrders::head(market.header)
    };
    if head_index == NIL {
        None
    } else {
        // Safety: The head index is a non-NIL sector index pointing to a valid order node.
        let order = unsafe { load_order_from_sector_index(market, head_index) };
        Some((
            order.base_remaining(),
            order.quote_remaining(),
            order.encoded_price(),
            order.user_seat(),
            head_index,
        ))
    }
}

#[inline(always)]
fn quote_filled_mul_div(
    base_not_filled_yet: u128,
    base_remaining_in_order: core::num::NonZeroU128,
    quote_remaining_in_order: u128,
) -> Result<u64, DropsetError> {
    let intermediate_result = price::checked_mul!(
        base_not_filled_yet,
        quote_remaining_in_order,
        DropsetError::ArithmeticOverflow
    )?;
    let res = intermediate_result / base_remaining_in_order.get();
    if res > u64::MAX as u128 {
        return Err(DropsetError::ArithmeticOverflow);
    }

    Ok(res as u64)
}
