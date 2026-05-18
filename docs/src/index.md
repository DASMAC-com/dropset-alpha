---
layout: home

hero:
  name: Dropset
  text: On-Chain Order Book for Solana
  tagline: A fully on-chain central limit order book (CLOB) — post orders, match trades, and build on top of a composable, low-latency protocol.
  actions:
    - theme: brand
      text: Get Started
      link: /quickstart/getting-started
    - theme: alt
      text: Architecture
      link: /architecture/overview

features:
  - title: Fully On-Chain CLOB
    details: Every order, match, and cancellation happens on-chain. No off-chain matching engine — the program is the exchange.
  - title: Composable by Design
    details: The dropset-interface crate is client-agnostic. Import it from on-chain or off-chain consumers without pulling in program logic.
  - title: TypeScript SDK
    details: A Codama IDL-generated SDK with a hand-written interface layer, price utilities, and full type safety.
  - title: Custom Instruction Macros
    details: Strongly-typed instruction builders and account context structs generated via procedural macros — no Anchor dependency.
---
