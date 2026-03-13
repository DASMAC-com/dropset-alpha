use client::e2e_helpers::{
    E2e,
    User,
};
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let user = Keypair::new();
    let e2e = E2e::new_users_and_market(None, [User::new(&user, 0, 0)]).await?;

    println!(
        "Transaction signature: {}",
        e2e.register_market_txn.parsed_transaction.signature
    );

    Ok(())
}
