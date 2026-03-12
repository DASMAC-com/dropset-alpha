use std::{
    fs,
    path::PathBuf,
    str::FromStr,
};

use anyhow::Context;
use reqwest::Url;
use solana_address::Address;
use solana_keypair::Keypair;

fn services_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("services")
}

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
    pub fn new(
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
            "Couldn't convert quote_mint `{}` to address",
            &quote_mint
        ))?;

        let kp_file = fs::File::open(&keypair_path)
            .with_context(|| anyhow::anyhow!("Couldn't open keypair file: {:#?}", keypair_path))?;
        let kp_bytes: Vec<u8> = serde_json::from_reader(kp_file)?;
        let keypair = Keypair::try_from(kp_bytes.as_slice())?;

        let rpc_url =
            Url::try_from(rpc_url.as_str()).context(format!("Invalid RPC url: {}", rpc_url))?;

        Ok(Self {
            keypair,
            base_mint,
            quote_mint,
            rpc_url,
        })
    }
}
