use std::time::Duration;

use serde::Deserialize;

use crate::taker::{ActivityProfile, ExecutionProfile};

/// Named presets that bundle sensible defaults for [`ActivityProfile`] +
/// [`crate::taker::TakerStrategy`] parameters. Each `[[agent]]` in the taker
/// config selects one of these, then optionally overrides individual fields.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    Passive,
    Retail,
    Aggressive,
    Whale,
    Sniper,
    Noise,
}

pub struct ArchetypeDefaults {
    pub profile: ActivityProfile,
    pub median_order_size: u64,
    pub spread_multiplier: f64,
    pub buy_bias: f64,
    pub execution_profile: ExecutionProfile,
}

impl Archetype {
    pub fn defaults(self) -> ArchetypeDefaults {
        match self {
            Self::Passive => ArchetypeDefaults {
                profile: ActivityProfile::passive(),
                median_order_size: 2_000,
                spread_multiplier: 1.5,
                buy_bias: 0.5,
                execution_profile: ExecutionProfile::patient(),
            },
            Self::Retail => ArchetypeDefaults {
                profile: ActivityProfile::retail(),
                median_order_size: 3_000,
                spread_multiplier: 2.0,
                buy_bias: 0.5,
                execution_profile: ExecutionProfile::balanced(),
            },
            Self::Aggressive => ArchetypeDefaults {
                profile: ActivityProfile::aggressive(),
                median_order_size: 5_000,
                spread_multiplier: 2.5,
                buy_bias: 0.5,
                execution_profile: ExecutionProfile::aggressive(),
            },
            Self::Whale => ArchetypeDefaults {
                profile: ActivityProfile::aggressive(),
                median_order_size: 15_000,
                spread_multiplier: 5.0,
                buy_bias: 0.5,
                execution_profile: ExecutionProfile::aggressive(),
            },
            Self::Sniper => ArchetypeDefaults {
                profile: ActivityProfile::passive(),
                median_order_size: 3_000,
                spread_multiplier: 1.5,
                buy_bias: 0.5,
                execution_profile: ExecutionProfile::sniper(),
            },
            // A steady, high-frequency noise trader — baseline CLOB chatter
            // with no directional bias and tight size variance.
            Self::Noise => ArchetypeDefaults {
                profile: ActivityProfile {
                    interval: tokio::time::interval(Duration::from_millis(500)),
                    lambda_quiet: 1.5,
                    lambda_burst: 2.0,
                    burst_entry_prob: 0.05,
                    burst_exit_prob: 0.5,
                },
                median_order_size: 1_000,
                spread_multiplier: 1.5,
                buy_bias: 0.5,
                execution_profile: ExecutionProfile::noise(),
            },
        }
    }
}
