# Market maker bot *(experimental)*

A prototype market-making bot implementing a naive version of the
[Avellaneda-Stoikov model] for a `dropset` market. Intended for
experimentation and local testing, not production use.

## Quoting model

In addition to the Avellaneda-Stoikov reservation price, the bot adapts its
quotes to local-book conditions:

- **Effective mid** blends an external fair-value anchor (OANDA `S5`) with a
  local microprice from the on-chain book, weighted by `local_book_weight_bps`.
- **Dynamic half-spread** widens with recent local-fair-price volatility and
  with one-sided "hit pressure" (counted by comparing maker-owned depth across
  market updates).
- **Hit detection** flags which side was most recently lifted/hit, applies a
  per-side refill delay, and skips re-posting at the touch on that side until
  the delay elapses.
- **Quote TTL** refreshes resting liquidity even without a websocket-triggered
  state change, so quotes age out instead of going stale.
- **Per-level jitter** (size and price) is applied across the ladder so the
  ladder looks less robotic.

### Style presets

Each maker run picks a `style` preset (overridable per field in `config.toml`):

| Style       | Quote TTL | Refill delay (min–max) | `replenish_ratio_bps` | `hit_widening_bps` | Max quote levels | Spread multiplier |
|-------------|-----------|------------------------|-----------------------|--------------------|------------------|-------------------|
| `tight`     | 1 500 ms  | 250–900 ms             | 9 000                 | 10                 | 10               | 0.90× |
| `balanced`  | 2 500 ms  | 600–2 000 ms           | 7 000                 | 18                 | 8                | 1.20× |
| `defensive` | 3 500 ms  | 1 200–4 000 ms         | 5 500                 | 28                 | 6                | 1.75× |

See `src/config.rs` (`MakerStyleDefaults`) for the full preset values and
`config.toml.example` for descriptions of the override knobs.

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
   cp services/maker-bot/config.toml.example \
      services/maker-bot/config.toml
   ```

   Then update any empty fields in the new `services/maker-bot/config.toml`:
      - `oanda_auth_token`
      - `base_mint`
      - `quote_mint`

   Update other fields as desired.

   By default, the service is configured to run on a local test validator.

3. Ensure there is a keypair file at `services/maker-bot/keypair.json`.

4. Either run the binary or start the `Docker` container.

   ```shell
   cargo run -p dropset-maker-bot
   # or, from the root directory:
   docker compose -f services/maker-bot/compose.yaml up --build
   ```

To get started on a local network, make sure that a local validator is running
and the `dropset` program has been built and deployed.

### Quick setup

For quick bootstrapping, run the [helper](../run-services-on-localnet.sh)
script.

It creates the base and token mints, creates a `dropset` market from them,
starts a local solana test validator if one isn't already running, then deploys
all services with properly updated `config.toml` files.


[Avellaneda-Stoikov model]: https://people.orie.cornell.edu/sfs33/LimitOrderBook.pdf
[version 4.34 or later]: https://docs.docker.com/engine/network/drivers/host/#docker-desktop
