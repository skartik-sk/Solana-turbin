# Solana & Rust Projects Collection

A collection of Solana smart contracts and Rust projects exploring various blockchain development patterns and technologies.

## Projects

| Project | Description | Technologies |
|---------|-------------|--------------|
| [whitelist-transfer-hook](./whitelist-transfer-hook/) | SPL Token 2022 transfer hook implementation that enforces whitelist restrictions on token transfers. Only whitelisted addresses can transfer tokens with this hook enabled. | Anchor 0.32.1, SPL Token 2022, TypeScript |
| [tuktuk-escrow](./tuktuk-escrow/) | Escrow program using TukTuk SDK for scheduled transactions and Cron SDK for time-based execution | Anchor 0.31.0, TukTuk SDK, Helium Cron SDK |
| [pricing-oracle](./pricing-oracle/) | Pricing oracle implementation with TukTuk scheduler integration for automated price updates | Anchor 0.32.1, TukTuk SDK, SPL Token |
| [magicblock-er-example](./magicblock-er-example/) | Example implementation using MagicBlock's ephemeral rollups SDK for high-performance state management | Anchor 0.32.1, MagicBlock ER SDK |
| [escrow-litesvm](./escrow-litesvm/) | Basic escrow contract tested with LiteSVM for fast local Solana program testing | Anchor 0.30.1, LiteSVM |
| [transfer-hook-vault](./transfer-hook-vault/) | Token transfer hook with vault functionality for securing tokens during transfers | Anchor 0.32.1, SPL Token 2022 |
| [tuktuk-gpt-oracle](./tuktuk-gpt-oracle/) | GPT-powered pricing oracle with TukTuk scheduling for AI-based price feeds | Anchor 0.32.1, TukTuk SDK, SPL Token |
| [rust-adv](./rust-adv/) | Serialization benchmark comparing Borsh, Serde JSON, and Wincode performance | Rust, Borsh, Serde, Wincode |
