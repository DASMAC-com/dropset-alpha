# Bots

## Market Maker *(experimental)*

A prototype market-making bot implementing a naive version of the
[Avellaneda-Stoikov model] for a `dropset` market. Intended for
experimentation and local testing, not production use.

### Running

1. Copy the maker config template and fill in your OANDA API token:

   ```shell
   cp services/maker-bot/maker.toml.example \
      services/maker-bot/maker.toml
   ```

   Then edit `maker.toml` and set `oanda_auth_token`.

   Update config values if desired. The default Solana network is the local
   validator network.

2. Make sure a local validator is running if you're running on a local network.

   ```shell
   solana-test-validator

   # Deploy the program if necessary
   pnpm run build:dropset
   pnpm run deploy
   ```

3. Either run the binary or start the `Docker` container.

   ```shell
   cargo run -p dropset-maker-bot
   # or
   docker compose -f compose.yaml up --build

[Avellaneda-Stoikov model]: https://people.orie.cornell.edu/sfs33/LimitOrderBook.pdf
