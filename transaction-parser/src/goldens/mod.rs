use std::{
    collections::HashMap,
    path::PathBuf,
    sync::LazyLock,
};

use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta,
    UiLoadedAddresses,
    UiTransactionTokenBalance,
};

use crate::client_rpc::{
    ParsedBalances,
    ParsedTransaction,
};

pub fn goldens_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("transaction-parser/src/goldens")
}

pub fn golden_encoded_metas() -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
    static GOLDEN_JSON_STRINGS: LazyLock<Vec<String>> = LazyLock::new(|| {
        let dir = goldens_dir();
        let mut paths: Vec<_> = std::fs::read_dir(&dir)
            .expect("Should read goldens directory")
            .map(|entry| entry.expect("Should read directory entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        assert!(
            !paths.is_empty(),
            "No golden JSON fixtures found in {dir:?}"
        );

        paths
            .iter()
            .map(|path| {
                std::fs::read_to_string(path).expect("Should read the JSON file as a string")
            })
            .collect()
    });

    GOLDEN_JSON_STRINGS
        .iter()
        .map(|json| {
            serde_json::from_str(json).expect("Should deserialize encoded transaction meta")
        })
        .collect()
}

pub fn golden_parsed_transactions() -> &'static HashMap<String, ParsedTransaction> {
    static PARSED_GOLDEN_TRANSACTIONS: LazyLock<HashMap<String, ParsedTransaction>> =
        LazyLock::new(|| {
            golden_encoded_metas()
                .into_iter()
                .map(ParsedTransaction::from_encoded_transaction)
                .collect::<anyhow::Result<Vec<_>>>()
                .expect("Should parse")
                .into_iter()
                .map(|txn| (txn.signature.to_string(), txn))
                .collect()
        });

    &PARSED_GOLDEN_TRANSACTIONS
}

pub fn golden_parsed_balances() -> &'static HashMap<String, ParsedBalances> {
    static PARSED_GOLDEN_BALANCES: LazyLock<HashMap<String, ParsedBalances>> =
        LazyLock::new(|| {
            let goldens = golden_parsed_transactions();
            goldens
                .iter()
                .map(|(sig, txn)| {
                    (
                        sig.clone(),
                        ParsedBalances::try_from(txn).expect("Should parse balances"),
                    )
                })
                .collect()
        });

    &PARSED_GOLDEN_BALANCES
}

pub fn has_loaded_addresses(meta: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
    if let Some(ref tx_meta) = meta.transaction.meta {
        let maybe_loaded = Option::<UiLoadedAddresses>::from(tx_meta.loaded_addresses.clone());
        if let Some(loaded) = maybe_loaded {
            !loaded.readonly.is_empty() || !loaded.writable.is_empty()
        } else {
            false
        }
    } else {
        panic!("Fixture should have transaction meta");
    }
}

pub fn load_golden_with_loaded_addresses(sig: &str) -> EncodedConfirmedTransactionWithStatusMeta {
    let path = goldens_dir()
        .join("with_loaded_addresses")
        .join(format!("{sig}.json"));
    let json = std::fs::read_to_string(&path).expect("Should read golden fixture");
    let meta: EncodedConfirmedTransactionWithStatusMeta =
        serde_json::from_str(&json).expect("Should deserialize");

    let msg = format!("Loaded address fixture at ({path:?}) doesn't have loaded addresses");
    assert!(has_loaded_addresses(&meta), "{msg}");

    meta
}

/// This helper function is useful because [EncodedConfirmedTransactionWithStatusMeta] is not
/// [Clone] and it typically needs to be an owned value.
pub fn load_golden_meta_with_sig(sig: &str) -> EncodedConfirmedTransactionWithStatusMeta {
    let path = goldens_dir().join(format!("{sig}.json"));
    let json = std::fs::read_to_string(&path).expect("Should read golden fixture");
    serde_json::from_str(&json).expect("Should deserialize")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_goldens() {
        golden_encoded_metas();
        golden_parsed_transactions();
        golden_parsed_balances();
    }

    #[test]
    fn load_goldens() {
        let goldens = golden_parsed_transactions();

        let sig = "5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN";
        let txn = goldens
            .get(sig)
            .expect("Should find txn with matching signature");

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
}
