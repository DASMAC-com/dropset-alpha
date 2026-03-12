use std::{
    fs,
    io::ErrorKind,
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::Context;
use dropset_services_shared::{
    config::ValidSharedConfig,
    oanda_types::CurrencyPair,
};
use reqwest::Url;
use serde::Deserialize;

pub fn maker_config_dir() -> PathBuf {
    std::env::var("MAKER_CONFIG_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string())
        .into()
}

pub fn maker_keypair_path() -> PathBuf {
    maker_config_dir().join("maker-keypair.json")
}

pub fn maker_config_path() -> PathBuf {
    maker_config_dir().join("maker.toml")
}

pub fn maker_example_config_path() -> PathBuf {
    maker_config_dir().join("maker.toml.example")
}

pub struct ValidMakerConfig {
    pub oanda_auth_token: String,
    pub pair: CurrencyPair,
    pub shared: ValidSharedConfig,
    pub target_base: u64,
    pub batch_replace: bool,
    pub base_order_size: u64,
    pub quote_order_size: u64,
    pub ws_url: Url,
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
    pub base_order_size: u64,
    pub quote_order_size: u64,
    pub rpc_url: String,
    pub ws_url: String,
    pub price_feed_poll_interval: u64,
    pub order_update_throttle_window: u64,
}

pub fn validate_config(path: &Path, input: MakerConfigInput) -> anyhow::Result<ValidMakerConfig> {
    let MakerConfigInput {
        oanda_auth_token,
        pair,
        target_base,
        batch_replace,
        base_mint: base_mint_input,
        quote_mint: quote_mint_input,
        base_order_size,
        quote_order_size,
        rpc_url: rpc_url_input,
        ws_url: ws_url_input,
        price_feed_poll_interval,
        order_update_throttle_window,
    } = input;

    if oanda_auth_token.is_empty() || oanda_auth_token == "your-token-here" {
        anyhow::bail!(
            "oanda_auth_token in '{}' is not set.\n\
                 Edit the file and replace the placeholder with your OANDA API token.",
            path.display()
        );
    }

    let ws_url = Url::try_from(ws_url_input.as_str())
        .context(format!("Invalid WS url: {}", ws_url_input))?;

    let shared = ValidSharedConfig::new(
        maker_keypair_path(),
        base_mint_input,
        quote_mint_input,
        rpc_url_input,
    )?;

    Ok(ValidMakerConfig {
        shared,
        oanda_auth_token,
        pair,
        target_base,
        batch_replace,
        base_order_size,
        quote_order_size,
        ws_url,
        price_feed_poll_interval,
        order_update_throttle_window,
    })
}

pub fn load_config_and_validate() -> anyhow::Result<ValidMakerConfig> {
    let path = &maker_config_path();
    let raw = fs::read_to_string(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow::anyhow!(
            "Config file not found at '{}'.\n\
                 Copy the template and fill in your OANDA token:\n\n\
                 \tcp {:#?} \\\n\
                 \t   {:#?}\n",
            path.display(),
            maker_example_config_path(),
            maker_config_path(),
        ),
        _ => anyhow::anyhow!("Failed to read config file: '{}': {e}", path.display()),
    })?;

    let config: MakerConfigInput = toml::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("Failed to parse '{}': {e}", path.display()))?;

    validate_config(path, config)
}
