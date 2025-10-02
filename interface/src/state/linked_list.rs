use crate::{
    error::DropsetError,
    state::{
        free_stack::Stack,
        market_header::MarketHeader,
        node::{Node, NODE_PAYLOAD_SIZE},
        sector::NonNilSectorIndex,
    },
};

/// A sorted, doubly linked list.
pub struct LinkedList<'a> {
    header: &'a mut MarketHeader,
    sectors: &'a mut [u8],
}

impl<'a> LinkedList<'a> {
    pub fn new_from_parts(header: &'a mut MarketHeader, sectors: &'a mut [u8]) -> Self {
        LinkedList { header, sectors }
    }

    pub fn insert_before(
        &mut self,
        // The sector index of the node to insert a new node before.
        next_index: NonNilSectorIndex,
        payload: &[u8; NODE_PAYLOAD_SIZE],
    ) -> Result<NonNilSectorIndex, DropsetError> {
        // Allocate a new node from the free stack.
        let mut free_stack =
            Stack::new_from_parts(self.header.free_stack_top_mut_ref(), self.sectors);
        let new_index = free_stack.remove_free_node()?;

        // Store the next node's `prev` index.
        let next_node = Node::from_non_nil_sector_index_mut(self.sectors, next_index)?;
        let prev_index = next_node.prev();
        // Set `next_node`'s `prev` to the new node.
        next_node.set_prev(new_index.get());

        // Create the new node.
        let new_node = Node::from_non_nil_sector_index_mut(self.sectors, new_index)?;
        new_node.set_prev(prev_index);
        new_node.set_next(next_index.get());
        new_node.set_payload(payload);

        if let Ok(prev_index) = NonNilSectorIndex::new(prev_index) {
            // If `prev_index` is non-NIL, set it's `next` to the new index.
            Node::from_non_nil_sector_index_mut(self.sectors, prev_index)?
                .set_next(new_index.get());
        } else {
            // If `prev_index` is NIL, that means `next_index` was the head prior to this insertion,
            // and the head needs to be updated to the new node's index.
            self.header.set_seat_dll_head(new_index.get());
        }

        self.header.increment_num_seats();

        Ok(new_index)
    }

    pub fn insert_after(
        &mut self,
        // The sector index of the node to insert a new node after.
        prev_index: NonNilSectorIndex,
        payload: &[u8; NODE_PAYLOAD_SIZE],
    ) -> Result<NonNilSectorIndex, DropsetError> {
        // Allocate a new node from the free stack.
        let mut free_stack =
            Stack::new_from_parts(self.header.free_stack_top_mut_ref(), self.sectors);
        let new_index = free_stack.remove_free_node()?;

        // Store the previous node's `next` index.
        let prev_node = Node::from_non_nil_sector_index_mut(self.sectors, prev_index)?;
        let next_index = prev_node.next();
        // Set `prev_node`'s `next` to the new node.
        prev_node.set_next(new_index.get());

        // Create the new node.
        let new_node = Node::from_non_nil_sector_index_mut(self.sectors, new_index)?;
        new_node.set_prev(prev_index.get());
        new_node.set_next(next_index);
        new_node.set_payload(payload);

        if let Ok(next_index) = NonNilSectorIndex::new(next_index) {
            // If `next_index` is non-NIL, set its `prev` to the new index.
            Node::from_non_nil_sector_index_mut(self.sectors, next_index)?
                .set_prev(new_index.get());
        } else {
            // If `next_index` is NIL, then `prev_index` was the tail prior to this insertion, and
            // the head needs to be updated to the new node's index.
            self.header.set_seat_dll_tail(new_index.get());
        }

        self.header.increment_num_seats();

        Ok(new_index)
    }
}
