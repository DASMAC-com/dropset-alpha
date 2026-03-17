use std::time::Duration;

use rand::prelude::*;
use rand_distr::{
    Distribution,
    LogNormal,
    Poisson,
};
use tokio::time::{
    Interval,
    MissedTickBehavior,
};

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
            interval: tokio::time::interval(Duration::from_millis(3000)),
            lambda_quiet: 0.2,
            lambda_burst: 2.5,
            burst_entry_prob: 0.05,
            burst_exit_prob: 0.4,
        }
    }

    /// A normal retail taker: occasional bursts.
    pub fn retail() -> Self {
        Self {
            interval: tokio::time::interval(Duration::from_millis(750)),
            lambda_quiet: 0.5,
            lambda_burst: 5.0,
            burst_entry_prob: 0.1,
            burst_exit_prob: 0.3,
        }
    }

    /// An aggressive taker: frequent, intense bursts.
    pub fn aggressive() -> Self {
        // This taker takes so often in the tests that it needs to skip missed ticks since `mollusk`
        // instruction processing is bottlenecked by the single-threaded runtime.
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self {
            interval,
            lambda_quiet: 1.0,
            lambda_burst: 12.0,
            burst_entry_prob: 0.2,
            burst_exit_prob: 0.2,
        }
    }
}

pub struct TakerStrategy {
    pub activity_profile: ActivityProfile,
    /// The median order size in atoms. A developer-friendly representation of `mu`.
    pub median_size: u64,
    /// The spread multiplier for order sizes, based around the [TakerStrategy::median_size].
    /// A value of 2 here would mean that order sizes range roughly from median/2 to median*2.
    /// A developer-friendly representation of `sigma`.
    pub spread_multiplier: f64,
    /// Probability this taker's next order is a buy.
    pub buy_bias: f64,
    /// Bias drifts: after a buy burst, lean sell (mean reversion behaviour).
    pub bias_reversion: f64,

    /// `mu` parameter for the underlying normal distribution.
    /// This is just the natural logarithm of [TakerStrategy::median_size].
    size_mu: f64,
    /// `sigma` parameter for the underlying normal distribution.
    /// This is just the natural logarithm of [TakerStrategy::spread_multiplier].
    size_sigma: f64,

    in_burst: bool,
    rng: StdRng,
}

impl TakerStrategy {
    pub fn new(
        activity_profile: ActivityProfile,
        median_size: u64,
        spread_multiplier: f64,
        buy_bias: f64,
        seed: u64,
    ) -> Self {
        Self {
            activity_profile,
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

                TakerFill { side, size }
            })
            .collect()
    }

    /// Creates the interval loop based on the taker's activity profile and taker strategy, calling
    /// `on_fill` every [ActivityProfile::interval] milliseconds.
    pub async fn interval_loop(mut self, mut on_fill: impl FnMut(TakerFill)) {
        loop {
            self.activity_profile.interval.tick().await;
            for fill in self.step() {
                on_fill(fill);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use client::{
        e2e_helpers::test_accounts,
        mollusk_helpers::{
            market_checker::MarketChecker,
            new_dropset_mollusk_context_with_default_market,
            utils::create_mock_user_account,
        },
    };
    use dropset_interface::{
        instructions::{
            BatchReplaceInstructionData,
            MarketOrderInstructionData,
            UnvalidatedOrders,
        },
        state::{
            sector::{
                MAX_PERMITTED_SECTOR_INCREASE,
                NIL,
            },
            user_order_sectors::MAX_ORDERS_USIZE,
        },
    };
    use price::client_helpers::to_order_info_args;
    use rust_decimal::{
        prelude::{
            FromPrimitive,
            ToPrimitive,
        },
        Decimal,
    };
    use solana_account::Account;
    use solana_address::Address;
    use solana_instruction::Instruction;
    use solana_keypair::Signer;

    use super::*;

    /// Creates a [Taker] with a moderate activity profile and order size.
    pub fn retail(median_size: u64, seed: u64) -> TakerStrategy {
        TakerStrategy::new(ActivityProfile::retail(), median_size, 2.0, 0.5, seed)
    }

    /// Creates a [Taker] with a high activity profile, large order sizes, and directional bias
    /// with fat tail sizes (high sigma, aka large spread multiplier).
    pub fn whale(median_size: u64, seed: u64) -> TakerStrategy {
        TakerStrategy::new(ActivityProfile::aggressive(), median_size, 5.0, 0.6, seed)
    }

    /// Creates a [Taker] with a passive activity profile, moderate order sizes, no directional
    /// bias, and very low spread multiplier.
    pub fn sniper(median_size: u64, seed: u64) -> TakerStrategy {
        TakerStrategy::new(ActivityProfile::passive(), median_size, 1.5, 0.5, seed)
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
            let all_fills: Vec<TakerFillWithAddress> = (0..self.n_steps)
                .flat_map(|_| {
                    self.taker_addresses
                        .iter()
                        .zip(&mut self.taker_strategies)
                        .flat_map(|(address, strategy)| {
                            strategy
                                .step()
                                .into_iter()
                                .map(|fill| TakerFillWithAddress {
                                    address: *address,
                                    side: fill.side,
                                    size: fill.size,
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

    #[test]
    fn poisson_takers_pure_simulation() {
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
    async fn poisson_takers_dropset_market() -> anyhow::Result<()> {
        const TEST_DURATION: u64 = 5;

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
        let bids = core::array::from_fn::<_, MAX_ORDERS_USIZE, _>(|i| {
            maker_order_at_price(10 - i, INITIAL_QUOTE / 10, false)
        });
        let asks = core::array::from_fn::<_, MAX_ORDERS_USIZE, _>(|i| {
            maker_order_at_price(11 + i, INITIAL_BASE / 10, true)
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

        checker.num_asks(10);
        checker.num_bids(10);
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

        tokio::select! {
            _ = taker_1.strategy.interval_loop(|TakerFill { side, size }| {
                mollusk.process_instruction(&market_ctx.market_order(
                    taker_1.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
            }) => {},
            _ = taker_2.strategy.interval_loop(|TakerFill { side, size }| {
                mollusk.process_instruction(&market_ctx.market_order(
                    taker_2.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
            }) => {},
            _ = taker_3.strategy.interval_loop(|TakerFill { side, size }| {
                mollusk.process_instruction(&market_ctx.market_order(
                    taker_3.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
            }) => {},
            _ = taker_4.strategy.interval_loop(|TakerFill { side, size }| {
                mollusk.process_instruction(&market_ctx.market_order(
                    taker_4.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
            }) => {},
            _ = taker_5.strategy.interval_loop(|TakerFill { side, size }| {
                mollusk.process_instruction(&market_ctx.market_order(
                    taker_5.address,
                    MarketOrderInstructionData::new(size, side.is_buy(), true),
                ));
            }) => {},
            _ = tokio::time::sleep(Duration::from_secs(TEST_DURATION)) => { println!("Test complete!") },
        }

        Ok(())
    }
}
