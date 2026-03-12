use std::collections::HashSet;

use anyhow::Context;
use client::{
    e2e_helpers::{
        test_accounts,
        E2e,
        Trader,
    },
    single_signer_instruction::SingleSignerInstruction,
    transactions::{
        airdrop,
        CustomRpcClient,
        SendTransactionConfig,
    },
};
use dropset_interface::state::sector::NIL;
use dropset_services_shared::config::Service;
use solana_keypair::Keypair;
use solana_sdk::signer::Signer;
use toml_edit::DocumentMut;

const FAUCET_INITIAL_BASE: u64 = 100_000_000_000;
const FAUCET_INITIAL_QUOTE: u64 = 100_000_000_000;

const MAKER_INITIAL_BASE: u64 = 100_000;
const MAKER_INITIAL_QUOTE: u64 = 100_000;

const TAKER_INITIAL_BASE: u64 = 100_000;
const TAKER_INITIAL_QUOTE: u64 = 100_000;

/// A helper example to bootstrap a market and a market maker on a localnet validator.
///
/// It does the following:
///
/// - Creates a market from two new tokens.
/// - Mints [`MAKER_INITIAL_BASE`] and [`MAKER_INITIAL_QUOTE`] and deposits them into the maker's
///   seat.
/// - Writes the maker, taker, and faucet's keypair to their appropriate, respective keypair files.
/// - Patches `base_mint` and `quote_mint` into the appropriate config files.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc = CustomRpcClient::new(
        None,
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(false),
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

    let e2e = E2e::new_traders_and_market(
        Some(rpc),
        [
            Trader::new(faucet, FAUCET_INITIAL_BASE, FAUCET_INITIAL_QUOTE),
            Trader::new(maker, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE),
            Trader::new(taker, TAKER_INITIAL_BASE, TAKER_INITIAL_QUOTE),
        ],
    )
    .await?;

    deposit_base_and_quote_to_market(faucet, &e2e, FAUCET_INITIAL_BASE, FAUCET_INITIAL_QUOTE)
        .await?;
    deposit_base_and_quote_to_market(maker, &e2e, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE).await?;
    deposit_base_and_quote_to_market(taker, &e2e, TAKER_INITIAL_BASE, TAKER_INITIAL_QUOTE).await?;

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
    if std::fs::exists(service.keypair_path())? && !should_force_overwrite() {
        anyhow::bail!(
            "{:#?} already exists. Pass `--force` to overwrite it.",
            service.keypair_path(),
        );
    }
    Ok(std::fs::write(
        service.keypair_path(),
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
    let config = service.toml_config_path();
    if !config.exists() {
        std::fs::copy(service.toml_config_example_path(), config.clone()).context(
            anyhow::anyhow!(
                "Failed to copy {:#?} to {:#?}",
                service.toml_config_example_path(),
                config,
            ),
        )?;
    }

    let raw = std::fs::read_to_string(&config)?;
    let mut doc: DocumentMut = raw.parse()?;
    doc["base_mint"] = toml_edit::value(e2e.market.base.mint_address.to_string());
    doc["quote_mint"] = toml_edit::value(e2e.market.quote.mint_address.to_string());
    std::fs::write(&config, doc.to_string())?;

    Ok(())
}

/// A simple CLI argument to indicate that rewriting keypair files is intentional.
fn should_force_overwrite() -> bool {
    std::env::args().any(|a| a == "--force")
}
