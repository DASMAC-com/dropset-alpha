use client::{
    context::market::MarketContext,
    transactions::{
        CustomRpcClient,
        ParsedTransactionWithEvents,
        TransactionSubmitError,
    },
};
use dropset_interface::instructions::MarketOrderInstructionData;
use dropset_services_shared::{
    config::ValidSharedConfig,
    faucet_client::FaucetClient,
};
use solana_address::Address;
use solana_keypair::{
    Keypair,
    Signer,
};

use crate::taker::{
    Side,
    TakerFill,
};

pub struct TakerContext {
    pub rpc: CustomRpcClient,
    pub keypair: Keypair,
    pub market_ctx: MarketContext,
    pub faucet_client: Option<FaucetClient>,
}

impl TakerContext {
    pub async fn init(rpc: CustomRpcClient, shared: ValidSharedConfig) -> anyhow::Result<Self> {
        let faucet_client = FaucetClient::new(&shared).await;
        let ValidSharedConfig {
            keypair,
            base,
            quote,
            ..
        } = shared;

        let market_ctx = MarketContext::new(base, quote);

        Ok(Self {
            rpc,
            keypair,
            market_ctx,
            faucet_client,
        })
    }

    pub fn address(&self) -> Address {
        self.keypair.pubkey()
    }

    /// Submits a market order for a fill produced by [`crate::taker::TakerStrategy::step`].
    /// Order size is treated as base atoms for both sides.
    pub async fn submit_fill(
        &self,
        fill: &TakerFill,
    ) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
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

    pub async fn submit_faucet_request(
        &self,
        side: Side,
    ) -> Result<Option<ParsedTransactionWithEvents>, TransactionSubmitError> {
        if let Some(ref faucet_client) = self.faucet_client {
            let res = match side {
                Side::Buy => {
                    faucet_client
                        .request_quote_sign_and_submit(
                            &self.address(),
                            &self.keypair,
                            &self.rpc,
                            None,
                        )
                        .await?
                }
                Side::Sell => {
                    faucet_client
                        .request_base_sign_and_submit(
                            &self.address(),
                            &self.keypair,
                            &self.rpc,
                            None,
                        )
                        .await?
                }
            };
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}
