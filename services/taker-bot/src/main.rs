pub mod archetype;
pub mod config;
pub mod taker;
pub mod taker_context;

use std::{
    collections::HashSet,
    sync::Arc,
};

use client::{
    context::market::MarketContext,
    transactions::{
        CustomRpcClient,
        SendTransactionConfig,
        TransactionSubmitError,
    },
};
use dropset_interface::error::DropsetError;
use dropset_services_shared::faucet_client::FaucetClient;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};
use spl_token_2022_interface::error::TokenError as Token2022Error;
use spl_token_interface::error::TokenError;
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

use crate::{
    config::{
        get_validated_config,
        ValidAgent,
    },
    taker::TakerStrategy,
    taker_context::TakerContext,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let health_check = std::env::args().any(|a| a == "--health-check");
    let cfg = get_validated_config().await?;
    if health_check {
        return Ok(());
    }

    // Default to `info`.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let rpc = Arc::new(CustomRpcClient::new(
        Some(RpcClient::new_with_commitment(
            cfg.shared.rpc_url.clone().to_string(),
            CommitmentConfig::confirmed(),
        )),
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(cfg.verbose),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    ));

    let faucet_client = match FaucetClient::new(&rpc, &cfg.shared).await {
        Ok(c) => Some(Arc::new(c)),
        Err(e) => {
            tracing::warn!(error = %e, "Faucet client is unavailable");
            None
        }
    };

    let market_ctx = Arc::new(MarketContext::new(
        cfg.shared.base.clone(),
        cfg.shared.quote.clone(),
    ));

    tracing::info!(count = cfg.agents.len(), "Launching taker agents");

    let mut handles = Vec::with_capacity(cfg.agents.len());
    for ValidAgent {
        name,
        keypair,
        strategy,
    } in cfg.agents
    {
        let ctx = TakerContext::new(rpc.clone(), market_ctx.clone(), faucet_client.clone(), keypair);
        let span = tracing::info_span!("agent", name = %name);
        tracing::info!(agent = %name, address = %ctx.address(), "Agent starting");
        let task = agent_loop(name.clone(), ctx, strategy, cfg.verbose).instrument(span);
        handles.push((name, tokio::spawn(task)));
    }

    // Wait for every agent task. If a task panics, log and keep the remaining
    // agents running — a single misbehaving archetype shouldn't kill the
    // container. The process exits once the last agent has ended.
    for (name, handle) in handles {
        match handle.await {
            Ok(()) => tracing::warn!(agent = %name, "Agent exited cleanly"),
            Err(e) if e.is_panic() => {
                tracing::error!(agent = %name, error = %e, "Agent panicked");
            }
            Err(e) => tracing::error!(agent = %name, error = %e, "Agent join error"),
        }
    }

    Ok(())
}

/// Runs a single agent's strategy loop forever. Non-fatal errors (empty book,
/// out-of-token) are logged and the loop keeps going; a panic is caught by
/// the `tokio::spawn` `JoinHandle` in `main`.
async fn agent_loop(
    name: String,
    ctx: TakerContext,
    mut strategy: TakerStrategy,
    verbose: bool,
) {
    loop {
        strategy.tick().await;
        for fill in strategy.step() {
            match ctx.submit_fill(&fill).await {
                Ok(_) => {}

                // The taker's ATA is out of the token being spent; ask the
                // faucet to top it up so the next tick can trade again.
                Err(TransactionSubmitError::Token(TokenError::InsufficientFunds))
                | Err(TransactionSubmitError::Token2022(Token2022Error::InsufficientFunds)) => {
                    if let Err(e) = ctx.submit_faucet_request(fill.side).await {
                        tracing::error!(agent = %name, error = ?e, "Faucet request failed");
                    }
                }

                // Book is dry — no liquidity to fill against on this side.
                Err(TransactionSubmitError::Dropset(DropsetError::AmountCannotBeZero)) => {
                    if verbose {
                        tracing::error!(
                            agent = %name,
                            "Fill returned zero amount, book likely empty — skipping"
                        );
                    }
                }

                Err(e) => {
                    tracing::error!(agent = %name, error = ?e, "Order submission error");
                }
            }
        }
    }
}
