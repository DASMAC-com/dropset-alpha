// use solana_client::rpc_client::RpcClient;
// use solana_sdk::{
//     signature::{Keypair, Signature},
//     signer::Signer,
// };

// use dropset_interface::instructions::CloseSeat;
// use dropset_interface::instructions::CloseSeatInstructionData;

// use crate::context::market::MarketContext;

// pub fn initialize_market(
//     rpc: &RpcClient,
//     ctx: MarketContext,
//     payer: &Keypair,
// ) -> anyhow::Result<Signature> {
//     let seat = CloseSeat {
//         user: payer.pubkey(),
//         market_account: ctx.market,
//         base_user_ata: ctx.base_market_ata,
//         quote_user_ata: ctx.base_market_ata,
//         base_market_ata: ctx.base_market_ata,
//         quote_market_ata: ctx.quote_market_ata,
//         base_mint: ctx.base_mint,
//         quote_mint: ctx.quote_mint,
//     };

//     let ixn = seat.create_instruction(CloseSeatInstructionData {
//         sector_index_hint: 2,
//     });

//     Err(())
//     // Instruction::new

//     // let ixn = Instruction::new_with_bytes(
//     //     dropset_interface::program::ID.into(),
//     //     &seat.pack(),
//     //     seat.create_account_metas().to_vec(),
//     // );

//     // let asdf = Instruction::new_with_bytes(Pubkey::new_from_array(dropset_interface::program::ID), data, accounts)

//     // send_transaction(rpc, payer, &[payer], vec![], Some(1_000_000))
// }
