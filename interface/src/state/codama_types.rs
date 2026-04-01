use _alloc::vec::Vec;
use instruction_macros::codama::*;

use crate::state::{
    market_header::{
        MarketHeader,
        MARKET_ACCOUNT_DISCRIMINANT,
    },
    market_seat::MarketSeat,
    order::{
        Order,
        ORDER_PADDING,
    },
    sector::{
        Sector,
        PAYLOAD_SIZE,
    },
    user_order_sectors::{
        OrderSectors,
        PriceToIndexEntry,
        UserOrderSectors,
        MAX_ORDERS_USIZE,
    },
};

impl CodamaType for MarketSeat {
    fn name() -> &'static str {
        "marketSeat"
    }

    fn type_node() -> TypeNode {
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::address("user"),
                StructFieldTypeNode::u64("baseAvailable"),
                StructFieldTypeNode::u64("quoteAvailable"),
                StructFieldTypeNode::new(
                    "userOrderSectors",
                    <UserOrderSectors as CodamaType>::type_node(),
                ),
            ]),
        })
    }
}

impl CodamaType for UserOrderSectors {
    fn name() -> &'static str {
        "userOrderSectors"
    }

    fn type_node() -> TypeNode {
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::new("bids", OrderSectors::type_node()),
                StructFieldTypeNode::new("asks", OrderSectors::type_node()),
            ]),
        })
    }
}

impl CodamaType for OrderSectors {
    fn name() -> &'static str {
        "orderSectors"
    }

    fn type_node() -> TypeNode {
        array_type(PriceToIndexEntry::type_node(), MAX_ORDERS_USIZE)
    }
}

impl CodamaType for PriceToIndexEntry {
    fn name() -> &'static str {
        "priceToIndexEntry"
    }

    fn type_node() -> TypeNode {
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::u32("encodedPrice"),
                StructFieldTypeNode::u32("sectorIndex"),
            ]),
        })
    }
}

impl CodamaType for Order {
    fn name() -> &'static str {
        "order"
    }

    fn type_node() -> TypeNode {
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::u32("encodedPrice"),
                StructFieldTypeNode::u32("userSeatIndex"),
                StructFieldTypeNode::u64("baseRemaining"),
                StructFieldTypeNode::u64("quoteRemaining"),
                StructFieldTypeNode::array::<u8>("_padding", ORDER_PADDING),
            ]),
        })
    }
}

impl CodamaType for Sector {
    fn name() -> &'static str {
        "sector"
    }

    fn type_node() -> TypeNode {
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::u32("next"),
                StructFieldTypeNode::u32("prev"),
                StructFieldTypeNode::array::<u8>("payload", PAYLOAD_SIZE),
            ]),
        })
    }
}

impl CodamaType for MarketHeader {
    fn name() -> &'static str {
        "marketHeader"
    }

    fn type_node() -> TypeNode {
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::u64("discriminant"),
                StructFieldTypeNode::u32("numSeats"),
                StructFieldTypeNode::u32("numBids"),
                StructFieldTypeNode::u32("numAsks"),
                StructFieldTypeNode::u32("numFreeSectors"),
                StructFieldTypeNode::u32("freeStackTop"),
                StructFieldTypeNode::u32("seatsDllHead"),
                StructFieldTypeNode::u32("seatsDllTail"),
                StructFieldTypeNode::u32("bidsDllHead"),
                StructFieldTypeNode::u32("bidsDllTail"),
                StructFieldTypeNode::u32("asksDllHead"),
                StructFieldTypeNode::u32("asksDllTail"),
                StructFieldTypeNode::address("baseMint"),
                StructFieldTypeNode::address("quoteMint"),
                StructFieldTypeNode::u8("marketBump"),
                StructFieldTypeNode::u64("numEvents"),
                StructFieldTypeNode::array::<u8>("_padding", 3),
            ]),
        })
    }
}

/// Returns the [`AccountNode`] describing the full market account layout:
/// a fixed [`MarketHeader`] followed by remainder bytes (sectors).
pub fn dropset_market_account_node() -> AccountNode {
    let mut account = AccountNode::new(
        "marketAccount",
        TypeNode::Struct(StructTypeNode {
            fields: Vec::from([
                StructFieldTypeNode::new("header", MarketHeader::type_node()),
                StructFieldTypeNode::bytes("sectors"),
            ]),
        }),
    );
    account.discriminators = Vec::from([discriminator(
        NumberFormat::U64,
        MARKET_ACCOUNT_DISCRIMINANT,
    )]);
    account
}
