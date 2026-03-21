use reqwest::Url;
use serde::{
    Deserialize,
    Serialize,
};
use solana_address::Address;
use solana_sdk::transaction::Transaction;

#[derive(Serialize, Deserialize)]
pub struct MintRequest {
    /// The recipient's address.
    pub address: String,
    /// Whether or not this is the `base` token. If false: it's the `quote` token.
    pub is_base: bool,
    /// Amount in whole tokens (will be multiplied by 10^mint_decimals).
    #[serde(default = "default_amount")]
    pub amount: u64,
}

fn default_amount() -> u64 {
    1
}

#[derive(Serialize, Deserialize)]
pub struct MintResponse {
    pub transaction: Transaction,
}

/// Requests base tokens from a running faucet service.
///
/// Returns the transaction signature on success.
pub async fn request_base(
    faucet_url: &Url,
    address: &Address,
    amount: u64,
) -> anyhow::Result<Transaction> {
    request_tokens(faucet_url, address, amount, true).await
}

/// Requests quote tokens from a running faucet service.
///
/// Returns the transaction signature on success.
pub async fn request_quote(
    faucet_url: &Url,
    address: &Address,
    amount: u64,
) -> anyhow::Result<Transaction> {
    request_tokens(faucet_url, address, amount, false).await
}

/// Requests base or quote tokens from a running faucet service.
///
/// Returns the transaction signature on success.
async fn request_tokens(
    faucet_url: &Url,
    address: &Address,
    amount: u64,
    is_base: bool,
) -> anyhow::Result<Transaction> {
    let url = faucet_url.join("/faucet")?;

    let resp = reqwest::Client::new()
        .post(url)
        .json(&MintRequest {
            address: address.to_string(),
            is_base,
            amount,
        })
        .send()
        .await?;

    if resp.status().is_success() {
        let body: MintResponse = resp.json().await?;
        Ok(body.transaction)
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Faucet request failed: ({status}): {body}");
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use client::transactions::{
        CustomRpcClient,
        SendTransactionConfig,
    };
    use solana_keypair::{
        Keypair,
        Signer,
    };
    use solana_rpc_client::api::{
        client_error::ErrorKind,
        request::RpcError,
    };

    use super::*;
    use crate::config::{
        Service,
        ValidSharedConfig,
    };

    async fn request_tokens_check(
        rpc: &CustomRpcClient,
        faucet_url: &Url,
        shared: &ValidSharedConfig,
        user: &Keypair,
        is_base: bool,
        requested: u64,
    ) -> anyhow::Result<()> {
        let user_address = user.pubkey();
        let token = if is_base { &shared.base } else { &shared.quote };

        let mut txn = if is_base {
            request_base(faucet_url, &user_address, requested).await
        } else {
            request_quote(faucet_url, &user_address, requested).await
        }
        .expect("Faucet request should succeed");

        #[rustfmt::skip]
        assert_eq!(txn.signatures.len(), 2, "Faucet transaction should have two signatures.");

        let txn_msg = txn.message().serialize();
        let faucet_addr_bytes = shared.keypair.pubkey().to_bytes();
        // Exactly one signer should be valid: the faucet's signature.
        let first_verified = txn.signatures[0].verify(&faucet_addr_bytes, &txn_msg);
        let second_verified = txn.signatures[1].verify(&faucet_addr_bytes, &txn_msg);
        assert!((first_verified && !second_verified) || (!first_verified && second_verified));

        // Neither should be valid signatures for the user already.
        for sig in &txn.signatures {
            assert!(!sig.verify(user_address.as_array(), &txn_msg));
        }

        #[rustfmt::skip]
        assert!(!txn.is_signed(), "Faucet transaction shouldn't be fully signed.");

        let blockhash_is_valid = rpc
            .client
            .is_blockhash_valid(&txn.message.recent_blockhash, rpc.client.commitment())
            .await
            .unwrap();

        assert!(blockhash_is_valid);

        txn.sign(&[user], txn.message.recent_blockhash);

        #[rustfmt::skip]
        assert!(txn.is_signed(), "Faucet transaction should be fully signed.");

        // The user's token account shouldn't exist.
        let ata = token.get_ata_for(&user_address);

        let is_nonexistent_account =
            rpc.client
                .get_account(&ata)
                .await
                .is_err_and(|e| match &e.kind() {
                    &ErrorKind::RpcError(RpcError::ForUser(s)) => s.starts_with("AccountNotFound"),
                    _ => false,
                });

        assert!(is_nonexistent_account);

        rpc.client.send_and_confirm_transaction(&txn).await?;
        let balance = rpc.client.get_token_account_balance(&ata).await?;
        let expected_amount_str = requested.to_string();
        assert_eq!(balance.ui_amount_string, expected_amount_str);

        Ok(())
    }

    fn faucet_url() -> Url {
        Url::parse("http://localhost:9090").unwrap()
    }

    /// Integration test that requires a running faucet and localnet.
    /// Reads everything from the faucet's config.toml and keypair.json.
    ///
    /// Run with: cargo test -p dropset-services-shared -- --ignored faucet
    #[tokio::test]
    #[ignore]
    async fn request_tokens_from_local_faucet() -> anyhow::Result<()> {
        let shared = ValidSharedConfig::from_service(Service::Faucet)
            .await
            .expect("Faucet config should be valid");

        let rpc = CustomRpcClient::new_from_url(
            shared.rpc_url.as_str(),
            SendTransactionConfig {
                compute_budget: None,
                debug_logs: Some(true),
                program_id_filter: HashSet::new(),
            },
        );
        let user = rpc.fund_new_account().await?;

        let base_amount = 5;
        let quote_amount = 6;

        request_tokens_check(&rpc, &faucet_url(), &shared, &user, true, base_amount).await?;
        request_tokens_check(&rpc, &faucet_url(), &shared, &user, false, quote_amount).await?;

        Ok(())
    }

    /// Integration test for the expected failures from the faucet.
    ///
    /// Run with: cargo test -p dropset-services-shared -- --ignored faucet
    #[tokio::test]
    #[ignore]
    async fn request_tokens_from_local_faucet_failure() {
        let res = request_base(&faucet_url(), &Address::new_unique(), 0).await;

        #[rustfmt::skip]
        assert!(res.is_err(), "Faucet should return an error when the amount is zero");
    }
}
