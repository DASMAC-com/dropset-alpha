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

use crate::config::ValidTakerConfig;

pub struct TakerContext {
    pub rpc: CustomRpcClient,
    /// The taker's keypair.
    pub keypair: Keypair,
    pub market_ctx: MarketContext,
    pub buy_order_size: u64,
    pub sell_order_size: u64,
    pub order_interval: u64,
    pub order_interval_jitter: u64,
}

impl TakerContext {
    /// Creates a new taker context from a [ValidTakerConfig].
    pub async fn init(rpc: CustomRpcClient, cfg: ValidTakerConfig) -> anyhow::Result<Self> {
        let ValidTakerConfig {
            shared,
            sell_order_size,
            buy_order_size,
            order_interval,
            order_interval_jitter,
        } = cfg;

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
            buy_order_size,
            sell_order_size,
            order_interval,
            order_interval_jitter,
        })
    }

    pub fn address(&self) -> Address {
        self.keypair.pubkey()
    }

    pub async fn buy(&self) -> anyhow::Result<ParsedTransactionWithEvents> {
        self.rpc
            .send_single_signer(
                &self.keypair,
                &[self.market_ctx.market_order(
                    self.address(),
                    // The buy order size is denominated in quote.
                    MarketOrderInstructionData::new(self.buy_order_size, true, false),
                )],
            )
            .await
    }

    pub async fn sell(&self) -> anyhow::Result<ParsedTransactionWithEvents> {
        self.rpc
            .send_single_signer(
                &self.keypair,
                &[self.market_ctx.market_order(
                    self.address(),
                    // The sell order size is denominated in base.
                    MarketOrderInstructionData::new(self.sell_order_size, false, true),
                )],
            )
            .await
    }
}
