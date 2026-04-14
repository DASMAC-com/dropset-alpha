# Services

The `services/` folder contains three experimental services for testing Dropset on devnet and testnet. These are **not intended for production use**.

## Faucet

Sends `base` and `quote` tokens to an address on request. Only works on test networks (localhost, devnet, testnet).

```bash
cargo run -p faucet
```

See `services/faucet/README.md` for environment variables and configuration.

## Maker Bot

A market-making bot implementing a naive version of the **Avellaneda-Stoikov model** — a stochastic control model that adjusts bid/ask quotes based on inventory risk and volatility estimates. The bot continuously posts and updates orders around a mid-price.

```bash
cargo run -p maker-bot
```

See `services/maker-bot/README.md` for configuration.

## Taker Bot

Periodically sends random market orders to a Dropset market. Used to stress-test the book and simulate taker flow during development.

```bash
cargo run -p taker-bot
```

See `services/taker-bot/README.md` for configuration.
