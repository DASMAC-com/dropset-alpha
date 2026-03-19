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
            debug_logs: Some(cfg.debug_logs),
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
                Err(TransactionSubmitError::Dropset(err)) => match err {
                    // Book is dry — most likely there is no liquidity to fill against, skip.
                    DropsetError::AmountCannotBeZero => {
                        let log_msg = format_timestamped_log(
                            "ERROR: Fill returned zero amount, book likely empty — skipping",
                        );
                        eprintln!("{log_msg}");
                    }
                    _ => return Err(TransactionSubmitError::Dropset(err).into()),
                },
                Err(e) => return Err(e.into()),
            }
        }
    }
}
