use dropset_interface::instructions::DropsetInstruction;
use instruction_macros::codama::CodamaProgram;

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

    let root = DropsetInstruction::codama_root("dropset", &PROGRAM_ID.to_string());
    let json = serde_json::to_string_pretty(&root).expect("Failed to serialize IDL");

    std::fs::write(&out, &json).expect("Failed to write IDL file");
    println!("Wrote Codama IDL to {}", out.display());
}
