use std::collections::HashSet;

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
use solana_sdk::signer::Signer;
use toml_edit::DocumentMut;

const MAKER_INITIAL_BASE: u64 = 10_000;
const MAKER_INITIAL_QUOTE: u64 = 10_000;

/// A helper example to bootstrap a market and a market maker. It does the following:
///
/// - Creates a market from two new tokens.
/// - Mints [`MAKER_INITIAL_BASE`] and [`MAKER_INITIAL_QUOTE`] and deposits them into the maker's
///   seat.
/// - Writes the maker's keypair to `maker-keypair.json`.
/// - Patches `base_mint` and `quote_mint` into `config.toml`.
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

    let maker = test_accounts::acc_FFFF();
    let maker_address = maker.pubkey();
    airdrop(&rpc.client, &test_accounts::default_payer().pubkey()).await?;
    airdrop(&rpc.client, &maker_address).await?;

    let e2e = E2e::new_traders_and_market(
        Some(rpc),
        [Trader::new(maker, MAKER_INITIAL_BASE, MAKER_INITIAL_QUOTE)],
    )
    .await?;

    e2e.market
        .deposit_base(maker_address, MAKER_INITIAL_BASE, NIL)
        .send_single_signer(&e2e.rpc, maker)
        .await?;

    let seat = e2e
        .fetch_seat(&maker_address)
        .await?
        .expect("Should have a seat")
        .index;

    e2e.market
        .deposit_quote(maker_address, MAKER_INITIAL_QUOTE, seat)
        .send_single_signer(&e2e.rpc, maker)
        .await?;

    // Write the maker's keypair.
    let keypair_bytes: Vec<u8> = maker.insecure_clone().to_bytes().to_vec();
    std::fs::write("maker-keypair.json", serde_json::to_string(&keypair_bytes)?)?;

    // Patch base_mint and quote_mint into config.toml in-place.
    let raw = std::fs::read_to_string("config.toml")?;
    let mut doc: DocumentMut = raw.parse()?;
    doc["base_mint"]  = toml_edit::value(e2e.market.base.mint_address.to_string());
    doc["quote_mint"] = toml_edit::value(e2e.market.quote.mint_address.to_string());
    std::fs::write("config.toml", doc.to_string())?;

    println!("Maker address : {maker_address}");
    println!("Base mint     : {}", e2e.market.base.mint_address);
    println!("Quote mint    : {}", e2e.market.quote.mint_address);
    println!("Market        : {}", e2e.market.market);

    Ok(())
}
