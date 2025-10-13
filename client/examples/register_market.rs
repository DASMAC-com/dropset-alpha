use client::{
    context::market::MarketContext,
    transactions::{
        fund_account,
        send_transaction,
    },
};
use dropset_interface::instructions::RegisterMarketInstructionData;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::signer::Signer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc =
        &RpcClient::new_with_commitment("http://localhost:8899", CommitmentConfig::confirmed());

    let payer = fund_account(rpc, None).await?;
    let market_ctx = MarketContext::new_market(rpc).await?;

    let register = market_ctx.register_market(
        payer.pubkey(),
        RegisterMarketInstructionData { num_sectors: 10 },
    );

    let res = send_transaction(rpc, &payer, &[&payer], &[register]).await?;

    println!("Transaction signature: {res}");

    Ok(())
}
