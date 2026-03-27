use std::collections::HashMap;

use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_address::Address;
use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::client_rpc::ParsedTransaction;

fn parse_u64_ui_amount(ui_amount: &UiTokenAmount) -> anyhow::Result<u64> {
    Ok(ui_amount.amount.parse()?)
}

/// Parsed, mapped balances of lamports and token accounts.
pub struct ParsedMappedBalances {
    /// Mapping of addresses to lamport pre-balances.
    pre_lamport_balances: HashMap<Address, u64>,
    /// Mapping of addresses to lamport post-balances.
    post_lamport_balances: HashMap<Address, u64>,
    /// Mapping of token accounts to their pre-transaction [u64] balance in atoms. Each [Address]
    /// key is very likely an associated token account but is possibly a non-ATA token account.
    pub pre_token_balances: HashMap<Address, u64>,
    /// Mapping of token accounts to their post-transaction [u64] balance in atoms. Each [Address]
    /// key is very likely an associated token account but is possibly a non-ATA token account.
    pub post_token_balances: HashMap<Address, u64>,
}

impl ParsedMappedBalances {
    pub fn get_pre_lamports(&self, address: &Address) -> Option<u64> {
        self.pre_lamport_balances.get(address).cloned()
    }

    pub fn get_post_lamports(&self, address: &Address) -> Option<u64> {
        self.post_lamport_balances.get(address).cloned()
    }

    pub fn get_user_pre_token_balance(&self, user: &Address, mint: &Address) -> Option<u64> {
        let ata = get_associated_token_address(user, mint);
        self.get_ata_pre_token_balance(&ata)
    }

    pub fn get_user_post_token_balance(&self, user: &Address, mint: &Address) -> Option<u64> {
        let ata = get_associated_token_address(user, mint);
        self.get_ata_post_token_balance(&ata)
    }

    pub fn get_ata_pre_token_balance(&self, ata: &Address) -> Option<u64> {
        self.pre_token_balances.get(ata).cloned()
    }

    pub fn get_ata_post_token_balance(&self, ata: &Address) -> Option<u64> {
        self.post_token_balances.get(ata).cloned()
    }
}

impl TryFrom<&ParsedTransaction> for ParsedMappedBalances {
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

        let indexed_addresses = HashMap::from_iter(
            addresses
                .iter()
                .enumerate()
                .map(|(i, addr)| {
                    if i > u8::MAX as usize {
                        anyhow::bail!(
                            "Got {} addresses, maximum supported is {}",
                            addresses.len(),
                            u8::MAX as usize + 1
                        );
                    }
                    Ok((i as u8, *addr))
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
        );

        fn get(index: usize, hash_map: &HashMap<u8, Address>) -> anyhow::Result<Address> {
            if index > u8::MAX as usize {
                anyhow::bail!("Index {index} is greater than u8::MAX ({})", u8::MAX);
            }
            hash_map.get(&(index as u8)).cloned().ok_or(anyhow::anyhow!(
                "Account index should exist in indexed addresses"
            ))
        }

        let pre_lamports = pre_balances
            .iter()
            .enumerate()
            .map(|(i, b)| Ok((get(i, &indexed_addresses)?, *b)))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let post_lamports = post_balances
            .iter()
            .enumerate()
            .map(|(i, b)| Ok((get(i, &indexed_addresses)?, *b)))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let pre_tokens = pre_token_balances
            .iter()
            .map(|ui_balance| {
                Ok((
                    get(ui_balance.account_index as usize, &indexed_addresses)?,
                    parse_u64_ui_amount(&ui_balance.ui_token_amount)?,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let post_tokens = post_token_balances
            .iter()
            .map(|ui_balance| {
                Ok((
                    get(ui_balance.account_index as usize, &indexed_addresses)?,
                    parse_u64_ui_amount(&ui_balance.ui_token_amount)?,
                ))
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
    use std::collections::HashMap;

    use itertools::Itertools;
    use solana_address::Address;
    use spl_associated_token_account_interface::address::get_associated_token_address;

    use crate::{
        client_rpc::ParsedMappedBalances,
        goldens::golden_parsed_transactions,
    };

    #[test]
    fn parse_balances() {
        let goldens = golden_parsed_transactions();
        let _try_from = goldens
            .iter()
            .map(ParsedMappedBalances::try_from)
            .collect::<anyhow::Result<Vec<_>>>()
            .expect("Should parse mapped balances");

        let _helper = goldens
            .iter()
            .map(|p| p.try_into_mapped_balances())
            .collect::<anyhow::Result<Vec<_>>>()
            .expect("Should parse mapped balances");
    }

    #[test]
    fn parse_correct_balances() {
        let goldens = golden_parsed_transactions();
        let balances = goldens
            .iter()
            .map(ParsedMappedBalances::try_from)
            .collect::<anyhow::Result<Vec<_>>>()
            .expect("Should parse mapped balances");

        let hash_map: HashMap<String, ParsedMappedBalances> = goldens
            .iter()
            .map(|txn| txn.signature.to_string())
            .zip_eq(balances)
            .collect();

        let balances = hash_map.get("5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN").unwrap();

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

        assert_eq!(balances.get_ata_pre_token_balance(expected_ata), Some(0));
        assert_eq!(balances.get_user_pre_token_balance(user, mint), Some(0));

        // There's a difference between 0 and None.
        assert_eq!(balances.get_ata_pre_token_balance(mint), None);

        assert_eq!(balances.post_token_balances.get(expected_ata), Some(&10000));

        assert_eq!(
            balances.get_ata_post_token_balance(expected_ata),
            Some(10000)
        );
        assert_eq!(
            balances.get_user_post_token_balance(user, mint),
            Some(10000)
        );
    }
}
