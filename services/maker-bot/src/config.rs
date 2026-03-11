use std::{
    fs,
    io::ErrorKind,
    path::{
        Path,
        PathBuf,
    },
    str::FromStr,
};

use anyhow::Context;
use dropset_services_shared::oanda_types::CurrencyPair;
use reqwest::Url;
use serde::Deserialize;
use solana_address::Address;
use solana_keypair::Keypair;

pub fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("maker.toml")
}

pub fn example_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("maker.toml.example")
}

pub struct ValidMakerConfig {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub target_base: u64,
    pub batch_replace: bool,
    pub base_mint: Address,
    pub quote_mint: Address,
    pub order_size: u64,
    pub rpc_url: Url,
    pub ws_url: Url,
    pub maker_keypair: Keypair,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
}

#[derive(Deserialize)]
pub struct MakerConfigInput {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub target_base: u64,
    pub batch_replace: bool,
    pub base_mint: String,
    pub quote_mint: String,
    pub order_size: u64,
    pub rpc_url: String,
    pub ws_url: String,
    pub maker_kp_file_path: String,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
}

pub fn validate_config(path: &Path, input: MakerConfigInput) -> anyhow::Result<ValidMakerConfig> {
    if input.oanda_auth_token.is_empty() || input.oanda_auth_token == "your-token-here" {
        anyhow::bail!(
            "oanda_auth_token in '{}' is not set.\n\
                 Edit the file and replace the placeholder with your OANDA API token.",
            path.display()
        );
    }

    let (base_mint_str, quote_mint_str) = (&input.base_mint, &input.quote_mint);
    let base_mint = Address::from_str(&input.base_mint).with_context(|| {
        anyhow::anyhow!("Couldn't convert base_mint `{base_mint_str}` to address")
    })?;
    let quote_mint = Address::from_str(&input.quote_mint).with_context(|| {
        anyhow::anyhow!("Couldn't convert quote_mint `{quote_mint_str}` to address")
    })?;

    let maker_kp_fp = &input.maker_kp_file_path;
    let maker_kp_file = fs::File::open(&input.maker_kp_file_path)
        .with_context(|| anyhow::anyhow!("Couldn't open maker keypair file: {maker_kp_fp}"))?;
    let maker_keypair_bytes: Vec<u8> = serde_json::from_reader(maker_kp_file)?;
    let maker_keypair = Keypair::try_from(maker_keypair_bytes.as_slice())?;

    let rpc_url = Url::try_from(input.rpc_url.as_str())
        .context(format!("Invalid RPC url: {}", input.rpc_url))?;
    let ws_url = Url::try_from(input.ws_url.as_str())
        .context(format!("Invalid WS url: {}", input.ws_url))?;

    Ok(ValidMakerConfig {
        oanda_auth_token: input.oanda_auth_token,
        pair: input.pair,
        target_base: input.target_base,
        batch_replace: input.batch_replace,
        base_mint,
        quote_mint,
        order_size: input.order_size,
        rpc_url,
        ws_url,
        maker_keypair,
        price_feed_poll_interval: input.price_feed_poll_interval,
        order_update_throttle_window: input.order_update_throttle_window,
    })
}

pub fn load_config_and_validate() -> anyhow::Result<ValidMakerConfig> {
    let path = &config_path();
    let raw = fs::read_to_string(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow::anyhow!(
            "Config file not found at '{}'.\n\
                 Copy the template and fill in your OANDA token:\n\n\
                 \tcp {:#?} \\\n\
                 \t   {:#?}\n",
            path.display(),
            example_config_path(),
            config_path(),
        ),
        _ => anyhow::anyhow!("Failed to read config file: '{}': {e}", path.display()),
    })?;

    let config: MakerConfigInput = toml::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("Failed to parse '{}': {e}", path.display()))?;

    validate_config(path, config)
}
