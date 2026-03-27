use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta,
    UiTransactionTokenBalance,
};

use crate::client_rpc::ParsedTransaction;

const GOLDENS: [&str; 25] = [
    "2aJ7c3wVZvuJiv9KNeaTwfdpCsoJJP2JVrU596BHVPYVMrADC843yPFEG1U1wo2aGnMExs9WocUqKqc6KRgrNv4q.json",
    "2BkwzZzZeVnV9cwoQYcvxWgAihvfmQf1sJTPKLuqLSyn48nngzjTmkuEwtPHpxQQNac5nZJvuQH86fcCsqLwfNDe.json",
    "2idyp3qsCFet76TMC2LGx3MQpu5455SSQuyASmQRKdCrCtgswxygjqhYVoh2WFQZqXNTD6QTTX43Tzkhcqw4S93b.json",
    "2KctbVWuZyZfkdyBKRs5uY5GSVxa62VBgWiG6ZBuKp52CjTfpZkgok17rHYSCp5FiyzPWde5u4YqoQrWPWVJ3xKY.json",
    "2nz31s4g3FR41eU3k11C9Co9qe295gqyEJJMZX6DX2H1dcrDKcS1iNHJpfJ9P3hrAy1WeSDtX4f1VN89tQzLSFvS.json",
    "2q85dFd4AHyCnpdaaK4ecs9nDT7if83Za5ejA3mvH7cDQCBWSKvw8YrZdCokvN9zSVHJJ65BSbLySoZgKogPyujx.json",
    "2Ucpd2j8EcZznqkwVdBv7ZCFdQtFx5GhnKw3TJca8Pa8cadcwfQBqJZQMtvZMxGsSQL8vvPmyd1cWKAQHPkA8nWS.json",
    "2yLyvWsrG3r4Un58itVVQULQe4chnEqPR78HHqbaGMW6Ke823ircn7xN9XTSeXBYjdHtyaTbRApk5EYXUD85X7W.json",
    "3fhUz9JetLdE9TYhfu9CMiBYRvdun8RD8MGQGkyyFnLZmfgaTp3ksSgtZ267kyr5Rmwi7UyC8QU4j2PnU5qMVr9v.json",
    "3knJCrDaoXitfS8mKuwUphhtLJyAz6DqLCoEC3VMCoveLZdqeZR18tZhkiLbx8UMEBugnAwyycMzxrpMniLKWZKn.json",
    "3Nm6wz6mGSDwCJ2tR36FhGoGd6DNirEoLVSf6CswykkcunokgUr7EDqGiP3VtNKZWLWxqdWsTPhMgxQm48iiUQhT.json",
    "3nnkh31MvXkYNeM75QMTuzRcttWkMVfd7FJQAL2YVnGT9yJrgbnUCgaHF7A2S1UiQWdRem1BKtnekPr2qWMfx6MF.json",
    "3p8NFLL8qm921hvkK2XXesbEiZY9pN9VWkdnAfhvdF1ve8pBaYswM18MAipmWY1aYtLqCqQxNvvsGz62DFYm8mvP.json",
    "42GkrQcKkP2uxztYt8aiLAFScqgZQnPc1oUv3dSkLZYjKd5AHpajTTJiyU9ZxqJmysaLUe7ksNnZW2LKrLUV4YiH.json",
    "4bQgAoSpYfTSTiBGJyqGdMHy2vGemN85e2SUF7vwksioj1YNwCo6stNwHDZdBK8MHMjk24D8NFy2XvDGDSeFGXre.json",
    "4Ph2Kpc8LnWS8eA9PbDQFUqhGcBWrvJpbegHA2KNxjqWR9qaxfUcxs75wGJmiMJeKgnV7icf2VB2uivYjb4oEQVm.json",
    "4W3MuLY176Y6JWZemKXV7skoZaVKJu9K1psZtR7VjZPBKytJK4cGC838tp55nL1NXoi7oroHZaQrVyu7V2KGq6wf.json",
    "4X69976BM32DZXKXtV4LLBdA3xKjxyPsuAqxpiFX3QTHmSDXfbYMPjmGuFatcyJj8MP7daZGQrRoKQswEvkPqFaW.json",
    "52KvUHPVu73gJQxGNxWmjTrCByfLBpabc3q8zFaq6tWA4FvfHBoMjyRHnPErEKpZEKd7PcdNfkpLwdWfSHxqAGm.json",
    "5FbKEWCZSK5SGGHP8MTbkHVGKNc3BUN5D5HK43fH4mkDtw1CEg655EdSRbRx18VvBajHjQ423g8y5AwRna8mwnsK.json",
    "5fYdZV8VxVhzqaHTS9ghU9KHiCBLBZMd4RxrHwknWraYNjv6j95VDkrXqEp8TGERTNb13LSaiDx1AAwAmJb1Zqtx.json",
    "5vCSiwWS4bAsQ7zAC7YfX4Rw4tTPXeQpXUj2caJPbM2GXn1orgazC3nrShcPxQ7MozSDtfzZyDZS1hhWnn3KR34z.json",
    "5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN.json",
    "5yEtYLoA5pwUiscJxdWjzNvkyxwpddMgFaWBUpMSnAwHNgmH22RsoWdUB85nJnN4jdq1u2RVbj8zEaPFT2xPadWy.json",
    "5Z6VsSbzrUGtDem41bPPCFFxB2dHCFndQizPZprLgc5LMycAc9Pc1wufs9A52wYbyRUuP4R6YVSie1UXRAwCxBLB.json",
];

pub fn golden_encoded_metas() -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
    GOLDENS
        .iter()
        .map(|json_file| {
            let goldens_dir =
                PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("transaction-parser/src/goldens");
            let file = File::open(goldens_dir.join(json_file))
                .expect("Golden JSON file should be readable");
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).expect("Should deserialize")
        })
        .collect()
}

pub fn golden_parsed_transactions() -> Vec<ParsedTransaction> {
    golden_encoded_metas()
        .into_iter()
        .map(ParsedTransaction::from_encoded_transaction)
        .collect::<anyhow::Result<Vec<_>>>()
        .expect("Should parse")
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
