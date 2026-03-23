//! See the tests for this crate at [dropset_services_shared::faucet_client].

pub mod config;
pub mod state;

use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        get,
        post,
    },
    Json,
    Router,
};
use client::transactions::{
    CustomRpcClient,
    SendTransactionConfig,
};
use dropset_services_shared::faucet_client::{
    MintRequest,
    MintResponse,
};
use serde::Serialize;
use solana_address::Address;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};
use solana_keypair::Signer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::{
    config::get_validated_config,
    state::FaucetState,
};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    cluster: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let health_check = std::env::args().any(|a| a == "--health-check");
    let cfg = get_validated_config().await?;
    if health_check {
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let port = cfg.port;

    let rpc = Arc::new(CustomRpcClient::new(
        Some(RpcClient::new_with_commitment(
            cfg.shared.rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        )),
        Some(SendTransactionConfig {
            compute_budget: Some(400_000),
            debug_logs: Some(true),
            program_id_filter: HashSet::new(),
        }),
    ));

    let state = Arc::new(FaucetState::new(cfg, rpc).await?);

    tracing::info!(port);
    tracing::info!(rpc_url = state.rpc.client.url());
    tracing::info!(cluster = ?state.cluster);
    tracing::info!(mint_authority = %state.keypair.pubkey());
    tracing::info!(base_mint = %state.base.mint_address);
    tracing::info!(quote_mint = %state.quote.mint_address);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/faucet", post(faucet_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler(State(state): State<Arc<FaucetState>>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        cluster: format!("{:?}", state.cluster),
    })
}

async fn faucet_handler(
    State(state): State<Arc<FaucetState>>,
    Json(req): Json<MintRequest>,
) -> impl IntoResponse {
    let address: Address = match req.address.parse() {
        Ok(a) => a,
        Err(_) => {
            return respond_err(
                StatusCode::BAD_REQUEST,
                format!("Invalid address: {}", req.address),
            );
        }
    };

    if req.amount == 0 {
        return respond_err(
            StatusCode::BAD_REQUEST,
            "Amount must be greater than zero".to_string(),
        );
    }

    match state.create_signed_mint_to_user_txn(&address, req.is_base, req.amount) {
        Ok(transaction) => {
            let token = if req.is_base { "base" } else { "quote" };
            tracing::info!(%address, token, amount = req.amount, "Signed mint transaction");
            Json(MintResponse { transaction }).into_response()
        }
        Err(e) => respond_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create transaction: {e}"),
        ),
    }
}

fn respond_err(status: StatusCode, error_msg: impl ToString) -> axum::http::Response<Body> {
    let body = ErrorResponse {
        error: error_msg.to_string(),
    };

    if !status.is_success() {
        tracing::warn!(%status, body = %serde_json::to_string(&body).unwrap_or_default());
    }
    (status, Json(body)).into_response()
}
