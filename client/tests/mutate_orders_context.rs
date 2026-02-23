use client::test_helpers::create_zeroed_mock_runtime_account;
use dropset_interface::instructions::{
    generated_client::{
        BatchReplace as ClientBatchReplace,
        CancelOrder as ClientCancelOrder,
        PostOrder as ClientPostOrder,
    },
    generated_program::{
        BatchReplace,
        CancelOrder,
        PostOrder,
    },
    BatchReplaceInstructionData,
    CancelOrderInstructionData,
    PostOrderInstructionData,
    UnvalidatedOrders,
};
use price::OrderInfoArgs;
use solana_account_view::{
    AccountView,
    RuntimeAccount,
};
use solana_address::Address;

#[test]
fn mutate_orders_account_order_invariant() {
    let mut runtime_accounts = [
        create_zeroed_mock_runtime_account(Address::new_from_array([0u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([1u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([2u8; 32])),
        create_zeroed_mock_runtime_account(Address::new_from_array([3u8; 32])),
    ];

    let accounts_ptr: *mut RuntimeAccount = runtime_accounts.as_mut_ptr();

    let account_views = unsafe {
        [
            AccountView::new_unchecked(accounts_ptr.add(0)),
            AccountView::new_unchecked(accounts_ptr.add(1)),
            AccountView::new_unchecked(accounts_ptr.add(2)),
            AccountView::new_unchecked(accounts_ptr.add(3)),
        ]
    };

    let post_order = PostOrder::load_accounts(&account_views).unwrap();
    let cancel_order = CancelOrder::load_accounts(&account_views).unwrap();
    let batch_replace = BatchReplace::load_accounts(&account_views).unwrap();

    let PostOrder {
        event_authority: po_event_authority,
        user: po_user,
        market_account: po_market_account,
        dropset_program: po_dropset_program,
    } = post_order;

    let CancelOrder {
        event_authority: co_event_authority,
        user: co_user,
        market_account: co_market_account,
        dropset_program: co_dropset_program,
    } = cancel_order;

    let BatchReplace {
        event_authority: br_event_authority,
        user: br_user,
        market_account: br_market_account,
        dropset_program: br_dropset_program,
    } = batch_replace;

    // Ensure the accounts are loaded in the same exact order by comparing each unique address.
    assert_eq!(co_event_authority.address(), po_event_authority.address());
    assert_eq!(co_user.address(), po_user.address());
    assert_eq!(co_market_account.address(), po_market_account.address());
    assert_eq!(co_dropset_program.address(), po_dropset_program.address());

    assert_eq!(br_event_authority.address(), po_event_authority.address());
    assert_eq!(br_user.address(), po_user.address());
    assert_eq!(br_market_account.address(), po_market_account.address());
    assert_eq!(br_dropset_program.address(), po_dropset_program.address());

    // Then check that the `AccountMeta`s created in the generated client code match for both
    // deposit and withdraw. This runs extra checks on the signer and writer status for each
    // account that aren't easy to run with the program generated code, since it doesn't expose
    // the intermediate instruction construction (it just immediately invokes the instruction).
    let post_instruction = ClientPostOrder {
        event_authority: *po_event_authority.address(),
        user: *po_user.address(),
        market_account: *po_market_account.address(),
        dropset_program: *po_dropset_program.address(),
    }
    .create_instruction(PostOrderInstructionData::new(
        OrderInfoArgs::order_at_price(10_000_000),
        true,
        0,
    ));

    let cancel_instruction = ClientCancelOrder {
        event_authority: *co_event_authority.address(),
        user: *co_user.address(),
        market_account: *co_market_account.address(),
        dropset_program: *co_dropset_program.address(),
    }
    .create_instruction(CancelOrderInstructionData::new(0, true, 0));

    let batch_instruction = ClientBatchReplace {
        event_authority: *br_event_authority.address(),
        user: *br_user.address(),
        market_account: *br_market_account.address(),
        dropset_program: *br_dropset_program.address(),
    }
    .create_instruction(BatchReplaceInstructionData::new(
        0,
        UnvalidatedOrders::new([]),
        UnvalidatedOrders::new([]),
    ));

    let post_accounts = post_instruction.accounts;
    let cancel_accounts = cancel_instruction.accounts;
    let batch_accounts = batch_instruction.accounts;
    assert_eq!(post_accounts.len(), cancel_accounts.len());
    assert_eq!(post_accounts.len(), batch_accounts.len());

    let zipped = post_accounts
        .into_iter()
        .zip(cancel_accounts)
        .zip(batch_accounts)
        .map(|((a, b), c)| (a, b, c));
    for (post_acc, cancel_acc, batch_acc) in zipped {
        assert_eq!(post_acc, cancel_acc);
        assert_eq!(post_acc, batch_acc);
    }
}
