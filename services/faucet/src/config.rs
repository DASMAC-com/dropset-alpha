use dropset_services_shared::config::{
    deserialize_service_config,
    Service,
    ValidSharedConfig,
};
use serde::Deserialize;

const SERVICE: Service = Service::Faucet;

pub struct ValidFaucetConfig {
    pub shared: ValidSharedConfig,
}

#[derive(Deserialize)]
pub struct FaucetConfigInput {
    pub base_mint: String,
    pub quote_mint: String,
    pub rpc_url: String,
}

pub async fn validate_config_and_endpoint(
    input: FaucetConfigInput,
) -> anyhow::Result<ValidFaucetConfig> {
    let FaucetConfigInput {
        base_mint: base_mint_input,
        quote_mint: quote_mint_input,
        rpc_url: rpc_url_input,
    } = input;

    let shared = ValidSharedConfig::new(
        SERVICE.keypair_path(),
        base_mint_input,
        quote_mint_input,
        rpc_url_input,
    )
    .await?;

    Ok(ValidFaucetConfig { shared })
}

pub async fn get_validated_config() -> anyhow::Result<ValidFaucetConfig> {
    let cfg: FaucetConfigInput = deserialize_service_config(SERVICE)?;

    validate_config_and_endpoint(cfg).await
}
