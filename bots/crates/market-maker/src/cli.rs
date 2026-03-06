use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;
use client::transactions::CustomRpcClient;
use serde::Deserialize;
use solana_address::Address;
use solana_keypair::Keypair;

use crate::{
    maker_context::{
        MakerContext,
        MakerContextInitArgs,
    },
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
    /// Path to the maker's keypair file.
    #[arg(short = 'k', long)]
    pub keypair: PathBuf,

    /// Path to the config file (defaults to <crate-dir>/config.toml).
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,
}

fn default_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml")
}

#[derive(Deserialize)]
pub struct Config {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub target_base: u64,
    pub batch_replace: bool,
    pub base_mint: String,
    pub quote_mint: String,
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => anyhow::anyhow!(
                "Config file not found at '{}'.\n\
                 Copy the template and fill in your OANDA token:\n\n\
                 \tcp bots/crates/market-maker/config.toml.example \\\n\
                 \t   bots/crates/market-maker/config.toml\n",
                path.display()
            ),
            _ => anyhow::anyhow!("Failed to read config file: '{}': {e}", path.display()),
        })?;

        let config: Self = toml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse '{}': {e}", path.display()))?;

        if config.oanda_auth_token.is_empty() || config.oanda_auth_token == "your-token-here" {
            anyhow::bail!(
                "oanda_auth_token in '{}' is not set.\n\
                 Edit the file and replace the placeholder with your OANDA API token.",
                path.display()
            );
        }

        Ok(config)
    }
}

/// Loads the maker context from the CLI args and config file.
/// Returns the context and the OANDA auth token (needed by the polling loop in main).
pub async fn initialize_context_from_cli(
    rpc: &CustomRpcClient,
    reqwest_client: &reqwest::Client,
) -> anyhow::Result<(MakerContext, String)> {
    let CliArgs { keypair, config } = CliArgs::parse();

    let config_path = config.unwrap_or_else(default_config_path);
    let cfg = Config::load(&config_path)?;
    let auth_token = cfg.oanda_auth_token;

    let initial_price_feed_response = query_price_feed(
        &OandaArgs {
            auth_token: auth_token.clone(),
            pair: cfg.pair,
            granularity: GRANULARITY,
            num_candles: NUM_CANDLES,
        },
        reqwest_client,
    )
    .await?;

    for (field, val) in [
        ("base_mint", &cfg.base_mint),
        ("quote_mint", &cfg.quote_mint),
    ] {
        if val.is_empty() {
            anyhow::bail!(
                "'{field}' in config.toml is not set.\n\
                 Run the script to initialize a market automatically:\n\n\
                 \tbash bots/crates/market-maker/market-maker.sh\n\n\
                 Or run initialization_helper manually and fill in the value."
            );
        }
    }

    let bytes: Vec<u8> = serde_json::from_reader(fs::File::open(&keypair)?)?;
    let maker_keypair = Keypair::try_from(bytes.as_slice())?;

    let ctx = MakerContext::init(MakerContextInitArgs {
        rpc,
        maker: maker_keypair,
        base_mint: Address::from_str(&cfg.base_mint)?,
        quote_mint: Address::from_str(&cfg.quote_mint)?,
        pair: cfg.pair,
        base_target_atoms: cfg.target_base,
        initial_price_feed_response,
        batch_replace: cfg.batch_replace,
    })
    .await?;

    Ok((ctx, auth_token))
}
