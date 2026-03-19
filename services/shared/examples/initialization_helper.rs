use std::{
    collections::HashSet,
    io::ErrorKind,
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
    Service,
};
use solana_keypair::Keypair;
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

const TAKER_INITIAL_BASE: u64 = 10_000_000_000;
const TAKER_INITIAL_QUOTE: u64 = 10_000_000_000;

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

    // Mint the initial amounts to each account.
    let e2e = E2e::new_users_and_market_with_mint_decimals(
        Some(rpc),
        [
            User::new(faucet, FAUCET_INITIAL_BASE, FAUCET_INITIAL_QUOTE),
            User::new(maker, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE),
            User::new(taker, TAKER_INITIAL_BASE, TAKER_INITIAL_QUOTE),
        ],
        Some(6),
        Some(6),
    )
    .await?;

    // Create the maker market seat by depositing base and quote. Note that the taker does not need
    // a market seat and must have the base/quote token in their token accounts, not market seats.
    deposit_base_and_quote_to_market(maker, &e2e, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE).await?;

    // Write each keypair to the appropriate file.
    write_keypair_to_file(Service::Faucet, faucet)?;
    write_keypair_to_file(Service::Maker, maker)?;
    write_keypair_to_file(Service::Taker, taker)?;

    // Patch base_mint and quote_mint into each toml file in-place.
    update_base_and_quote_mints(Service::Faucet, &e2e)?;
    update_base_and_quote_mints(Service::Maker, &e2e)?;
    update_base_and_quote_mints(Service::Taker, &e2e)?;

    println!("Faucet address : {}", faucet.pubkey());
    println!("Maker address : {}", maker.pubkey());
    println!("Taker address : {}", taker.pubkey());
    println!("Base mint     : {}", e2e.market.base.mint_address);
    println!("Quote mint    : {}", e2e.market.quote.mint_address);
    println!("Market        : {}", e2e.market.market);

    Ok(())
}

fn write_keypair_to_file(service: Service, kp: &Keypair) -> anyhow::Result<()> {
    let kp_path = service.keypair_path();
    if std::fs::exists(kp_path.clone())? && !should_force_overwrite() {
        anyhow::bail!(
            "{:#?} already exists. Pass `--force` to overwrite it.",
            kp_path.clone(),
        );
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

fn update_base_and_quote_mints(service: Service, e2e: &E2e) -> anyhow::Result<()> {
    let cfg_path = service.toml_config_path();

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
                    load_raw_service_config(service)?;
                }
                e_kind => anyhow::bail!("Failed to remove empty directory: {e_kind}"),
            },
        }
    }

    // Copy from the example template if there's nothing at the config path.
    if !cfg_path.exists() {
        std::fs::copy(service.toml_config_example_path(), cfg_path.clone()).context(
            anyhow::anyhow!(
                "Failed to copy {:#?} to {:#?}",
                service.toml_config_example_path(),
                cfg_path,
            ),
        )?;
    }

    let raw = load_raw_service_config(service)?;
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
