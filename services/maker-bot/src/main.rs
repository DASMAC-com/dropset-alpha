//! Creates a market making bot that utilizes the strategy defined in
//! [`crate::model::calculate_spreads`].

pub mod config;
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
    rc::Rc,
};

use client::transactions::{
    CustomRpcClient,
    SendTransactionConfig,
};
use dropset_services_shared::oanda_types::CandlestickGranularity;
use maker_state::*;
use order_flow::*;
use strum_macros::Display;
use tokio::sync::watch;

use crate::{
    config::load_config_and_validate,
    maker_context::MakerContext,
    oanda_price_feed::OandaArgs,
};

const WS_URL: &str = "ws://localhost:8900";
pub const GRANULARITY: CandlestickGranularity = CandlestickGranularity::M15;
pub const NUM_CANDLES: u64 = 1;

#[derive(Debug, Copy, Clone, Display)]
pub enum TaskUpdate {
    MakerState,
    Price,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let rpc = CustomRpcClient::new(
        None,
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(true),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    );

    rpc.validate_endpoint().await?;
    let reqwest_client = reqwest::Client::new();

    let cfg = load_config_and_validate()?;
    let poll_interval = cfg.price_feed_poll_interval;
    let throttle_window = cfg.order_update_throttle_window;

    let oanda_args = OandaArgs {
        auth_token: cfg.oanda_auth_token.clone(),
        pair: cfg.pair,
        granularity: GRANULARITY,
        num_candles: NUM_CANDLES,
    };

    let ctx = MakerContext::init(&rpc, &oanda_args, &reqwest_client, cfg).await?;
    let maker_ctx = Rc::new(RefCell::new(ctx));

    // Create the sender/receiver to facilitate notifications of mutations from the program
    // subscription and price feed poller tasks.
    let (sender, receiver) = watch::channel(TaskUpdate::MakerState);

    tokio::select! {
        r1 = tasks::program_subscribe(maker_ctx.clone(), sender.clone(), WS_URL) => {
            println!("Program subscription terminated: {r1:#?}");
        },
        r2 = tasks::poll_price_feed(maker_ctx.clone(), sender.clone(), reqwest_client, &oanda_args, poll_interval) => {
            println!("Price feed poll loop terminated: {r2:#?}");
        },
        r3 = tasks::throttled_order_update(maker_ctx.clone(), receiver, &rpc, throttle_window) => {
            println!("Throttled order update loop terminated: {r3:#?}");
        }
    }

    Ok(())
}
