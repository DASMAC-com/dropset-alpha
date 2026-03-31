//! Lightweight, nonblocking RPC client utilities for funding accounts, sending transactions,
//! and pretty-printing `dropset`-related transaction logs.

mod instruction_data_at_index;
mod transaction_submit_error;

use std::collections::HashSet;

use anyhow::{
    bail,
    Context,
};
pub use instruction_data_at_index::*;
use solana_address::Address;
use solana_client::{
    client_error::{
        ClientError,
        ClientErrorKind,
    },
    nonblocking::rpc_client::RpcClient,
    rpc_request::{
        RpcError,
        RpcResponseErrorData,
    },
    rpc_response::RpcSimulateTransactionResult,
};
use solana_cluster_type::ClusterType;
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    message::{
        Instruction,
        Message,
    },
    signature::{
        Keypair,
        Signature,
        Signer,
    },
    transaction::Transaction,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta,
    UiTransactionEncoding,
};
use transaction_parser::{
    client_rpc::{
        find_inner_instruction_custom_error_info,
        parse_transaction,
        ParsedTransaction,
    },
    events::dropset_event::DropsetEvent,
    ParseDropsetEvents,
};
pub use transaction_submit_error::*;

use crate::{
    pretty::{
        transaction::PrettyTransaction,
        transaction_error::PrettyTransactionError,
    },
    print_kv,
    LogColor,
};

pub struct CustomRpcClient {
    pub client: RpcClient,
    pub config: SendTransactionConfig,
}

impl Default for CustomRpcClient {
    fn default() -> Self {
        CustomRpcClient {
            client: RpcClient::new_with_commitment(
                "http://localhost:8899".into(),
                CommitmentConfig::confirmed(),
            ),
            config: Default::default(),
        }
    }
}

impl CustomRpcClient {
    pub fn new(client: Option<RpcClient>, config: Option<SendTransactionConfig>) -> Self {
        match (client, config) {
            (Some(client), Some(config)) => Self { client, config },
            (client, config) => {
                let CustomRpcClient {
                    client: default_client,
                    config: default_config,
                } = Default::default();
                Self {
                    client: client.unwrap_or(default_client),
                    config: config.unwrap_or(default_config),
                }
            }
        }
    }

    pub fn new_from_url(url: &str, config: SendTransactionConfig) -> Self {
        CustomRpcClient {
            client: RpcClient::new_with_commitment(url.into(), CommitmentConfig::confirmed()),
            config,
        }
    }

    pub async fn validate_endpoint(&self) -> anyhow::Result<()> {
        self.client
            .get_version()
            .await
            .with_context(|| format!("Failed to connect to Solana RPC at {}", self.client.url()))?;

        Ok(())
    }

    /// Resolves which Solana cluster the RPC is connected to by matching the genesis hash returned
    /// from the RPC.
    ///
    /// Returns the detected [`ClusterType`].
    pub async fn resolve_cluster(&self) -> anyhow::Result<ClusterType> {
        let genesis = self
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

        Ok(cluster)
    }

    pub async fn fund_account(&self, address: &Address) -> anyhow::Result<()> {
        airdrop(&self.client, address).await
    }

    pub async fn fund_new_account(&self) -> anyhow::Result<Keypair> {
        let kp = Keypair::new();
        airdrop(&self.client, &kp.pubkey()).await?;

        Ok(kp)
    }

    /// Signs, submits and confirms a single signer [Transaction] with the signer passed in as the
    /// payer and sole signer.
    /// Instructions that require multiple signers should not be used here as they will obviously
    /// fail.
    pub async fn send_single_signer(
        &self,
        signer: &Keypair,
        instructions: impl AsRef<[Instruction]>,
    ) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
        self.sign_and_submit_instructions(signer, &[signer], instructions.as_ref())
            .await
    }

    /// Signs instructions and then creates, submits and confirms the resulting [Transaction].
    pub async fn sign_and_submit_instructions(
        &self,
        payer: &Keypair,
        signers: &[&Keypair],
        instructions: &[Instruction],
    ) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
        let transaction =
            sign_transaction_with_config(&self.client, payer, signers, instructions, &self.config)
                .await?;

        self.submit_and_confirm_transaction(payer.pubkey(), transaction)
            .await
    }

    /// Submits and confirms an already signed [Transaction].
    pub async fn submit_and_confirm_transaction(
        &self,
        payer_addr: Address,
        transaction: Transaction,
    ) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
        send_transaction_with_config(&self.client, payer_addr, transaction, &self.config).await
    }
}

const MAX_TRIES: u8 = 20;

pub const DEFAULT_FUND_AMOUNT: u64 = 10_000_000_000;

pub async fn airdrop(rpc: &RpcClient, address: &Address) -> anyhow::Result<()> {
    let airdrop_signature: Signature = rpc
        .request_airdrop(address, DEFAULT_FUND_AMOUNT)
        .await
        .context("Failed to request airdrop")?;

    let mut i = 0;
    // Wait for airdrop confirmation.
    while !rpc
        .confirm_transaction(&airdrop_signature)
        .await
        .context("Couldn't confirm transaction")?
        && i < MAX_TRIES
    {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        i += 1;
    }

    if i == MAX_TRIES {
        bail!("Airdrop did not land.");
    }

    Ok(())
}

#[derive(Clone)]
pub struct SendTransactionConfig {
    pub compute_budget: Option<u32>,
    pub debug_logs: Option<bool>,
    pub program_id_filter: HashSet<Address>,
}

impl Default for SendTransactionConfig {
    fn default() -> Self {
        SendTransactionConfig {
            compute_budget: Default::default(),
            debug_logs: Some(true),
            program_id_filter: HashSet::new(),
        }
    }
}

/// A parsed transaction together with all `DropsetEvent`s derived from it.
///
/// This bundles the decoded transaction data with the events extracted from
/// its execution logs, making it easier for callers to work with both in one
/// value.
pub struct ParsedTransactionWithEvents {
    /// The parsed representation of the confirmed transaction.
    pub parsed_transaction: ParsedTransaction,
    /// All `DropsetEvent`s parsed in the transaction.
    pub events: Vec<DropsetEvent>,
}

async fn sign_transaction_with_config(
    rpc: &RpcClient,
    payer: &Keypair,
    signers: &[&Keypair],
    instructions: &[Instruction],
    config: &SendTransactionConfig,
) -> anyhow::Result<Transaction> {
    let bh = rpc
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow::anyhow!("Couldn't get latest blockhash: ({e})"))?;

    let final_instructions: &[Instruction] = &[
        config.compute_budget.map_or(vec![], |budget| {
            vec![
                ComputeBudgetInstruction::set_compute_unit_limit(budget),
                ComputeBudgetInstruction::set_compute_unit_price(1),
            ]
        }),
        instructions.to_vec(),
    ]
    .concat();

    let msg = Message::new(final_instructions, Some(&payer.pubkey()));

    // The payer must always sign since it is paying, so chain it with the rest of the signers.
    let mut tx = Transaction::new_unsigned(msg);
    tx.try_sign(
        &[std::iter::once(payer)
            .chain(signers.iter().cloned())
            .collect::<Vec<_>>()]
        .concat(),
        bh,
    )
    .context("Failed to sign transaction")?;

    Ok(tx)
}

async fn send_transaction_with_config(
    rpc: &RpcClient,
    payer_addr: Address,
    transaction: Transaction,
    config: &SendTransactionConfig,
) -> Result<ParsedTransactionWithEvents, TransactionSubmitError> {
    let res = rpc.send_and_confirm_transaction(&transaction).await;
    match res {
        Ok(signature) => {
            let encoded = fetch_transaction_json(rpc, signature).await?;
            let parsed_transaction = parse_transaction(encoded).map_err(|e| {
                TransactionSubmitError::Other(e.context("Failed to parse transaction"))
            })?;

            let dropset_events = {
                let mut res = vec![];
                for outer in &parsed_transaction.instructions {
                    for inner in &outer.inner_instructions {
                        let parsed_events = inner
                            .parse_events()
                            .map_err(|e| TransactionSubmitError::Other(e.into()))?;
                        res.extend(parsed_events);
                    }
                }

                res
            };

            if matches!(config.debug_logs, Some(true)) {
                let pretty = PrettyTransaction {
                    sender: payer_addr,
                    signature,
                    indent_size: 2,
                    transaction: &parsed_transaction,
                    instruction_filter: &config.program_id_filter,
                }
                .to_string();
                if !pretty.is_empty() {
                    println!("{pretty}");
                }

                for event in dropset_events.iter() {
                    println!("{event:?}");
                }
            }

            Ok(ParsedTransactionWithEvents {
                parsed_transaction,
                events: dropset_events,
            })
        }
        Err(error) => {
            let txn_submit_error = TransactionSubmitError::from_client_error(error, &transaction);
            if matches!(config.debug_logs, Some(true)) {
                let err = PrettyTransactionError::new(&txn_submit_error);
                print!("{err}");
                print_kv!("Sender", payer_addr, LogColor::Gray);
                println!();
            }

            Err(txn_submit_error)
        }
    }
}

async fn fetch_transaction_json(
    rpc: &RpcClient,
    sig: Signature,
) -> anyhow::Result<EncodedConfirmedTransactionWithStatusMeta> {
    rpc.get_transaction_with_config(
        &sig,
        solana_client::rpc_config::RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        },
    )
    .await
    .context("Should be able to fetch transaction with config")
}

/// Checks if an account at the given address exists on-chain.
pub async fn account_exists(rpc: &RpcClient, address: &Address) -> anyhow::Result<bool> {
    Ok(rpc
        .get_account_with_commitment(address, CommitmentConfig::confirmed())
        .await
        .context("Couldn't retrieve account data")?
        .value
        .is_some())
}

/// Returns the program ID and error code for the first inner instruction error, if the simulation
/// resulted in one.
pub fn inner_simulation_error(error: &ClientError) -> Option<(Address, u32)> {
    if let ClientErrorKind::RpcError(RpcError::RpcResponseError {
        data:
            RpcResponseErrorData::SendTransactionPreflightFailure(RpcSimulateTransactionResult {
                logs: Some(logs),
                ..
            }),
        ..
    }) = error.kind()
    {
        return find_inner_instruction_custom_error_info(logs);
    }

    None
}
