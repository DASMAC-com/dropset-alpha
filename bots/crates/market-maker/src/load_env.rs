use std::sync::LazyLock;

use solana_sdk::{
    bs58,
    signature::Keypair,
};

pub fn oanda_auth_token() -> String {
    static TOKEN: LazyLock<String> = LazyLock::new(|| {
        std::env::var("OANDA_AUTH").expect("Environment variable OANDA_AUTH must be set.")
    });

    TOKEN.clone()
}

pub fn maker_keypair() -> &'static Keypair {
    static KEYPAIR: LazyLock<Keypair> = LazyLock::new(|| {
        let kp_str = std::env::var("MAKER_SECRET_KEY")
            .expect("Environment variable MAKER_SECRET_KEY must be set.");
        let byte_vec = if kp_str.starts_with('[') {
            serde_json::from_str(kp_str.as_str()).expect("Invalid JSON keypair")
        } else {
            bs58::decode(kp_str)
                .into_vec()
                .expect("Invalid base58 keypair")
        };
        let bytes = byte_vec.as_slice();

        Keypair::try_from(bytes).expect("Invalid keypair bytes")
    });

    LazyLock::force(&KEYPAIR)
}
