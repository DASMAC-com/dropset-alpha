use std::{
    collections::HashSet,
    str::FromStr,
};

use anyhow::Context;
use dropset_services_shared::config::{
    deserialize_service_config,
    Service,
    ValidSharedConfig,
};
use serde::Deserialize;
use solana_address::Address;

const SERVICE: Service = Service::Faucet;

pub struct ValidFaucetConfig {
    pub shared: ValidSharedConfig,
    pub port: u16,
    pub max_public_tokens: u64,
    pub max_allowlist_tokens: u64,
    pub allowlist: HashSet<Address>,
}

#[derive(Deserialize)]
pub struct FaucetConfigInput {
    pub rpc_url: String,
    pub base_mint: String,
    pub quote_mint: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_public")]
    pub max_public_tokens: u64,
    #[serde(default = "default_max_allowlist")]
    pub max_allowlist_tokens: u64,
    #[serde(default)]
    pub allowlist: Vec<String>,
}

fn default_port() -> u16 {
    9090
}
fn default_max_public() -> u64 {
    10
}
fn default_max_allowlist() -> u64 {
    1000
}

async fn validate_config_and_endpoint(
    input: FaucetConfigInput,
) -> anyhow::Result<ValidFaucetConfig> {
    let shared = ValidSharedConfig::new(
        SERVICE.keypair_path(),
        input.base_mint,
        input.quote_mint,
        input.rpc_url,
    )
    .await?;

    let max_public_tokens = match input.max_public_tokens {
        0 => anyhow::bail!("Max public tokens must be greater than zero"),
        _ => input.max_public_tokens,
    };

    let max_allowlist_tokens = match input.max_allowlist_tokens {
        0 => anyhow::bail!("Max allowlist tokens must be greater than zero"),
        _ => input.max_allowlist_tokens,
    };

    let allowlist = input
        .allowlist
        .iter()
        .map(|s| Address::from_str(s).with_context(|| format!("Invalid allowlist address: {s}")))
        .collect::<anyhow::Result<HashSet<Address>>>()?;

    Ok(ValidFaucetConfig {
        shared,
        port: input.port,
        max_public_tokens,
        max_allowlist_tokens,
        allowlist,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidFaucetConfig> {
    let cfg: FaucetConfigInput = deserialize_service_config(SERVICE)?;
    validate_config_and_endpoint(cfg).await
}
