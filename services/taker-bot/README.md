# Market taker bot *(experimental)*

A prototype taker bot that generates realistic order flow against a `dropset`
market. Order arrival is modeled as a Poisson process with two states (quiet /
burst), and order sizes are drawn from a LogNormal distribution. Intended for
experimentation and local testing, not production use.

## Multiple agents and archetypes

The taker service runs **one or more agents** in the same process — each
configured under a `[[agent]]` block in `config.toml` with its own keypair and
behavior. Each agent picks a named **archetype** preset, and may override
individual fields on top of the preset's defaults.

| Archetype     | Activity profile                | Execution profile | Median order size | Notes |
|---------------|---------------------------------|-------------------|-------------------|-------|
| `passive`     | Slow Poisson arrivals           | `patient`         | 2 000             | Light continuous flow; tolerates wider spreads. |
| `retail`      | Moderate, occasional bursts     | `balanced`        | 3 000             | Generic background retail-style trader. |
| `aggressive`  | Fast / bursty arrivals          | `aggressive`      | 5 000             | Sweeps multiple levels, low spread tolerance. |
| `whale`       | Fast / bursty arrivals          | `aggressive`      | 15 000            | Large parent orders, willing to pay up. |
| `sniper`      | Slow base, opportunistic spikes | `sniper`          | 3 000             | Sits idle, then fires when conditions align. |
| `noise`       | High-frequency, low size        | `noise`           | 1 000             | Steady CLOB chatter; no directional bias. |

Each archetype is a *style* preset: it pairs an `ActivityProfile` (Poisson
rates, burst entry/exit probabilities) with an `ExecutionProfile` (max spread
tolerated, sweep depth, child-sizing fractions, etc.). See
`src/archetype.rs` and `src/taker.rs` for the full preset values, and
`config.toml.example` for the override knobs you can tune per agent.

### Agent registry (`agents.json`)

When you bootstrap localnet via `services/run-services-on-localnet.sh`, the
`initialization_helper` writes a `services/taker-bot/agents.json` registry of
the form `[{ name, kind, pubkey }, ...]`. The frontend reads this registry
through `/api/agents` to label fills in the transaction log by trader
personality. The file is gitignored — re-running the helper regenerates it,
and `--force` will overwrite the underlying agent keypairs in `keypairs/` as
well.

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
