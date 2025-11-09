use std::collections::{
    HashMap,
    HashSet,
};

use client::{
    context::market::MarketContext,
    test_accounts::*,
    transactions::{
        CustomRpcClient,
        SendTransactionConfig,
    },
};
use dropset_interface::state::sector::SectorIndex;
use itertools::Itertools;
use solana_instruction::Instruction;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc = &CustomRpcClient::new(
        None,
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(true),
            program_id_filter: HashSet::from([dropset_interface::program::ID.into()]),
        }),
    );

    let payer = rpc.fund_new_account().await?;

    let market_ctx = MarketContext::new_market(rpc).await?;

    rpc.send_and_confirm_txn(
        &payer,
        &[&payer],
        &[market_ctx.register_market(payer.pubkey(), 10)],
    )
    .await?;

    // Insert out of order to ensure that it's ordered later.
    let signers: Vec<&Keypair> = vec![&USER_5, &USER_2, &USER_4, &USER_1, &USER_3];

    for user in signers.iter() {
        rpc.fund_account(&user.pubkey()).await?;
        market_ctx.base.create_ata_for(rpc, user).await?;
        market_ctx.quote.create_ata_for(rpc, user).await?;
        market_ctx.base.mint_to(rpc, user, 10000).await?;
        market_ctx.quote.mint_to(rpc, user, 10000).await?;
    }

    let user_pks: Vec<Pubkey> = signers.iter().map(|u| u.pubkey()).collect();

    let seat_creations: Vec<Instruction> = user_pks
        .iter()
        // Deposits 1 base token in order to create the seat.
        .map(|pk| market_ctx.create_seat(*pk))
        .collect();

    rpc.send_and_confirm_txn(signers[0], &signers, &seat_creations)
        .await?;

    let seats: Vec<SectorIndex> = user_pks
        .iter()
        .map(|user| {
            market_ctx
                .find_seat(rpc, user)
                .ok()
                .flatten()
                .expect("User should have a seat")
                .index
        })
        .collect();

    // HashMap<Pubkey, (deposit_amount, withdraw_amount)>
    let base_amounts: HashMap<Pubkey, (u64, u64)> = HashMap::from([
        (USER_1.pubkey(), (100, 10)),
        (USER_2.pubkey(), (100, 20)),
        (USER_3.pubkey(), (100, 30)),
        (USER_4.pubkey(), (100, 40)),
        (USER_5.pubkey(), (100, 50)),
    ]);

    let deposits_and_withdraws: Vec<Instruction> = user_pks
        .iter()
        .zip(seats)
        .flat_map(|(user, seat)| {
            let (deposit, withdraw) = base_amounts.get(user).unwrap();
            [
                market_ctx.deposit_base(*user, *deposit, seat),
                market_ctx.withdraw_base(*user, *withdraw, seat),
            ]
        })
        .collect();

    rpc.send_and_confirm_txn(signers[0], &signers, &deposits_and_withdraws)
        .await?;

    let expected_base = base_amounts
        .into_iter()
        .map(|pk_and_amts| {
            let (pubkey, (deposit, withdraw)) = pk_and_amts;
            (pubkey, deposit, withdraw)
        })
        // Sort by the pubkey.
        .sorted_by_key(|v| v.0)
        .collect_vec();

    let market = market_ctx.view_market(rpc)?;

    // Check that seats are ordered by pubkey (ascending) and compare the final state of each user's
    // seat to the expected state.
    for (seat, expected_seat) in market.sectors.iter().zip_eq(expected_base) {
        let (expected_pk, expected_base_dep, expected_base_wd) = expected_seat;
        assert_eq!(seat.user, expected_pk);
        let amount_from_create_seat = 1;
        let base_remaining = (expected_base_dep + amount_from_create_seat) - expected_base_wd;
        assert_eq!(seat.base_available, base_remaining);
        assert_eq!(seat.base_deposited, base_remaining);
    }

    Ok(())
}
