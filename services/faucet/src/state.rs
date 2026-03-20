use std::{
    collections::HashSet,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};

use anyhow::Context;
use client::{
    context::token::TokenContext,
    transactions::CustomRpcClient,
};
use dashmap::DashMap;
use solana_address::Address;
use solana_client::rpc_config::CommitmentConfig;
use solana_cluster_type::ClusterType;
use solana_keypair::{
    Keypair,
    Signer,
};
use solana_sdk::message::Instruction;
use tokio::{
    sync::{
        mpsc,
        oneshot,
    },
    time::Instant,
};

use crate::rate_window::RateWindow;

pub struct FaucetRequest {
    pub address: Address,
    pub mint: Address,
    pub amount: u64,
    pub respond: oneshot::Sender<Result<String, String>>,
}

pub struct FaucetState {
    pub keypair: Arc<Keypair>,
    pub rpc: CustomRpcClient,
    pub base_mint: Address,
    pub quote_mint: Address,
    pub cooldown: Duration,
    pub max_public_tokens: u64,
    pub max_whitelist_tokens: u64,
    pub whitelist: HashSet<Address>,
    /// Cache of fetched mint TokenContexts, keyed by mint address.
    pub mint_cache: DashMap<Address, Arc<TokenContext>>,
    /// Per-(address, mint) cooldown tracking.
    pub cooldowns: DashMap<(Address, Address), Instant>,
    /// Whether the processor is currently in slow (batched) mode.
    pub slow_mode: AtomicBool,
    /// Maximum mint requests per Solana transaction.
    pub max_batch_size: usize,
    /// Minimum delay between consecutive `sendTransaction` RPC calls.
    pub min_tx_interval: Duration,
    /// Channel for submitting requests to the processor.
    pub tx: mpsc::UnboundedSender<FaucetRequest>,
}

impl FaucetState {
    /// Returns `true` if the mint is one of the configured base/quote mints.
    pub fn is_known_mint(&self, mint: &Address) -> bool {
        *mint == self.base_mint || *mint == self.quote_mint
    }

    /// Resolves which Solana cluster the RPC is connected to by matching
    /// the genesis hash. Refuses to operate against mainnet-beta.
    ///
    /// Returns the detected [`ClusterType`] for logging/display.
    pub async fn resolve_cluster(&self) -> anyhow::Result<ClusterType> {
        let genesis = self
            .rpc
            .client
            .get_genesis_hash()
            .await
            .context("Failed to fetch genesis hash")?;

        let cluster = [
            ClusterType::MainnetBeta,
            ClusterType::Testnet,
            ClusterType::Devnet,
        ]
        .into_iter()
        .find(|c| c.get_genesis_hash().is_some_and(|h| h == genesis))
        .unwrap_or(ClusterType::Development);

        anyhow::ensure!(
            cluster != ClusterType::MainnetBeta,
            "Refusing to operate against mainnet-beta. \
             The faucet is only for testnet/devnet/localnet."
        );

        Ok(cluster)
    }

    /// Looks up or fetches the [`TokenContext`] for `mint`, caching the result.
    /// Returns an error if the mint doesn't exist or this keypair is not the
    /// mint authority.
    pub async fn resolve_mint(&self, mint: &Address) -> anyhow::Result<Arc<TokenContext>> {
        if let Some(entry) = self.mint_cache.get(mint) {
            return Ok(Arc::clone(entry.value()));
        }

        let account = self
            .rpc
            .client
            .get_account_with_commitment(mint, CommitmentConfig::confirmed())
            .await
            .with_context(|| format!("RPC error fetching mint {mint}"))?
            .value
            .ok_or_else(|| anyhow::anyhow!("Mint account {mint} does not exist on-chain"))?;

        let ctx = TokenContext::from_account_data(*mint, account.owner, &account.data)?;

        let authority = ctx
            .mint_authority
            .ok_or_else(|| anyhow::anyhow!("Mint {mint} has no mint authority"))?;

        anyhow::ensure!(
            authority == self.keypair.pubkey(),
            "Faucet keypair {} is not the mint authority for {mint} (authority: {authority})",
            self.keypair.pubkey(),
        );

        let ctx = Arc::new(ctx);
        self.mint_cache.insert(*mint, Arc::clone(&ctx));
        Ok(ctx)
    }

    /// Checks and updates the per-address cooldown. Returns `Err` if the
    /// address is still in cooldown.
    pub fn check_cooldown(&self, address: &Address, mint: &Address) -> anyhow::Result<()> {
        let key = (*address, *mint);
        if let Some(last) = self.cooldowns.get(&key) {
            let elapsed = last.elapsed();
            if elapsed < self.cooldown {
                let remaining = self.cooldown - elapsed;
                anyhow::bail!(
                    "Cooldown active for {address} on mint {mint}. \
                     Try again in {remaining:.0?}."
                );
            }
        }
        self.cooldowns.insert(key, Instant::now());
        Ok(())
    }

    /// Caps the requested amount based on whitelist membership.
    /// `amount` is in whole tokens — this returns the capped value in atoms.
    pub fn cap_amount(&self, address: &Address, amount: u64, decimals: u8) -> u64 {
        let max_tokens = if self.whitelist.contains(address) {
            self.max_whitelist_tokens
        } else {
            self.max_public_tokens
        };
        let capped = amount.min(max_tokens);
        capped.saturating_mul(10u64.pow(decimals as u32))
    }
}

/// Runs the processor loop. Reads requests from `rx`, decides fast vs. slow
/// mode based on the sliding window, and submits transactions.
pub async fn processor_loop(
    state: Arc<FaucetState>,
    mut rx: mpsc::UnboundedReceiver<FaucetRequest>,
) {
    let mut rate = RateWindow::from_interval(state.min_tx_interval.as_millis() as u64);

    loop {
        // Wait for at least one request.
        let first = match rx.recv().await {
            Some(req) => req,
            None => return, // channel closed
        };

        let mut batch = vec![first];

        rate.prune();

        if rate.is_slow() {
            // Sleep to accumulate more requests, then re-evaluate.
            tokio::time::sleep(rate.drain_interval()).await;
            rate.try_exit_slow();
        }

        // Drain all pending requests from the channel.
        while let Ok(req) = rx.try_recv() {
            batch.push(req);
        }

        // Record this batch and update mode.
        let slow = rate.record(batch.len());
        state.slow_mode.store(slow, Ordering::Relaxed);

        // Process in sub-batches to stay within transaction size limits.
        // Delay between sub-batches to respect the Solana RPC rate limit.
        let mut remaining = batch;
        let mut first_chunk = true;
        while !remaining.is_empty() {
            if !first_chunk {
                tokio::time::sleep(state.min_tx_interval).await;
            }
            first_chunk = false;
            let take = remaining.len().min(state.max_batch_size);
            let chunk: Vec<FaucetRequest> = remaining.drain(..take).collect();
            process_batch(&state, chunk).await;
        }
    }
}

/// Periodic task that evicts stale cooldown entries every 5 minutes.
pub async fn cooldown_eviction_loop(state: Arc<FaucetState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    interval.tick().await; // first tick is immediate

    loop {
        interval.tick().await;
        let cooldown = state.cooldown;
        state
            .cooldowns
            .retain(|_, last_request| last_request.elapsed() < cooldown);
    }
}

async fn process_batch(state: &FaucetState, batch: Vec<FaucetRequest>) {
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut responders: Vec<oneshot::Sender<Result<String, String>>> = Vec::new();

    for req in batch {
        let ctx = match state.resolve_mint(&req.mint).await {
            Ok(ctx) => ctx,
            Err(e) => {
                let _ = req.respond.send(Err(e.to_string()));
                continue;
            }
        };

        let amount = state.cap_amount(&req.address, req.amount, ctx.mint_decimals);
        if amount == 0 {
            let _ = req.respond.send(Err("Amount resolves to zero".into()));
            continue;
        }

        let faucet_addr = state.keypair.pubkey();
        instructions.push(ctx.create_ata_idempotent(&faucet_addr, &req.address));

        match ctx.mint_to_user(&req.address, amount) {
            Ok(ix) => instructions.push(ix),
            Err(e) => {
                // Remove the create_ata instruction that was just pushed.
                instructions.pop();
                let _ = req.respond.send(Err(e.to_string()));
                continue;
            }
        }

        responders.push(req.respond);
    }

    if instructions.is_empty() {
        return;
    }

    let result = state
        .rpc
        .send_single_signer(&state.keypair, &instructions)
        .await;

    match result {
        Ok(parsed) => {
            let sig = parsed.parsed_transaction.signature.to_string();
            for tx in responders {
                let _ = tx.send(Ok(sig.clone()));
            }
        }
        Err(e) => {
            let err_msg = anyhow::Error::from(e).to_string();
            for tx in responders {
                let _ = tx.send(Err(err_msg.clone()));
            }
        }
    }
}
