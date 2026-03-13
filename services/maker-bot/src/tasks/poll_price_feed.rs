use std::{
    cell::RefCell,
    rc::Rc,
    time::Duration,
};

use client::print_kv;
use tokio::sync::watch;

use crate::{
    maker_context::MakerContext,
    oanda_price_feed::{
        query_price_feed,
        OandaArgs,
    },
    TaskUpdate,
};

/// The indefinite task loop for polling the price feed endpoint.
///
/// On each loop iteration, it updates the maker context price info and notifies the
/// [`throttled_order_update`] task of a [`TaskUpdate::Price`] update.
pub async fn poll_price_feed(
    maker_ctx: Rc<RefCell<MakerContext>>,
    sender: watch::Sender<TaskUpdate>,
    client: reqwest::Client,
    oanda_args: &OandaArgs,
    poll_interval_ms: u64,
) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

    loop {
        interval.tick().await;

        match query_price_feed(oanda_args, &client).await {
            Ok(response) => {
                // Update the price in the maker context and then notify with `watch::Sender` that
                // the context has updated.
                maker_ctx
                    .try_borrow_mut()?
                    .update_price_from_candlestick(response)?;
                sender.send(TaskUpdate::Price)?;
                print_kv!("New mid price", maker_ctx.try_borrow()?.mid_price());
            }
            Err(e) => eprintln!("Price feed error: {e:#?}"),
        }
    }
}
