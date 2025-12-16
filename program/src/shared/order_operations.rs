//! Core logic for manipulating and traversing [`Order`]s in the [`OrdersLinkedList`].

use dropset_interface::{
    error::DropsetError,
    state::{
        market::{
            Market,
            MarketRef,
            MarketRefMut,
        },
        market_header::MarketHeader,
        node::Node,
        order::Order,
        orders_dll::OrdersLinkedList,
        sector::{
            SectorIndex,
            NIL,
            SECTOR_SIZE,
        },
        transmutable::Transmutable,
    },
};
use pinocchio::pubkey::{
    pubkey_eq,
    Pubkey,
};
use price::EncodedPrice;

pub fn insert_order(
    list: &mut OrdersLinkedList,
    order: Order,
) -> Result<SectorIndex, DropsetError> {
}

fn find_bid_insert_before_index(
    list: &OrdersLinkedList,
    encoded_order_price: &EncodedPrice,
) -> (SectorIndex, SectorIndex) {
    for (index, node) in list.iter() {
        let order = node.load_payload::<Order>();
    }
    (0, 0)
}
