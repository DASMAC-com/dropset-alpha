use std::{
    cell::RefCell,
    rc::Rc,
    time::Duration,
};

use client::{
    fmt_kv,
    transactions::{
        CustomRpcClient,
        ParsedTransactionWithEvents,
        TransactionSubmitError,
    },
};
use dropset_interface::error::DropsetError;
use dropset_services_shared::{
    debug_logs::format_timestamped_log,
    faucet_client::FaucetClient,
};
use solana_address::Address;
use solana_keypair::{
    Keypair,
    Signer,
};
use tokio::sync::watch;

use crate::{
    maker_context::MakerContext,
    TaskUpdate,
};

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
        // Wait until the value has changed. Not equality wise, but a sender posting a new value.
        rx.changed().await?;

        let update = *rx.borrow();
        let log_msg = format_timestamped_log(update);
        maker_ctx.try_borrow_mut()?.logger.log(log_msg);

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
                    let (kp, addr, quote_amount, base_amount, base_ata, quote_ata) = {
                        let ctx = maker_ctx.try_borrow_mut()?;
                        let kp = ctx.keypair.insecure_clone();
                        let (ask_size, bid_size) = (ctx.ask_order_size, ctx.bid_order_size);
                        let quote_amount = bid_size * 100;
                        let base_amount = ask_size * 100;
                        let addr = kp.pubkey();
                        let base_ata = ctx.market_ctx.get_base_ata(&addr);
                        let quote_ata = ctx.market_ctx.get_quote_ata(&addr);

                        (kp, addr, quote_amount, base_amount, base_ata, quote_ata)
                    };

                    if let Some(ref faucet_client) = faucet_client {
                        faucet_client
                            .request_quote_sign_and_submit(&addr, &kp, rpc, Some(quote_amount))
                            .await?;
                        faucet_client
                            .request_base_sign_and_submit(&addr, &kp, rpc, Some(base_amount))
                            .await?;
                        let base_after = rpc.client.get_token_account_balance(&base_ata).await?;
                        let quote_after = rpc.client.get_token_account_balance(&quote_ata).await?;

                        let deposits = {
                            let ctx = maker_ctx.try_borrow_mut()?;
                            [
                                ctx.deposit_base(base_after.amount.parse::<u64>()?),
                                ctx.deposit_quote(quote_after.amount.parse::<u64>()?),
                            ]
                        };

                        rpc.sign_and_submit_instructions(&kp, &[&kp], &deposits)
                            .await?;
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
    maker_ctx: Rc<RefCell<MakerContext>>,
    faucet_client: &FaucetClient,
    rpc: &CustomRpcClient,
    kp: Keypair,
    addr: Address,
    bid_size: u64,
    ask_size: u64,
    base_ata: Address,
    quote_ata: Address,
) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
    let (kp, addr, bid_size, ask_size, base_ata, quote_ata) = {
        let ctx = maker_ctx.try_borrow()?;
        let kp = ctx.keypair.insecure_clone();
        let quote_amount = ctx.bid_order_size * 100;
        let base_amount = ctx.ask_order_size * 100;
        let addr = kp.pubkey();
        let base_ata = ctx.market_ctx.get_base_ata(&addr);
        let quote_ata = ctx.market_ctx.get_quote_ata(&addr);

        (kp, addr, quote_amount, base_amount, base_ata, quote_ata)
    };

    faucet_client
        .request_quote_sign_and_submit(&addr, &kp, rpc, Some(quote_amount))
        .await?;
    faucet_client
        .request_base_sign_and_submit(&addr, &kp, rpc, Some(base_amount))
        .await?;
    let base_after = rpc.client.get_token_account_balance(&base_ata).await?;
    let quote_after = rpc.client.get_token_account_balance(&quote_ata).await?;

    let deposits = {
        let ctx = maker_ctx.try_borrow_mut()?;
        [
            ctx.deposit_base(base_after.amount.parse::<u64>()?),
            ctx.deposit_quote(quote_after.amount.parse::<u64>()?),
        ]
    };

    rpc.sign_and_submit_instructions(&kp, &[&kp], &deposits)
        .await
}
