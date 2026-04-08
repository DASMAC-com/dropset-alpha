# Market taker bot *(experimental)*

A prototype taker bot that generates realistic order flow against a `dropset`
market. Order arrival is modeled as a Poisson process with two states (quiet /
burst), and order sizes are drawn from a LogNormal distribution. Intended for
experimentation and local testing, not production use.

## Running

1. If you're using Docker Desktop, make sure `Enable host networking` is checked
   ***on***. The `compose.yaml` files specify `network_mode: host`, so the
   containers can only access ports on the host machine if you have that setting
   enabled.

   You must have Docker [version 4.34 or later]. As of Docker Desktop version
   v4.68.0, the setting is at:

   `Settings -> Resources -> Network -> Enable host networking`

2. Copy the config template:

   ```shell
   cp services/taker-bot/config.toml.example \
      services/taker-bot/config.toml
   ```

   Then update any empty fields in the new `services/taker-bot/config.toml`:
      - `base_mint`
      - `quote_mint`

   Update other fields as desired.

   By default, the service is configured to run on a local test validator.

3. Ensure there is a keypair file at `services/taker-bot/keypair.json`.

4. Either run the binary or start the `Docker` container.

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

[version 4.34 or later]: https://docs.docker.com/engine/network/drivers/host/#docker-desktop
