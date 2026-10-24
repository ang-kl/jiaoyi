# jiaoyi

A daily trend-following trading system for cTrader accounts, written in Rust, with PostgreSQL storage, deployed on Railway.

It is the ground-up rebuild ordered by the owner on 24-10-2026, replacing the bot-trade program.

## Status

Phase 0: specification, a draft awaiting the owner's approval. No trading code exists yet.

- [`docs/phase-0-rulebook.md`](docs/phase-0-rulebook.md) — the trading rules, with every number fixed in advance, and the gates each phase must pass.
- [`docs/storage.md`](docs/storage.md) — the PostgreSQL design: event log, intent before send, broker as the authority.

Nothing trades until phase 3, the history test, passes. Live trading needs the owner's word.
