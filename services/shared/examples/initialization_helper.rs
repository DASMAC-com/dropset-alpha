use std::{
    collections::HashSet,
    io::ErrorKind,
    path::{
        Path,
        PathBuf,
    },
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
    deserialize_service_config,
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

/// Minimal view of `services/taker-bot/config.toml` — just the fields the
/// helper needs to know about per agent. Extra fields (archetype, overrides,
/// etc.) are tolerated by serde's default behavior.
#[derive(Deserialize)]
struct TakerManifest {
    #[serde(default, rename = "agent")]
    agents: Vec<AgentManifestEntry>,
}

#[derive(Deserialize)]
struct AgentManifestEntry {
    name: String,
    keypair_path: PathBuf,
}

/// A helper example to bootstrap a market and all participants on a localnet validator.
///
/// It does the following:
///
/// - Creates a market from two new tokens.
/// - Mints initial base/quote amounts to the faucet, maker, and every taker agent declared in
///   `services/taker-bot/config.toml`.
/// - For the maker, the base/quote amounts are deposited to the `dropset` market, creating a seat.
/// - Each taker agent keeps its balance in its associated token accounts, since the MarketOrder
///   instruction expects the balance to be in the ATA, not a seat.
/// - Writes the maker, faucet, and each taker agent's keypair to their configured keypair files.
///   Reruns are idempotent: an existing keypair file is reused unless `--force` is passed.
/// - Patches `base_mint` and `quote_mint` into the shared config file.
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

    let force = should_force_overwrite();

    let faucet = test_accounts::default_payer();
    let maker = test_accounts::acc_FFFF();
    airdrop(&rpc.client, &faucet.pubkey()).await?;
    airdrop(&rpc.client, &maker.pubkey()).await?;

    // Load the taker-bot agent manifest before touching the chain so a config
    // error fails fast.
    let manifest: TakerManifest = deserialize_service_config(ServiceConfig::Taker)?;
    if manifest.agents.is_empty() {
        anyhow::bail!(
            "No `[[agent]]` entries in services/taker-bot/config.toml — \
             copy config.toml.example as a starting point"
        );
    }
    ensure_unique_agent_names(&manifest.agents)?;

    // Materialize (or load) a keypair per agent. We keep these in a Vec so the
    // &Keypair references passed to `User::new` stay valid for the whole helper.
    let agent_keypairs: Vec<Keypair> = manifest
        .agents
        .iter()
        .map(|entry| {
            let path = resolve_agent_keypair_path(&entry.keypair_path);
            load_or_create_keypair(&path, force).with_context(|| {
                format!(
                    "Failed to prepare keypair for agent `{}` at {}",
                    entry.name,
                    path.display()
                )
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    for kp in &agent_keypairs {
        airdrop(&rpc.client, &kp.pubkey()).await?;
    }

    // Mint the initial amounts to each account.
    // Pass the faucet keypair as the mint authority so the faucet service
    // can mint tokens on demand.
    let mut users = vec![
        User::new(faucet, FAUCET_INITIAL_BASE, FAUCET_INITIAL_QUOTE),
        User::new(maker, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE),
    ];
    users.extend(
        agent_keypairs
            .iter()
            .map(|kp| User::new(kp, TAKER_INITIAL_BASE, TAKER_INITIAL_QUOTE)),
    );

    let e2e = E2e::new_users_and_market_with_options(
        Some(rpc),
        users,
        Some(6),
        Some(6),
        Some(faucet),
    )
    .await?;

    // Create the maker market seat by depositing base and quote. Note that taker agents do not
    // need a market seat and must keep their base/quote tokens in their token accounts.
    deposit_base_and_quote_to_market(maker, &e2e, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE).await?;

    // Write the faucet + maker keypair files (the taker-bot service-level
    // keypair file is still written below for shared-config compatibility).
    write_keypair_to_file(ServiceConfig::Faucet, faucet)?;
    write_keypair_to_file(ServiceConfig::Maker, maker)?;

    // The shared config loader for the taker service still reads
    // `services/taker-bot/keypair.json`. Its content is unused at runtime
    // (each agent signs with its own keypair), but the file needs to exist.
    // Reuse the first agent's keypair so we don't create a second dangling file.
    let service_identity = agent_keypairs
        .first()
        .expect("At least one agent exists — checked above");
    write_keypair_to_file(ServiceConfig::Taker, service_identity)?;

    // Patch base_mint and quote_mint into the shared config file in-place.
    update_base_and_quote_mints(&e2e)?;

    println!("Faucet address : {}", faucet.pubkey());
    println!("Maker address  : {}", maker.pubkey());
    for (entry, kp) in manifest.agents.iter().zip(agent_keypairs.iter()) {
        println!("Agent `{}` : {}", entry.name, kp.pubkey());
    }
    println!("Base mint      : {}", e2e.market.base.mint_address);
    println!("Quote mint     : {}", e2e.market.quote.mint_address);
    println!("Market         : {}", e2e.market.market);

    Ok(())
}

fn ensure_unique_agent_names(agents: &[AgentManifestEntry]) -> anyhow::Result<()> {
    let mut seen = HashSet::new();
    for a in agents {
        if !seen.insert(a.name.as_str()) {
            anyhow::bail!("Duplicate agent name `{}` in taker-bot config", a.name);
        }
    }
    Ok(())
}

/// Relative paths are resolved against `services/taker-bot/` so configs can
/// say `keypair_path = "keypairs/retail-1.json"` without the caller having to
/// care about the current working directory.
fn resolve_agent_keypair_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        ServiceConfig::Taker.config_dir().join(path)
    }
}

/// Loads an existing keypair at `path`, or generates and persists a new one
/// if the file is missing. When `force` is true the file is overwritten with
/// a fresh keypair regardless.
fn load_or_create_keypair(path: &Path, force: bool) -> anyhow::Result<Keypair> {
    if path.exists() && !force {
        return read_keypair_file(path)
            .map_err(|e| anyhow::anyhow!("Couldn't read keypair file `{}`: {e}", path.display()));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory for {}", path.display())
        })?;
    }

    let kp = Keypair::new();
    std::fs::write(path, serde_json::to_string(&kp.to_bytes().to_vec())?)
        .with_context(|| format!("Failed to write keypair file {}", path.display()))?;
    Ok(kp)
}

fn write_keypair_to_file(service: ServiceConfig, kp: &Keypair) -> anyhow::Result<()> {
    let kp_path = service.keypair_path();
    if std::fs::exists(kp_path.clone())? {
        let existing_keypair = read_keypair_file(&kp_path).map_err(|e| {
            anyhow::anyhow!("Couldn't open existing keypair file: {kp_path:#?}, err: ({e})")
        })?;

        if existing_keypair == *kp {
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
