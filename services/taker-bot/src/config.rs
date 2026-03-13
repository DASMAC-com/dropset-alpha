use dropset_services_shared::config::{
    deserialize_service_config,
    Service,
    ValidSharedConfig,
};
use serde::Deserialize;

const SERVICE: Service = Service::Taker;

pub struct ValidTakerConfig {
    pub shared: ValidSharedConfig,
    pub sell_order_size: u64,
    pub buy_order_size: u64,
}

#[derive(Deserialize)]
pub struct TakerConfigInput {
    pub base_mint: String,
    pub quote_mint: String,
    pub sell_order_size: u64,
    pub buy_order_size: u64,
    pub rpc_url: String,
}

pub async fn validate_config_and_endpoint(
    input: TakerConfigInput,
) -> anyhow::Result<ValidTakerConfig> {
    let TakerConfigInput {
        base_mint: base_mint_input,
        quote_mint: quote_mint_input,
        sell_order_size,
        buy_order_size,
        rpc_url: rpc_url_input,
    } = input;

    let shared = ValidSharedConfig::new(
        SERVICE.keypair_path(),
        base_mint_input,
        quote_mint_input,
        rpc_url_input,
    )
    .await?;

    Ok(ValidTakerConfig {
        shared,
        sell_order_size,
        buy_order_size,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidTakerConfig> {
    let cfg: TakerConfigInput = deserialize_service_config(SERVICE)?;

    validate_config_and_endpoint(cfg).await
}
