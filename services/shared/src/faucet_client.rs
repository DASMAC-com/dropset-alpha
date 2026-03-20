use reqwest::Url;
use serde::{
    Deserialize,
    Serialize,
};
use solana_address::Address;

#[derive(Serialize)]
struct MintRequest {
    address: String,
    mint: String,
    amount: u64,
}

#[derive(Deserialize)]
struct MintResponse {
    signature: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

/// Requests tokens from a running faucet service.
///
/// Returns the transaction signature on success.
pub async fn request_tokens(
    faucet_url: &Url,
    address: &Address,
    mint: &Address,
    amount: u64,
) -> anyhow::Result<String> {
    let url = faucet_url.join("/faucet")?;

    let resp = reqwest::Client::new()
        .post(url)
        .json(&MintRequest {
            address: address.to_string(),
            mint: mint.to_string(),
            amount,
        })
        .send()
        .await?;

    if resp.status().is_success() {
        let body: MintResponse = resp.json().await?;
        Ok(body.signature)
    } else {
        let body: ErrorResponse = resp.json().await.unwrap_or(ErrorResponse {
            error: "Unknown faucet error".into(),
        });
        anyhow::bail!("Faucet request failed: {}", body.error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        Service,
        ValidSharedConfig,
    };

    /// Integration test that requires a running faucet and localnet.
    /// Reads everything from the faucet's config.toml and keypair.json — no env vars needed.
    ///
    /// Run with: cargo test -p dropset-services-shared -- --ignored faucet
    #[tokio::test]
    #[ignore]
    async fn request_tokens_from_local_faucet() {
        let shared = ValidSharedConfig::from_service(Service::Faucet)
            .await
            .expect("Faucet config should be valid");

        let faucet_url = Url::parse("http://localhost:9090").unwrap();
        let address = shared.address();

        let sig = request_tokens(&faucet_url, &address, &shared.base, 5)
            .await
            .expect("Faucet request should succeed");

        assert!(!sig.is_empty(), "Signature should not be empty");
        println!("Got signature: {sig}");
    }
}
