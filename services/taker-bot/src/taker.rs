use std::time::Duration;

use rand::prelude::*;
use rand_distr::{
    Distribution,
    LogNormal,
    Poisson,
};
use solana_address::Address;

#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct TakerFill {
    pub address: Address,
    pub side: Side,
    pub size: u64,
}

/// Controls the activity profile for a taker. This indicates how frequently a taker places orders
/// and how often they "burst" orders. A burst is a short window of
/// elevated λ, followed by quiet. This is the key to realistic CLOB flow.
#[derive(Clone)]
pub struct ActivityProfile {
    /// The time in milliseconds between periods of activity.
    pub interval: u64,
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
            interval: 2000,
            lambda_quiet: 0.2,
            lambda_burst: 2.5,
            burst_entry_prob: 0.05,
            burst_exit_prob: 0.4,
        }
    }

    /// A normal retail taker: occasional bursts.
    pub fn retail() -> Self {
        Self {
            interval: 400,
            lambda_quiet: 0.5,
            lambda_burst: 5.0,
            burst_entry_prob: 0.1,
            burst_exit_prob: 0.3,
        }
    }

    /// An aggressive taker: frequent, intense bursts.
    pub fn aggressive() -> Self {
        Self {
            interval: 50,
            lambda_quiet: 1.0,
            lambda_burst: 12.0,
            burst_entry_prob: 0.2,
            burst_exit_prob: 0.2,
        }
    }
}

pub struct Taker {
    pub address: Address,
    pub activity_profile: ActivityProfile,
    /// The median order size in atoms. A developer-friendly representation of `mu`.
    pub median_size: u64,
    /// The spread multiplier for order sizes, based around the [Taker::median_size].
    /// A value of 2 here would mean that order sizes range roughly from median/2 to median*2.
    /// A developer-friendly representation of `sigma`.
    pub spread_multiplier: f64,
    /// Probability this taker's next order is a buy.
    pub buy_bias: f64,
    /// Bias drifts: after a buy burst, lean sell (mean reversion behaviour).
    pub bias_reversion: f64,

    /// `mu` parameter for the underlying normal distribution.
    /// This is just the natural logarithm of [Taker::median_size].
    size_mu: f64,
    /// `sigma` parameter for the underlying normal distribution.
    /// This is just the natural logarithm of [Taker::spread_multiplier].
    size_sigma: f64,

    in_burst: bool,
    rng: StdRng,
}

impl Taker {
    pub fn new(
        address: Address,
        profile: ActivityProfile,
        median_size: u64,
        spread_multiplier: f64,
        buy_bias: f64,
        seed: u64,
    ) -> Self {
        Self {
            address,
            activity_profile: profile,
            median_size,
            spread_multiplier,
            buy_bias,
            bias_reversion: 0.05,
            size_mu: (median_size as f64).ln(),
            size_sigma: spread_multiplier.ln(),
            in_burst: false,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Convenience constructors for demo variety
    pub fn retail(address: Address, seed: u64) -> Self {
        Self::new(address, ActivityProfile::retail(), 3000, 2.0, 0.5, seed)
    }

    pub fn whale(address: Address, seed: u64) -> Self {
        // Large sizes, fat tail (high sigma), directional bias
        Self::new(
            address,
            ActivityProfile::aggressive(),
            15000,
            4.0,
            0.6,
            seed,
        )
    }

    pub fn sniper(address: Address, seed: u64) -> Self {
        // Rare but precise: quiet most of the time, sudden sharp bursts
        Self::new(address, ActivityProfile::passive(), 3000, 2.0, 0.5, seed)
    }

    /// A single moment of market activity between idle intervals.
    /// Called repeatedly by the taker's task loop every `interval_ms`.
    /// Returns zero or more fills depending on burst state and Poisson draw.
    pub fn step(&mut self) -> Vec<TakerFill> {
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
        let n_orders = Poisson::new(lambda).unwrap().sample(&mut self.rng) as u64;

        if n_orders == 0 {
            return vec![];
        }

        let size_dist = LogNormal::new(self.size_mu, self.size_sigma).unwrap();

        (0..n_orders)
            .map(|_| {
                let side = if self.rng.random_bool(self.buy_bias) {
                    Side::Buy
                } else {
                    Side::Sell
                };

                // Nudge bias toward 0.5 after each order (slight mean reversion)
                self.buy_bias += self.bias_reversion * (0.5 - self.buy_bias);

                let size = size_dist.sample(&mut self.rng).max(1.0) as u64;

                TakerFill {
                    address: self.address,
                    side,
                    size,
                }
            })
            .collect()
    }

    /// Spawns a [tokio] task based on the taker's configuration where `on_fill` is called every
    /// [ActivityProfile::interval] milliseconds.
    pub fn into_task(
        mut self,
        mut on_fill: impl FnMut(TakerFill) + Send + 'static,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(self.activity_profile.interval));
            loop {
                interval.tick().await;
                for fill in self.step() {
                    on_fill(fill);
                }
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use client::e2e_helpers::test_accounts;
    use solana_keypair::Signer;

    use super::*;

    pub struct Simulation {
        pub takers: Vec<Taker>,
        pub n_steps: usize,
    }

    impl Simulation {
        pub fn new(takers: Vec<Taker>, n_steps: usize) -> Self {
            Self { takers, n_steps }
        }

        /// Step each taker `n_steps` times, collecting all fills.
        pub fn run(&mut self) -> Vec<TakerFill> {
            let all_fills: Vec<TakerFill> = (0..self.n_steps)
                .flat_map(|_| {
                    self.takers
                        .iter_mut()
                        .flat_map(|taker| taker.step())
                        .collect::<Vec<_>>()
                })
                .collect();

            all_fills
        }
    }

    fn display_addr(taker_addr: &Address) -> String {
        format!("0x{}", &taker_addr.to_string()[0..4])
    }

    #[test]
    fn poisson_takers_pure_simulation() {
        let takers = vec![
            Taker::retail(test_accounts::acc_1111().pubkey(), 42),
            Taker::retail(test_accounts::acc_4444().pubkey(), 555),
            Taker::retail(test_accounts::acc_AAAA().pubkey(), 137),
            Taker::whale(test_accounts::acc_CCCC().pubkey(), 9001),
            Taker::sniper(test_accounts::acc_FFFF().pubkey(), 31337),
        ];

        let taker_addresses: Vec<Address> = takers.iter().map(|t| t.address).collect();
        let mut sim = Simulation::new(takers, 50);
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
        for address in taker_addresses {
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
}
