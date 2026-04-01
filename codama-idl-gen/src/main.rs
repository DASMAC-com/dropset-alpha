use dropset_interface::{
    instructions::DropsetInstruction,
    state::{
        codama_types::dropset_market_account_node,
        market_header::MarketHeader,
        market_seat::MarketSeat,
        order::Order,
        sector::Sector,
        user_order_sectors::{
            OrderSectors,
            PriceToIndexEntry,
            UserOrderSectors,
        },
    },
};
use instruction_macros::codama::{
    CodamaProgram,
    CodamaType,
};

const PROGRAM_ID: solana_address::Address = dropset_interface::program::ID;

/// Serializes the Codama IDL for the dropset program to a JSON file.
///
/// - Relative paths are resolved from the workspace root.
/// - Absolute paths are used as-is.
///
/// ```sh
/// # Default output path:
/// # <workspace>/codama-idl-gen/idl.json
/// cargo run -p codama-idl-gen
///
/// # Relative output path (from workspace root):
/// # <workspace>/my-idl.json
/// cargo run -p codama-idl-gen -- my-idl.json
///
/// # Absolute output path:
/// # /tmp/idl.json
/// cargo run -p codama-idl-gen -- /tmp/idl.json
/// ```
fn main() {
    let workspace_dir = std::path::PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
    let out = match std::env::args().nth(1) {
        Some(arg) => {
            let path = std::path::PathBuf::from(&arg);
            if path.is_absolute() {
                path
            } else {
                workspace_dir.join(path)
            }
        }
        None => workspace_dir.join("codama-idl-gen/idl.json"),
    };

    let mut root = DropsetInstruction::codama_root("dropset", &PROGRAM_ID.to_string());

    // Add the manual implementations.
    root.program.defined_types.extend([
        MarketSeat::defined_type_node(),
        UserOrderSectors::defined_type_node(),
        OrderSectors::defined_type_node(),
        PriceToIndexEntry::defined_type_node(),
        Order::defined_type_node(),
        Sector::defined_type_node(),
        MarketHeader::defined_type_node(),
    ]);
    root.program.accounts = vec![dropset_market_account_node()];

    let json = serde_json::to_string_pretty(&root).expect("Failed to serialize IDL");
    std::fs::write(&out, &json).expect("Failed to write IDL file");
    println!("Wrote Codama IDL to {}", out.display());
}
