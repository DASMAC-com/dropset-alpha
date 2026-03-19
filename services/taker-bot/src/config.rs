use std::time::Duration;

use dropset_services_shared::config::{
    deserialize_service_config,
    Service,
    ValidSharedConfig,
};
use serde::Deserialize;

use crate::taker::{
    ActivityProfile,
    TakerStrategy,
};

const SERVICE: Service = Service::Taker;

pub struct ValidTakerConfig {
    pub shared: ValidSharedConfig,
    pub debug_logs: bool,
    pub taker_strategy: TakerStrategy,
}

#[derive(Deserialize)]
pub struct TakerConfigInput {
    pub base_mint: String,
    pub quote_mint: String,
    pub rpc_url: String,
    pub debug_logs: bool,
    pub strategy: TakerStrategyConfig,
}

/// Flat config for an [ActivityProfile] + [TakerStrategy], suitable for TOML deserialization.
#[derive(Deserialize)]
pub struct TakerStrategyConfig {
    /// Milliseconds between ticks of the interval loop.
    pub interval_ms: u64,
    /// Base arrival rate (orders/tick) during quiet periods.
    pub lambda_quiet: f64,
    /// Arrival rate during an active burst.
    pub lambda_burst: f64,
    /// Probability of entering a burst each quiet tick.
    pub burst_entry_prob: f64,
    /// Probability of exiting the burst each tick (controls burst duration).
    pub burst_exit_prob: f64,
    /// Median order size in atoms.
    pub median_order_size: u64,
    /// Spread multiplier: order sizes range roughly from `median / x` to `median * x`.
    pub spread_multiplier: f64,
    /// Probability the next order is a buy (0.0–1.0).
    pub buy_bias: f64,
    /// Optional RNG seed for reproducible runs.
    pub seed: Option<u64>,
}

impl TakerStrategyConfig {
    fn into_strategy(self) -> anyhow::Result<TakerStrategy> {
        let mut interval = tokio::time::interval(Duration::from_millis(self.interval_ms));
        // The taker doesn't need to make up for missed intervals, the idea is for it to be
        // stochastic behavior. The desired behavior here is to skip missed ticks.
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let profile = ActivityProfile {
            interval,
            lambda_quiet: self.lambda_quiet,
            lambda_burst: self.lambda_burst,
            burst_entry_prob: self.burst_entry_prob,
            burst_exit_prob: self.burst_exit_prob,
        };
        TakerStrategy::new(
            profile,
            self.median_order_size,
            self.spread_multiplier,
            self.buy_bias,
            self.seed,
        )
    }
}

pub async fn validate_config_and_endpoint(
    input: TakerConfigInput,
) -> anyhow::Result<ValidTakerConfig> {
    let TakerConfigInput {
        base_mint,
        quote_mint,
        rpc_url,
        debug_logs,
        strategy,
    } = input;

    if strategy.interval_ms == 0 {
        anyhow::bail!("The taker strategy's interval value must be greater than zero");
    }

    let shared =
        ValidSharedConfig::new(SERVICE.keypair_path(), base_mint, quote_mint, rpc_url).await?;

    let taker_strategy = strategy.into_strategy()?;

    Ok(ValidTakerConfig {
        shared,
        debug_logs,
        taker_strategy,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidTakerConfig> {
    let cfg: TakerConfigInput = deserialize_service_config(SERVICE)?;
    validate_config_and_endpoint(cfg).await
}
