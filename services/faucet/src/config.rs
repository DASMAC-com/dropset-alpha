use std::{collections::HashSet, str::FromStr};

use anyhow::Context;
use dropset_services_shared::config::{deserialize_service_config, Service, ValidSharedConfig};
use serde::Deserialize;
use solana_address::Address;

const SERVICE: Service = Service::Faucet;

pub struct ValidFaucetConfig {
    pub shared: ValidSharedConfig,
    pub port: u16,
    pub cooldown_secs: u64,
    pub max_public_tokens: u64,
    pub max_whitelist_tokens: u64,
    pub whitelist: HashSet<Address>,
}

#[derive(Deserialize)]
pub struct FaucetConfigInput {
    pub rpc_url: String,
    pub base_mint: String,
    pub quote_mint: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_cooldown")]
    pub cooldown_secs: u64,
    #[serde(default = "default_max_public")]
    pub max_public_tokens: u64,
    #[serde(default = "default_max_whitelist")]
    pub max_whitelist_tokens: u64,
    #[serde(default)]
    pub whitelist: Vec<String>,
}

fn default_port() -> u16 {
    9090
}
fn default_cooldown() -> u64 {
    10
}
fn default_max_public() -> u64 {
    10
}
fn default_max_whitelist() -> u64 {
    1000
}

async fn validate_config(input: FaucetConfigInput) -> anyhow::Result<ValidFaucetConfig> {
    let shared = ValidSharedConfig::new(
        SERVICE.keypair_path(),
        input.base_mint,
        input.quote_mint,
        input.rpc_url,
    )
    .await?;

    let whitelist = input
        .whitelist
        .iter()
        .map(|s| {
            Address::from_str(s)
                .with_context(|| format!("Invalid whitelist address: {s}"))
        })
        .collect::<anyhow::Result<HashSet<Address>>>()?;

    Ok(ValidFaucetConfig {
        shared,
        port: input.port,
        cooldown_secs: input.cooldown_secs,
        max_public_tokens: input.max_public_tokens,
        max_whitelist_tokens: input.max_whitelist_tokens,
        whitelist,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidFaucetConfig> {
    let cfg: FaucetConfigInput = deserialize_service_config(SERVICE)?;
    validate_config(cfg).await
}
