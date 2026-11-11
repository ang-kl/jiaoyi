# Infrastructure repair release — №18,425

№18,425 §§1–8 · 2026-11-11T08:46:28.063456+08:00 SGT · by Codex (GPT-6), effort not exposed.

## §1. TLDR and release scope

Three infrastructure defects are repaired: a carried crypto arm no longer holds its sibling's peer context in the previous session; genuine first quotes arriving up to 15 seconds after native opening can supply the opening reference; and every normally completed consumed-trigger dispatch produces its own correlated outcome receipt. This is not evidence of a new trading edge. Financial qualification remains **false**.

This report is committed before merge. The PR's final release-verification comment records the actual merge SHA, Railway deployment and before/after checks; deployment success is not asserted in advance.

## §2. Carried arm source versus current peer session

`Sym.pullback_source_day` preserves the original opening quote, band table and causal macro context separately from current `Sym.day`. Preparation now advances the current session. Admission and dispatch read the immutable arm source, while `peer_evidence` continues to read the sibling's current session. Arm receipts persist their source; legacy receipts reconstruct their original preparation without replacing current peer context. The registered four-hour carry and execution deadline rules are unchanged.

The regression crosses a crypto window boundary, checks original source band/origin, changes the current peer origin, verifies stale/missing/weak/opposing peers still refuse, and restores both new and legacy receipt formats. Existing human ownership and paired-risk regressions remain in the full suite.

## §3. Versioned opening quote evidence

The bounded database query ends at `opening_observation_start + 15 seconds` (or current time if earlier). It still selects the first recorded valid two-sided quote. No later market price can fill a missing historical opening. The existing metals observation-start adjustment is unchanged.

Previously valid quotes within six seconds retain their exact legacy receipt metadata. Newly admitted quotes after six and through fifteen seconds carry `native_open_first_quote_v18425_15s`, exact `bid_at`, `ask_at`, `observed_at`, observation start and window end. Admission validates the original policy's bounds. A healthy macro preparation with a missing opening now reports `opening_reference_unavailable` in assessments and dashboard gates.

Tests cover 6.0s, 7.1s, exactly 15s, >15s, no two-sided quote, late replay, restart equality and invalid source-policy metadata. These tests exercise the evidenced upstream reopening delay without permitting holiday or mid-session substitution.

## §4. Consumed-trigger outcome attribution

Consumption retains its existing durable-before-dispatch ordering. Its actual `trading_decisions.id` now identifies one `s3_trigger_outcome` receipt. Both prepared and synchronous dispatch paths record refusal, broker result or unresolved error. Repetitive idle diagnostics retain their existing deduplication. A crash or database-write failure remains an unmatched consumption; no historical outcome is invented.

The actual actor regression drives three separate arm/touch/turn triggers into the same pre-intent refusal. Expected result: three consumed rows, exactly one outcome each, zero missing outcomes, zero order intents or broker orders, and only one deduplicated idle diagnostic. Additional tests preserve known fills when later persistence fails and leave uncertain execution unresolved.

## §5. Unchanged trading and human protections

No alpha, admission threshold, remaining-horizon experiment, exit geometry, risk value, margin value or pair cap changes. Breakeven remains +1.00R to entry, target +1.38R, risk 0.6%, margin floor 250%. Exact `jiaoyi-s3` ownership, account-local yield to external positions/orders, and permanent NatGas/JPN225 exclusion are unchanged. The two human-protected live metal symbols must not be forced into preparation merely to display 32/32.

## §6. Verification evidence

| Check | Status |
|---|---|
| `cargo_check_workspace_all_targets` | passed |
| `cargo_test_workspace` | passed |
| `cargo_clippy_workspace_deny_warnings` | passed |
| `cargo_clippy_all_targets_locked` | passed |
| Pre-change strategy-source backup hashes | 12/12 verified |
| Independent code review | Complete; missing explicit source-validation refusal found and corrected |
| Required GitHub gates | `check`, `docker`, `engineering`; exact head must pass before merge |

Completed local test totals in the accompanying JSON: {'passed': 798, 'failed': 0, 'ignored': 1}. Test logs are hashed there. All SQL integration tests use an isolated local PostgreSQL container, not production.

## §7. Deployment acceptance and baseline

The 08:27 SGT production baseline has 4/4 connected and unpaused accounts and 26/32 valid opening references. Four Crude references are missing; live XAUUSD yields to a human pending order and live XAGUSD respects pair occupancy. Therefore 30/32 may be the correct post-repair total. Compare only matching native windows and preserve original quote values/timestamps/source exactly. Any newly recovered Crude references must show their actual 7.02s arrival, never a relabelled opening timestamp.

Release checks require the merged SHA active on Railway, progressing account monitors, unchanged risk settings and policy, preserved references, and zero recorded bot interference on live 43002148. Existing external positions and pending-order geometry are compared, with broker-side changes treated as separate evidence. A post-deployment interval with no consumed trigger is explicitly 0/0 observation, not proof of live receipt coverage.

## §8. Goal, boundaries and next observation

The goal remains at least 73% net wins (15 of the latest 20 completed positions), separately for each strategy/account, with the 1:1.38 reference objective. This release makes peer context, opening references and refusal evidence more reliable; it does not claim improved win rate. Proposal 4 stays offline. No verification order, new backtest broker collection, Telegram message or risk-setting change is part of this release.
