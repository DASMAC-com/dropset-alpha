use std::collections::HashSet;

use client::{
    e2e_helpers::{
        E2e,
        User,
    },
    transactions::{
        CustomRpcClient,
        SendTransactionConfig,
    },
};
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let user = Keypair::new();
    let rpc = CustomRpcClient::new(
        None,
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(true),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    );
    let e2e = E2e::new_users_and_market(Some(rpc), [User::new(&user, 0, 0)]).await?;

    println!(
        "Transaction signature: {}",
        e2e.register_market_txn.parsed_transaction.signature
    );

    Ok(())
}
