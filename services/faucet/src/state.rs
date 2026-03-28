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

/// The blockhash refresh. This is, effectively, how often the same address can sign and submit a
/// [Transaction] signed by the faucet to transfer base or mint tokens with the same amount of
/// atoms. If the same amount of atoms is requested twice for the same token and the two blockhashes
/// are equal, the sender will get an `AlreadyProcessed` error when submitting the transaction.
pub const BLOCKHASH_REFRESH: u64 = 5;

impl FaucetState {
    pub async fn new(config: ValidFaucetConfig, rpc: Arc<CustomRpcClient>) -> anyhow::Result<Self> {
        let ValidFaucetConfig {
            keypair,
            base,
            quote,
            cluster,
            max_public_tokens,
            max_allowlist_tokens,
            allowlist,
            ..
        } = config;

        let (initial_blockhash, _) = rpc
            .client
            .get_latest_blockhash_with_commitment(rpc.client.commitment())
            .await?;

        let recent_blockhash = Arc::new(RwLock::new(initial_blockhash));

        // Refresh the blockhash every BLOCKHASH_REFRESH seconds. Solana blockhashes are valid
        // for ~60s (~150 slots), so this gives users 60 - BLOCKHASH_REFRESH seconds to sign and
        // submit. `get_new_latest_blockhash` retries until a genuinely new hash appears.
        {
            let blockhash = Arc::clone(&recent_blockhash);
            let rpc = Arc::clone(&rpc);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(BLOCKHASH_REFRESH));
                // Skip immediate first tick.
                interval.tick().await;
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

    /// Caps the requested amount based on allowlist membership. The input `amount` should be in
    /// whole tokens.
    ///
    /// This returns the capped value as a pair of (atoms, whole_tokens).
    pub fn cap_amount(&self, receiver: &Address, amount: u64, is_base: bool) -> (u64, u64) {
        let max_tokens = if self.allowlist.contains(receiver) {
            self.max_allowlist_tokens
        } else {
            self.max_public_tokens
        };
        let capped_whole_tokens = amount.min(max_tokens);
        let decimals = if is_base {
            self.base.mint_decimals
        } else {
            self.quote.mint_decimals
        } as u32;

        let capped_atoms = capped_whole_tokens.saturating_mul(10u64.saturating_pow(decimals));

        (capped_atoms, capped_whole_tokens)
    }

    /// Build and partially sign a transaction that mints tokens to the receiver, where the receiver
    /// is set as the fee payer.
    ///
    /// Returns the partially signed [Transaction] that the receiver must add their signature to and
    /// submit, paired with the capped token amount (as whole tokens) sent in the transaction.
    pub fn create_signed_mint_to_user_txn(
        &self,
        receiver: &Address,
        is_base: bool,
        amount: u64,
    ) -> anyhow::Result<(Transaction, u64)> {
        let (capped_atoms, capped_whole_tokens) = self.cap_amount(receiver, amount, is_base);
        let token_ctx = if is_base { &self.base } else { &self.quote };
        let create_ata = token_ctx.create_ata_idempotent(receiver, receiver);
        let mint_to = token_ctx.mint_to_user(receiver, capped_atoms)?;

        let message = Message::new(&[create_ata, mint_to], Some(receiver));
        let mut tx = Transaction::new_unsigned(message);

        tx.try_partial_sign(&[&self.keypair], self.recent_blockhash())?;

        Ok((tx, capped_whole_tokens))
    }
}
