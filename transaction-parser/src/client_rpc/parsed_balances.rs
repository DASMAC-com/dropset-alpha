use std::collections::HashMap;

use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_address::Address;
use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::client_rpc::ParsedTransaction;

fn parse_u64_ui_amount(ui_amount: &UiTokenAmount) -> u64 {
    ui_amount
        .amount
        .parse()
        .expect("All ui token amounts should be parseable u64 strings")
}

/// Parsed, mapped balances of lamports and token accounts.
pub struct ParsedMappedBalances {
    /// Mapping of addresses to lamport pre-balances.
    pre_lamport_balances: HashMap<Address, u64>,
    /// Mapping of addresses to lamport post-balances.
    post_lamport_balances: HashMap<Address, u64>,
    /// Mapping of associated token account pre-balances to [UiTokenAmount]. Entry keys are ATAs.
    pub pre_ui_token_amounts: HashMap<Address, UiTokenAmount>,
    /// Mapping of associated token account post-balances to [UiTokenAmount]. Entry keys are ATAs.
    pub post_ui_token_amounts: HashMap<Address, UiTokenAmount>,
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
        self.pre_ui_token_amounts.get(ata).map(parse_u64_ui_amount)
    }

    pub fn get_ata_post_token_balance(&self, ata: &Address) -> Option<u64> {
        self.post_ui_token_amounts.get(ata).map(parse_u64_ui_amount)
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
                            "Got {} addresses, expected fewer than {}",
                            addresses.len(),
                            u8::MAX
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
                    ui_balance.ui_token_amount.clone(),
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let post_tokens = post_token_balances
            .iter()
            .map(|ui_balance| {
                Ok((
                    get(ui_balance.account_index as usize, &indexed_addresses)?,
                    ui_balance.ui_token_amount.clone(),
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            pre_lamport_balances: HashMap::from_iter(pre_lamports),
            post_lamport_balances: HashMap::from_iter(post_lamports),
            pre_ui_token_amounts: HashMap::from_iter(pre_tokens),
            post_ui_token_amounts: HashMap::from_iter(post_tokens),
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

        assert_eq!(
            balances
                .pre_ui_token_amounts
                .get(expected_ata)
                .unwrap()
                .ui_amount,
            None,
        );

        assert_eq!(balances.get_ata_pre_token_balance(expected_ata), Some(0));
        assert_eq!(balances.get_user_pre_token_balance(user, mint), Some(0));

        // There's a difference between 0 and None.
        assert_eq!(balances.get_ata_pre_token_balance(mint), None);

        assert_eq!(
            balances
                .post_ui_token_amounts
                .get(expected_ata)
                .unwrap()
                .amount,
            "10000",
        );

        assert_eq!(
            balances.get_ata_post_token_balance(expected_ata).unwrap(),
            10000
        );
        assert_eq!(
            balances.get_user_post_token_balance(user, mint),
            Some(10000)
        );
    }

    #[test]
    fn all_goldens_have_u64_ui_amounts() {
        let goldens = golden_parsed_transactions();
        let maps = goldens
            .iter()
            .map(ParsedMappedBalances::try_from)
            .collect::<anyhow::Result<Vec<_>>>()
            .expect("Should parse mapped balances");

        // For all parsed transactions in `goldens`, test the u64 string parsing
        // that's occurring in the `get_ata_*_balance` methods.
        // If any of the strings are invalid `u64`s, this test will panic.
        for mapped_balances in maps {
            for ata in mapped_balances.pre_ui_token_amounts.keys() {
                mapped_balances.get_ata_pre_token_balance(ata).unwrap();
            }
            for ata in mapped_balances.post_ui_token_amounts.keys() {
                mapped_balances.get_ata_post_token_balance(ata).unwrap();
            }
        }
    }
}
