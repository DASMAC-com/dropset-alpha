use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use dropset_services_shared::config::{
    deserialize_service_config, ServiceConfig, ValidSharedConfig,
};
use serde::Deserialize;
use solana_keypair::{read_keypair_file, Keypair};

use crate::{
    archetype::{Archetype, ArchetypeDefaults},
    taker::{ActivityProfile, ExecutionProfile, TakerStrategy},
};

const SERVICE: ServiceConfig = ServiceConfig::Taker;

pub struct ValidTakerConfig {
    pub shared: ValidSharedConfig,
    pub verbose: bool,
    pub agents: Vec<ValidAgent>,
}

pub struct ValidAgent {
    pub name: String,
    pub keypair: Keypair,
    pub strategy: TakerStrategy,
}

#[derive(Deserialize)]
pub struct TakerConfigInput {
    #[serde(default)]
    pub verbose: bool,
    #[serde(default, rename = "agent")]
    pub agents: Vec<AgentConfigInput>,
}

/// One `[[agent]]` entry in `taker-bot/config.toml`. Every field except
/// `name`, `archetype`, and `keypair_path` is an optional override on top of
/// the archetype's preset defaults.
#[derive(Deserialize)]
pub struct AgentConfigInput {
    pub name: String,
    pub archetype: Archetype,
    pub keypair_path: PathBuf,
    pub interval_ms: Option<u64>,
    pub lambda_quiet: Option<f64>,
    pub lambda_burst: Option<f64>,
    pub burst_entry_prob: Option<f64>,
    pub burst_exit_prob: Option<f64>,
    pub median_order_size: Option<u64>,
    pub spread_multiplier: Option<f64>,
    pub buy_bias: Option<f64>,
    pub parent_multiplier_min: Option<f64>,
    pub parent_multiplier_max: Option<f64>,
    pub child_depth_fraction_min: Option<f64>,
    pub child_depth_fraction_max: Option<f64>,
    pub max_sweep_levels: Option<usize>,
    pub max_spread_bps: Option<f64>,
    pub cooldown_ticks: Option<u8>,
    pub parent_slice_count_min: Option<u8>,
    pub parent_slice_count_max: Option<u8>,
    pub imbalance_bias: Option<f64>,
    pub patience_ticks: Option<u8>,
    pub seed: Option<u64>,
}

impl AgentConfigInput {
    fn into_valid(self) -> anyhow::Result<ValidAgent> {
        let ArchetypeDefaults {
            profile: default_profile,
            median_order_size: default_size,
            spread_multiplier: default_spread,
            buy_bias: default_bias,
            execution_profile: default_execution,
        } = self.archetype.defaults();

        let interval = match self.interval_ms {
            Some(0) => anyhow::bail!("Agent `{}`: interval_ms must be > 0", self.name),
            Some(ms) => {
                let mut i = tokio::time::interval(Duration::from_millis(ms));
                i.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                i
            }
            None => {
                let mut i = default_profile.interval;
                i.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                i
            }
        };

        let profile = ActivityProfile {
            interval,
            lambda_quiet: self.lambda_quiet.unwrap_or(default_profile.lambda_quiet),
            lambda_burst: self.lambda_burst.unwrap_or(default_profile.lambda_burst),
            burst_entry_prob: self
                .burst_entry_prob
                .unwrap_or(default_profile.burst_entry_prob),
            burst_exit_prob: self
                .burst_exit_prob
                .unwrap_or(default_profile.burst_exit_prob),
        };

        let execution_profile = ExecutionProfile {
            parent_multiplier_min: self
                .parent_multiplier_min
                .unwrap_or(default_execution.parent_multiplier_min),
            parent_multiplier_max: self
                .parent_multiplier_max
                .unwrap_or(default_execution.parent_multiplier_max),
            child_depth_fraction_min: self
                .child_depth_fraction_min
                .unwrap_or(default_execution.child_depth_fraction_min),
            child_depth_fraction_max: self
                .child_depth_fraction_max
                .unwrap_or(default_execution.child_depth_fraction_max),
            max_sweep_levels: self
                .max_sweep_levels
                .unwrap_or(default_execution.max_sweep_levels),
            max_spread_bps: self
                .max_spread_bps
                .unwrap_or(default_execution.max_spread_bps),
            cooldown_ticks: self
                .cooldown_ticks
                .unwrap_or(default_execution.cooldown_ticks),
            parent_slice_count_min: self
                .parent_slice_count_min
                .unwrap_or(default_execution.parent_slice_count_min),
            parent_slice_count_max: self
                .parent_slice_count_max
                .unwrap_or(default_execution.parent_slice_count_max),
            imbalance_bias: self
                .imbalance_bias
                .unwrap_or(default_execution.imbalance_bias),
            patience_ticks: self
                .patience_ticks
                .unwrap_or(default_execution.patience_ticks),
        };

        let strategy = TakerStrategy::new(
            profile,
            self.median_order_size.unwrap_or(default_size),
            self.spread_multiplier.unwrap_or(default_spread),
            self.buy_bias.unwrap_or(default_bias),
            execution_profile,
            self.seed,
        )
        .with_context(|| format!("Invalid strategy for agent `{}`", self.name))?;

        let keypair_path = resolve_keypair_path(&self.keypair_path);
        let keypair = read_keypair_file(&keypair_path).map_err(|e| {
            anyhow::anyhow!(
                "Agent `{}`: couldn't open keypair file `{}`: {e}",
                self.name,
                keypair_path.display()
            )
        })?;

        Ok(ValidAgent {
            name: self.name,
            keypair,
            strategy,
        })
    }
}

/// Relative keypair paths are resolved against the taker-bot service directory
/// (so a config like `keypair_path = "keypairs/retail-1.json"` works without
/// every caller having to care about the current working directory).
fn resolve_keypair_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        SERVICE.config_dir().join(path)
    }
}

pub async fn validate_config_and_endpoint(
    input: TakerConfigInput,
) -> anyhow::Result<ValidTakerConfig> {
    let TakerConfigInput { verbose, agents } = input;

    if agents.is_empty() {
        anyhow::bail!("At least one `[[agent]]` entry is required in the taker-bot config");
    }

    let mut seen_names = std::collections::HashSet::new();
    for agent in &agents {
        if !seen_names.insert(agent.name.as_str()) {
            anyhow::bail!("Duplicate agent name `{}` in taker-bot config", agent.name);
        }
    }

    let shared = ValidSharedConfig::new_validated(SERVICE).await?;

    let agents = agents
        .into_iter()
        .map(AgentConfigInput::into_valid)
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(ValidTakerConfig {
        shared,
        verbose,
        agents,
    })
}

pub async fn get_validated_config() -> anyhow::Result<ValidTakerConfig> {
    let cfg: TakerConfigInput = deserialize_service_config(SERVICE)?;
    validate_config_and_endpoint(cfg).await
}
