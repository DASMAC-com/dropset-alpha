use client::{
    e2e_helpers::{
        E2e,
        User,
    },
    single_signer_instruction::SingleSignerInstruction,
};
use dropset_interface::{
    instructions::{
        BatchReplaceInstructionData,
        UnvalidatedOrders,
    },
    state::sector::NIL,
};
use price::OrderInfoArgs;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let user = Keypair::new();
    let e2e =
        E2e::new_users_and_market(None, [User::new(&user, 1_000_000_000, 1_000_000_000)]).await?;

    let base = e2e.get_base_balance(&user.pubkey()).await?;
    let quote = e2e.get_quote_balance(&user.pubkey()).await?;
    println!("{base} {quote}");

    e2e.market
        .deposit_base(user.pubkey(), 1_000_000_000, NIL)
        .send_single_signer(&e2e.rpc, &user)
        .await?;
    let seat = e2e
        .fetch_seat(&user.pubkey())
        .await?
        .expect("Trader should have a seat");
    e2e.market
        .deposit_quote(user.pubkey(), 1_000_000_000, seat.index)
        .send_single_signer(&e2e.rpc, &user)
        .await?;

    let res = e2e
        .market
        .batch_replace(
            user.pubkey(),
            BatchReplaceInstructionData::new(
                seat.index,
                UnvalidatedOrders::new([OrderInfoArgs::new_unscaled(11_000_000, 1)]),
                UnvalidatedOrders::new([
                    OrderInfoArgs::new_unscaled(12_000_000, 1),
                    OrderInfoArgs::new_unscaled(13_000_000, 2),
                    OrderInfoArgs::new_unscaled(14_000_000, 3),
                    OrderInfoArgs::new_unscaled(15_000_000, 4),
                    OrderInfoArgs::new_unscaled(16_000_000, 5),
                ]),
            ),
        )
        .send_single_signer(&e2e.rpc, &user)
        .await?;

    for msg in res.parsed_transaction.log_messages {
        println!("{msg}");
    }

    println!(
        "Transaction signature: {}",
        e2e.register_market_txn.parsed_transaction.signature
    );

    Ok(())
}
