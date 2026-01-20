//! Creates a market making bot that utilizes the strategy defined in [`crate::calculate_spreads`].

use crate::{
    cli::initialize_context_from_cli,
    oanda::{
        poll_price_feed,
        CandlestickGranularity,
        OandaArgs,
    },
};

mod program_subscribe;
use program_subscribe::program_subscribe;

pub mod calculate_spreads;
pub mod maker_context;
pub mod oanda;

pub mod cli;
pub mod load_env;

const WS_URL: &str = "ws://localhost:8900";
pub const GRANULARITY: CandlestickGranularity = CandlestickGranularity::M15;
pub const NUM_CANDLES: u64 = 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize things.
    let reqwest_client = reqwest::Client::new();
    let maker_context = initialize_context_from_cli(&reqwest_client).await?;

    tokio::select! {
        result_1 = program_subscribe(WS_URL) => {
            println!("Program subscription errored out: {result_1:#?}");
        },
        result_2 = poll_price_feed(reqwest_client, OandaArgs {
            auth_token: load_env::oanda_auth_token(),
            pair: maker_context.pair,
            granularity: GRANULARITY,
            num_candles: NUM_CANDLES
        }) => {
            println!("Price feed poller errored out: {result_2:#?}");
        }
    }

    Ok(())
}
