use std::path::PathBuf;

use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransaction,
    TransactionBinaryEncoding,
    UiMessage,
    UiTransactionTokenBalance,
};

pub fn goldens_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("transaction-parser/src/goldens")
}

/// Load a golden fixture by signature, using the default `json` encoding.
///
/// For a different encoding (e.g., `"base64"`, `"base58"`, `"jsonParsed"`),
/// use [load_golden_encoding] instead.
pub fn load_golden(sig: &str) -> EncodedConfirmedTransactionWithStatusMeta {
    load_golden_encoding(sig, "json")
}

/// Load a golden fixture that has full (non-truncated) logs.
pub fn load_golden_full_logs(sig: &str) -> EncodedConfirmedTransactionWithStatusMeta {
    load_golden_encoding(sig, "json_full_logs")
}

/// Load a golden fixture for a specific encoding variant (e.g., "json", "base64", "base58").
/// These live in a subdirectory named after the transaction signature.
pub fn load_golden_encoding(
    sig: &str,
    encoding: &str,
) -> EncodedConfirmedTransactionWithStatusMeta {
    let path = goldens_dir().join(sig).join(format!("{encoding}.json"));
    let json = std::fs::read_to_string(&path).expect("Should read golden fixture");
    let value: serde_json::Value = serde_json::from_str(&json).expect("Should parse JSON");
    serde_json::from_value(value).expect("Should deserialize")
}

/// The supported RPC transaction encoding names, mapped to how they appear in deserialized
/// [EncodedTransaction] variants.
///
/// - `"json"` → [EncodedTransaction::Json] with [UiMessage::Raw]
/// - `"jsonParsed"` → [EncodedTransaction::Json] with [UiMessage::Parsed]
/// - `"base64"` → [EncodedTransaction::Binary](_, [TransactionBinaryEncoding::Base64])
/// - `"base58"` → [EncodedTransaction::Binary](_, [TransactionBinaryEncoding::Base58])
pub const SUPPORTED_ENCODINGS: &[&str] = &["json", "jsonParsed", "base64", "base58"];

/// Returns the encoding name for a deserialized [EncodedConfirmedTransactionWithStatusMeta].
pub fn detect_encoding(meta: &EncodedConfirmedTransactionWithStatusMeta) -> &'static str {
    match &meta.transaction.transaction {
        EncodedTransaction::Json(ui_txn) => match &ui_txn.message {
            UiMessage::Raw(_) => "json",
            UiMessage::Parsed(_) => "jsonParsed",
        },
        EncodedTransaction::Binary(_, TransactionBinaryEncoding::Base64) => "base64",
        EncodedTransaction::Binary(_, TransactionBinaryEncoding::Base58)
        | EncodedTransaction::LegacyBinary(_) => "base58",
        EncodedTransaction::Accounts(_) => "accounts",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::client_rpc::{
        ParsedBalances,
        ParsedTransaction,
    };

    #[test]
    fn deserialize_all_goldens() {
        let dir = goldens_dir();
        let mut paths: Vec<_> = std::fs::read_dir(&dir)
            .expect("Should read goldens directory")
            .map(|entry| entry.expect("Should read directory entry").path())
            .filter(|path| path.is_dir())
            .map(|dir| dir.join("json.json"))
            .collect();
        paths.sort();
        assert!(!paths.is_empty(), "No golden directories found in {dir:?}");

        for path in &paths {
            let json =
                std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Should read {path:?}"));
            let value: serde_json::Value = serde_json::from_str(&json).expect("Should parse JSON");
            let encoded: EncodedConfirmedTransactionWithStatusMeta =
                serde_json::from_value(value).expect("Should deserialize");
            let txn = ParsedTransaction::from_encoded_transaction(encoded)
                .expect("Should parse transaction");
            ParsedBalances::try_from(&txn).expect("Should parse balances");
        }
    }

    #[test]
    fn load_goldens() {
        let sig = "5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN";
        let encoded = load_golden(sig);
        let txn =
            ParsedTransaction::from_encoded_transaction(encoded).expect("Should parse transaction");

        assert_eq!(txn.pre_token_balances.len(), 1);
        assert_eq!(txn.post_token_balances.len(), 1);

        assert_eq!(
            txn.pre_token_balances[0],
            UiTransactionTokenBalance {
                account_index: 3,
                mint: "EmgRPxqMnGFFYTqRY6L6gVRNyzaadjHhkTDB4Jq8h9kV".into(),
                ui_token_amount: UiTokenAmount {
                    ui_amount: None,
                    decimals: 8,
                    amount: "0".into(),
                    ui_amount_string: "0".into(),
                },
                owner: OptionSerializer::Some("11113MwGAy1Aq8qkfPuukq892Zn3tV6uGHWoRYLaUBS".into()),
                program_id: OptionSerializer::Some(
                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".into()
                ),
            }
        );

        assert_eq!(
            txn.post_token_balances[0],
            UiTransactionTokenBalance {
                account_index: 3,
                mint: "EmgRPxqMnGFFYTqRY6L6gVRNyzaadjHhkTDB4Jq8h9kV".into(),
                ui_token_amount: UiTokenAmount {
                    ui_amount: Some(0.0001),
                    decimals: 8,
                    amount: "10000".into(),
                    ui_amount_string: "0.0001".into(),
                },
                owner: OptionSerializer::Some("11113MwGAy1Aq8qkfPuukq892Zn3tV6uGHWoRYLaUBS".into()),
                program_id: OptionSerializer::Some(
                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".into()
                ),
            }
        );
    }

    /// Walks all subdirectories in `goldens/` that contain encoding-named JSON files
    /// (e.g., `json.json`, `base64.json`) and verifies each file deserializes to the
    /// encoding indicated by its filename.
    #[test]
    fn encoding_fixtures_match_filenames() {
        let dir = goldens_dir();
        let mut checked = 0;

        for entry in std::fs::read_dir(&dir).expect("Should read goldens directory") {
            let entry = entry.expect("Should read entry");
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            for file in std::fs::read_dir(&path).expect("Should read subdirectory") {
                let file = file.expect("Should read file entry");
                let file_path = file.path();

                let stem = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .expect("Should have file stem");

                if !SUPPORTED_ENCODINGS.contains(&stem) {
                    continue;
                }

                let json = std::fs::read_to_string(&file_path).expect("Should read fixture");
                let value: serde_json::Value =
                    serde_json::from_str(&json).expect("Should parse JSON");
                let meta: EncodedConfirmedTransactionWithStatusMeta =
                    serde_json::from_value(value).expect("Should deserialize");

                let detected = detect_encoding(&meta);
                assert_eq!(
                    detected, stem,
                    "Fixture {file_path:?} has encoding {detected:?} but filename says {stem:?}"
                );
                checked += 1;
            }
        }

        assert!(
            checked > 0,
            "Should have checked at least one encoding fixture"
        );
    }
}
