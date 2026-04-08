# Faucet service *(experimental)*

A `faucet` service intended to send `base` and `quote` mint tokens associated
with a `dropset` market to an address upon request.

It's only intended to run on a Solana test network like `localhost`, `devnet`,
or `testnet`.

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
   cp services/faucet/config.toml.example \
      services/faucet/config.toml
   ```

   Then update any empty fields in the new `services/faucet/config.toml`:
      - `base_mint`
      - `quote_mint`

   Update other fields as desired.

   By default, the service is configured to run on a local test validator.

3. Ensure there is a keypair file at `services/faucet/keypair.json`.

4. Either run the binary or start the `Docker` container.

   ```shell
   cargo run -p dropset-faucet
   # or, from the root directory:
   docker compose -f services/faucet/compose.yaml up --build
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
