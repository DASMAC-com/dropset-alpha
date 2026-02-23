use dropset_interface::state::SYSTEM_PROGRAM_ID;
use solana_account_view::RuntimeAccount;
use solana_address::Address;

/// Creates a mock runtime account with only the address field not set to zeros.
pub fn create_zeroed_mock_runtime_account(address: Address) -> RuntimeAccount {
    RuntimeAccount {
        borrow_state: 0,
        is_signer: 0,
        is_writable: 0,
        executable: 0,
        resize_delta: 0,
        address,
        owner: SYSTEM_PROGRAM_ID,
        lamports: 0,
        data_len: 0,
    }
}
