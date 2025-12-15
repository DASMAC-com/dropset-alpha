//! Doubly linked list utilities for traversing, inserting, and removing nodes containing
//! [`crate::state::order::Order`] payloads.

use crate::state::{
    linked_list::{
        LinkedList,
        LinkedListOperations,
    },
    market_header::MarketHeader,
    sector::SectorIndex,
};

pub struct Orders;

pub type OrdersLinkedList<'a> = LinkedList<'a, Orders>;

/// Operations for the sorted, doubly linked list of nodes containing
/// [`crate::state::order::Order`] payloads.
impl LinkedListOperations for Orders {
    fn head(header: &MarketHeader) -> SectorIndex {
        header.orders_dll_head()
    }

    fn set_head(header: &mut MarketHeader, new_index: SectorIndex) {
        header.set_orders_dll_head(new_index);
    }

    fn tail(header: &MarketHeader) -> SectorIndex {
        header.orders_dll_tail()
    }

    fn set_tail(header: &mut MarketHeader, new_index: SectorIndex) {
        header.set_orders_dll_tail(new_index);
    }

    fn increment_num_nodes(header: &mut MarketHeader) {
        header.increment_num_orders();
    }

    fn decrement_num_nodes(header: &mut MarketHeader) {
        header.decrement_num_orders();
    }
}
