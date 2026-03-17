use anyhow::Context;
use client::{
    context::{
        market::MarketContext,
        token::TokenContext,
    },
    transactions::{
        CustomRpcClient,
        ParsedTransactionWithEvents,
    },
};
use dropset_interface::instructions::MarketOrderInstructionData;
use dropset_services_shared::config::ValidSharedConfig;
use solana_address::Address;
use solana_keypair::{
    Keypair,
    Signer,
};

use crate::taker::TakerFill;

pub struct TakerContext {
    pub rpc: CustomRpcClient,
    pub keypair: Keypair,
    pub market_ctx: MarketContext,
}

impl TakerContext {
    pub async fn init(rpc: CustomRpcClient, shared: ValidSharedConfig) -> anyhow::Result<Self> {
        let ValidSharedConfig {
            keypair,
            base_mint,
            quote_mint,
            ..
        } = shared;

        let base_account = rpc
            .client
            .get_account(&base_mint)
            .await
            .context("Couldn't find base mint account on-chain")?;
        let base =
            TokenContext::from_account_data(base_mint, base_account.owner, &base_account.data)?;

        let quote_account = rpc
            .client
            .get_account(&quote_mint)
            .await
            .context("Couldn't find quote mint account on-chain")?;
        let quote =
            TokenContext::from_account_data(quote_mint, quote_account.owner, &quote_account.data)?;

        let market_ctx = MarketContext::new(base, quote);

        Ok(Self {
            rpc,
            keypair,
            market_ctx,
        })
    }

    pub fn address(&self) -> Address {
        self.keypair.pubkey()
    }

    /// Submits a market order for a fill produced by [TakerStrategy::step].
    /// Order size is treated as base atoms for both sides.
    pub async fn submit_fill(
        &self,
        fill: &TakerFill,
    ) -> anyhow::Result<ParsedTransactionWithEvents> {
        self.rpc
            .send_single_signer(
                &self.keypair,
                &[self.market_ctx.market_order(
                    self.address(),
                    MarketOrderInstructionData::new(fill.size, fill.side.is_buy(), true),
                )],
            )
            .await
    }
}
