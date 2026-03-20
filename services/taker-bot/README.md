# Market taker bot *(experimental)*

A prototype taker bot that generates realistic order flow against a `dropset`
market. Order arrival is modeled as a Poisson process with two states (quiet /
burst), and order sizes are drawn from a LogNormal distribution. Intended for
experimentation and local testing, not production use.

## Running

1. Copy the config template:

   ```shell
   cp services/taker-bot/config.toml.example \
      services/taker-bot/config.toml
   ```

   Then update any empty fields in the new `services/taker-bot/config.toml`:
      - `base_mint`
      - `quote_mint`

   Update other fields as desired.

   By default, the service is configured to run on a local test validator.

2. Ensure there is a keypair file at `services/taker-bot/keypair.json`.

3. Either run the binary or start the `Docker` container.

   ```shell
   cargo run -p dropset-taker-bot
   # or, from the root directory:
   docker compose -f services/taker-bot/compose.yaml up --build
   ```

To get started on a local network, make sure that a local validator is running
and the `dropset` program has been built and deployed.

### Quick setup

For quick bootstrapping, run the [helper](../run-services-on-localnet.sh)
script.

It creates the base and token mints, creates a `dropset` market from them,
starts a local solana test validator if one isn't already running, then deploys
all services with properly updated `config.toml` files.
