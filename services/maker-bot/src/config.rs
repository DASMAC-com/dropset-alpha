use std::path::Path;

use anyhow::Context;
use dropset_services_shared::{
    config::{
        deserialize_service_config,
        ServiceConfig,
        ValidSharedConfig,
    },
    oanda_types::{
        CurrencyPair,
        OandaCandlestickResponse,
    },
};
use reqwest::Url;
use serde::Deserialize;

use crate::{
    oanda_price_feed::{
        query_price_feed,
        OandaArgs,
    },
    GRANULARITY,
    NUM_CANDLES,
};

const SERVICE: ServiceConfig = ServiceConfig::Maker;

pub struct ValidMakerConfig {
    pub shared: ValidSharedConfig,
    pub target_base: u64,
    pub batch_replace: bool,
    pub ask_order_size: u64,
    pub bid_order_size: u64,
    pub visualize: bool,
    pub ws_url: Url,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
    pub oanda_args: OandaArgs,
    pub initial_price_feed_response: OandaCandlestickResponse,
}

#[derive(Deserialize)]
pub struct MakerConfigInput {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub target_base: u64,
    pub batch_replace: bool,
    pub ask_order_size: u64,
    pub bid_order_size: u64,
    pub ws_url: String,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
    #[serde(default)]
    pub visualize: bool,
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
        ask_order_size,
        bid_order_size,
        ws_url: ws_url_input,
        price_feed_poll_interval,
        order_update_throttle_window,
        visualize,
    } = input;

    if oanda_auth_token.is_empty() || oanda_auth_token == "your-token-here" {
        anyhow::bail!(
            "oanda_auth_token in '{}' is not set.\n\
                 Edit the file and replace the placeholder with your OANDA API token.",
            path.display()
        );
    }

    let oanda_args = OandaArgs {
        auth_token: oanda_auth_token,
        pair,
        granularity: GRANULARITY,
        num_candles: NUM_CANDLES,
    };

    let initial_price_feed_response = query_price_feed(&oanda_args, &reqwest::Client::new())
        .await
        .with_context(|| anyhow::anyhow!("Couldn't query OANDA price feed."))?;

    let ws_url = Url::try_from(ws_url_input.as_str())
        .context(format!("Invalid WS url: {}", ws_url_input))?;

    let shared = ValidSharedConfig::new_validated(SERVICE).await?;

    Ok(ValidMakerConfig {
        shared,
        target_base,
        batch_replace,
        ask_order_size,
        bid_order_size,
        visualize,
        ws_url,
        price_feed_poll_interval,
        order_update_throttle_window,
        oanda_args,
        initial_price_feed_response,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidMakerConfig> {
    let cfg: MakerConfigInput = deserialize_service_config(SERVICE)?;
    let path = &SERVICE.toml_config_path();

    validate_config_and_endpoint(path, cfg).await
}
