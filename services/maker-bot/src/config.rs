use std::path::Path;

use anyhow::Context;
use dropset_services_shared::{
    config::{
        deserialize_service_config,
        Service,
        ValidSharedConfig,
    },
    oanda_types::CurrencyPair,
};
use reqwest::Url;
use serde::Deserialize;

const SERVICE: Service = Service::Maker;

pub struct ValidMakerConfig {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub shared: ValidSharedConfig,
    pub target_base: u64,
    pub batch_replace: bool,
    pub base_order_size: u64,
    pub quote_order_size: u64,
    pub ws_url: Url,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
}

#[derive(Deserialize)]
pub struct MakerConfigInput {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub target_base: u64,
    pub batch_replace: bool,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_order_size: u64,
    pub quote_order_size: u64,
    pub rpc_url: String,
    pub ws_url: String,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
}

pub async fn validate_config_and_endpoint(
    path: &Path,
    input: MakerConfigInput,
) -> anyhow::Result<ValidMakerConfig> {
    let MakerConfigInput {
        oanda_auth_token,
        pair,
        target_base,
        batch_replace,
        base_mint: base_mint_input,
        quote_mint: quote_mint_input,
        base_order_size,
        quote_order_size,
        rpc_url: rpc_url_input,
        ws_url: ws_url_input,
        price_feed_poll_interval,
        order_update_throttle_window,
    } = input;

    if oanda_auth_token.is_empty() || oanda_auth_token == "your-token-here" {
        anyhow::bail!(
            "oanda_auth_token in '{}' is not set.\n\
                 Edit the file and replace the placeholder with your OANDA API token.",
            path.display()
        );
    }

    let ws_url = Url::try_from(ws_url_input.as_str())
        .context(format!("Invalid WS url: {}", ws_url_input))?;

    let shared = ValidSharedConfig::new(
        SERVICE.keypair_path(),
        base_mint_input,
        quote_mint_input,
        rpc_url_input,
    )
    .await?;

    Ok(ValidMakerConfig {
        shared,
        oanda_auth_token,
        pair,
        target_base,
        batch_replace,
        base_order_size,
        quote_order_size,
        ws_url,
        price_feed_poll_interval,
        order_update_throttle_window,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidMakerConfig> {
    let cfg: MakerConfigInput = deserialize_service_config(SERVICE)?;
    let path = &SERVICE.toml_config_path();

    validate_config_and_endpoint(path, cfg).await
}
