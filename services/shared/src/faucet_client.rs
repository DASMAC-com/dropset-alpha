use client::transactions::CustomRpcClient;
use reqwest::Url;
use serde::{
    Deserialize,
    Serialize,
};
use solana_address::Address;
use solana_keypair::{
    Keypair,
    Signature,
};
use solana_sdk::transaction::Transaction;

use crate::config::ValidSharedConfig;

pub struct FaucetClient {
    client: reqwest::Client,
    faucet_url: Url,
}

impl FaucetClient {
    pub async fn new(shared: &ValidSharedConfig) -> Option<FaucetClient> {
        let client = reqwest::Client::new();
        let faucet_url = shared.faucet_url();
        get_health(&client, &faucet_url)
            .await
            .is_ok()
            .then(|| Self { client, faucet_url })
    }

    /// Wrapper for [get_health].
    pub async fn get_health(&self) -> anyhow::Result<HealthResponse> {
        get_health(&self.client, &self.faucet_url).await
    }

    /// Wrapper for passing the result of [request_base] into [sign_faucet_transaction_and_submit].
    pub async fn request_base_sign_and_submit(
        &self,
        to: &Address,
        keypair: &Keypair,
        rpc: &CustomRpcClient,
        amount: Option<u64>,
    ) -> anyhow::Result<Signature> {
        let txn = request_base(&self.client, &self.faucet_url, to, amount).await?;
        sign_faucet_transaction_and_submit(txn, keypair, rpc).await
    }

    /// Wrapper for passing the result of [request_quote] into [sign_faucet_transaction_and_submit].
    pub async fn request_quote_sign_and_submit(
        &self,
        to: &Address,
        keypair: &Keypair,
        rpc: &CustomRpcClient,
        amount: Option<u64>,
    ) -> anyhow::Result<Signature> {
        let txn = request_quote(&self.client, &self.faucet_url, to, amount).await?;
        sign_faucet_transaction_and_submit(txn, keypair, rpc).await
    }
}

pub const DEFAULT_FAUCET_AMOUNT: u64 = 1;

#[derive(Serialize, Deserialize)]
pub struct MintRequest {
    /// The recipient's address.
    pub address: String,
    /// Whether or not this is the `base` token. If false: it's the `quote` token.
    pub is_base: bool,
    /// Amount in whole tokens (will be multiplied by 10^mint_decimals).
    pub amount: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub cluster: String,
}

#[derive(Serialize, Deserialize)]
pub struct MintResponse {
    pub transaction: Transaction,
}

pub enum FaucetEndpoint {
    Health,
    Faucet,
}

impl FaucetEndpoint {
    pub fn route(&self) -> &'static str {
        match self {
            FaucetEndpoint::Health => "/health",
            FaucetEndpoint::Faucet => "/faucet",
        }
    }
}

/// Calls the health endpoint for the faucet.
///
/// Returns a [HealthResponse] on success.
pub async fn get_health(
    client: &reqwest::Client,
    faucet_url: &Url,
) -> anyhow::Result<HealthResponse> {
    let url = faucet_url.join(FaucetEndpoint::Health.route())?;

    let resp = client.get(url).send().await?;

    if resp.status().is_success() {
        let body: HealthResponse = resp.json().await?;
        Ok(body)
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Faucet health check failed: ({status}): {body}");
    }
}

/// Requests base tokens from a running faucet service.
///
/// Returns a [Transaction] on success, partially signed by the faucet keypair.
pub async fn request_base(
    client: &reqwest::Client,
    faucet_url: &Url,
    address: &Address,
    amount: Option<u64>,
) -> anyhow::Result<Transaction> {
    request_tokens(client, faucet_url, address, amount, true).await
}

/// Requests quote tokens from a running faucet service.
///
/// Returns a [Transaction] on success, partially signed by the faucet keypair.
pub async fn request_quote(
    client: &reqwest::Client,
    faucet_url: &Url,
    address: &Address,
    amount: Option<u64>,
) -> anyhow::Result<Transaction> {
    request_tokens(client, faucet_url, address, amount, false).await
}

/// Requests base or quote tokens from a running faucet service.
///
/// Returns a [Transaction] on success, partially signed by the faucet keypair.
async fn request_tokens(
    client: &reqwest::Client,
    faucet_url: &Url,
    address: &Address,
    amount: Option<u64>,
    is_base: bool,
) -> anyhow::Result<Transaction> {
    let url = faucet_url.join(FaucetEndpoint::Faucet.route())?;

    let resp = client
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

/// Handles signing and submitting the received [Transaction] from [request_tokens].
async fn sign_faucet_transaction_and_submit(
    mut faucet_transaction: Transaction,
    keypair: &Keypair,
    rpc: &CustomRpcClient,
) -> anyhow::Result<Signature> {
    faucet_transaction.sign(&[keypair], faucet_transaction.message.recent_blockhash);

    Ok(rpc
        .client
        .send_and_confirm_transaction(&faucet_transaction)
        .await?)
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
        ServiceConfig,
        ValidSharedConfig,
    };

    async fn request_tokens_check(
        client: &reqwest::Client,
        rpc: &CustomRpcClient,
        faucet_url: &Url,
        shared: &ValidSharedConfig,
        user: &Keypair,
        is_base: bool,
        requested: Option<u64>,
    ) -> anyhow::Result<()> {
        let user_address = user.pubkey();
        let token = if is_base { &shared.base } else { &shared.quote };

        let mut txn = if is_base {
            request_base(client, faucet_url, &user_address, requested).await
        } else {
            request_quote(client, faucet_url, &user_address, requested).await
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
        let expected_amount_str = requested.unwrap_or(DEFAULT_FAUCET_AMOUNT).to_string();
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
        let shared = ValidSharedConfig::new_validated(ServiceConfig::Faucet)
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

        let base_amt = Some(5);
        let quote_amt = Some(6);

        let req = reqwest::Client::new();
        request_tokens_check(&req, &rpc, &faucet_url(), &shared, &user, true, base_amt).await?;
        request_tokens_check(&req, &rpc, &faucet_url(), &shared, &user, false, quote_amt).await?;

        Ok(())
    }

    /// Integration test for the expected failures from the faucet.
    ///
    /// Run with: cargo test -p dropset-services-shared -- --ignored faucet
    #[tokio::test]
    #[ignore]
    async fn request_tokens_from_local_faucet_failure() {
        let req = reqwest::Client::new();
        let res = request_base(&req, &faucet_url(), &Address::new_unique(), Some(0)).await;

        #[rustfmt::skip]
        assert!(res.is_err(), "Faucet should return an error when the amount is zero");
    }

    /// Integration test for the default fund amount and the [sign_faucet_transaction_and_submit]
    /// helper function.
    ///
    /// Run with: cargo test -p dropset-services-shared -- --ignored faucet
    #[tokio::test]
    #[ignore]
    async fn request_default_amount() -> anyhow::Result<()> {
        let shared = ValidSharedConfig::new_validated(ServiceConfig::Faucet)
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

        let req = reqwest::Client::new();
        let user = Keypair::new();
        let user_address = user.pubkey();
        rpc.fund_account(&user_address).await?;
        let txn = request_base(&req, &faucet_url(), &user_address, None).await?;
        sign_faucet_transaction_and_submit(txn, &user, &rpc).await?;

        let ata = shared.base.get_ata_for(&user_address);
        let balance = rpc.client.get_token_account_balance(&ata).await?;
        assert_eq!(balance.ui_amount_string, DEFAULT_FAUCET_AMOUNT.to_string());

        Ok(())
    }
}
