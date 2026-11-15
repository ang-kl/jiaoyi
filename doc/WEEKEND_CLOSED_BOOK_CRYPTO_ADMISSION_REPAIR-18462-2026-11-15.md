# №18,462 — Weekend closed-book crypto admission repair

## §1. Outcome and scope

The shared margin gate previously required a live FX conversion quote for every existing foreign position. Consequently a healthy USD crypto candidate could be refused solely because an existing S1 AUS200/GER40 position needed AUD/USD or EUR/USD while FX was closed. This release separates existing-position accounting from candidate and pending-entry valuation. Existing stop liabilities may use broker-calendar-verified pre-close evidence with a 5% adverse FX reserve; candidate quotes and conversion legs remain live and executable.

This is an execution/accounting repair, not strategy qualification or a change in entry/exit rules. The pre-deployment report does not claim a successful deployment or successful future 12:00 SGT opening. The release PR will carry the timestamped post-deployment attestation and any remaining evidence or legitimate risk blockers.

## §2. Recorded refusals and comparison fills

The read-only PostgreSQL audit covers **14 November 05:35 through 15 November 08:00 SGT, end exclusive**. Account 49236051 recorded the three specified S3 fills: ETHUSD at 14 November 10:41:08 SGT, BTCUSD at 15 November 03:25:53 and 04:21:31 SGT; S1 ETHUSD followed at 06:37:45 SGT. The other two demos recorded no opening fills in that interval.

Both 46130058 and 47790949 recorded `conversion quote stale/incomplete for broker symbol 5` (AUDUSD) during S1 margin admission at **15 November 06:36:45 SGT**, followed by corresponding refused receipts. These are two actual refusals with two diagnostic rows each, not four trading attempts. Live 43002148's contemporaneous S1 refusal was its existing open-risk budget, which remains enforced.

No persisted S3 terminal margin refusal was found at the three S3 comparison entry times on the affected demos. Background preparation failures were not always persisted as terminal receipts. The audit therefore confirms the shared FX failure path but does **not** claim it explains every missed S3 trade. Additional health fields expose known S3 stop risk and commission evidence age/error to separate these issues after release.

## §3. Existing Gold stop liability is already deterministic

The prior `quote_to_deposit` implementation already returned 1 for USD/USD before inspecting market prices. Missing Gold ticks therefore did not, by themselves, prove a broken USD conversion path. This release makes that existing-book identity explicit and tests it with an empty quote cache; authority recovery, an absent stop, or incomplete asset metadata can still correctly refuse.

| Account | Position | Short entry | Initial/current stop | Volume (cents) | Nominal stop loss from entry |
|---|---|---:|---:|---:|---:|
| 46130058 | 262583980 | 3878.12 | 3893.62 | 1000 | $155.00 |
| 47790949 | 262583136 | 3877.19 | 3892.65 | 900 | $139.14 |

The account-wide margin reserve continues to account for broker-reported gross floating P&L relative to the stop. The table is the S3 loss from entry, not a replacement for the full margin calculation. No Gold stop, target, volume, or recorded exit deadline is changed.

## §4. Closed-market FX evidence and boundaries

1. The actual account's broker symbol contract supplies timezone, weekly intervals and holidays. Staleness alone never proves that a market is closed; missing, invalid, disabled or ambiguous calendar evidence refuses.
2. Only **existing open positions** enter the closed-book bucket. Candidate and pending foreign exposures retain live 5-second conversion requirements. The candidate conversion market must also be open even if a subscription response has a recent arrival time.
3. The most recent two-sided pre-close evidence must have both original side timestamps strictly before the current closure and within five seconds of its start. Account-specific broker-dated memory is preferred; otherwise the bounded PostgreSQL tick query reconstructs both sides from one recording session. Historical data never enters the executable quote cache.
4. Evidence is bound to connection generation, account and invalidation revision. Calendar/evidence verification expires after five minutes and is refreshed in the background. Reopening immediately disables historical fallback and reinstates live freshness.
5. For a conversion interval `[low, high]`, apply `low / 1.05` and `high * 1.05` once to the closed conversion chain. Losses use high, profits use low. Missing stops or pre-close evidence refuse; there is no unit-rate substitute for foreign currency.
6. The final guarded writer rechecks these conditions. `crates/jiaoyi-ctrader/src/trading.rs` is byte-identical to the pre-change file.

Stored Friday pre-close evidence was found for AUDUSD and USDJPY across all four accounts. **EURUSD was absent from the bounded Friday tick query.** A broker-dated last quote may be available from the existing subscription on restart; it must satisfy the same provenance checks. If it does not, GER40 exposure correctly remains an admission blocker. This report does not fill that gap with a guessed rate or another account's spread.

## §5. Preserved risk and ownership rules

Strategy policy remains `s3_macro_pullback_owner_18409_v1`. The 0.6% demo trade sizing, 3% S3 open-risk cap, 250% projected margin floor, 1.00R breakeven, 1.38R target, pair reservation and all Three Human Protection Laws remain unchanged. The existing stressed-margin calculation is also unchanged: it requires the stressed level to exceed the broker's recorded margin-call/stop-out warning. The code did not previously impose 250% on both metrics, so this release does not misstate or change that policy.

Live minimum-volume sizing and all S1 trading rules remain intact. No verification order is sent. This change does not guarantee entry when macro/peer/ownership, risk, fee evidence or margin gates refuse.

The prior MARKET_CLOSED deferral implementation is untouched, including its quote-triggered reopen path and retry limits. Regression coverage retains carried Gold deadlines and verifies human/S1 isolation.

## §6. Verification

- `cargo check --workspace --all-targets`: PASS (14.04 seconds).
- `cargo test --workspace`: PASS (322.99 seconds).
- `cargo clippy --workspace -- -D warnings`: PASS (13.43 seconds).
- `cargo fmt --all --check`: PASS (1.13 seconds).
- `cargo test -p jiaoyi --lib cold_start_hydrates_only_existing_book`: PASS (0.94 seconds).
- `cargo test -p jiaoyi-ctrader market_hours::tests`: PASS (10.06 seconds).
- `cargo test -p jiaoyi-ctrader broker_dated_accounting_quotes`: PASS (0.14 seconds).
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (18.29 seconds).

Workspace test result: **810 passed**, **1 ignored**. Focused cases cover cold-start PostgreSQL hydration without executable quotes; $155 and $139.14 Gold risk; direct/inverse and profit-side adverse reserves; wrong-account/missing/old evidence; weekday 5,001ms rejection; closed-FX candidate rejection; reopen freshness; original broker timestamps and invalidation; no-stop refusal; projected/stressed margin failure; existing restart and MARKET_CLOSED protections.

The initial test environment had a stopped local development PostgreSQL container. Once started, an existing Sunday-only fixture error appeared: the restart test used Friday but only created Saturday-onward tick partitions. Its fixture now creates the actual selected session date. The S1 conversion test also now supplies the open FX contract required by candidate admission, rather than relying on missing calendar metadata. Production session definitions are unchanged.

## §7. Release verification required

CI must pass `check`, `docker`, and `engineering` on the exact PR head before squash merge. After Railway deploys the merge commit, verify four connected/unpaused account loops, the exact 08:00 crypto opening references, known Gold reserves, closed-book quote provenance and per-symbol preparation/refusal diagnostics. Compare live human/S1 inventory and Gold deadlines against the pre-release snapshot; distinguish ordinary autonomous market activity from release interference. Observe the closed-market retry lane over a measured interval, not just a single log line.

12:00 SGT preparation can be inspected in advance; its actual opening quote cannot be certified before that time. Any missing EURUSD evidence or legitimate admission refusal must remain explicit in the release attestation. Financial qualification is not asserted.

## §8. Inspectable evidence

The companion JSON includes exact refusal groups, comparison fill identifiers, Gold geometry, available pre-close quotes, command results, protected-file hash, and SHA256 hashes of the read-only source snapshots. Raw authenticated health and database captures remain private; no tokens or passwords are published. Post-deployment evidence will be linked on the PR so another reviewer can read it on GitHub.
