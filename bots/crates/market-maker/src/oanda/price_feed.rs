use std::time::Duration;

use crate::oanda::{
    CandlestickGranularity,
    CurrencyPair,
    OandaCandlestickResponse,
};

const OANDA_BASE_URL: &str = "https://api-fxpractice.oanda.com/v3";

pub struct OandaArgs {
    pub auth_token: String,
    pub pair: CurrencyPair,
    pub granularity: CandlestickGranularity,
    pub num_candles: u64,
}

pub async fn query_price_feed(
    oanda_args: &OandaArgs,
    client: &reqwest::Client,
) -> anyhow::Result<OandaCandlestickResponse> {
    let OandaArgs {
        auth_token,
        pair,
        granularity,
        num_candles,
    } = oanda_args;
    let url = format!(
        "{OANDA_BASE_URL}/instruments/{pair}/candles?count={num_candles}&granularity={granularity}"
    );
    let response = client.get(url).bearer_auth(auth_token).send().await?;
    let text = response.text().await?;

    serde_json::from_str(text.as_str()).map_err(|e| e.into())
}

pub async fn poll_price_feed(client: reqwest::Client, oanda_args: OandaArgs) -> anyhow::Result<()> {
    const POLL_INTERVAL_MS: u64 = 5000;
    let mut interval = tokio::time::interval(Duration::from_millis(POLL_INTERVAL_MS));

    loop {
        interval.tick().await;

        match query_price_feed(&oanda_args, &client).await {
            Ok(response) => println!("{response:#?}"),
            Err(e) => eprintln!("Price feed error: {e:#?}"),
        }
    }
}
