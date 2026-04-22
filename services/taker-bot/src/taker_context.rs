use std::sync::Arc;

use client::{
    context::market::MarketContext,
    transactions::{
        CustomRpcClient,
        ParsedTransactionWithEvents,
        TransactionSubmitError,
    },
};
use dropset_interface::instructions::MarketOrderInstructionData;
use dropset_services_shared::faucet_client::FaucetClient;
use solana_address::Address;
use solana_keypair::{
    Keypair,
    Signer,
};

use crate::taker::{
    Side,
    TakerFill,
};

/// Per-agent submission context. Each `[[agent]]` in the taker config gets one
/// of these: its own keypair, sharing the service-wide RPC and faucet clients.
pub struct TakerContext {
    pub rpc: Arc<CustomRpcClient>,
    pub keypair: Keypair,
    pub market_ctx: Arc<MarketContext>,
    pub faucet_client: Option<Arc<FaucetClient>>,
}

impl TakerContext {
    pub fn new(
        rpc: Arc<CustomRpcClient>,
        market_ctx: Arc<MarketContext>,
        faucet_client: Option<Arc<FaucetClient>>,
        keypair: Keypair,
    ) -> Self {
        Self {
            rpc,
            keypair,
            market_ctx,
            faucet_client,
        }
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

    /// Tries to request a faucet airdrop and then submits the transaction, returning the
    /// [TransactionSubmitError] if it fails or if the faucet service isn't available.
    pub async fn submit_faucet_request(
        &self,
        side: Side,
    ) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
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
            Ok(res)
        } else {
            let msg = "Out of tokens and couldn't request from faucet";
            tracing::error!("{msg}");
            Err(TransactionSubmitError::Other(anyhow::anyhow!(msg)))
        }
    }
}
