use std::{
    cell::RefCell,
    rc::Rc,
    time::Duration,
};

use client::{
    fmt_kv,
    transactions::{
        CustomRpcClient,
        TransactionSubmitError,
    },
};
use dropset_interface::error::DropsetError;
use dropset_services_shared::debug_logs::format_timestamped_log;
use solana_keypair::Signer;
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
                .send_and_confirm_txn(&maker_keypair, &[&maker_keypair], &instructions)
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
                Err(e) => return Err(e.into()),
            }
        }

        // Sleep for the throttle window in milliseconds before doing work again.
        // This effectively means the loop only does the cancel/post work once every window of time.
        tokio::time::sleep(Duration::from_millis(throttle_window_ms)).await;
    }
}
