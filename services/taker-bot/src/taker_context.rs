use std::sync::Arc;

use client::{
    context::market::MarketContext,
    transactions::{CustomRpcClient, ParsedTransactionWithEvents, TransactionSubmitError},
};
use dropset_interface::instructions::MarketOrderInstructionData;
use dropset_services_shared::faucet_client::FaucetClient;
use price::client_helpers::try_encoded_u32_to_decoded_decimal;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use solana_address::Address;
use solana_keypair::{Keypair, Signer};
use transaction_parser::views::try_market_view_all_from_owner_and_data;

use crate::taker::{BookLevel, BookSideSnapshot, MarketSnapshot, Side, TakerFill};

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

    pub async fn fetch_market_snapshot(&self) -> anyhow::Result<MarketSnapshot> {
        let market_account = self.rpc.client.get_account(&self.market_ctx.market).await?;
        let market =
            try_market_view_all_from_owner_and_data(market_account.owner, &market_account.data)?;

        let decode_side =
            |orders: &[transaction_parser::views::OrderView]| -> anyhow::Result<BookSideSnapshot> {
                let levels = orders
                    .iter()
                    .map(|order| {
                        Ok(BookLevel {
                            price: try_encoded_u32_to_decoded_decimal(
                                order.encoded_price.as_u32(),
                            )?,
                            base_remaining: order.base_remaining,
                            quote_remaining: order.quote_remaining,
                        })
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;
                let total_base_depth = levels.iter().map(|level| level.base_remaining).sum();
                Ok(BookSideSnapshot {
                    levels,
                    total_base_depth,
                })
            };

        let bids = decode_side(&market.bids)?;
        let asks = decode_side(&market.asks)?;
        let visible_bid = bids.visible_base_depth(3);
        let visible_ask = asks.visible_base_depth(3);
        let imbalance_denom = visible_bid + visible_ask;
        let imbalance = if imbalance_denom == 0 {
            0.0
        } else {
            (visible_bid as f64 - visible_ask as f64) / (imbalance_denom as f64)
        };

        let best_bid = bids.levels.first();
        let best_ask = asks.levels.first();
        let mid_price = match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / Decimal::from(2u8)),
            _ => None,
        };
        let spread_bps = match (best_bid, best_ask, mid_price) {
            (Some(bid), Some(ask), Some(mid)) if mid > Decimal::ZERO => {
                let spread = ask.price - bid.price;
                let bps = spread
                    .checked_mul(Decimal::from(10_000u64))
                    .and_then(|scaled| scaled.checked_div(mid))
                    .and_then(|res| res.to_f64());
                bps
            }
            _ => None,
        };
        let microprice = match (best_bid, best_ask) {
            (Some(bid), Some(ask)) if bid.base_remaining + ask.base_remaining > 0 => {
                let bid_weight = Decimal::from(bid.base_remaining);
                let ask_weight = Decimal::from(ask.base_remaining);
                let numerator = ask.price * bid_weight + bid.price * ask_weight;
                numerator.checked_div(bid_weight + ask_weight)
            }
            _ => None,
        };

        Ok(MarketSnapshot {
            bids,
            asks,
            spread_bps,
            mid_price,
            microprice,
            imbalance,
        })
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
