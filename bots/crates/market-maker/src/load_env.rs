use std::sync::LazyLock;

pub fn oanda_auth_token() -> String {
    static TOKEN: LazyLock<String> = LazyLock::new(|| {
        std::env::var("OANDA_AUTH").expect("Environment variable OANDA_AUTH must be set.")
    });

    TOKEN.clone()
}
