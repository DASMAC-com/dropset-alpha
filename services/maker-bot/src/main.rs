//! Creates a market making bot that utilizes the strategy defined in
//! [`crate::model::calculate_spreads`].

pub mod config;
pub mod logger;
pub mod maker_context;
pub mod maker_state;
pub mod model;
pub mod oanda_price_feed;
pub mod order_flow;
pub mod tasks;
pub mod utils;

use std::{
    cell::RefCell,
    collections::HashSet,
    fmt,
    rc::Rc,
};

use client::transactions::{
    CustomRpcClient,
    SendTransactionConfig,
};
use dropset_services_shared::{
    faucet_client::FaucetClient,
    oanda_types::CandlestickGranularity,
};
use maker_state::*;
use order_flow::*;
use rust_decimal::Decimal;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};
use tokio::sync::watch;

use crate::{
    config::get_validated_config,
    maker_context::MakerContext,
};

pub const GRANULARITY: CandlestickGranularity = CandlestickGranularity::M15;
pub const NUM_CANDLES: u64 = 1;

#[derive(Debug, Copy, Clone)]
pub enum TaskUpdate {
    MakerState,
    Price(Decimal),
}

impl fmt::Display for TaskUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskUpdate::MakerState => write!(f, "Orders filled"),
            TaskUpdate::Price(p) => write!(f, "Price feed update — {p}"),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let health_check = std::env::args().any(|a| a == "--health-check");
    let cfg = get_validated_config().await?;
    let reqwest_client = reqwest::Client::new();
    if health_check {
        return Ok(());
    }

    let poll_interval = cfg.price_feed_poll_interval;
    let throttle_window = cfg.order_update_throttle_window;
    let ws_url = cfg.ws_url.clone();

    let rpc = CustomRpcClient::new(
        Some(RpcClient::new_with_commitment(
            cfg.shared.rpc_url.clone().to_string(),
            CommitmentConfig::confirmed(),
        )),
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(!cfg.visualize),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    );

    let oanda_args = cfg.oanda_args.clone();
    let faucet_client = FaucetClient::new(&cfg.shared).await;
    let ctx = MakerContext::init(&rpc, cfg).await?;
    let maker_ctx = Rc::new(RefCell::new(ctx));

    // Create the sender/receiver to facilitate notifications of mutations from the program
    // subscription and price feed poller tasks.
    let (sender, receiver) = watch::channel(TaskUpdate::MakerState);

    tokio::select! {
        r1 = tasks::program_subscribe(maker_ctx.clone(), sender.clone(), ws_url.as_str()) => {
            println!("Program subscription terminated: {r1:#?}");
        },
        r2 = tasks::poll_price_feed(maker_ctx.clone(), sender.clone(), reqwest_client, &oanda_args, poll_interval) => {
            println!("Price feed poll loop terminated: {r2:#?}");
        },
        r3 = tasks::throttled_order_update(maker_ctx.clone(), receiver, &rpc, throttle_window, faucet_client) => {
            println!("Throttled order update loop terminated: {r3:#?}");
        }
    }

    Ok(())
}
