pub mod config;
pub mod state;

use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{atomic::Ordering, Arc},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use client::transactions::{CustomRpcClient, SendTransactionConfig};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use solana_address::Address;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
use solana_keypair::Signer;
use tokio::sync::{mpsc, oneshot};

use crate::{
    config::get_validated_config,
    state::{cooldown_eviction_loop, processor_loop, FaucetRequest, FaucetState},
};

#[derive(Deserialize)]
struct MintRequest {
    /// Recipient address.
    address: String,
    /// Mint address of the token to dispense.
    mint: String,
    /// Amount in whole tokens (will be multiplied by 10^decimals).
    #[serde(default = "default_amount")]
    amount: u64,
}

fn default_amount() -> u64 {
    1
}

#[derive(Serialize)]
struct MintResponse {
    signature: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    cluster: String,
    slow_mode: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let health_check = std::env::args().any(|a| a == "--health-check");
    let cfg = get_validated_config().await?;
    if health_check {
        return Ok(());
    }

    let shared = cfg.shared;

    let rpc = CustomRpcClient::new(
        Some(RpcClient::new_with_commitment(
            shared.rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        )),
        Some(SendTransactionConfig {
            compute_budget: Some(400_000),
            debug_logs: Some(true),
            program_id_filter: HashSet::new(),
        }),
    );

    let (tx, rx) = mpsc::unbounded_channel::<FaucetRequest>();

    let state = Arc::new(FaucetState {
        keypair: Arc::new(shared.keypair),
        rpc,
        base_mint: shared.base_mint,
        quote_mint: shared.quote_mint,
        cooldown: std::time::Duration::from_secs(cfg.cooldown_secs),
        max_public_tokens: cfg.max_public_tokens,
        max_whitelist_tokens: cfg.max_whitelist_tokens,
        whitelist: cfg.whitelist,
        mint_cache: DashMap::new(),
        cooldowns: DashMap::new(),
        slow_mode: false.into(),
        tx,
    });

    let cluster = state.resolve_cluster().await?;
    println!("Faucet starting on cluster: {cluster:?}");
    println!("Faucet address: {}", state.keypair.pubkey());
    println!("Base mint: {}", state.base_mint);
    println!("Quote mint: {}", state.quote_mint);

    // Eagerly resolve both mints at startup to verify authority.
    state.resolve_mint(&state.base_mint).await?;
    state.resolve_mint(&state.quote_mint).await?;
    println!("Mint authority verified for both mints.");

    println!("Listening on port {}", cfg.port);

    // Spawn the processor and cooldown eviction tasks.
    let processor_state = Arc::clone(&state);
    tokio::spawn(async move { processor_loop(processor_state, rx).await });

    let eviction_state = Arc::clone(&state);
    tokio::spawn(async move { cooldown_eviction_loop(eviction_state).await });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/faucet", post(faucet_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler(State(state): State<Arc<FaucetState>>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        cluster: format!("{:?}", "unknown"), // TODO: cache cluster at startup
        slow_mode: state.slow_mode.load(Ordering::Relaxed),
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
                    state.base_mint, state.quote_mint
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

    // Check cooldown before queuing.
    if let Err(e) = state.check_cooldown(&address, &mint) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response();
    }

    let (respond_tx, respond_rx) = oneshot::channel();

    let faucet_req = FaucetRequest {
        address,
        mint,
        amount: req.amount,
        respond: respond_tx,
    };

    if state.tx.send(faucet_req).is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Faucet processor is down".into(),
            }),
        )
            .into_response();
    }

    match respond_rx.await {
        Ok(Ok(signature)) => (StatusCode::OK, Json(MintResponse { signature })).into_response(),
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: err }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Request dropped".into(),
            }),
        )
            .into_response(),
    }
}
