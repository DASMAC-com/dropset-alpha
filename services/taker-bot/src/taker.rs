use std::time::Duration;

use anyhow::Context;
use rand::{prelude::*, random};
use rand_distr::{Distribution, LogNormal, Poisson};
use rust_decimal::Decimal;
use tokio::time::Interval;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn is_buy(&self) -> bool {
        matches!(self, Side::Buy)
    }
}

#[derive(Debug, Clone)]
pub struct TakerFill {
    pub side: Side,
    pub size: u64,
    pub planned_levels: usize,
    pub parent_remaining: u64,
}

/// Controls the activity profile for a taker. This indicates how frequently a taker places orders
/// and how often they "burst" orders. A burst is a short window of
/// elevated λ, followed by quiet. This is the key to realistic CLOB flow.
pub struct ActivityProfile {
    /// The time in milliseconds between periods of activity.
    pub interval: Interval,
    /// Base arrival rate (orders/slot) during quiet periods.
    pub lambda_quiet: f64,
    /// Arrival rate during an active burst.
    pub lambda_burst: f64,
    /// Probability of entering a burst in any given quiet slot.
    pub burst_entry_prob: f64,
    /// Probability of exiting the burst each slot (controls burst duration).
    pub burst_exit_prob: f64,
}

impl ActivityProfile {
    /// A passive, slow taker: rare, small pokes.
    pub fn passive() -> Self {
        Self {
            interval: tokio::time::interval(Duration::from_millis(4000)),
            lambda_quiet: 0.2,
            lambda_burst: 2.5,
            burst_entry_prob: 0.05,
            burst_exit_prob: 0.4,
        }
    }

    /// A normal retail taker: occasional bursts.
    pub fn retail() -> Self {
        Self {
            interval: tokio::time::interval(Duration::from_millis(2000)),
            lambda_quiet: 0.5,
            lambda_burst: 5.0,
            burst_entry_prob: 0.1,
            burst_exit_prob: 0.3,
        }
    }

    /// An aggressive taker: frequent, intense bursts.
    pub fn aggressive() -> Self {
        Self {
            interval: tokio::time::interval(Duration::from_millis(750)),
            lambda_quiet: 1.0,
            lambda_burst: 12.0,
            burst_entry_prob: 0.2,
            burst_exit_prob: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BookLevel {
    pub price: Decimal,
    pub base_remaining: u64,
    pub quote_remaining: u64,
}

#[derive(Debug, Clone, Default)]
pub struct BookSideSnapshot {
    pub levels: Vec<BookLevel>,
    pub total_base_depth: u64,
}

impl BookSideSnapshot {
    pub fn best_base_depth(&self) -> u64 {
        self.levels.first().map_or(0, |level| level.base_remaining)
    }

    pub fn visible_base_depth(&self, levels: usize) -> u64 {
        self.levels
            .iter()
            .take(levels)
            .map(|level| level.base_remaining)
            .sum()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarketSnapshot {
    pub bids: BookSideSnapshot,
    pub asks: BookSideSnapshot,
    pub spread_bps: Option<f64>,
    pub mid_price: Option<Decimal>,
    pub microprice: Option<Decimal>,
    pub imbalance: f64,
}

impl MarketSnapshot {
    pub fn synthetic(best_level_depth: u64) -> Self {
        let make_level = |price: Decimal, multiplier: u64| BookLevel {
            price,
            base_remaining: best_level_depth.saturating_mul(multiplier),
            quote_remaining: best_level_depth.saturating_mul(multiplier),
        };

        Self {
            bids: BookSideSnapshot {
                levels: vec![
                    make_level(Decimal::new(9990, 4), 1),
                    make_level(Decimal::new(9980, 4), 2),
                    make_level(Decimal::new(9970, 4), 3),
                ],
                total_base_depth: best_level_depth.saturating_mul(6),
            },
            asks: BookSideSnapshot {
                levels: vec![
                    make_level(Decimal::new(10010, 4), 1),
                    make_level(Decimal::new(10020, 4), 2),
                    make_level(Decimal::new(10030, 4), 3),
                ],
                total_base_depth: best_level_depth.saturating_mul(6),
            },
            spread_bps: Some(20.0),
            mid_price: Some(Decimal::ONE),
            microprice: Some(Decimal::ONE),
            imbalance: 0.0,
        }
    }

    fn opposing_side(&self, side: Side) -> &BookSideSnapshot {
        match side {
            Side::Buy => &self.asks,
            Side::Sell => &self.bids,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExecutionProfile {
    pub parent_multiplier_min: f64,
    pub parent_multiplier_max: f64,
    pub child_depth_fraction_min: f64,
    pub child_depth_fraction_max: f64,
    pub max_sweep_levels: usize,
    pub max_spread_bps: f64,
    pub cooldown_ticks: u8,
    pub parent_slice_count_min: u8,
    pub parent_slice_count_max: u8,
    pub imbalance_bias: f64,
    pub patience_ticks: u8,
}

impl ExecutionProfile {
    pub fn patient() -> Self {
        Self {
            parent_multiplier_min: 1.5,
            parent_multiplier_max: 4.0,
            child_depth_fraction_min: 0.04,
            child_depth_fraction_max: 0.12,
            max_sweep_levels: 2,
            max_spread_bps: 8.0,
            cooldown_ticks: 2,
            parent_slice_count_min: 2,
            parent_slice_count_max: 6,
            imbalance_bias: 0.04,
            patience_ticks: 3,
        }
    }

    pub fn balanced() -> Self {
        Self {
            parent_multiplier_min: 2.0,
            parent_multiplier_max: 6.0,
            child_depth_fraction_min: 0.08,
            child_depth_fraction_max: 0.22,
            max_sweep_levels: 3,
            max_spread_bps: 15.0,
            cooldown_ticks: 1,
            parent_slice_count_min: 2,
            parent_slice_count_max: 5,
            imbalance_bias: 0.08,
            patience_ticks: 2,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            parent_multiplier_min: 4.0,
            parent_multiplier_max: 12.0,
            child_depth_fraction_min: 0.18,
            child_depth_fraction_max: 0.85,
            max_sweep_levels: 5,
            max_spread_bps: 30.0,
            cooldown_ticks: 0,
            parent_slice_count_min: 1,
            parent_slice_count_max: 4,
            imbalance_bias: 0.12,
            patience_ticks: 1,
        }
    }

    pub fn noise() -> Self {
        Self {
            parent_multiplier_min: 1.0,
            parent_multiplier_max: 2.5,
            child_depth_fraction_min: 0.03,
            child_depth_fraction_max: 0.10,
            max_sweep_levels: 1,
            max_spread_bps: 10.0,
            cooldown_ticks: 0,
            parent_slice_count_min: 1,
            parent_slice_count_max: 3,
            imbalance_bias: 0.03,
            patience_ticks: 1,
        }
    }

    /// Opportunistic single-shot taker: only acts when the spread is tight,
    /// follows imbalance strongly, and pauses between hits.
    pub fn sniper() -> Self {
        Self {
            parent_multiplier_min: 1.0,
            parent_multiplier_max: 2.0,
            child_depth_fraction_min: 0.05,
            child_depth_fraction_max: 0.20,
            max_sweep_levels: 1,
            max_spread_bps: 4.0,
            cooldown_ticks: 3,
            parent_slice_count_min: 1,
            parent_slice_count_max: 2,
            imbalance_bias: 0.20,
            patience_ticks: 0,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TakerStepStats {
    pub attempted_children: u64,
    pub submitted_children: u64,
    pub parent_orders_started: u64,
    pub skipped_empty_book: u64,
    pub skipped_wide_spread: u64,
    pub skipped_cooldown: u64,
}

impl TakerStepStats {
    pub fn accumulate(&mut self, other: Self) {
        self.attempted_children += other.attempted_children;
        self.submitted_children += other.submitted_children;
        self.parent_orders_started += other.parent_orders_started;
        self.skipped_empty_book += other.skipped_empty_book;
        self.skipped_wide_spread += other.skipped_wide_spread;
        self.skipped_cooldown += other.skipped_cooldown;
    }
}

#[derive(Debug, Default)]
pub struct TakerStep {
    pub fills: Vec<TakerFill>,
    pub stats: TakerStepStats,
    fill_commit: Option<FillCommit>,
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct PlannedTick {
    pub attempts: u64,
    pub stats: TakerStepStats,
}

#[derive(Debug, Clone, Copy)]
struct FillCommit {
    next_parent_order: Option<ParentOrder>,
    enters_cooldown: bool,
}

#[derive(Debug, Clone, Copy)]
struct ParentOrder {
    side: Side,
    remaining_base: u64,
    children_remaining: u8,
    urgency: f64,
    patience_ticks_remaining: u8,
}

pub struct TakerStrategy {
    /// Controls burst/quiet switching, arrival rates, and tick interval.
    activity_profile: ActivityProfile,
    /// Probability this taker's next order is a buy. Always in [0.0, 1.0].
    buy_bias: f64,
    /// Fraction by which `buy_bias` reverts toward 0.5 each step. Always in [0.0, 1.0].
    bias_reversion: f64,
    /// `mu` for the LogNormal order size distribution. Computed as `ln(median_order_size)`.
    size_mu: f64,
    /// `sigma` for the LogNormal order size distribution. Computed as `ln(spread_multiplier)`.
    /// A `spread_multiplier` of 2 means sizes range roughly from `median/2` to `median*2`.
    size_sigma: f64,
    execution_profile: ExecutionProfile,
    /// Whether the taker is currently in an active burst period.
    in_burst: bool,
    parent_order: Option<ParentOrder>,
    cooldown_ticks_remaining: u8,
    /// Random number generator, seeded from config or randomly at startup.
    rng: StdRng,
}

impl TakerStrategy {
    /// `median_order_size` is the median order size in base atoms (`mu = ln(median_order_size)`).
    /// `spread_multiplier` controls the width of the size distribution: a value of 2 means
    /// sizes range roughly from `median/2` to `median*2` (`sigma = ln(spread_multiplier)`).
    pub fn new(
        activity_profile: ActivityProfile,
        median_order_size: u64,
        spread_multiplier: f64,
        buy_bias: f64,
        execution_profile: ExecutionProfile,
        seed: Option<u64>,
    ) -> anyhow::Result<Self> {
        if median_order_size == 0 {
            anyhow::bail!("Median size must be greater than zero");
        }

        if spread_multiplier <= 0.0 {
            anyhow::bail!("Spread multiplier must be greater than zero");
        }

        let size_mu = (median_order_size as f64).ln();
        let size_sigma = spread_multiplier.ln();
        LogNormal::new(size_mu, size_sigma).with_context(|| {
            let msg = format!("Invalid (size_mu, size_sigma): ({size_mu}, {size_sigma}) when calculating LogNormal");
            anyhow::anyhow!(msg)
        })?;

        if !(0.0..=1.0).contains(&buy_bias) {
            anyhow::bail!("Buy bias must be between 0.0 and 1.0");
        }

        if !(0.0..=1.0).contains(&activity_profile.burst_entry_prob) {
            anyhow::bail!("burst_entry_prob must be between 0.0 and 1.0");
        }

        if !(0.0..=1.0).contains(&activity_profile.burst_exit_prob) {
            anyhow::bail!("burst_exit_prob must be between 0.0 and 1.0");
        }

        if execution_profile.parent_multiplier_min <= 0.0
            || execution_profile.parent_multiplier_max < execution_profile.parent_multiplier_min
        {
            anyhow::bail!("Execution profile parent multipliers must be positive and ordered");
        }

        if execution_profile.child_depth_fraction_min <= 0.0
            || execution_profile.child_depth_fraction_max
                < execution_profile.child_depth_fraction_min
        {
            anyhow::bail!("Execution profile child depth fractions must be positive and ordered");
        }

        if execution_profile.max_sweep_levels == 0 {
            anyhow::bail!("Execution profile max_sweep_levels must be greater than zero");
        }

        if execution_profile.parent_slice_count_min == 0
            || execution_profile.parent_slice_count_max < execution_profile.parent_slice_count_min
        {
            anyhow::bail!("Execution profile parent slice counts must be positive and ordered");
        }

        if !execution_profile.imbalance_bias.is_finite()
            || execution_profile.imbalance_bias.abs() > 1.0
        {
            anyhow::bail!("Execution profile imbalance_bias must be finite and within [-1.0, 1.0]");
        }

        if execution_profile.patience_ticks > 32 {
            anyhow::bail!("Execution profile patience_ticks must be <= 32");
        }

        Poisson::new(activity_profile.lambda_burst).with_context(|| {
            let msg = format!(
                "Invalid `lambda_burst` when calculating Poisson::new({})",
                activity_profile.lambda_burst
            );
            anyhow::anyhow!(msg)
        })?;

        Poisson::new(activity_profile.lambda_quiet).with_context(|| {
            let msg = format!(
                "Invalid `lambda_quiet` when calculating Poisson::new({})",
                activity_profile.lambda_quiet
            );
            anyhow::anyhow!(msg)
        })?;

        Ok(Self {
            activity_profile,
            buy_bias,
            bias_reversion: 0.05,
            size_mu,
            size_sigma,
            execution_profile,
            in_burst: false,
            parent_order: None,
            cooldown_ticks_remaining: 0,
            rng: StdRng::seed_from_u64(seed.unwrap_or(random::<u64>())),
        })
    }

    pub async fn tick(&mut self) {
        self.activity_profile.interval.tick().await;
    }

    /// A single moment of market activity between idle intervals. Takers now
    /// maintain a parent order and adapt child-order sizing to visible book depth.
    pub fn step(&mut self, snapshot: &MarketSnapshot) -> TakerStep {
        self.step_with_snapshot_provider(|| snapshot.clone())
    }

    fn step_with_snapshot_provider(
        &mut self,
        mut snapshot: impl FnMut() -> MarketSnapshot,
    ) -> TakerStep {
        let planned = self.begin_tick();
        let mut step = TakerStep {
            fills: Vec::with_capacity(planned.attempts as usize),
            stats: planned.stats,
            fill_commit: None,
        };

        for _ in 0..planned.attempts {
            let attempt = self.execute_attempt(&snapshot());
            step.stats.accumulate(attempt.stats);
            if !attempt.fills.is_empty() {
                self.confirm_attempt(&attempt);
                step.stats.submitted_children += 1;
            }
            step.fills.extend(attempt.fills);
        }

        step
    }

    pub(crate) fn begin_tick(&mut self) -> PlannedTick {
        let bp = &self.activity_profile;

        // Burst state machine
        self.in_burst = if self.in_burst {
            !self.rng.random_bool(bp.burst_exit_prob)
        } else {
            self.rng.random_bool(bp.burst_entry_prob)
        };

        let lambda = if self.in_burst {
            bp.lambda_burst
        } else {
            bp.lambda_quiet
        };

        // Poisson distribution is a discrete probability distribution, so it always outputs
        // an integer type, but the authors of the `rust_random` crate decided to defer
        // truncation to the user. `as u64` used below is the expected and suggested solution.
        // See here: https://rust-random.github.io/book/guide-dist.html#integers
        let n_orders = Poisson::new(lambda)
            .unwrap_or_else(|_| panic!("Poisson::new({}) was checked in the constructor", lambda))
            .sample(&mut self.rng) as u64;

        if n_orders == 0 {
            return PlannedTick::default();
        }

        if self.cooldown_ticks_remaining > 0 {
            self.cooldown_ticks_remaining -= 1;
            return PlannedTick {
                attempts: 0,
                stats: TakerStepStats {
                    skipped_cooldown: n_orders,
                    ..TakerStepStats::default()
                },
            };
        }

        PlannedTick {
            attempts: n_orders,
            stats: TakerStepStats::default(),
        }
    }

    pub(crate) fn execute_attempt(&mut self, snapshot: &MarketSnapshot) -> TakerStep {
        let mut step = TakerStep::default();
        let size_dist = LogNormal::new(self.size_mu, self.size_sigma).unwrap_or_else(|_| {
            panic!(
                "LogNormal::new({}, {}) was checked in the constructor",
                self.size_mu, self.size_sigma
            )
        });

        step.stats.attempted_children += 1;

        if self.parent_order.is_none() {
            match self.start_parent_order(snapshot, &size_dist) {
                Some(parent) => {
                    self.parent_order = Some(parent);
                    step.stats.parent_orders_started += 1;
                }
                None => {
                    step.stats.skipped_empty_book += 1;
                    return step;
                }
            }
        }

        let mut parent = self
            .parent_order
            .expect("parent_order is set above when start_parent_order returns Some");
        let opposing_side = snapshot.opposing_side(parent.side);
        if opposing_side.levels.is_empty() {
            self.parent_order = None;
            step.stats.skipped_empty_book += 1;
            return step;
        }

        let spread_bps = snapshot.spread_bps.unwrap_or_default();
        let too_wide = spread_bps > self.execution_profile.max_spread_bps && parent.urgency < 0.85;
        if too_wide {
            if parent.patience_ticks_remaining > 0 {
                parent.patience_ticks_remaining -= 1;
            } else {
                parent.urgency = (parent.urgency + 0.10).min(1.0);
            }
            self.parent_order = Some(parent);
            step.stats.skipped_wide_spread += 1;
            return step;
        }

        let visible_depth = opposing_side
            .visible_base_depth(self.execution_profile.max_sweep_levels)
            .max(opposing_side.best_base_depth());
        let child_sample = size_dist.sample(&mut self.rng).max(1.0) as u64;
        let depth_fraction = self.rng.random_range(
            self.execution_profile.child_depth_fraction_min
                ..=self.execution_profile.child_depth_fraction_max,
        );
        let urgency_multiplier = 0.5 + parent.urgency;
        let depth_target =
            ((visible_depth as f64) * depth_fraction * urgency_multiplier).round() as u64;
        let min_child_size = ((opposing_side.best_base_depth() as f64)
            * self.execution_profile.child_depth_fraction_min.max(0.01))
        .round() as u64;

        let size = child_sample
            .max(depth_target)
            .max(min_child_size.max(1))
            .min(parent.remaining_base)
            .min(visible_depth.max(1));

        let planned_levels = planned_levels_for_size(size, &opposing_side.levels);
        let fill = TakerFill {
            side: parent.side,
            size,
            planned_levels,
            parent_remaining: parent.remaining_base.saturating_sub(size),
        };

        parent.remaining_base = fill.parent_remaining;
        parent.children_remaining = parent.children_remaining.saturating_sub(1);
        let parent_done = parent.remaining_base == 0 || parent.children_remaining == 0;

        step.fills.push(fill);
        step.fill_commit = Some(FillCommit {
            next_parent_order: (!parent_done).then_some(parent),
            enters_cooldown: parent_done,
        });
        step
    }

    pub(crate) fn confirm_attempt(&mut self, attempt: &TakerStep) {
        let Some(commit) = attempt.fill_commit else {
            return;
        };

        self.parent_order = commit.next_parent_order;
        if commit.enters_cooldown {
            self.cooldown_ticks_remaining = self.execution_profile.cooldown_ticks;
        }
    }

    fn start_parent_order(
        &mut self,
        snapshot: &MarketSnapshot,
        size_dist: &LogNormal<f64>,
    ) -> Option<ParentOrder> {
        let imbalance_adjustment = snapshot.imbalance * self.execution_profile.imbalance_bias;
        let buy_probability = (self.buy_bias + imbalance_adjustment).clamp(0.05, 0.95);
        let side = if self.rng.random_bool(buy_probability) {
            Side::Buy
        } else {
            Side::Sell
        };

        self.buy_bias += self.bias_reversion * (0.5 - self.buy_bias);

        let opposing_side = snapshot.opposing_side(side);
        if opposing_side.levels.is_empty() {
            return None;
        }

        let child_base = size_dist.sample(&mut self.rng).max(1.0) as u64;
        let parent_multiplier = self.rng.random_range(
            self.execution_profile.parent_multiplier_min
                ..=self.execution_profile.parent_multiplier_max,
        );
        let visible_depth = opposing_side
            .visible_base_depth(self.execution_profile.max_sweep_levels)
            .max(opposing_side.best_base_depth());
        let capped_visible_depth =
            ((visible_depth as f64) * (1.0 + self.rng.random_range(0.1..=0.6))).round() as u64;
        let remaining_base = ((child_base as f64) * parent_multiplier).round() as u64;
        let remaining_base = remaining_base
            .max(child_base)
            .min(capped_visible_depth.max(1));
        let children_remaining = self.rng.random_range(
            self.execution_profile.parent_slice_count_min
                ..=self.execution_profile.parent_slice_count_max,
        );

        Some(ParentOrder {
            side,
            remaining_base,
            children_remaining,
            urgency: self.rng.random_range(0.25..=1.0),
            patience_ticks_remaining: self.execution_profile.patience_ticks,
        })
    }

    /// Creates the interval loop based on the taker's activity profile and taker strategy, calling
    /// `on_fill` every [ActivityProfile::interval] milliseconds.
    pub async fn interval_loop(
        mut self,
        mut snapshot: impl FnMut() -> MarketSnapshot,
        mut on_fill: impl FnMut(TakerFill),
    ) {
        loop {
            self.activity_profile.interval.tick().await;
            for fill in self.step_with_snapshot_provider(&mut snapshot).fills {
                on_fill(fill);
            }
        }
    }
}

fn planned_levels_for_size(size: u64, levels: &[BookLevel]) -> usize {
    let mut remaining = size;
    let mut touched = 0;

    for level in levels {
        if remaining == 0 {
            break;
        }
        touched += 1;
        remaining = remaining.saturating_sub(level.base_remaining);
    }

    touched
}
#[cfg(test)]
mod tests {
    use std::sync::atomic;

    use client::{
        e2e_helpers::test_accounts,
        mollusk_helpers::{
            market_checker::MarketChecker, new_dropset_mollusk_context_with_default_market,
            utils::create_mock_user_account,
        },
    };
    use dropset_interface::{
        instructions::{
            BatchReplaceInstructionData, MarketOrderInstructionData, UnvalidatedOrders,
        },
        state::{
            sector::{MAX_PERMITTED_SECTOR_INCREASE, NIL},
            user_order_sectors::MAX_ORDERS_USIZE,
        },
    };
    use price::client_helpers::to_order_info_args;
    use rust_decimal::{
        prelude::{FromPrimitive, ToPrimitive},
        Decimal,
    };
    use solana_account::Account;
    use solana_address::Address;
    use solana_instruction::Instruction;
    use solana_keypair::Signer;

    use super::*;

    /// Creates a [TakerStrategy] with a moderate activity profile and order size.
    pub fn retail(median_order_size: u64, seed: u64) -> TakerStrategy {
        TakerStrategy::new(
            ActivityProfile::retail(),
            median_order_size,
            2.0,
            0.5,
            ExecutionProfile::balanced(),
            Some(seed),
        )
        .expect("Should be valid inputs")
    }

    /// Creates a [TakerStrategy] with a high activity profile, large order sizes, and directional
    /// bias with fat tail sizes (high sigma, aka large spread multiplier).
    pub fn whale(median_order_size: u64, seed: u64) -> TakerStrategy {
        TakerStrategy::new(
            ActivityProfile::aggressive(),
            median_order_size,
            5.0,
            0.6,
            ExecutionProfile::aggressive(),
            Some(seed),
        )
        .expect("Should be valid inputs")
    }

    /// Creates a [TakerStrategy] with a passive activity profile, moderate order sizes, no
    /// directional bias, and very low spread multiplier.
    pub fn sniper(median_order_size: u64, seed: u64) -> TakerStrategy {
        TakerStrategy::new(
            ActivityProfile::passive(),
            median_order_size,
            1.5,
            0.5,
            ExecutionProfile::patient(),
            Some(seed),
        )
        .expect("Should be valid inputs")
    }

    pub struct Simulation {
        pub taker_addresses: Vec<Address>,
        pub taker_strategies: Vec<TakerStrategy>,
        pub n_steps: usize,
    }

    pub struct TakerFillWithAddress {
        pub address: Address,
        pub side: Side,
        pub size: u64,
    }

    impl Simulation {
        pub fn new(
            taker_addresses: Vec<Address>,
            taker_strategies: Vec<TakerStrategy>,
            n_steps: usize,
        ) -> Self {
            Self {
                taker_addresses,
                taker_strategies,
                n_steps,
            }
        }

        /// Step each taker `n_steps` times, collecting all fills.
        pub fn run(&mut self) -> Vec<TakerFillWithAddress> {
            let snapshot = MarketSnapshot::synthetic(500_000);
            let all_fills: Vec<TakerFillWithAddress> = (0..self.n_steps)
                .flat_map(|_| {
                    self.taker_addresses
                        .iter()
                        .zip(&mut self.taker_strategies)
                        .flat_map(|(address, strategy)| {
                            strategy.step(&snapshot).fills.into_iter().map(|fill| {
                                TakerFillWithAddress {
                                    address: *address,
                                    side: fill.side,
                                    size: fill.size,
                                }
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            all_fills
        }
    }

    fn display_addr(taker_addr: &Address) -> String {
        format!("0x{}", &taker_addr.to_string()[0..4])
    }

    /// Runs a deterministic taker-flow simulation and prints the generated fills.
    /// Ignored by default because this is a manual inspection/debug harness,
    /// not an assertion-based test.
    #[tokio::test]
    #[ignore]
    async fn poisson_takers_pure_simulation() {
        let (taker_addresses, takers): (Vec<Address>, Vec<_>) = vec![
            (test_accounts::acc_2222().pubkey(), retail(3000, 42)),
            (test_accounts::acc_4444().pubkey(), retail(3000, 555)),
            (test_accounts::acc_AAAA().pubkey(), retail(3000, 137)),
            (test_accounts::acc_CCCC().pubkey(), whale(15000, 9001)),
            (test_accounts::acc_FFFF().pubkey(), sniper(3000, 31337)),
        ]
        .into_iter()
        .unzip();

        let mut sim = Simulation::new(taker_addresses, takers, 50);
        let fills = sim.run();

        println!("{:<10} {:<6} {:<10}", "taker", "side", "size");
        println!("{}", "-".repeat(40));

        for f in &fills {
            let bar = match f.side {
                Side::Buy => format!("\x1b[32m{:>6} BUY \x1b[0m", "▲"),
                Side::Sell => format!("\x1b[31m{:>6} SELL\x1b[0m", "▼"),
            };
            println!("{}  {}  {}", display_addr(&f.address), bar, f.size);
        }

        println!("\n── Per-taker summary ──────────────────────────────");
        for address in sim.taker_addresses {
            let mine: Vec<_> = fills.iter().filter(|f| f.address == address).collect();
            let buys = mine.iter().filter(|f| f.side == Side::Buy).count();
            let vol: u64 = mine.iter().map(|f| f.size).sum();
            println!(
                "Taker {}: {:>3} fills | {:>3} buys / {:>3} sells | vol {}",
                display_addr(&address),
                mine.len(),
                buys,
                mine.len() - buys,
                vol
            );
        }
        println!("Total fills: {}", fills.len());
    }

    #[tokio::test]
    async fn step_refreshes_liquidity_between_child_attempts() {
        let mut strategy = whale(50_000, 7);
        let deep = MarketSnapshot::synthetic(500_000);
        let shallow = MarketSnapshot::synthetic(25);

        for _ in 0..256 {
            let mut snapshot_calls = 0;
            let step = strategy.step_with_snapshot_provider(|| {
                snapshot_calls += 1;
                if snapshot_calls == 1 {
                    deep.clone()
                } else {
                    shallow.clone()
                }
            });

            if step.fills.len() < 2 {
                continue;
            }

            let second_fill = &step.fills[1];
            let second_visible_depth = shallow
                .opposing_side(second_fill.side)
                .visible_base_depth(strategy.execution_profile.max_sweep_levels)
                .max(1);

            assert!(second_fill.size <= second_visible_depth);
            return;
        }

        panic!("expected a multi-fill step to validate refreshed liquidity");
    }

    fn test_strategy(seed: u64) -> TakerStrategy {
        let mut interval = tokio::time::interval(Duration::from_millis(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        TakerStrategy::new(
            ActivityProfile {
                interval,
                lambda_quiet: 1.0,
                lambda_burst: 1.0,
                burst_entry_prob: 0.0,
                burst_exit_prob: 1.0,
            },
            100,
            1.1,
            0.5,
            ExecutionProfile {
                parent_multiplier_min: 2.0,
                parent_multiplier_max: 2.0,
                child_depth_fraction_min: 0.1,
                child_depth_fraction_max: 0.1,
                max_sweep_levels: 3,
                max_spread_bps: 100.0,
                cooldown_ticks: 2,
                parent_slice_count_min: 2,
                parent_slice_count_max: 2,
                imbalance_bias: 0.0,
                patience_ticks: 0,
            },
            Some(seed),
        )
        .expect("test strategy should be valid")
    }

    #[tokio::test]
    async fn failed_submission_keeps_parent_order_open() {
        let mut strategy = test_strategy(11);
        let snapshot = MarketSnapshot::synthetic(10_000);

        let attempt = strategy.execute_attempt(&snapshot);
        let fill = attempt
            .fills
            .first()
            .expect("attempt should plan a child fill");
        let parent_before_submit = strategy
            .parent_order
            .expect("starting a parent order should persist before submission");

        assert_eq!(
            parent_before_submit.remaining_base,
            fill.size + fill.parent_remaining
        );
        assert_eq!(parent_before_submit.children_remaining, 2);
    }

    #[tokio::test]
    async fn successful_submission_commits_parent_progress() {
        let mut strategy = test_strategy(11);
        let snapshot = MarketSnapshot::synthetic(10_000);

        let attempt = strategy.execute_attempt(&snapshot);
        let fill = attempt
            .fills
            .first()
            .cloned()
            .expect("attempt should plan a child fill");

        strategy.confirm_attempt(&attempt);

        if fill.parent_remaining == 0 {
            assert!(strategy.parent_order.is_none());
            assert_eq!(
                strategy.cooldown_ticks_remaining,
                strategy.execution_profile.cooldown_ticks
            );
        } else {
            let parent_after_submit = strategy
                .parent_order
                .expect("a partially filled parent order should remain open");
            assert_eq!(parent_after_submit.remaining_base, fill.parent_remaining);
            assert_eq!(parent_after_submit.children_remaining, 1);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn poisson_takers_dropset_market() -> anyhow::Result<()> {
        // This isn't a literal duration; with the `tokio` flag `start_paused = true`, each
        // sleep is simulated by lurching the wall clock forward instead of literally waiting.
        // The real test duration should be less than a few seconds.
        const TEST_DURATION: u64 = 20;

        let maker_keypair = test_accounts::acc_1111();
        let taker_keypairs = [
            test_accounts::acc_2222(),
            test_accounts::acc_4444(),
            test_accounts::acc_AAAA(),
            test_accounts::acc_CCCC(),
            test_accounts::acc_FFFF(),
        ];
        let mollusk_account_pairs: Vec<(Address, Account)> = std::iter::once(&maker_keypair)
            .chain(taker_keypairs.iter())
            .map(|acc| create_mock_user_account(acc.pubkey(), 1_000_000_000))
            .collect();

        let mollusk_addresses: Vec<Address> = mollusk_account_pairs
            .iter()
            .map(|(addr, _)| *addr)
            .collect();

        const INITIAL_BASE: u64 = 1_000_000_000;
        const INITIAL_QUOTE: u64 = 1_000_000_000;
        let (mollusk, market_ctx) =
            new_dropset_mollusk_context_with_default_market(&mollusk_account_pairs);
        let checker = MarketChecker::new(&mollusk, &market_ctx);

        // Create all maker and taker ATAs for base/quote, then mint large amounts to them to use.
        let create_token_accounts: Vec<Instruction> = mollusk_addresses
            .iter()
            .flat_map(|user| {
                vec![
                    market_ctx.base.create_ata_idempotent(user, user),
                    market_ctx.quote.create_ata_idempotent(user, user),
                    market_ctx.quote.mint_to_user(user, INITIAL_QUOTE).unwrap(),
                    market_ctx.base.mint_to_user(user, INITIAL_BASE).unwrap(),
                ]
            })
            .collect();
        /// The maker seat's index post-creation should be 0 since it's the first market seat.
        const MAKER_SEAT_INDEX: u32 = 0;
        // Have the maker deposit all of their base and quote to the market.
        let maker_deposits: Vec<Instruction> = vec![
            market_ctx.deposit_base(maker_keypair.pubkey(), INITIAL_BASE, NIL),
            // Maker should create the first seat.
            market_ctx.deposit_quote(maker_keypair.pubkey(), INITIAL_QUOTE, MAKER_SEAT_INDEX),
        ];

        let expand_instruction =
            market_ctx.expand(maker_keypair.pubkey(), MAX_PERMITTED_SECTOR_INCREASE as u16);

        let initialization_instructions = [
            create_token_accounts,
            maker_deposits,
            vec![expand_instruction],
        ]
        .concat();

        assert!(mollusk
            .process_instruction_chain(&initialization_instructions)
            .program_result
            .is_ok());

        checker.has_seat(maker_keypair.pubkey());
        checker.seat_index(maker_keypair.pubkey(), MAKER_SEAT_INDEX);

        let maker_order_at_price = |price: usize, atoms: u64, in_base: bool| {
            let decimal_price = Decimal::from_usize(price).unwrap();
            let base_atoms = if in_base {
                atoms
            } else {
                (Decimal::from_u64(atoms).unwrap() / decimal_price)
                    .round()
                    .to_u64()
                    .unwrap()
            };
            to_order_info_args(decimal_price, base_atoms).unwrap()
        };

        // Have the maker create thick, layered orders. The purpose of this test is to employ the
        // takers to fill orders, so the maker just needs to make sure takers have liquidity.
        const N: usize = MAX_ORDERS_USIZE;
        let bids = core::array::from_fn::<_, N, _>(|i| {
            maker_order_at_price(N - i, INITIAL_QUOTE / N as u64, false)
        });
        let asks = core::array::from_fn::<_, N, _>(|i| {
            maker_order_at_price(N + 1 + i, INITIAL_BASE / N as u64, true)
        });
        assert!(mollusk
            .process_instruction_chain(&[market_ctx.batch_replace(
                maker_keypair.pubkey(),
                BatchReplaceInstructionData::new(
                    MAKER_SEAT_INDEX,
                    UnvalidatedOrders::new(bids),
                    UnvalidatedOrders::new(asks),
                ),
            )])
            .program_result
            .is_ok());

        checker.num_asks(MAX_ORDERS_USIZE);
        checker.num_bids(MAX_ORDERS_USIZE);
        checker.num_seats(1);

        // Spawn the taker tasks and have them fill orders for a period of time, then print out
        // the results.

        struct Taker {
            pub address: Address,
            pub strategy: TakerStrategy,
        }

        impl Taker {
            pub fn new(address: Address, strategy: TakerStrategy) -> Self {
                Self { address, strategy }
            }
        }

        let taker_1 = Taker::new(test_accounts::acc_2222().pubkey(), retail(3000, 42));
        let taker_2 = Taker::new(test_accounts::acc_4444().pubkey(), retail(3000, 555));
        let taker_3 = Taker::new(test_accounts::acc_AAAA().pubkey(), retail(3000, 137));
        let taker_4 = Taker::new(test_accounts::acc_CCCC().pubkey(), whale(15000, 9001));
        let taker_5 = Taker::new(test_accounts::acc_FFFF().pubkey(), sniper(3000, 31337));

        let num_takes = atomic::AtomicU64::new(0);
        let increment = || num_takes.fetch_add(1, atomic::Ordering::Relaxed);

        tokio::select! {
            _ = taker_1.strategy.interval_loop(|| MarketSnapshot::synthetic(100_000_000), |TakerFill { side, size, .. }| {
                let res = mollusk.process_instruction(&market_ctx.market_order(
                    taker_1.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
                increment();
                assert!(res.program_result.is_ok());
            }) => {},
            _ = taker_2.strategy.interval_loop(|| MarketSnapshot::synthetic(100_000_000), |TakerFill { side, size, .. }| {
                let res = mollusk.process_instruction(&market_ctx.market_order(
                    taker_2.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
                increment();
                assert!(res.program_result.is_ok());
            }) => {},
            _ = taker_3.strategy.interval_loop(|| MarketSnapshot::synthetic(100_000_000), |TakerFill { side, size, .. }| {
                let res = mollusk.process_instruction(&market_ctx.market_order(
                    taker_3.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
                increment();
                assert!(res.program_result.is_ok());
            }) => {},
            _ = taker_4.strategy.interval_loop(|| MarketSnapshot::synthetic(100_000_000), |TakerFill { side, size, .. }| {
                let res = mollusk.process_instruction(&market_ctx.market_order(
                    taker_4.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
                increment();
                assert!(res.program_result.is_ok());
            }) => {},
            _ = taker_5.strategy.interval_loop(|| MarketSnapshot::synthetic(100_000_000), |TakerFill { side, size, .. }| {
                let res = mollusk.process_instruction(&market_ctx.market_order(
                    taker_5.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
                increment();
                assert!(res.program_result.is_ok());
            }) => {},
            _ = tokio::time::sleep(Duration::from_secs(TEST_DURATION)) => { println!("Test complete!") },
        }

        // Ensure the `start_paused` `tokio` feature works with the taker bot activity profile
        // intervals as expected by checking that the number of takes is a reasonably large amount.
        assert!(num_takes.into_inner() > 100);

        Ok(())
    }
}
