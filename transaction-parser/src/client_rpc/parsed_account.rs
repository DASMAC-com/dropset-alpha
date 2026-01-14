//! Typed wrapper for transaction accounts, including signer and writable metadata.

use derive_more::{
    AsRef,
    Deref,
    Index,
    IntoIterator,
};
use solana_address::Address;
use solana_transaction_status_client_types::ParsedAccount as SdkParsedAccount;

#[derive(Copy, Clone, Debug)]
pub struct ParsedAccount {
    pub address: Address,
    pub writable: bool,
    pub signer: bool,
}

impl From<&ParsedAccount> for Address {
    fn from(account: &ParsedAccount) -> Self {
        account.address
    }
}

impl From<SdkParsedAccount> for ParsedAccount {
    fn from(account: SdkParsedAccount) -> Self {
        Self {
            address: Address::from_str_const(&account.pubkey),
            writable: account.writable,
            signer: account.signer,
        }
    }
}

#[derive(Clone, Debug, Default, Deref, Index, IntoIterator, AsRef)]
pub struct ParsedAccounts(Vec<ParsedAccount>);

impl ParsedAccounts {
    pub fn addresses(&self) -> Vec<Address> {
        self.iter().map(|p| p.address).collect()
    }
}

impl FromIterator<ParsedAccount> for ParsedAccounts {
    fn from_iter<I: IntoIterator<Item = ParsedAccount>>(iter: I) -> Self {
        ParsedAccounts(iter.into_iter().collect())
    }
}
