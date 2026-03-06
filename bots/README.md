# Bots

## Market Maker *(experimental)*

A prototype market-making bot implementing a naive version of the
[Avellaneda-Stoikov model] for a `dropset` market. Intended for
experimentation and local testing, not production use.

### Running

1. Copy the config template and fill in your OANDA API token:

   ```shell
   cp bots/crates/market-maker/config.toml.example \
      bots/crates/market-maker/config.toml
   ```

   Then edit `config.toml` and set `oanda_auth_token`. Everything else has
   sensible defaults.

2. Run:

   ```shell
   bash bots/crates/market-maker/market-maker.sh
   ```

   The script starts localnet if it is not already running, builds and deploys
   the program, initializes a market, and starts the bot.

[Avellaneda-Stoikov model]: https://people.orie.cornell.edu/sfs33/LimitOrderBook.pdf
