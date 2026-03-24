use dropset_services_shared::config::{
    ServiceConfig,
    SharedConfigInput,
    ValidSharedConfig,
};
use solana_keypair::Signer;

const SERVICE: ServiceConfig = ServiceConfig::Faucet;

pub type ValidFaucetConfig = ValidSharedConfig;

pub type FaucetConfigInput = SharedConfigInput;

async fn validate_config_and_endpoint() -> anyhow::Result<ValidFaucetConfig> {
    let shared = ValidSharedConfig::new_validated(SERVICE).await?;
    validate_mint_authorities(&shared)?;

    Ok(shared)
}

fn validate_mint_authorities(shared: &ValidSharedConfig) -> anyhow::Result<()> {
    for is_base in [true, false] {
        let token = if is_base { &shared.base } else { &shared.quote };
        let token_type = if is_base { "Base" } else { "Quote" };

        match token.mint_authority {
            Some(ma) => anyhow::ensure!(
                ma == shared.keypair.pubkey(),
                "{token_type} token mint authority {:#?} doesn't match faucet address {:#?}",
                token.mint_authority,
                shared.keypair.pubkey(),
            ),
            None => anyhow::bail!(
                "Faucet {} token doesn't have a mint authority",
                token_type.to_lowercase()
            ),
        };
    }

    Ok(())
}

pub async fn get_validated_config() -> anyhow::Result<ValidFaucetConfig> {
    validate_config_and_endpoint().await
}
