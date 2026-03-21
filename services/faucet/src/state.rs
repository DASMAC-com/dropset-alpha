use std::{
    collections::HashSet,
    sync::{
        Arc,
        RwLock,
    },
    time::Duration,
};

use client::{
    context::token::TokenContext,
    transactions::CustomRpcClient,
};
use dropset_services_shared::config::ValidSharedConfig;
use solana_address::Address;
use solana_cluster_type::ClusterType;
use solana_keypair::Keypair;
use solana_sdk::{
    message::Message,
    transaction::Transaction,
};

use crate::config::ValidFaucetConfig;

pub struct FaucetState {
    pub keypair: Keypair,
    pub rpc: Arc<CustomRpcClient>,
    pub base: TokenContext,
    pub quote: TokenContext,
    pub max_public_tokens: u64,
    pub max_allowlist_tokens: u64,
    pub allowlist: HashSet<Address>,
    pub cluster: ClusterType,
    recent_blockhash: Arc<RwLock<solana_sdk::hash::Hash>>,
}

impl FaucetState {
    pub async fn new(config: ValidFaucetConfig, rpc: Arc<CustomRpcClient>) -> anyhow::Result<Self> {
        let ValidFaucetConfig {
            shared,
            max_public_tokens,
            max_allowlist_tokens,
            allowlist,
            ..
        } = config;

        let ValidSharedConfig {
            keypair,
            base,
            quote,
            cluster,
            ..
        } = shared;

        let (initial_blockhash, _) = rpc
            .client
            .get_latest_blockhash_with_commitment(rpc.client.commitment())
            .await?;

        let recent_blockhash = Arc::new(RwLock::new(initial_blockhash));

        // Refresh the blockhash every 20 seconds. Solana blockhashes are valid
        // for ~60s (~150 slots), so this gives users ~40s to sign and submit.
        // `get_new_latest_blockhash` retries until a genuinely new hash appears.
        {
            let blockhash = Arc::clone(&recent_blockhash);
            let rpc = Arc::clone(&rpc);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(20));
                interval.tick().await; // skip immediate first tick
                loop {
                    interval.tick().await;
                    let current = *blockhash
                        .read()
                        .expect("Blockhash lock should not be poisoned");
                    match rpc.client.get_new_latest_blockhash(&current).await {
                        Ok(hash) => {
                            *blockhash
                                .write()
                                .expect("Blockhash lock should not be poisoned") = hash;
                        }
                        Err(e) => {
                            tracing::error!("Failed to refresh blockhash: {e}");
                        }
                    }
                }
            });
        }

        Ok(Self {
            keypair,
            rpc,
            base,
            quote,
            max_public_tokens,
            max_allowlist_tokens,
            allowlist,
            cluster,
            recent_blockhash,
        })
    }

    pub fn recent_blockhash(&self) -> solana_sdk::hash::Hash {
        *self
            .recent_blockhash
            .read()
            .expect("Blockhash lock should not be poisoned")
    }

    /// Caps the requested amount based on allowlist membership.
    /// `amount` in is in whole tokens; this returns the capped value in atoms.
    pub fn cap_amount(&self, receiver: &Address, amount: u64, is_base: bool) -> u64 {
        let max_tokens = if self.allowlist.contains(receiver) {
            self.max_allowlist_tokens
        } else {
            self.max_public_tokens
        };
        let capped = amount.min(max_tokens);
        let decimals = if is_base {
            self.base.mint_decimals
        } else {
            self.quote.mint_decimals
        } as u32;
        capped.saturating_mul(10u64.saturating_pow(decimals))
    }

    /// Build and partially sign a transaction that mints tokens to the receiver, where the receiver
    /// is set as the fee payer.
    ///
    /// Returns the partially signed [Transaction] that the receiver must add their signature to and
    /// submit.
    pub fn create_signed_transfer(
        &self,
        receiver: &Address,
        is_base: bool,
        amount: u64,
    ) -> anyhow::Result<Transaction> {
        let capped_amount = self.cap_amount(receiver, amount, is_base);
        let token_ctx = if is_base { &self.base } else { &self.quote };
        let create_ata = token_ctx.create_ata_idempotent(receiver, receiver);
        let mint_to = token_ctx.mint_to_user(receiver, capped_amount)?;

        let message = Message::new(&[create_ata, mint_to], Some(receiver));
        let mut tx = Transaction::new_unsigned(message);

        tx.try_partial_sign(&[&self.keypair], self.recent_blockhash())?;

        Ok(tx)
    }
}
