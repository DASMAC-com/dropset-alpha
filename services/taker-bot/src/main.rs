pub mod config;
pub mod taker;
pub mod taker_context;

use std::collections::HashSet;

use client::transactions::{
    extract_dropset_error,
    CustomRpcClient,
    SendTransactionConfig,
};
use dropset_interface::error::DropsetError;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};

use crate::{
    config::get_validated_config,
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
            debug_logs: Some(true),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    );

    let taker_ctx = TakerContext::init(rpc, cfg.shared).await?;
    let mut strategy = cfg.taker_strategy;

    loop {
        strategy.activity_profile.interval.tick().await;
        for fill in strategy.step() {
            match taker_ctx.submit_fill(&fill).await {
                Ok(_) => {}
                Err(e) => match extract_dropset_error(&e) {
                    // Book is dry — no liquidity to fill against, skip.
                    Some(DropsetError::AmountCannotBeZero) => {}
                    _ => return Err(e),
                },
            }
        }
    }
}
