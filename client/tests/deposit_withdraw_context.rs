use core::iter::zip;

use client::test_helpers::create_zeroed_mock_runtime_account;
use dropset_interface::instructions::{
    generated_client::{
        Deposit as DepositClient,
        Withdraw as WithdrawClient,
    },
    generated_program::{
        Deposit,
        Withdraw,
    },
    DepositInstructionData,
    WithdrawInstructionData,
};
use solana_account_view::{
    AccountView,
    RuntimeAccount,
};
use solana_address::Address;

#[test]
fn deposit_withdraw_account_order_invariant() {
    let mut runtime_accounts = [
        create_zeroed_mock_runtime_account(Address::new_from_array([0u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([1u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([2u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([3u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([4u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([5u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([6u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([7u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([8u8; 32])),
    ];

    let accounts_ptr: *mut RuntimeAccount = runtime_accounts.as_mut_ptr();

    let account_views = unsafe {
        [
            AccountView::new_unchecked(accounts_ptr.add(0)),
            AccountView::new_unchecked(accounts_ptr.add(1)),
            AccountView::new_unchecked(accounts_ptr.add(2)),
            AccountView::new_unchecked(accounts_ptr.add(3)),
            AccountView::new_unchecked(accounts_ptr.add(4)),
            AccountView::new_unchecked(accounts_ptr.add(5)),
            AccountView::new_unchecked(accounts_ptr.add(6)),
            AccountView::new_unchecked(accounts_ptr.add(7)),
            AccountView::new_unchecked(accounts_ptr.add(8)),
        ]
    };

    let deposit = Deposit::load_accounts(&account_views).unwrap();
    let withdraw = Withdraw::load_accounts(&account_views).unwrap();

    let Deposit {
        event_authority: dep_event_authority,
        user: dep_user,
        market_account: dep_market_account,
        user_ata: dep_user_ata,
        market_ata: dep_market_ata,
        mint: dep_mint,
        token_program: dep_token_program,
        system_program: dep_system_program,
        dropset_program: dep_dropset_program,
    } = deposit;

    let Withdraw {
        event_authority: wd_event_authority,
        user: wd_user,
        market_account: wd_market_account,
        user_ata: wd_user_ata,
        market_ata: wd_market_ata,
        mint: wd_mint,
        token_program: wd_token_program,
        system_program: wd_system_program,
        dropset_program: wd_dropset_program,
    } = withdraw;

    // Ensure that each account is loaded in the same exact order by comparing addresses.
    assert_eq!(dep_event_authority.address(), wd_event_authority.address());
    assert_eq!(dep_user.address(), wd_user.address());
    assert_eq!(dep_market_account.address(), wd_market_account.address());
    assert_eq!(dep_user_ata.address(), wd_user_ata.address());
    assert_eq!(dep_market_ata.address(), wd_market_ata.address());
    assert_eq!(dep_mint.address(), wd_mint.address());
    assert_eq!(dep_token_program.address(), wd_token_program.address());
    assert_eq!(dep_system_program.address(), wd_system_program.address());
    assert_eq!(dep_dropset_program.address(), wd_dropset_program.address());

    // Then checkthat the `AccountMeta`s created in the generated client code match for both
    // deposit and withdraw. This runs extra checks on the signer and writer status for each
    // account that aren't easy to run with the program generated code, since it doesn't expose
    // the intermediate instruction construction (it just immediately invokes the instruction).
    let deposit_instruction = DepositClient {
        event_authority: *dep_event_authority.address(),
        user: *dep_user.address(),
        market_account: *dep_market_account.address(),
        user_ata: *dep_user_ata.address(),
        market_ata: *dep_market_ata.address(),
        mint: *dep_mint.address(),
        token_program: *dep_token_program.address(),
        system_program: *dep_system_program.address(),
        dropset_program: *dep_dropset_program.address(),
    }
    .create_instruction(DepositInstructionData::new(1, 0));

    let withdraw_instruction = WithdrawClient {
        event_authority: *wd_event_authority.address(),
        user: *wd_user.address(),
        market_account: *wd_market_account.address(),
        user_ata: *wd_user_ata.address(),
        market_ata: *wd_market_ata.address(),
        mint: *wd_mint.address(),
        token_program: *wd_token_program.address(),
        system_program: *wd_system_program.address(),
        dropset_program: *wd_dropset_program.address(),
    }
    .create_instruction(WithdrawInstructionData::new(1, 0));

    let deposit_accounts = deposit_instruction.accounts;
    let withdraw_accounts = withdraw_instruction.accounts;
    assert_eq!(deposit_accounts.len(), withdraw_accounts.len());
    for (dep_acc, wd_acc) in zip(deposit_accounts, withdraw_accounts) {
        assert_eq!(dep_acc, wd_acc);
    }
}
