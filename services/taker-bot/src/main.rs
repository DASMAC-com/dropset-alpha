pub mod config;
pub mod taker_context;

use std::{
    collections::HashSet,
    time::Duration,
};

use client::transactions::{
    CustomRpcClient,
    SendTransactionConfig,
};
use rand::random_bool;
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

    let taker_ctx = TakerContext::init(rpc, cfg).await?;

    loop {
        let jitter = rand::random::<u64>() % taker_ctx.order_interval_jitter;
        tokio::time::sleep(Duration::from_millis(taker_ctx.order_interval + jitter)).await;

        if random_bool(0.5) {
            taker_ctx.buy().await?;
        } else {
            taker_ctx.sell().await?;
        }
    }
}
