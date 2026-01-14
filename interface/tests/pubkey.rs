#[cfg(test)]
mod tests {
    extern crate std;
    use solana_address::Address;

    #[test]
    fn address_comparisons() {
        let addresses: std::vec::Vec<Address> = [
            "00000000000000000000000000000001",
            "00000000000000000000000000000002",
            "00000000000000000000000000000010",
            "10000000000000001000000000000000",
            "21000000000000000000000000000000",
            "30000000000000000000000000000000",
        ]
        .iter()
        .map(|s| {
            let mut arr = [0u8; 32];
            arr.iter_mut()
                .zip(s.chars())
                .for_each(|(byte, c)| *byte = c.to_digit(10).unwrap() as u8);
            arr.into()
        })
        .collect();

        for (i, pk_i) in addresses.iter().enumerate() {
            if i > 0 {
                assert!(pk_i > &addresses[i - 1]);
                assert!(pk_i.as_ref() > addresses[i - 1].as_ref());
            }
        }
    }
}
