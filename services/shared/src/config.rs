use std::{
    collections::HashSet,
    fs,
    io::ErrorKind,
    path::{
        Path,
        PathBuf,
    },
    str::FromStr,
};

use anyhow::Context;
use client::{
    context::token::TokenContext,
    transactions::CustomRpcClient,
};
use reqwest::Url;
use serde::de::DeserializeOwned;
use solana_address::Address;
use solana_cluster_type::ClusterType;
use solana_keypair::{
    read_keypair_file,
    Keypair,
    Signer,
};

fn services_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("services")
}

#[derive(Copy, Clone)]
pub enum ServiceConfig {
    Maker,
    Taker,
    Shared,
    Faucet,
}

impl ServiceConfig {
    pub fn config_dir(&self) -> PathBuf {
        match self {
            Self::Maker => services_dir().join("maker-bot"),
            Self::Taker => services_dir().join("taker-bot"),
            Self::Faucet => services_dir().join("faucet"),
            Self::Shared => services_dir().join("shared"),
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
    pub base: TokenContext,
    pub quote: TokenContext,
    pub rpc_url: Url,
    pub cluster: ClusterType,
    pub faucet_base_url: Url,
    pub faucet_port: u16,
    pub max_public_tokens: u64,
    pub max_allowlist_tokens: u64,
    pub allowlist: HashSet<Address>,
}
#[derive(serde::Deserialize)]
pub struct SharedConfigInput {
    pub base_mint: String,
    pub quote_mint: String,
    pub rpc_url: String,
    pub faucet_base_url: String,
    pub faucet_port: u16,
    pub max_public_tokens: u64,
    pub max_allowlist_tokens: u64,
    #[serde(default)]
    pub allowlist: Vec<String>,
}

impl ValidSharedConfig {
    pub async fn new_validated(service_config: ServiceConfig) -> Result<Self, anyhow::Error> {
        let shared_input = deserialize_service_config(ServiceConfig::Shared)?;
        let SharedConfigInput {
            base_mint,
            quote_mint,
            rpc_url,
            faucet_base_url: faucet_base_url_str,
            faucet_port,
            max_public_tokens,
            max_allowlist_tokens,
            allowlist,
        } = shared_input;

        // --- Validate the faucet and RPC urls.
        let faucet_base_url = Url::try_from(faucet_base_url_str.as_str())
            .with_context(|| format!("Invalid faucet url: {}", faucet_base_url_str))?;
        full_faucet_url(&faucet_base_url, faucet_port)?;

        let rpc_url = Url::try_from(rpc_url.as_str())
            .with_context(|| format!("Invalid RPC url: {}", rpc_url))?;
        let rpc = CustomRpcClient::new_from_url(rpc_url.as_str(), Default::default());
        rpc.validate_endpoint().await?;

        // --- Validate base/quote mint addresses
        let base_mint = Address::from_str(&base_mint).context(anyhow::anyhow!(
            "Couldn't convert base mint `{}` to address",
            &base_mint
        ))?;
        let quote_mint = Address::from_str(&quote_mint).context(anyhow::anyhow!(
            "Couldn't convert quote mint `{}` to address",
            &quote_mint
        ))?;

        // --- Validate base/quote token contexts.
        let base_msg = format!("Couldn't find base mint account on-chain: {base_mint}");
        let base_account = rpc
            .client
            .get_account(&base_mint)
            .await
            .with_context(|| base_msg)?;
        let base =
            TokenContext::from_account_data(base_mint, base_account.owner, &base_account.data)?;

        let quote_msg = format!("Couldn't find quote mint account on-chain: {quote_mint}");
        let quote_account = rpc
            .client
            .get_account(&quote_mint)
            .await
            .with_context(|| quote_msg)?;
        let quote =
            TokenContext::from_account_data(quote_mint, quote_account.owner, &quote_account.data)?;

        // --- Validate base/quote token contexts.
        let keypair_path = service_config.keypair_path();
        let keypair = read_keypair_file(&keypair_path).map_err(|e| {
            anyhow::anyhow!("Couldn't open keypair file: {keypair_path:#?}, err: ({e})",)
        })?;

        // --- Validate the cluster.
        let cluster = rpc.resolve_cluster().await?;
        anyhow::ensure!(
            cluster != ClusterType::MainnetBeta,
            "Refusing to operate against mainnet-beta. \
             These services are only for testnet/devnet/localnet."
        );

        // --- Validate the faucet config.
        if max_public_tokens == 0 {
            anyhow::bail!("Max public tokens must be greater than zero");
        }
        if max_allowlist_tokens == 0 {
            anyhow::bail!("Max allowlist tokens must be greater than zero");
        }
        let allowlist = allowlist
            .iter()
            .map(|s| {
                Address::from_str(s).with_context(|| format!("Invalid allowlist address: {s}"))
            })
            .collect::<anyhow::Result<HashSet<Address>>>()?;

        Ok(Self {
            keypair,
            base,
            quote,
            rpc_url,
            cluster,
            faucet_base_url,
            faucet_port,
            max_public_tokens,
            max_allowlist_tokens,
            allowlist,
        })
    }

    pub fn address(&self) -> Address {
        self.keypair.pubkey()
    }

    pub fn faucet_url(&self) -> Url {
        full_faucet_url(&self.faucet_base_url, self.faucet_port)
            .expect("Faucet base url and port should be valid")
    }
}

fn full_faucet_url(base_url: &Url, port: u16) -> anyhow::Result<Url> {
    if base_url.port().is_some() {
        anyhow::bail!("Base faucet url should not have a port number, got: {base_url}");
    }

    let mut res = base_url.clone();
    res.set_port(Some(port))
        .map_err(|_| anyhow::anyhow!("Couldn't set port on the faucet url"))?;

    Ok(res)
}

/// Converts a service's toml config file to a raw string, with helpful error messages
/// if the file doesn't exist or the expected file path is actually a directory.
pub fn load_raw_service_config(service_config: ServiceConfig) -> anyhow::Result<String> {
    let path = &service_config.toml_config_path();
    let example_path = &service_config.toml_config_example_path();

    /// Only display the path after `services/`, since if this is running in a Docker container it
    /// will display the container's path instead of the host path.
    /// The host path is the intended displayed path here since it's what's actually mounted to the
    /// container, but that's not readily available, so a relative path will have to suffice.
    fn relative_path_starting_from_services(path: &Path) -> anyhow::Result<&Path> {
        Ok(path.strip_prefix(
            services_dir()
                .parent()
                .expect("Services directory should have a parent"),
        )?)
    }

    let relative_config_path =
        relative_path_starting_from_services(path).expect("Services config path should exist");
    let relative_config_example_path = relative_path_starting_from_services(example_path)
        .expect("Services example config path should exist");

    fs::read_to_string(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow::anyhow!(
            "Config file not found at '{}'.\n\
                 Copy the template and fill in empty fields:\n\n\
                 \tcp {} \\\n\
                 \t   {}\n",
            relative_config_path.display(),
            relative_config_example_path.display(),
            relative_config_path.display(),
        ),
        ErrorKind::IsADirectory => anyhow::anyhow!(
            "Expected a config file at '{}' but found a directory.\n\
                 If running via Docker, this means the host-side config file does not exist — \
                 Docker created a directory at the volume mount target.\n\
                 Remove the directory, then copy the template and fill in empty fields:\n\n\
                 \trmdir {} && \\\n\
                 \tcp {} \\\n\
                 \t   {}\n",
            relative_config_path.display(),
            relative_config_path.display(),
            relative_config_example_path.display(),
            relative_config_path.display(),
        ),
        _ => anyhow::anyhow!("Failed to read config file: '{path:#?}': {e}"),
    })
}

/// Loads the raw toml config file for a service, printing helpful messages upon error.
pub fn deserialize_service_config<T: DeserializeOwned>(
    service_config: ServiceConfig,
) -> anyhow::Result<T> {
    let path = &service_config.toml_config_path();
    let raw = load_raw_service_config(service_config)?;

    toml::from_str(&raw).map_err(|e| anyhow::anyhow!("Failed to parse '{path:#?}': {e}"))
}
