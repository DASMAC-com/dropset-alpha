use std::{cell::RefCell, rc::Rc, time::Duration};

use client::{
    fmt_kv,
    transactions::{CustomRpcClient, ParsedTransactionWithEvents, TransactionSubmitError},
};
use dropset_interface::error::DropsetError;
use dropset_services_shared::faucet_client::FaucetClient;
use solana_keypair::Signer;
use tokio::sync::watch;

use crate::{maker_context::MakerContext, TaskUpdate};

/// The indefinite task loop to update orders whenever the [`watch::Receiver`] receives a message
/// from another task that indicates a [`TaskUpdate`] has occurred. Order submissions are
/// throttled so that they're updated at most one time per interval window.
///
/// It cancels old orders and posts new orders whenever the maker's orders would change due to a new
/// price from the price feed response or new market state.
pub async fn throttled_order_update(
    maker_ctx: Rc<RefCell<MakerContext>>,
    mut rx: watch::Receiver<TaskUpdate>,
    rpc: &CustomRpcClient,
    throttle_window_ms: u64,
    faucet_client: Option<FaucetClient>,
) -> anyhow::Result<()> {
    loop {
        let quote_ttl = maker_ctx.try_borrow()?.quote_ttl();
        let update = match tokio::time::timeout(quote_ttl, rx.changed()).await {
            Ok(res) => {
                res?;
                Some(*rx.borrow())
            }
            Err(_) => None,
        };

        {
            let mut ctx = maker_ctx.try_borrow_mut()?;
            match update {
                Some(update) => ctx.logger.log(update.to_string()),
                None if ctx.should_refresh_quotes() => ctx.logger.log(fmt_kv!(
                    "Quote TTL",
                    "refreshing resting liquidity",
                    client::LogColor::Info,
                )),
                None => continue,
            }
        }

        // Then cancel all orders and post new ones.
        let (maker_keypair, instructions) = {
            let mut ctx = maker_ctx.try_borrow_mut()?;
            let maker_keypair = ctx.keypair.insecure_clone();
            let instructions = ctx.create_update_book_instructions()?;
            (maker_keypair, instructions)
        };

        if !instructions.is_empty() {
            match rpc
                .sign_and_submit_instructions(&maker_keypair, &[&maker_keypair], &instructions)
                .await
            {
                Ok(_) => {
                    let balance = rpc.client.get_balance(&maker_keypair.pubkey()).await;
                    let mut ctx = maker_ctx.try_borrow_mut()?;
                    ctx.needs_expand = false;
                    ctx.mark_orders_submitted();
                    if let Ok(balance) = balance {
                        ctx.update_sol_balance(balance);
                    }
                    ctx.render_chart();
                }
                Err(TransactionSubmitError::Dropset(DropsetError::NoFreeSectorsRemaining)) => {
                    let mut ctx = maker_ctx.try_borrow_mut()?;
                    ctx.logger.log(fmt_kv!(
                        "Expanding market",
                        "no free sectors remaining",
                        client::LogColor::Info,
                    ));
                    ctx.needs_expand = true;
                }
                Err(TransactionSubmitError::Dropset(DropsetError::InsufficientUserBalance)) => {
                    if let Some(ref faucet_client) = faucet_client {
                        handle_faucet_request(&maker_ctx, faucet_client, rpc).await?;
                    } else {
                        anyhow::bail!("Out of tokens and couldn't request from faucet");
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Sleep for the throttle window in milliseconds before doing work again.
        // This effectively means the loop only does the cancel/post work once every window of time.
        tokio::time::sleep(Duration::from_millis(throttle_window_ms)).await;
    }
}

async fn handle_faucet_request(
    maker_ctx: &Rc<RefCell<MakerContext>>,
    faucet_client: &FaucetClient,
    rpc: &CustomRpcClient,
) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
    let (kp, addr, quote_amount, base_amount, base_ata, quote_ata) = {
        let ctx = maker_ctx
            .try_borrow()
            .map_err(|e| TransactionSubmitError::Other(e.into()))?;
        let kp = ctx.keypair.insecure_clone();
        let (ask_size, bid_size) = (ctx.ask_order_size, ctx.bid_order_size);
        let quote_amount = bid_size.saturating_mul(100);
        let base_amount = ask_size.saturating_mul(100);
        let addr = kp.pubkey();
        let base_ata = ctx.market_ctx.get_base_ata(&addr);
        let quote_ata = ctx.market_ctx.get_quote_ata(&addr);

        (kp, addr, quote_amount, base_amount, base_ata, quote_ata)
    };

    let quote_request = faucet_client
        .request_quote_sign_and_submit(&addr, &kp, rpc, Some(quote_amount))
        .await?;
    let base_request = faucet_client
        .request_base_sign_and_submit(&addr, &kp, rpc, Some(base_amount))
        .await?;

    let quote_after = quote_request
        .parsed_transaction
        .parsed_balances()?
        .get_post_token_balance(&quote_ata)
        .ok_or_else(|| {
            TransactionSubmitError::Other(anyhow::anyhow!(
                "Couldn't find maker's quote ATA ({quote_ata}) in parsed balances"
            ))
        })?;
    let base_after = base_request
        .parsed_transaction
        .parsed_balances()?
        .get_post_token_balance(&base_ata)
        .ok_or_else(|| {
            TransactionSubmitError::Other(anyhow::anyhow!(
                "Couldn't find maker's base ATA ({base_ata}) in parsed balances"
            ))
        })?;

    let deposits = {
        let ctx = maker_ctx
            .try_borrow()
            .map_err(|e| TransactionSubmitError::Other(e.into()))?;
        [ctx.deposit_base(base_after), ctx.deposit_quote(quote_after)]
    };

    rpc.sign_and_submit_instructions(&kp, &[&kp], &deposits)
        .await
}
