use std::{
    path::PathBuf,
    sync::LazyLock,
};

use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta,
    UiTransactionTokenBalance,
};

use crate::client_rpc::ParsedTransaction;

fn goldens_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("transaction-parser/src/goldens")
}

fn golden_encoded_metas() -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
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

pub fn golden_parsed_transactions() -> &'static [ParsedTransaction] {
    static PARSED_GOLDENS: LazyLock<Vec<ParsedTransaction>> = LazyLock::new(|| {
        golden_encoded_metas()
            .into_iter()
            .map(ParsedTransaction::from_encoded_transaction)
            .collect::<anyhow::Result<Vec<_>>>()
            .expect("Should parse")
    });

    &PARSED_GOLDENS
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_goldens() {
        let _ = golden_encoded_metas();
    }

    #[test]
    fn load_goldens() {
        let goldens = golden_parsed_transactions();

        let txn = goldens.iter().find(|v| v.signature.to_string() == "5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN").unwrap();

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
