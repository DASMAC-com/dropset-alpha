use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    str::FromStr,
};

use anyhow::Context;
use client::transactions::CustomRpcClient;
use reqwest::Url;
use serde::de::DeserializeOwned;
use solana_address::Address;
use solana_keypair::{Keypair, Signer};

fn services_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("services")
}

#[derive(Copy, Clone)]
pub enum Service {
    Maker,
    Taker,
    Faucet,
}

impl Service {
    pub fn config_dir(&self) -> PathBuf {
        match self {
            Self::Maker => services_dir().join("maker-bot"),
            Self::Taker => services_dir().join("taker-bot"),
            Self::Faucet => services_dir().join("faucet"),
        }
    }

    pub fn keypair_path(&self) -> PathBuf {
        self.config_dir().join("keypair.json")
    }

    pub fn toml_config_path(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    pub fn toml_config_example_path(&self) -> PathBuf {
        self.config_dir().join("config.toml.example")
    }
}

/// Validated config inputs.
pub struct ValidSharedConfig {
    pub keypair: Keypair,
    pub base_mint: Address,
    pub quote_mint: Address,
    pub rpc_url: Url,
}

impl ValidSharedConfig {
    pub async fn new(
        keypair_path: PathBuf,
        base_mint: String,
        quote_mint: String,
        rpc_url: String,
    ) -> Result<Self, anyhow::Error> {
        let base_mint = Address::from_str(&base_mint).context(anyhow::anyhow!(
            "Couldn't convert base mint `{}` to address",
            &base_mint
        ))?;
        let quote_mint = Address::from_str(&quote_mint).context(anyhow::anyhow!(
            "Couldn't convert quote mint `{}` to address",
            &quote_mint
        ))?;

        let kp_file = fs::File::open(&keypair_path)
            .with_context(|| anyhow::anyhow!("Couldn't open keypair file: {:#?}", keypair_path))?;
        let kp_bytes: Vec<u8> = serde_json::from_reader(kp_file)?;
        let keypair = Keypair::try_from(kp_bytes.as_slice())?;

        let rpc_url =
            Url::try_from(rpc_url.as_str()).context(format!("Invalid RPC url: {}", rpc_url))?;

        CustomRpcClient::new_from_url(rpc_url.as_str(), Default::default())
            .validate_endpoint()
            .await?;

        Ok(Self {
            keypair,
            base_mint,
            quote_mint,
            rpc_url,
        })
    }

    /// Builds a [`ValidSharedConfig`] directly from a [`Service`]'s config.toml and keypair.json.
    pub async fn from_service(service: Service) -> anyhow::Result<Self> {
        #[derive(serde::Deserialize)]
        struct SharedFields {
            rpc_url: String,
            base_mint: String,
            quote_mint: String,
        }

        let cfg: SharedFields = deserialize_service_config(service)?;
        Self::new(service.keypair_path(), cfg.base_mint, cfg.quote_mint, cfg.rpc_url).await
    }

    pub fn address(&self) -> Address {
        self.keypair.pubkey()
    }
}

/// Converts a service's toml config file to a raw string, with helpful error messages
/// if the file doesn't exist or the expected file path is actually a directory.
pub fn load_raw_service_config(service: Service) -> anyhow::Result<String> {
    let path = &service.toml_config_path();
    let example_path = &service.toml_config_example_path();

    fs::read_to_string(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow::anyhow!(
            "Config file not found at '{}'.\n\
                 Copy the template and fill in empty fields:\n\n\
                 \tcp {} \\\n\
                 \t   {}\n",
            path.display(),
            example_path.display(),
            path.display(),
        ),
        ErrorKind::IsADirectory => anyhow::anyhow!(
            "Expected a config file at '{}' but found a directory.\n\
                 If running via Docker, this means the host-side config file does not exist — \
                 Docker created a directory at the volume mount target.\n\
                 Remove the directory, then copy the template and fill in empty fields:\n\n\
                 \trmdir {} && \\\n\
                 \tcp {} \\\n\
                 \t   {}\n",
            path.display(),
            path.display(),
            example_path.display(),
            path.display(),
        ),
        _ => anyhow::anyhow!("Failed to read config file: '{path:#?}': {e}"),
    })
}

/// Loads the raw toml config file for a service, printing helpful messages upon error.
pub fn deserialize_service_config<T: DeserializeOwned>(service: Service) -> anyhow::Result<T> {
    let path = &service.toml_config_path();
    let raw = load_raw_service_config(service)?;

    toml::from_str(&raw).map_err(|e| anyhow::anyhow!("Failed to parse '{path:#?}': {e}"))
}
