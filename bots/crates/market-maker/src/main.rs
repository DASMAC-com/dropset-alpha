//! Creates a market making bot that utilizes the strategy defined in [`crate::calculate_spreads`].

use std::{
    str::FromStr,
    sync::LazyLock,
    time::Duration,
};

use anyhow::Context;
use dropset_interface::state::market_header::MARKET_ACCOUNT_DISCRIMINANT;
use solana_address::Address;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{
        CommitmentConfig,
        RpcAccountInfoConfig,
        RpcProgramAccountsConfig,
    },
    rpc_filter::{
        Memcmp,
        RpcFilterType,
    },
};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use transaction_parser::views::try_market_view_all_from_owner_and_data;

use crate::oanda::{
    query_price_feed,
    CandlestickGranularity,
    Currency,
    CurrencyPair,
};

pub mod calculate_spreads;
pub mod maker_context;
pub mod oanda;

const WS_URL: &str = "ws://localhost:8900";
const CURRENCY_PAIR: CurrencyPair = CurrencyPair {
    base: Currency::EUR,
    quote: Currency::USD,
};
const GRANULARITY: CandlestickGranularity = CandlestickGranularity::M15;
const NUM_CANDLES: u64 = 1;
const POLL_INTERVAL_MS: u64 = 1000;

fn oanda_auth_token() -> String {
    static TOKEN: LazyLock<String> = LazyLock::new(|| {
        std::env::var("OANDA_AUTH_TOKEN")
            .expect("Environment variable OANDA_AUTH_TOKEN must be set")
    });

    TOKEN.clone()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::select! {
        result_1 = spawn_program_subscription() => {
            println!("Program subscription errored out: {result_1:#?}");
        },
        result_2 = spawn_price_feed_poller() => {
            println!("Price feed poller errored out: {result_2:#?}");
        }
    }

    Ok(())
}

async fn spawn_program_subscription() -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let ws_client = PubsubClient::new(WS_URL).await?;

    let task = tokio::spawn(async move {
        let config = RpcProgramAccountsConfig {
            filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                0,
                MARKET_ACCOUNT_DISCRIMINANT.to_le_bytes().to_vec(),
            ))]),
            account_config: RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::confirmed()),
                encoding: Some(solana_client::rpc_config::UiAccountEncoding::Base64),
                data_slice: None,
                min_context_slot: None,
            },
            with_context: Some(true),
            sort_results: Some(true),
        };

        let (mut stream, _) = ws_client
            .program_subscribe(&dropset_interface::program::ID, Some(config))
            .await
            .context("Couldn't subscribe to program")?;

        while let Some(account) = stream.next().await {
            // This could be an unchecked transmutation since the account discriminant indicates
            // it's a valid market account, but it's a simple extra check.
            let owner = Address::from_str(account.value.account.owner.as_str())
                .expect("Should be a valid address");
            let account_data = account
                .value
                .account
                .data
                .decode()
                .expect("Should decode account data");
            let market_view = try_market_view_all_from_owner_and_data(owner, &account_data)
                .expect("Should convert to a valid market account's data");

            // For now debug with print statement, eventually, this will mutate the MakerContext
            // state and update it.
            println!("new maker state\n{market_view:#?}");
        }

        Ok::<(), anyhow::Error>(())
    });

    Ok(task)
}

async fn spawn_price_feed_poller() -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_millis(POLL_INTERVAL_MS));

    let task = tokio::spawn(async move {
        loop {
            interval.tick().await;

            match query_price_feed(
                &oanda_auth_token(),
                CURRENCY_PAIR,
                GRANULARITY,
                NUM_CANDLES,
                client.clone(),
            )
            .await
            {
                Ok(response) => println!("{response:?}"),
                Err(e) => eprintln!("Price feed error: {e:?}"),
            }
        }
    });

    Ok(task)
}
