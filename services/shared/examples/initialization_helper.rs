use std::{
    collections::HashSet,
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::Context;
use client::{
    e2e_helpers::{
        test_accounts,
        E2e,
        User,
    },
    single_signer_instruction::SingleSignerInstruction,
    transactions::{
        airdrop,
        CustomRpcClient,
        SendTransactionConfig,
    },
};
use dropset_interface::state::sector::NIL;
use dropset_services_shared::config::{
    load_raw_service_config,
    ServiceConfig,
};
use serde::Deserialize;
use solana_keypair::{
    read_keypair_file,
    Keypair,
};
use solana_sdk::signer::Signer;
use toml_edit::DocumentMut;

// If you change these amounts, make sure the corresponding `config.toml` files still make sense.
// Inventory targets, order sizes, and similar parameters are expressed in base/quote atoms, so
// they need to be consistent with the amounts deposited here. For example, if the maker deposits
// 10_000_000_000 base atoms but `target_base` is still 100_000, it will be heavily sell-biased.
const FAUCET_INITIAL_BASE: u64 = 100_000_000_000;
const FAUCET_INITIAL_QUOTE: u64 = 100_000_000_000;

const MAKER_INITIAL_BASE: u64 = 10_000_000_000;
const MAKER_INITIAL_QUOTE: u64 = 10_000_000_000;

const TAKER_INITIAL_BASE: u64 = 1_000_000_000_000;
const TAKER_INITIAL_QUOTE: u64 = 1_000_000_000_000;

const AGENT_INITIAL_BASE: u64 = 1_000_000_000_000;
const AGENT_INITIAL_QUOTE: u64 = 1_000_000_000_000;

/// A helper example to bootstrap a market and a market maker on a localnet validator.
///
/// It does the following:
///
/// - Creates a market from two new tokens.
/// - Mints initial base/quote amounts to the faucet, maker, and taker.
/// - For the maker, the base/quote amounts are deposited to the `dropset` market, creating a seat.
/// - The taker keeps their balance in their associated token accounts, since the MarketOrder
///   instruction expects the balance to be in their ATA, not their seat.
/// - Writes the maker, taker, and faucet's keypair to their appropriate, respective keypair files.
/// - Patches `base_mint` and `quote_mint` into the appropriate config files.
#[tokio::main]
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

    let faucet = test_accounts::default_payer();
    let maker = test_accounts::acc_FFFF();
    let taker = test_accounts::acc_1111();
    airdrop(&rpc.client, &faucet.pubkey()).await?;
    airdrop(&rpc.client, &maker.pubkey()).await?;
    airdrop(&rpc.client, &taker.pubkey()).await?;

    // Per-agent taker keypairs (one per `[[agent]]` in `taker-bot/config.toml`).
    // Each one signs its own market orders, so each one needs SOL for fees plus
    // base/quote tokens to trade with. They are created on demand if missing.
    let agents = load_or_create_agent_keypairs()?;
    for agent in &agents {
        airdrop(&rpc.client, &agent.keypair.pubkey()).await?;
    }

    // Mint the initial amounts to each account.
    // Pass the faucet keypair as the mint authority so the faucet service
    // can mint tokens on demand.
    let mut users = vec![
        User::new(faucet, FAUCET_INITIAL_BASE, FAUCET_INITIAL_QUOTE),
        User::new(maker, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE),
        User::new(taker, TAKER_INITIAL_BASE, TAKER_INITIAL_QUOTE),
    ];
    users.extend(
        agents
            .iter()
            .map(|agent| User::new(&agent.keypair, AGENT_INITIAL_BASE, AGENT_INITIAL_QUOTE)),
    );
    let e2e = E2e::new_users_and_market_with_options(
        Some(rpc),
        users,
        Some(6),
        Some(6),
        Some(faucet),
    )
    .await?;

    // Create the maker market seat by depositing base and quote. Note that the taker does not need
    // a market seat and must have the base/quote token in their token accounts, not market seats.
    deposit_base_and_quote_to_market(maker, &e2e, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE).await?;

    // Write each keypair to the appropriate file.
    write_keypair_to_file(ServiceConfig::Faucet, faucet)?;
    write_keypair_to_file(ServiceConfig::Maker, maker)?;
    write_keypair_to_file(ServiceConfig::Taker, taker)?;

    // Patch base_mint and quote_mint into the shared config file in-place.
    update_base_and_quote_mints(&e2e)?;

    // Write the agent registry that the frontend reads to label each fill with
    // the trader personality that submitted it.
    write_agent_registry(maker, &agents)?;

    println!("Faucet address : {}", faucet.pubkey());
    println!("Maker address : {}", maker.pubkey());
    println!("Taker address : {}", taker.pubkey());
    for agent in &agents {
        println!("Agent {:<8}: {}", agent.name, agent.keypair.pubkey());
    }
    println!("Base mint     : {}", e2e.market.base.mint_address);
    println!("Quote mint    : {}", e2e.market.quote.mint_address);
    println!("Market        : {}", e2e.market.market);

    Ok(())
}

#[derive(Deserialize)]
struct TakerConfigAgents {
    #[serde(default, rename = "agent")]
    agents: Vec<TakerAgentEntry>,
}

#[derive(Deserialize)]
struct TakerAgentEntry {
    name: String,
    keypair_path: PathBuf,
}

pub struct AgentEntry {
    pub name: String,
    pub keypair: Keypair,
}

/// Reads `services/taker-bot/config.toml`, loads each agent's keypair file, and
/// generates a fresh keypair (writing it to disk) for any path that doesn't
/// exist yet. Relative `keypair_path` entries are resolved against the
/// taker-bot config directory, mirroring `taker-bot/src/config.rs`.
fn load_or_create_agent_keypairs() -> anyhow::Result<Vec<AgentEntry>> {
    let raw = load_raw_service_config(ServiceConfig::Taker)?;
    let parsed: TakerConfigAgents = toml::from_str(&raw)
        .context("Failed to parse taker-bot config.toml while loading agent keypairs")?;

    let taker_dir = ServiceConfig::Taker.config_dir();
    parsed
        .agents
        .into_iter()
        .map(|entry| {
            let path = if entry.keypair_path.is_absolute() {
                entry.keypair_path
            } else {
                taker_dir.join(&entry.keypair_path)
            };
            let keypair = load_or_create_keypair_file(&path)?;
            Ok(AgentEntry {
                name: entry.name,
                keypair,
            })
        })
        .collect()
}

/// Writes `services/taker-bot/agents.json` with one entry per known trader
/// (the maker plus every taker agent), so the frontend can label each fill
/// with the personality that submitted it.
fn write_agent_registry(maker: &Keypair, agents: &[AgentEntry]) -> anyhow::Result<()> {
    let mut entries: Vec<serde_json::Value> =
        Vec::with_capacity(agents.len() + 1);
    entries.push(serde_json::json!({
        "name": "maker",
        "kind": "maker",
        "pubkey": maker.pubkey().to_string(),
    }));
    for agent in agents {
        entries.push(serde_json::json!({
            "name": agent.name,
            "kind": "taker",
            "pubkey": agent.keypair.pubkey().to_string(),
        }));
    }

    let path = ServiceConfig::Taker.config_dir().join("agents.json");
    std::fs::write(&path, serde_json::to_string_pretty(&entries)?)
        .with_context(|| format!("Failed to write agent registry to {path:#?}"))?;
    Ok(())
}

/// Loads a keypair from `path` if it exists, otherwise generates a new one and
/// writes it to disk in the same JSON-array format that `solana-keygen` uses.
///
/// When `--force` is passed, an existing file is overwritten with a freshly
/// generated keypair so localnet identities can be reset.
fn load_or_create_keypair_file(path: &std::path::Path) -> anyhow::Result<Keypair> {
    if path.exists() && !should_force_overwrite() {
        return read_keypair_file(path).map_err(|e| {
            anyhow::anyhow!("Couldn't open agent keypair file: {path:#?}, err: ({e})")
        });
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create agent keypair directory: {parent:#?}")
        })?;
    }
    let kp = Keypair::new();
    std::fs::write(path, serde_json::to_string(&kp.to_bytes().to_vec())?)
        .with_context(|| format!("Failed to write new agent keypair to {path:#?}"))?;
    Ok(kp)
}

fn write_keypair_to_file(service: ServiceConfig, kp: &Keypair) -> anyhow::Result<()> {
    let kp_path = service.keypair_path();
    if std::fs::exists(kp_path.clone())? {
        let existing_keypair = read_keypair_file(&kp_path).map_err(|e| {
            anyhow::anyhow!("Couldn't open existing keypair file: {kp_path:#?}, err: ({e})")
        })?;

        if existing_keypair == kp {
            return Ok(());
        }

        // If the keypair already exists and doesn't match the one passed in, throw an error if a
        // forced overwrite flag wasn't passed. Otherwise, just write it.
        if !should_force_overwrite() {
            let existing_pub = existing_keypair.pubkey();
            let new_pub = kp.pubkey();
            anyhow::bail!(
                "{kp_path:#?} already exists.\n\
                 Pass `--force` to overwrite the existing keypair \
                 for {existing_pub} with the keypair for {new_pub}"
            );
        }
    }

    Ok(std::fs::write(
        kp_path,
        serde_json::to_string(&kp.to_bytes().to_vec())?,
    )?)
}

async fn deposit_base_and_quote_to_market(
    user: &Keypair,
    e2e: &E2e,
    base_amount: u64,
    quote_amount: u64,
) -> anyhow::Result<()> {
    e2e.market
        .deposit_base(user.pubkey(), base_amount, NIL)
        .send_single_signer(&e2e.rpc, user)
        .await?;

    let seat_index = e2e
        .fetch_seat(&user.pubkey())
        .await?
        .expect("Should have a seat")
        .index;

    e2e.market
        .deposit_quote(user.pubkey(), quote_amount, seat_index)
        .send_single_signer(&e2e.rpc, user)
        .await?;

    Ok(())
}

fn update_base_and_quote_mints(e2e: &E2e) -> anyhow::Result<()> {
    let shared_config = ServiceConfig::Shared;
    let cfg_path = shared_config.toml_config_path();

    // Try to `rmdir` the path if it's a directory, which fails if it's empty.
    // If the directory is empty, it's most likely because Docker mounted an empty directory
    // to the path because it didn't exist, so it's safe to try to remove.
    if cfg_path.is_dir() {
        let res = std::fs::remove_dir(cfg_path.clone());
        match res {
            Ok(_) => {}
            Err(e) => match e.kind() {
                ErrorKind::DirectoryNotEmpty => {
                    // Force an early return with an appropriate error message if it's not empty.
                    load_raw_service_config(shared_config)?;
                }
                e_kind => anyhow::bail!("Failed to remove empty directory: {e_kind}"),
            },
        }
    }

    // Copy from the example template if there's nothing at the config path.
    if !cfg_path.exists() {
        std::fs::copy(shared_config.toml_config_example_path(), cfg_path.clone()).context(
            anyhow::anyhow!(
                "Failed to copy {:#?} to {:#?}",
                shared_config.toml_config_example_path(),
                cfg_path,
            ),
        )?;
    }

    let raw = load_raw_service_config(shared_config)?;
    let mut doc: DocumentMut = raw.parse()?;
    doc["base_mint"] = toml_edit::value(e2e.market.base.mint_address.to_string());
    doc["quote_mint"] = toml_edit::value(e2e.market.quote.mint_address.to_string());
    std::fs::write(&cfg_path, doc.to_string())?;

    Ok(())
}

/// A simple CLI argument to indicate that rewriting keypair files is intentional.
fn should_force_overwrite() -> bool {
    std::env::args().any(|a| a == "--force")
}
