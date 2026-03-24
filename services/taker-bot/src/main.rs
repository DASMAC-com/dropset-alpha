pub mod config;
pub mod taker;
pub mod taker_context;

use std::collections::HashSet;

use client::transactions::{
    CustomRpcClient,
    SendTransactionConfig,
    TransactionSubmitError,
};
use dropset_interface::error::DropsetError;
use dropset_services_shared::debug_logs::format_timestamped_log;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};
use spl_token_2022_interface::error::TokenError;

use crate::{
    config::get_validated_config,
    taker::Side,
    taker_context::TakerContext,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let health_check = std::env::args().any(|a| a == "--health-check");
    let cfg = get_validated_config().await?;
    if health_check {
        return Ok(());
    }

    let rpc = CustomRpcClient::new(
        Some(RpcClient::new_with_commitment(
            cfg.shared.rpc_url.clone().to_string(),
            CommitmentConfig::confirmed(),
        )),
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(cfg.verbose),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    );

    let taker_ctx = TakerContext::init(rpc, cfg.shared).await?;
    let mut strategy = cfg.taker_strategy;

    loop {
        strategy.tick().await;
        for fill in strategy.step() {
            match taker_ctx.submit_fill(&fill).await {
                Ok(_) => {}
                Err(TransactionSubmitError::Token { error, .. }) => match error {
                    // Taker has run out of tokens. Airdrop more if the faucet client exists.
                    // Note that this is a match on the token program error, not dropset error,
                    // because taker's balances don't go through their seats, they go directly
                    // through the token program.
                    TokenError::InsufficientFunds => {
                        if let Some(ref faucet_client) = taker_ctx.faucet_client {
                            match fill.side {
                                Side::Buy => {
                                    faucet_client
                                        .request_quote_sign_and_submit(
                                            &taker_ctx.address(),
                                            &taker_ctx.keypair,
                                            &taker_ctx.rpc,
                                            None,
                                        )
                                        .await?;
                                }
                                Side::Sell => {
                                    faucet_client
                                        .request_base_sign_and_submit(
                                            &taker_ctx.address(),
                                            &taker_ctx.keypair,
                                            &taker_ctx.rpc,
                                            None,
                                        )
                                        .await?;
                                }
                            }
                        }
                    }
                    _ => return Err(error.into()),
                },
                Err(TransactionSubmitError::Dropset(err)) => match err {
                    // Book is dry — most likely there is no liquidity to fill against, skip.
                    DropsetError::AmountCannotBeZero => {
                        let log_message =
                            "ERROR: Fill returned zero amount, book likely empty — skipping";
                        if cfg.verbose {
                            eprintln!("{}", format_timestamped_log(log_message));
                        }
                    }
                    _ => return Err(TransactionSubmitError::Dropset(err).into()),
                },
                Err(e) => return Err(e.into()),
            }
        }
    }
}
