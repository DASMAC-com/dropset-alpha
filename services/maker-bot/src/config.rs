use std::path::Path;

use anyhow::Context;
use dropset_services_shared::{
    config::{deserialize_service_config, ServiceConfig, ValidSharedConfig},
    oanda_types::{CurrencyPair, OandaCandlestickResponse},
};
use reqwest::Url;
use serde::Deserialize;

use crate::{
    oanda_price_feed::{query_price_feed, OandaArgs},
    GRANULARITY, NUM_CANDLES,
};

const SERVICE: ServiceConfig = ServiceConfig::Maker;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MakerStyle {
    Tight,
    #[default]
    Balanced,
    Defensive,
}

#[derive(Clone, Copy)]
pub struct MakerStyleDefaults {
    pub quote_ttl_ms: u64,
    pub min_refill_delay_ms: u64,
    pub max_refill_delay_ms: u64,
    pub replenish_ratio_bps: u16,
    pub size_jitter_bps: u16,
    pub price_jitter_pct: u16,
    pub hit_widening_bps: u16,
    pub local_book_weight_bps: u16,
    pub max_quote_levels: usize,
    pub spread_multiplier_bps: u16,
}

impl MakerStyle {
    pub fn defaults(self) -> MakerStyleDefaults {
        match self {
            Self::Tight => MakerStyleDefaults {
                quote_ttl_ms: 1_500,
                min_refill_delay_ms: 250,
                max_refill_delay_ms: 900,
                replenish_ratio_bps: 9_000,
                size_jitter_bps: 1_000,
                price_jitter_pct: 6,
                hit_widening_bps: 10,
                local_book_weight_bps: 2_500,
                max_quote_levels: 10,
                spread_multiplier_bps: 9_000,
            },
            Self::Balanced => MakerStyleDefaults {
                quote_ttl_ms: 2_500,
                min_refill_delay_ms: 600,
                max_refill_delay_ms: 2_000,
                replenish_ratio_bps: 7_000,
                size_jitter_bps: 1_800,
                price_jitter_pct: 12,
                hit_widening_bps: 18,
                local_book_weight_bps: 4_000,
                max_quote_levels: 8,
                spread_multiplier_bps: 12_000,
            },
            Self::Defensive => MakerStyleDefaults {
                quote_ttl_ms: 3_500,
                min_refill_delay_ms: 1_200,
                max_refill_delay_ms: 4_000,
                replenish_ratio_bps: 5_500,
                size_jitter_bps: 2_500,
                price_jitter_pct: 18,
                hit_widening_bps: 28,
                local_book_weight_bps: 5_500,
                max_quote_levels: 6,
                spread_multiplier_bps: 17_500,
            },
        }
    }
}

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
    pub style: MakerStyle,
    pub quote_ttl_ms: u64,
    pub min_refill_delay_ms: u64,
    pub max_refill_delay_ms: u64,
    pub replenish_ratio_bps: u16,
    pub size_jitter_bps: u16,
    pub price_jitter_pct: u16,
    pub hit_widening_bps: u16,
    pub local_book_weight_bps: u16,
    pub max_quote_levels: usize,
    pub spread_multiplier_bps: u16,
    pub seed: u64,
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
    pub style: MakerStyle,
    pub quote_ttl_ms: Option<u64>,
    pub min_refill_delay_ms: Option<u64>,
    pub max_refill_delay_ms: Option<u64>,
    pub replenish_ratio_bps: Option<u16>,
    pub size_jitter_bps: Option<u16>,
    pub price_jitter_pct: Option<u16>,
    pub hit_widening_bps: Option<u16>,
    pub local_book_weight_bps: Option<u16>,
    pub max_quote_levels: Option<usize>,
    pub spread_multiplier_bps: Option<u16>,
    pub seed: Option<u64>,
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
        style,
        quote_ttl_ms,
        min_refill_delay_ms,
        max_refill_delay_ms,
        replenish_ratio_bps,
        size_jitter_bps,
        price_jitter_pct,
        hit_widening_bps,
        local_book_weight_bps,
        max_quote_levels,
        spread_multiplier_bps,
        seed,
        visualize,
    } = input;

    let defaults = style.defaults();
    let quote_ttl_ms = quote_ttl_ms.unwrap_or(defaults.quote_ttl_ms);
    let min_refill_delay_ms = min_refill_delay_ms.unwrap_or(defaults.min_refill_delay_ms);
    let max_refill_delay_ms = max_refill_delay_ms.unwrap_or(defaults.max_refill_delay_ms);
    let replenish_ratio_bps = replenish_ratio_bps.unwrap_or(defaults.replenish_ratio_bps);
    let size_jitter_bps = size_jitter_bps.unwrap_or(defaults.size_jitter_bps);
    let price_jitter_pct = price_jitter_pct.unwrap_or(defaults.price_jitter_pct);
    let hit_widening_bps = hit_widening_bps.unwrap_or(defaults.hit_widening_bps);
    let local_book_weight_bps = local_book_weight_bps.unwrap_or(defaults.local_book_weight_bps);
    let max_quote_levels = max_quote_levels.unwrap_or(defaults.max_quote_levels);
    let spread_multiplier_bps = spread_multiplier_bps.unwrap_or(defaults.spread_multiplier_bps);
    let seed = seed.unwrap_or(7);

    if oanda_auth_token.is_empty() || oanda_auth_token == "your-token-here" {
        anyhow::bail!(
            "oanda_auth_token in '{}' is not set.\n\
                 Edit the file and replace the placeholder with your OANDA API token.",
            path.display()
        );
    }

    if quote_ttl_ms == 0 {
        anyhow::bail!("quote_ttl_ms must be greater than zero");
    }
    if min_refill_delay_ms == 0 || max_refill_delay_ms < min_refill_delay_ms {
        anyhow::bail!("Refill delay window must be positive and ordered");
    }
    if max_quote_levels == 0 || max_quote_levels > 10 {
        anyhow::bail!("max_quote_levels must be between 1 and 10");
    }
    for (name, value) in [
        ("replenish_ratio_bps", replenish_ratio_bps),
        ("size_jitter_bps", size_jitter_bps),
        ("hit_widening_bps", hit_widening_bps),
        ("spread_multiplier_bps", spread_multiplier_bps),
    ] {
        anyhow::ensure!(value <= 20_000, "{name} must be <= 20000 bps");
    }
    anyhow::ensure!(
        local_book_weight_bps <= 10_000,
        "local_book_weight_bps must be <= 10000 bps (convex weight in [0, 1])"
    );
    anyhow::ensure!(
        price_jitter_pct <= 100,
        "price_jitter_pct must be <= 100 (percent of step)"
    );

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
        style,
        quote_ttl_ms,
        min_refill_delay_ms,
        max_refill_delay_ms,
        replenish_ratio_bps,
        size_jitter_bps,
        price_jitter_pct,
        hit_widening_bps,
        local_book_weight_bps,
        max_quote_levels,
        spread_multiplier_bps,
        seed,
        oanda_args,
        initial_price_feed_response,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidMakerConfig> {
    let cfg: MakerConfigInput = deserialize_service_config(SERVICE)?;
    let path = &SERVICE.toml_config_path();

    validate_config_and_endpoint(path, cfg).await
}
