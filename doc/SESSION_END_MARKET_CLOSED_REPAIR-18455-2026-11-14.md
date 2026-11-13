# Session-end closed-market retry repair — №18,455

TL;DR: The old session-end handler retried broker `MARKET_CLOSED` every ten seconds. This repair persists the deferral across restart, wakes on fresh reopening quotes after a quiet feed, and otherwise waits for a scheduled opening or a 30-minute probe. New traditional positions receive an execution-only deadline two minutes before the recorded broker daily/Friday close. Existing deadlines and the trading model remain unchanged.

## §1. Scope and measured fault

Source main: `9a2137a40af6afdc647c02f6a70df40169ce7f84`. Trading at capture: `aebf404dedf89dbcdb8cee78c95a582eea042d51`. This is an execution-hygiene release, not evidence of a better financial strategy. The policy remains `s3_macro_pullback_owner_18409_v1`.

Read-only PostgreSQL evidence captured at **2026-11-13T22:57:54.201222+00:00** records:

| Symbol | MARKET_CLOSED rejections | First dispatch UTC | Last dispatch UTC |
|---|---:|---|---|
| XAUUSD | 707 | 2026-11-13T21:55:00.847529+00:00 | 2026-11-13T22:57:52.866998+00:00 |
| XAGUSD | 700 | 2026-11-12T21:59:00.827447+00:00 | 2026-11-12T23:00:55.859806+00:00 |

The XAUUSD count was still increasing under the old deployment. Positions **262583136** on Demo **47790949** and **262583980** on Demo **46130058** remain open with their original Friday `21:55 UTC` deadline. The release must not rewrite their plans or claim that a weekend market is executable.

## §2. Recorded calendar authority and execution buffer

All **24** stored symbol contracts (six symbols × four accounts) specify `America/New_York`, normal weekday close **16:59**, Friday close **16:55**. Exact decoded intervals, account-specific symbol IDs and raw receipt SHA256 values are in the companion JSON. Read-only stored fill inspection found five deals in the bounded near-close query; it did not establish the last executable minute for all six instruments. The buffer is a preventive margin against the verified schedule, not a fabricated last-fill calibration.

| Scope | Broker close (New York) | New exit deadline (New York) | November SGT deadline |
|---|---|---|---|
| Mon–Thu final traditional session | 16:59 | 16:57 | Next day 05:57 |
| Friday final traditional session | 16:55 | 16:53 | Saturday 05:53 |
| Earlier traditional session boundaries | Existing native close | Unchanged | Unchanged |
| BTCUSD / ETHUSD | Existing four-hour window | Unchanged | Unchanged |

`production_sessions::exit_deadline()` uses version **`native_preclose_v18455_120s`**. `Session.close`, entry eligibility, preparation marks and `sigma_table` are untouched. Both entry writers store the new deadline and policy tag. Already-open plans retain their stored deadline. An entry admitted after the new buffered deadline is immediately due for session exit; this preserves the requested entry rules and is disclosed rather than silently extending its holding period. Broker holiday exceptions can still reject an early close; the retry guard handles that case.

## §3. Closed-market retry state and restart

1. After a terminal `MARKET_CLOSED` response, record one attributable deferral with position ID, rejection time, original deadline, retry-policy tag and next probe.
2. For a closed scheduled market, choose a strictly future native observation opening (including the metals one-minute reopen offset and DST). For a mid-session halt or unknown schedule, use 30 minutes.
3. After a quiet/stale feed, a fresh valid two-sided quote newer than the rejected attempt wakes the close path directly. Preserve the ten-second minimum rejection spacing. Continuous quotes alone cannot prove the broker accepts orders; they do not cause an endless ten-second loop.
4. On reconnect, recover the latest terminal rejection from the account/position/label-scoped persisted ledger. A later filled, unconfirmed or unresolved order prevents recovery of an older rejection. Idle timer checks write no repeated intents or deferrals.

For the two existing gold positions, the scheduled next observation is **Monday 16 November 2026, 07:01 SGT**. An actual earlier reopening quote can wake them, subject to the safeguards above. Monday execution has not yet occurred and cannot be certified by this Saturday report.

## §4. Protection and unchanged policy

There are no changes to entry direction, impulse/retest/turn rules, macro or peer filters, pair caps, 0.6% risk, margin floor, 1.00R gross breakeven activation, 1.38R gross target, labels, human-reserved instruments or S1/S2 logic. No manual order, account pause, setting update, data deletion or broker historical request was issued for this repair. Production database evidence reads used repeatable-read read-only transactions. The authorized Trading image rollout is a deliberate deployment; Engineering stays unchanged.

At **2026-11-13T23:15:04.658783+00:00**, all four S3 actors were connected and unpaused; two gold S3 positions and no crypto S3 positions were open. Live contained five human/external positions, five external pending entries and two S1 positions. Post-deployment comparison must verify their exact geometry and absence of S3 intervention. Eight existing BTC/ETH opening references must survive within the same window; a normal next-window rollover must be reported separately.

## §5. Verification

Focused actor tests cover quiet-feed reopen, 10-second floor, repeated continuous quotes, 30-minute fallback, unique retry intents and restart restoration. Calendar tests cover all six traditional instruments, weekday/Friday, winter/summer DST, strictly future metal reopen, unchanged crypto deadlines, entry windows and sigma marks. A deadline regression covers existing plans, valid versioned late entries and forged past deadlines. The workspace suite also exercises owned-position reconciliation, PostgreSQL opening-reference restore, crypto and human/S1 isolation.

- `cargo check --workspace --all-targets`: PASS (15.63 seconds).
- `cargo test --workspace`: PASS (312.09 seconds).
- `cargo clippy --workspace -- -D warnings`: PASS (15.65 seconds).
- `cargo fmt --all --check`: PASS (0.89 seconds).

Completed workspace test totals: **{'passed': 805, 'failed': 0, 'ignored': 1}**. Pre-change strategy sources were privately archived before editing; the companion JSON contains their manifest and hashes. These backups are excluded from Cargo builds.

## §6. Release verification and limits

At this commit, Railway rollout is **pending**. Merge is conditional on GitHub `check`, `docker` and `engineering` passing. The release PR will receive the measured post-deployment JSON: actual merged commit/image, four connected/unpaused accounts, preserved opening references/positions, latest deferral timestamps and a timed interval with no continuing `MARKET_CLOSED` storm. No success flag is pre-filled here.

The evidence distinguishes a tested immediate quote-triggered retry from an actual broker fill. A continuous-quote unannounced halt can require the scheduled/30-minute probe to discover execution has resumed. A two-minute buffer reduces normal cutoff risk but does not guarantee fills, remove gap risk, or prove a trading edge. `financial_pass` remains **false**.

Companion: [SESSION_END_MARKET_CLOSED_REPAIR-18455-2026-11-14.json](SESSION_END_MARKET_CLOSED_REPAIR-18455-2026-11-14.json).
