pub mod config;
pub mod rate_window;
pub mod state;

use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::Arc,
};

use axum::{
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
use serde::{
    Deserialize,
    Serialize,
};
use solana_address::Address;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};
use solana_keypair::Signer;

use crate::{
    config::get_validated_config,
    state::FaucetState,
};

#[derive(Deserialize)]
struct MintRequest {
    /// Recipient address.
    address: String,
    /// Mint address of the token to dispense.
    mint: String,
    /// Amount in whole tokens (will be multiplied by 10^mint_decimals).
    #[serde(default = "default_amount")]
    amount: u64,
}

fn default_amount() -> u64 {
    1
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    cluster: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let health_check = std::env::args().any(|a| a == "--health-check");
    let cfg = get_validated_config().await?;
    if health_check {
        return Ok(());
    }

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

    let cluster = state.resolve_cluster().await?;
    println!("Faucet starting on cluster: {cluster:?}");
    println!("Faucet address: {}", state.keypair.pubkey());
    println!("Base mint: {}", state.base.mint_address);
    println!("Quote mint: {}", state.quote.mint_address);
    println!("Listening on port {port}");
    println!("Cluster {:#?}", state.cluster);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/faucet", post(faucet_handler))
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
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid address: {}", req.address),
                }),
            )
                .into_response();
        }
    };

    let mint: Address = match req.mint.parse() {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid mint: {}", req.mint),
                }),
            )
                .into_response();
        }
    };

    if !state.is_known_mint(&mint) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "Unknown mint: {mint}. Known mints: base={}, quote={}",
                    state.base.mint_address, state.quote.mint_address
                ),
            }),
        )
            .into_response();
    }

    if req.amount == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Amount must be greater than zero".into(),
            }),
        )
            .into_response();
    }

    let is_base = mint == state.base.mint_address;

    match state.create_signed_transfer(&address, is_base, req.amount) {
        Ok(tx) => Json(tx).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create transaction: {e}"),
            }),
        )
            .into_response(),
    }
}
