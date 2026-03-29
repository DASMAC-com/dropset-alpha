use std::collections::HashMap;

use anyhow::Context;
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_address::Address;

use crate::client_rpc::ParsedTransaction;

fn parse_u64_ui_amount(ui_amount: &UiTokenAmount) -> anyhow::Result<u64> {
    ui_amount
        .amount
        .parse()
        .with_context(|| anyhow::anyhow!("Couldn't parse UI token amount: {}", ui_amount.amount))
}

/// Parsed lamport and token account balances as hashmaps from addresses to pre/post transaction
/// amounts in atoms.
///
/// This is intended to be created from [ParsedTransaction] data.
pub struct ParsedBalances {
    /// Mapping of addresses to lamport pre-balances.
    pub pre_lamport_balances: HashMap<Address, u64>,
    /// Mapping of addresses to lamport post-balances.
    pub post_lamport_balances: HashMap<Address, u64>,
    /// Mapping of token account addresses to their pre-transaction [u64] balance in atoms.
    ///
    /// Note that the [Address] keys here are always token accounts but not necessarily
    /// *associated* token accounts.
    pub pre_token_balances: HashMap<Address, u64>,
    /// Mapping of token account addresses to their post-transaction [u64] balance in atoms.
    ///
    /// Note that the [Address] keys here are always token accounts but not necessarily
    /// *associated* token accounts.
    pub post_token_balances: HashMap<Address, u64>,
}

impl ParsedBalances {
    /// Get a user's pre lamport balance.
    pub fn get_pre_lamports(&self, address: &Address) -> Option<u64> {
        self.pre_lamport_balances.get(address).cloned()
    }

    /// Get a user's post lamport balance.
    pub fn get_post_lamports(&self, address: &Address) -> Option<u64> {
        self.post_lamport_balances.get(address).cloned()
    }

    /// Get the pre token balance for the passed address. Typically, this will be an ATA address.
    pub fn get_pre_token_balance(&self, token_account: &Address) -> Option<u64> {
        self.pre_token_balances.get(token_account).cloned()
    }

    /// Get the post token balance for the passed address. Typically, this will be an ATA address.
    pub fn get_post_token_balance(&self, token_account: &Address) -> Option<u64> {
        self.post_token_balances.get(token_account).cloned()
    }
}

impl TryFrom<&ParsedTransaction> for ParsedBalances {
    type Error = anyhow::Error;

    fn try_from(parsed_transaction: &ParsedTransaction) -> Result<Self, Self::Error> {
        let ParsedTransaction {
            pre_balances,
            post_balances,
            addresses,
            pre_token_balances,
            post_token_balances,
            ..
        } = parsed_transaction;

        fn get(index: usize, addresses: &[Address]) -> anyhow::Result<Address> {
            addresses.get(index).cloned().ok_or_else(|| {
                anyhow::anyhow!(
                    "Account index ({index}) not found in the transaction's {} indexed addresses",
                    addresses.len(),
                )
            })
        }

        let pre_lamports = pre_balances
            .iter()
            .enumerate()
            .map(|(i, b)| Ok((get(i, addresses)?, *b)))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let post_lamports = post_balances
            .iter()
            .enumerate()
            .map(|(i, b)| Ok((get(i, addresses)?, *b)))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let pre_tokens = pre_token_balances
            .iter()
            .map(|ui_balance| {
                let address = get(ui_balance.account_index as usize, addresses)?;
                let amount = parse_u64_ui_amount(&ui_balance.ui_token_amount)
                    .with_context(|| anyhow::anyhow!("Couldn't parse ui amount for {address}"))?;
                Ok((address, amount))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let post_tokens = post_token_balances
            .iter()
            .map(|ui_balance| {
                let address = get(ui_balance.account_index as usize, addresses)?;
                let amount = parse_u64_ui_amount(&ui_balance.ui_token_amount)
                    .with_context(|| anyhow::anyhow!("Couldn't parse ui amount for {address}"))?;
                Ok((address, amount))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            pre_lamport_balances: HashMap::from_iter(pre_lamports),
            post_lamport_balances: HashMap::from_iter(post_lamports),
            pre_token_balances: HashMap::from_iter(pre_tokens),
            post_token_balances: HashMap::from_iter(post_tokens),
        })
    }
}

#[cfg(test)]
mod test {

    use solana_address::Address;
    use spl_associated_token_account_interface::address::get_associated_token_address;

    use crate::{
        client_rpc::{
            ParsedBalances,
            ParsedTransaction,
        },
        goldens::{
            load_golden,
            load_golden_full_logs,
        },
    };

    #[test]
    fn parse_correct_balances() {
        let sig = "5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN";
        let txn = ParsedTransaction::from_encoded_transaction(load_golden(sig))
            .expect("Should parse transaction");
        let balances = ParsedBalances::try_from(&txn).expect("Should parse balances");

        let user = &Address::from_str_const("11113MwGAy1Aq8qkfPuukq892Zn3tV6uGHWoRYLaUBS");

        assert_eq!(balances.pre_lamport_balances.get(user), Some(&179832313698));
        assert_eq!(balances.get_pre_lamports(user), Some(179832313698));

        assert_eq!(
            balances.post_lamport_balances.get(user),
            Some(&179832303696)
        );
        assert_eq!(balances.get_post_lamports(user), Some(179832303696));

        let mint = &Address::from_str_const("EmgRPxqMnGFFYTqRY6L6gVRNyzaadjHhkTDB4Jq8h9kV");
        let expected_ata = &get_associated_token_address(user, mint);

        assert_eq!(balances.pre_token_balances.get(expected_ata), Some(&0));

        assert_eq!(balances.get_pre_token_balance(expected_ata), Some(0));

        // There's a difference between 0 and None.
        assert_eq!(balances.get_pre_token_balance(mint), None);

        assert_eq!(balances.post_token_balances.get(expected_ata), Some(&10000));

        assert_eq!(balances.get_post_token_balance(expected_ata), Some(10000));
    }

    #[test]
    fn parse_balances_with_loaded_addresses() {
        let sig = "3apVSExwHE5PuoMGHdpBZWbjV79bhcjP2cUTHGwysCKjhBfFcRs2JLDnjpxc6jNhsLu7bNCScNoP2mzrv9dBKCYA";
        let encoded_meta = load_golden_full_logs(sig);
        let parsed_txn =
            ParsedTransaction::from_encoded_transaction(encoded_meta).expect("Should parse txn");
        assert!(parsed_txn.used_address_lookup_table);

        let parsed_balances = ParsedBalances::try_from(&parsed_txn).expect("Should parse balances");

        for balances in [
            parsed_txn.pre_token_balances,
            parsed_txn.post_token_balances,
        ] {
            for b in balances {
                let token_account = parsed_txn
                    .addresses
                    .get(b.account_index as usize)
                    .expect("account_index should map to a valid address");

                // Verify at least one of the balance maps contain this token account.
                let is_in_pre_token = parsed_balances
                    .pre_token_balances
                    .contains_key(token_account);
                let is_in_post_token = parsed_balances
                    .post_token_balances
                    .contains_key(token_account);

                assert!(is_in_pre_token || is_in_post_token);
            }
        }
    }
}
