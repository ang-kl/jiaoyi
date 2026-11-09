# Exact pre-change Rust archive — №18,403

Created before code edits at 2026-11-09T07:19:38.083073+00:00; source commit `1d279e8980d1c31b6ebebd496a644bea7f755510`.

All 119 `crates/*/src/**/*.rs` files are preserved as `.rs.bak`, including all strategy modules and shared execution dependencies. Total 2,421,318 bytes. Tests/examples are not in the source backup scope. The archive is outside Cargo modules; it is evidence, not executable code.

`manifest.json` records original path, backup path, exact byte length and SHA256. Every archive file was compared with its Git base blob before commit. Run `python3 -m unittest discover -s ops/research -p test_strategy_backup_18403.py` to verify every archived hash.

No credentials or broker data are included. Restore only following a separately reviewed owner instruction; restoring the old code would remove these protections.
