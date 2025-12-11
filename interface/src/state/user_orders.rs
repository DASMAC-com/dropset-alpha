use price::{
    EncodedPrice,
    LeEncodedPrice,
};
use static_assertions::const_assert_eq;

use crate::{
    error::{
        DropsetError,
        DropsetResult,
    },
    state::{
        sector::{
            LeSectorIndex,
            SectorIndex,
            LE_NIL,
        },
        transmutable::Transmutable,
    },
};

/// The max number of orders a single user/address can have for a single market.
const MAX_ORDERS: u8 = 10;

/// A lookup structure that indexes all of the user's unique order prices to the sector index of the
/// corresponding order.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserOrderSectors {
    pub orders: [PriceIndexNode; MAX_ORDERS as usize],
}

impl Default for UserOrderSectors {
    fn default() -> Self {
        Self::new()
    }
}

impl UserOrderSectors {
    /// Creates a new collection of user orders to sector indices with all freed nodes.
    pub fn new() -> Self {
        Self {
            orders: [
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
            ],
        }
    }

    /// Create a new collection of user orders to sector indices with only one valid price to index
    /// node and [`MAX_ORDERS`] - 1 freed nodes.
    pub fn new_from_order(encoded_price: EncodedPrice, sector_index: &SectorIndex) -> Self {
        Self {
            orders: [
                PriceIndexNode::new(encoded_price, sector_index),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
                PriceIndexNode::new_free(),
            ],
        }
    }

    /// Attempt to retrieve the sector index of an encoded price in a user's orders.
    #[inline(always)]
    pub fn get(&self, target_le_encoded_price: &LeEncodedPrice) -> Option<SectorIndex> {
        // Compare each price against the encoded price passed in.
        // Returns Some(sector_index) if it exists, otherwise None.
        self.orders.iter().find_map(
            |PriceIndexNode {
                 le_encoded_price,
                 le_sector_index,
             }| {
                match le_encoded_price.get() == target_le_encoded_price.get() {
                    true => Some(u32::from_le_bytes(*le_sector_index)),
                    false => None,
                }
            },
        )
    }

    /// Fallibly add a `PriceAndSectorIndex` to a user's orders.
    ///
    /// Fails if the user already has [`MAX_ORDERS`] or the price already has an existing order.
    ///
    /// The `sector_index` passed to this method should be non-NIL or the node after mutation will
    /// continue be treated as a free node.
    #[inline(always)]
    pub fn add(
        &mut self,
        target_le_encoded_price: &LeEncodedPrice,
        le_sector_index: &LeSectorIndex,
    ) -> DropsetResult {
        // Check if the price already exists in a node and fail early if it does.
        if self
            .orders
            .iter()
            .any(|node| node.le_encoded_price.get() == target_le_encoded_price.get())
        {
            return Err(DropsetError::OrderWithPriceAlreadyExists);
        }

        let node = self
            .orders
            .iter_mut()
            .find(|node| node.is_free())
            .ok_or(DropsetError::UserHasMaxOrders)?;

        node.le_encoded_price = *target_le_encoded_price;
        node.le_sector_index = *le_sector_index;

        Ok(())
    }

    /// Fallibly remove a `PriceAndSectorIndex` from a user's orders.
    ///
    /// Fails if the user does not have an order corresponding to the passed encoded price.
    #[inline(always)]
    pub fn remove(&mut self, le_encoded_price: &LeEncodedPrice) -> DropsetResult {
        let node = self
            .orders
            .iter_mut()
            .find(|node| node.le_encoded_price.get() == le_encoded_price.get())
            .ok_or(DropsetError::OrderNotFound)?;

        node.le_encoded_price = LeEncodedPrice::zero();
        node.le_sector_index = LE_NIL;

        Ok(())
    }
}

/// The paired encoded price and sector index for an order.
///
/// If the sector index equals [`NIL`], it's considered a freed node, otherwise, it contains an
/// existing, valid pair of encoded price to sector index.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceIndexNode {
    pub le_encoded_price: LeEncodedPrice,
    pub le_sector_index: LeSectorIndex,
}

impl PriceIndexNode {
    /// Create a new free node.
    #[inline(always)]
    pub fn new_free() -> Self {
        Self {
            le_encoded_price: LeEncodedPrice::zero(),
            le_sector_index: LE_NIL,
        }
    }

    /// Create a new encoded price to sector index node.
    #[inline(always)]
    pub fn new(encoded_price: EncodedPrice, sector_index: &SectorIndex) -> Self {
        Self {
            le_encoded_price: encoded_price.into(),
            le_sector_index: sector_index.to_le_bytes(),
        }
    }

    #[inline(always)]
    pub fn is_free(&self) -> bool {
        self.le_sector_index == LE_NIL
    }
}

// Safety:
//
// - Stable layout with `#[repr(C)]`.
// - `size_of` and `align_of` are checked below.
// - All bit patterns are valid.
unsafe impl Transmutable for UserOrderSectors {
    const LEN: usize = size_of::<PriceIndexNode>() * MAX_ORDERS as usize;

    #[inline(always)]
    fn validate_bit_patterns(_bytes: &[u8]) -> crate::error::DropsetResult {
        // All bit patterns are valid.
        Ok(())
    }
}

const_assert_eq!(UserOrderSectors::LEN, size_of::<UserOrderSectors>());
const_assert_eq!(align_of::<UserOrderSectors>(), 1);

// Safety:
//
// - Stable layout with `#[repr(C)]`.
// - `size_of` and `align_of` are checked below.
// - All bit patterns are valid.
unsafe impl Transmutable for PriceIndexNode {
    const LEN: usize = size_of::<PriceIndexNode>();

    #[inline(always)]
    fn validate_bit_patterns(_bytes: &[u8]) -> crate::error::DropsetResult {
        // All bit patterns are valid.
        Ok(())
    }
}

const_assert_eq!(PriceIndexNode::LEN, size_of::<PriceIndexNode>());
const_assert_eq!(align_of::<PriceIndexNode>(), 1);
