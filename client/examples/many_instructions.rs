use std::collections::{
    HashMap,
    HashSet,
};

use client::{
    e2e_helpers::{
        test_accounts,
        E2e,
        User,
    },
    transactions::{
        CustomRpcClient,
        SendTransactionConfig,
    },
};
use dropset_interface::state::sector::SectorIndex;
use itertools::Itertools;
use solana_address::Address;
use solana_instruction::Instruction;
use solana_sdk::signer::Signer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc = CustomRpcClient::new(
        None,
        Some(SendTransactionConfig {
            compute_budget: Some(2000000),
            debug_logs: Some(true),
            program_id_filter: HashSet::from([dropset_interface::program::ID]),
        }),
    );
    // Create the collection of users out of order so that the order must change when they're
    // sorted on insert later.
    let users = [
        User::new(test_accounts::acc_5555(), 10000, 10000),
        User::new(test_accounts::acc_2222(), 10000, 10000),
        User::new(test_accounts::acc_4444(), 10000, 10000),
        User::new(test_accounts::acc_1111(), 10000, 10000),
        User::new(test_accounts::acc_3333(), 10000, 10000),
    ];
    let e2e = E2e::new_users_and_market(Some(rpc), &users).await?;

    // Create the seats for each user.
    let seat_creations: Vec<Instruction> = users
        .iter()
        .map(|pk| -> Instruction { e2e.market.create_seat(pk.address()) })
        .collect();
    e2e.rpc
        .send_and_confirm_txn(
            test_accounts::default_payer(),
            &users.iter().map(|tr| tr.keypair).collect_vec(),
            &seat_creations,
        )
        .await?;

    let market_seats = e2e.view_market().await?.seats;
    let user_seats: Vec<SectorIndex> = users
        .iter()
        .map(|user| {
            e2e.find_seat(&market_seats, &user.address())
                .expect("User should have a seat")
                .index
        })
        .collect();

    // HashMap<Address, (deposit_amount, withdraw_amount)>
    let base_amounts: HashMap<Address, (u64, u64)> = HashMap::from([
        (test_accounts::acc_1111().pubkey(), (100, 10)),
        (test_accounts::acc_2222().pubkey(), (100, 20)),
        (test_accounts::acc_3333().pubkey(), (100, 30)),
        (test_accounts::acc_4444().pubkey(), (100, 40)),
        (test_accounts::acc_5555().pubkey(), (100, 50)),
    ]);

    let (deposits, withdraws): (Vec<Instruction>, Vec<Instruction>) = users
        .iter()
        .zip(user_seats)
        .map(|(user, seat)| {
            let user_addr = user.address();
            let (deposit, withdraw) = base_amounts.get(&user_addr).unwrap();
            (
                e2e.market.deposit_base(user_addr, *deposit, seat),
                e2e.market.withdraw_base(user_addr, *withdraw, seat),
            )
        })
        .unzip();

    let user_keypairs = &users.into_iter().map(|tr| tr.keypair).collect_vec();
    e2e.rpc
        .send_and_confirm_txn(test_accounts::default_payer(), user_keypairs, &deposits)
        .await?;

    e2e.rpc
        .send_and_confirm_txn(test_accounts::default_payer(), user_keypairs, &withdraws)
        .await?;

    let expected_base = base_amounts
        .into_iter()
        .map(|pk_and_amts| {
            let (pubkey, (deposit, withdraw)) = pk_and_amts;
            (pubkey, deposit, withdraw)
        })
        // Sort by the address.
        .sorted_by_key(|v| v.0)
        .collect_vec();

    let market = e2e.view_market().await?;

    // Check that seats are ordered by address (ascending) and compare the final state of each
    // user's seat to the expected state.
    for (seat, expected_seat) in market.seats.iter().zip_eq(expected_base) {
        let (expected_pk, expected_base_dep, expected_base_wd) = expected_seat;
        assert_eq!(seat.user, expected_pk);
        let amount_from_create_seat = 1;
        let base_remaining = (expected_base_dep + amount_from_create_seat) - expected_base_wd;
        assert_eq!(seat.base_available, base_remaining);
    }

    Ok(())
}
