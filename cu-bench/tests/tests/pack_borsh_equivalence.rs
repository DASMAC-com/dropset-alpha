use borsh::BorshDeserialize;
use cu_bench_pack_orders::{
    BorshBatchReplaceData,
    BorshUnvalidatedOrders,
};
use dropset_interface::instructions::{
    BatchReplaceInstructionData,
    UnvalidatedOrders,
};
use instruction_macros_traits::{
    Pack,
    Unpack,
};
use price::OrderInfoArgs;

#[test]
fn pack_borsh_round_trip_equivalence() {
    // Create test data using Pack
    let pack_data = BatchReplaceInstructionData::new(
        42,
        UnvalidatedOrders::new([OrderInfoArgs::new_unscaled(11_000_000, 1)]),
        UnvalidatedOrders::new([
            OrderInfoArgs::new_unscaled(12_000_000, 1),
            OrderInfoArgs::new_unscaled(13_000_000, 2),
            OrderInfoArgs::new_unscaled(14_000_000, 3),
            OrderInfoArgs::new_unscaled(15_000_000, 4),
            OrderInfoArgs::new_unscaled(16_000_000, 5),
        ]),
    );

    // Serialize with Pack
    let packed_bytes = pack_data.pack();

    // Create equivalent Borsh data
    let borsh_data = BorshBatchReplaceData {
        user_sector_index_hint: 42,
        new_bids: BorshUnvalidatedOrders {
            order_args: [
                OrderInfoArgs::new_unscaled(11_000_000, 1).into(),
                OrderInfoArgs::new(0, 0, 0, 0).into(),
                OrderInfoArgs::new(0, 0, 0, 0).into(),
                OrderInfoArgs::new(0, 0, 0, 0).into(),
                OrderInfoArgs::new(0, 0, 0, 0).into(),
            ],
        },
        new_asks: BorshUnvalidatedOrders {
            order_args: [
                OrderInfoArgs::new_unscaled(12_000_000, 1).into(),
                OrderInfoArgs::new_unscaled(13_000_000, 2).into(),
                OrderInfoArgs::new_unscaled(14_000_000, 3).into(),
                OrderInfoArgs::new_unscaled(15_000_000, 4).into(),
                OrderInfoArgs::new_unscaled(16_000_000, 5).into(),
            ],
        },
    };

    // Serialize with Borsh
    let borsh_bytes = borsh::to_vec(&borsh_data).expect("Borsh serialization failed");

    // Compare byte representations
    assert_eq!(
        packed_bytes.as_ref(),
        borsh_bytes.as_slice(),
        "Pack and Borsh serialization should produce identical bytes"
    );

    // Round-trip: deserialize Pack bytes with Borsh
    let borsh_deserialized = BorshBatchReplaceData::try_from_slice(packed_bytes.as_ref())
        .expect("Borsh deserialization of Pack bytes failed");

    assert_eq!(
        borsh_deserialized, borsh_data,
        "Deserializing Pack bytes with Borsh should produce equivalent data"
    );

    // Round-trip: deserialize Borsh bytes with Pack
    let pack_deserialized = BatchReplaceInstructionData::unpack(&borsh_bytes)
        .expect("Pack deserialization of Borsh bytes failed");

    assert_eq!(pack_deserialized.pack().as_ref(), borsh_bytes.as_slice());
}
