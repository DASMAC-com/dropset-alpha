use std::{
    fs::File,
    path::PathBuf,
};

use clap::Parser;
use client::transactions::CustomRpcClient;
use solana_address::Address;
use solana_keypair::Keypair;

use crate::{
    load_env::oanda_auth_token,
    maker_context::MakerContext,
    oanda::{
        query_price_feed,
        CurrencyPair,
        OandaArgs,
    },
    GRANULARITY,
    NUM_CANDLES,
};

#[derive(Parser)]
#[command(name = "market-maker")]
pub struct CliArgs {
    /// Base mint address.
    #[arg(short = 'b', long)]
    pub base_mint: Address,

    /// Quote mint address.
    #[arg(short = 'q', long)]
    pub quote_mint: Address,

    /// The [`CurrencyPair`] as a string. The format is `{BASE}_{QUOTE}`; e.g. `EUR_USD`.
    #[arg(short = 'p', long)]
    pub pair: CurrencyPair,

    /// The target base inventory in atoms that the model implementation will gravitate towards.
    /// This value is absolute, meaning a passed value of zero when the maker has existing base
    /// already will result in the maker immediately placing aggressive asks and passive/wide bids.
    #[arg(long)]
    pub target_base: u64,

    /// Path to the maker's keypair file.
    #[arg(short = 'k', long)]
    pub keypair: PathBuf,
}

/// Loads the maker context from passed CLI arguments and a few expected environment variables.
/// See [`crate::load_env`] for the expected environment variables.
pub async fn initialize_context_from_cli(
    rpc: &CustomRpcClient,
    reqwest_client: &reqwest::Client,
) -> anyhow::Result<MakerContext> {
    let CliArgs {
        base_mint,
        quote_mint,
        pair,
        target_base,
        keypair,
    } = CliArgs::parse();

    let initial_price_feed_response = query_price_feed(
        &OandaArgs {
            auth_token: oanda_auth_token(),
            pair,
            granularity: GRANULARITY,
            num_candles: NUM_CANDLES,
        },
        reqwest_client,
    )
    .await?;

    let bytes: Vec<u8> = serde_json::from_reader(File::open(&keypair)?)?;
    let maker_keypair = Keypair::try_from(bytes.as_slice())?;

    MakerContext::init(
        rpc,
        maker_keypair,
        base_mint,
        quote_mint,
        pair,
        target_base,
        initial_price_feed_response,
    )
    .await
}
