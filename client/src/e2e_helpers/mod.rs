use solana_address::Address;
use solana_sdk::{
    program_pack::Pack,
    signature::{
        Keypair,
        Signer,
    },
};
use spl_token_interface::state::{
    Account,
    Mint,
};
use transaction_parser::views::{
    try_market_view_all_from_owner_and_data,
    MarketSeatView,
    MarketViewAll,
};

use crate::{
    context::{
        market::MarketContext,
        token::TokenContext,
    },
    single_signer_instruction::SingleSignerInstruction,
    token_instructions::create_and_initialize_token_instructions,
    transactions::{
        account_exists,
        CustomRpcClient,
        ParsedTransactionWithEvents,
        DEFAULT_FUND_AMOUNT,
    },
};

pub mod test_accounts;

/// Convenience harness for end-to-end tests and examples.
///
/// Upon instantiation it:
/// - Airdrops [`crate::transactions::DEFAULT_FUND_AMOUNT`] lamports to the
///   [`test_accounts::default_payer`] account.
/// - Creates and registers a new market backed by two newly-created SPL token mints (base/quote).
///   The [`test_accounts::default_payer`] account is the registrant.
/// - Airdrops [`crate::transactions::DEFAULT_FUND_AMOUNT`] lamports to each user.
/// - Creates base/quote associated token accounts (ATAs) for each user.
/// - Mints the specified `base` and `quote` amounts to each user's ATAs if the amount is != 0.
pub struct E2e {
    pub rpc: CustomRpcClient,
    pub market: MarketContext,
    pub register_market_txn: ParsedTransactionWithEvents,
}

/// Setup config for a user in [`E2e::new_users_and_market`].
///
/// Bundles a signer with initial `base` / `quote` amounts.
pub struct User<'a> {
    pub base: u64,
    pub quote: u64,
    pub keypair: &'a Keypair,
}

impl<'a> User<'a> {
    pub fn new(keypair: &'a Keypair, base: u64, quote: u64) -> Self {
        Self {
            base,
            quote,
            keypair,
        }
    }

    pub fn address(&self) -> Address {
        self.keypair.pubkey()
    }
}

impl E2e {
    pub async fn new_users_and_market(
        rpc: Option<CustomRpcClient>,
        users: impl AsRef<[User<'_>]>,
    ) -> anyhow::Result<Self> {
        E2e::new_users_and_market_with_options(rpc, users, None, None, None).await
    }

    pub async fn new_users_and_market_with_options(
        rpc: Option<CustomRpcClient>,
        users: impl AsRef<[User<'_>]>,
        base_mint_decimals: Option<u8>,
        quote_mint_decimals: Option<u8>,
        mint_authority: Option<&Keypair>,
    ) -> anyhow::Result<Self> {
        let rpc = rpc.unwrap_or_default();

        let default_payer = test_accounts::default_payer().insecure_clone();
        if !account_exists(&rpc.client, &default_payer.pubkey()).await? {
            rpc.fund_account(&default_payer.pubkey()).await?;
        }

        // Create new random base/quote tokens and derive the market context from them.
        let base_decimals = base_mint_decimals.unwrap_or(8);
        let quote_decimals = quote_mint_decimals.unwrap_or(8);
        let (base, base_mint_authority) =
            create_token(&rpc, None, base_decimals, mint_authority).await?;
        let (quote, quote_mint_authority) =
            create_token(&rpc, None, quote_decimals, mint_authority).await?;
        let market = MarketContext::new(base, quote);

        let register_market_txn = market
            .register_market(default_payer.pubkey(), 10)
            .send_single_signer(&rpc, &default_payer)
            .await?;

        // Fund and create the user accounts, create their base/quote associated token accounts,
        // and mint + deposit the specified base/quote amounts to each user if the amount
        // != 0.
        for user in users.as_ref().iter() {
            rpc.fund_account(&user.address()).await?;

            create_ata(&rpc, &market.base, user.keypair).await?;
            create_ata(&rpc, &market.quote, user.keypair).await?;

            if user.base != 0 {
                mint_to(
                    &rpc,
                    &market.base,
                    &base_mint_authority,
                    user.keypair,
                    user.base,
                )
                .await?;
            }
            if user.quote != 0 {
                mint_to(
                    &rpc,
                    &market.quote,
                    &quote_mint_authority,
                    user.keypair,
                    user.quote,
                )
                .await?;
            }
        }

        Ok(Self {
            rpc,
            market,
            register_market_txn,
        })
    }

    pub async fn view_market(&self) -> anyhow::Result<MarketViewAll> {
        let market_account = self.rpc.client.get_account(&self.market.market).await?;
        try_market_view_all_from_owner_and_data(market_account.owner, &market_account.data)
    }

    pub async fn fetch_seat(&self, user: &Address) -> anyhow::Result<Option<MarketSeatView>> {
        let market = self.view_market().await?;
        Ok(self.market.find_seat(&market.seats, user))
    }

    pub fn find_seat(&self, seats: &[MarketSeatView], user: &Address) -> Option<MarketSeatView> {
        self.market.find_seat(seats, user)
    }

    pub async fn get_base_balance(&self, user: &Address) -> anyhow::Result<u64> {
        get_token_balance(&self.rpc, &self.market.base, user).await
    }

    pub async fn get_quote_balance(&self, user: &Address) -> anyhow::Result<u64> {
        get_token_balance(&self.rpc, &self.market.quote, user).await
    }
}

/// Creates a new token mint on-chain. Returns the [`TokenContext`] and the mint authority keypair.
///
/// If `mint_authority` is provided, it is used as the mint authority (and funded if needed).
/// Otherwise a fresh keypair is generated.
async fn create_token(
    rpc: &CustomRpcClient,
    token_program: Option<Address>,
    mint_decimals: u8,
    mint_authority: Option<&Keypair>,
) -> anyhow::Result<(TokenContext, Keypair)> {
    let authority = match mint_authority {
        Some(kp) => {
            if account_exists(&rpc.client, &kp.pubkey()).await? {
                let balance = rpc.client.get_balance(&kp.pubkey()).await?;
                // Only airdrop lamports if the account needs it. Otherwise this will fail with
                // an overflow error after multiple airdrops because the amount is huge.
                if balance < DEFAULT_FUND_AMOUNT {
                    rpc.fund_account(&kp.pubkey()).await?;
                }
            } else {
                rpc.fund_account(&kp.pubkey()).await?;
            }
            kp.insecure_clone()
        }
        None => rpc.fund_new_account().await?,
    };
    let mint = Keypair::new();
    let token_program = token_program.unwrap_or(spl_token_interface::ID);

    let mint_rent = rpc
        .client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await?;

    let (create_mint_account, initialize_mint) = create_and_initialize_token_instructions(
        &authority.pubkey(),
        &mint.pubkey(),
        mint_rent,
        mint_decimals,
        &token_program,
    )?;

    rpc.sign_and_submit_instructions(
        &authority,
        &[&mint],
        &[create_mint_account, initialize_mint],
    )
    .await?;

    let token = TokenContext::new(
        Some(authority.pubkey()),
        mint.pubkey(),
        token_program,
        mint_decimals,
    );
    Ok((token, authority))
}

/// Creates an associated token account for a user.
async fn create_ata(
    rpc: &CustomRpcClient,
    token: &TokenContext,
    user: &Keypair,
) -> anyhow::Result<Address> {
    let ix = token.create_ata(&user.pubkey(), &user.pubkey());
    rpc.sign_and_submit_instructions(user, &[], &[ix]).await?;
    Ok(token.get_ata_for(&user.pubkey()))
}

/// Mints tokens to a user's ATA.
async fn mint_to(
    rpc: &CustomRpcClient,
    token: &TokenContext,
    mint_authority: &Keypair,
    user: &Keypair,
    amount: u64,
) -> anyhow::Result<()> {
    let ix = token.mint_to_user(&user.pubkey(), amount)?;
    rpc.sign_and_submit_instructions(user, &[mint_authority], &[ix])
        .await?;
    Ok(())
}

/// Fetches a token balance for a user.
async fn get_token_balance(
    rpc: &CustomRpcClient,
    token: &TokenContext,
    user: &Address,
) -> anyhow::Result<u64> {
    let ata = token.get_ata_for(user);
    let account_data = rpc.client.get_account_data(&ata).await?;
    let account_data = Account::unpack(&account_data)?;
    Ok(account_data.amount)
}
