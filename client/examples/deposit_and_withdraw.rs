use client::{
    e2e_helpers::{
        E2e,
        User,
    },
    single_signer_instruction::SingleSignerInstruction,
};
use dropset_interface::state::sector::NIL;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let user = Keypair::new();
    let e2e = E2e::new_users_and_market(None, [User::new(&user, 10000, 10000)]).await?;

    e2e.market
        .deposit_base(user.pubkey(), 1000, NIL)
        .send_single_signer(&e2e.rpc, &user)
        .await?;

    println!("{:#?}", e2e.view_market().await?);

    let user_seat = e2e
        .fetch_seat(&user.pubkey())
        .await?
        .expect("User should have been registered on deposit");

    let res = e2e
        .market
        .withdraw_base(user.pubkey(), 100, user_seat.index)
        .send_single_signer(&e2e.rpc, &user)
        .await?;

    println!(
        "Transaction signature: {}",
        res.parsed_transaction.signature
    );

    Ok(())
}
