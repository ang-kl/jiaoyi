# Production upgrade: S3 macro pullback and eight-symbol core

№18,377 §§1–12 · 08-11-2026 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed>.

## TLDR

The owner-authorised production change replaces immediate peak-breakout entry with an armed, macro-aligned 50% midpoint retracement. New S3 entries are limited to eight symbols. New positions have an original structural −1R stop, an entry-price breakeven amendment after +1R, and a full-position +1.38R gross target. Existing positions retain their recorded exit policy and original risk.

**Financial progress remains unverified.** This change creates no measured improvement in win rate and no new independent backtest. The invariant remains at least 73% net wins over the latest 20 fully closed positions, separately per strategy/account: 15 wins are required. Gross target geometry does not establish realised net payoff or positive expectancy. Live permission is explicit owner authority, not an RFC financial PASS.

Model family: `s3_macro_pullback_v1`. Exact production policy: `s3_macro_pullback_owner_18377_v1`. The earlier research candidate and its receipts remain frozen. This revision follows the pasted production directive's observed impulse distance and midpoint retest; its returns cannot be inferred from the earlier noise-band-distance/executable-touch experiments.

## §1. Authority and scope

Source: owner's pasted Production Upgrade Directive №18,377 §§1–6, dated 08-11-2026 19:38 SGT. The pasted request controls over the supporting attachment: all eleven named noncore instruments are excluded from new S3 entries, and the stop moves to entry-price breakeven rather than an assumed fee buffer. No protected validation dates or fresh historical backtest collection were opened. Local integration tests use an isolated PostgreSQL database and a scripted protobuf broker; they do not send market orders to an external account.

The executable registration is [the owner production registration](../docs/preregistration-2026-11-08-strategy3-macro-pullback.md). Its hash scopes new plans, durable setup state, cost vetoes and Kelly samples. The [current contract](../docs/current-system-contract.md) points to that registration. Existing owner pauses, risk settings, the margin floor, ownership checks and the one broker-minimum live-position rule remain in force.

## §2. Core universe and native clocks

| Class | Symmetric pair | Native strategy session |
| --- | --- | --- |
| Energy | SpotCrude / SpotBrent | 07:00–20:00 UTC, weekdays |
| Equity index | US500 / NAS100 | 09:30–16:00 America/New_York, DST-aware |
| Metal | XAUUSD / XAGUSD | 07:00–20:00 UTC, weekdays |
| Crypto | BTCUSD / ETHUSD | 00:00–24:00 UTC, every day |

These are the existing strategy clocks, not a claim that every broker venue trades continuously throughout them. Holidays, missing quotes, broker constraints and account admission can still prevent an order.

Retired from new S3 entries: NatGas, AUDUSD, NZDUSD, GBPUSD, USDJPY, EURUSD, USDCAD, AUS200, JPN225, UK100 and GER40. Any other noncore name receives `symbol_retired_from_s3_core`. Its presence in the data recorder or account monitor is not entry permission.

An older migration removed US500 from the shared configured universe. The data loader and recorder now include the eight core names without rewriting S1's configured entry universe. S3 also retains historically resolved names for existing-position subscriptions and exits, even when removed from current settings. Missing or ambiguous broker identities are not aliased or invented. Reconciliation can add a core symbol resolved after actor startup, subscribe to its quotes and refresh preparation metadata without discarding held-position or arm state.

## §3. Deterministic state machine

`Native preparation → observed impulse → macro/peer approval → armed midpoint retracement → fresh admitted entry → two-stage exit`

| State | Exact action | Refusal or expiry |
| --- | --- | --- |
| Preparation | Read interval-only M30 history; build the existing sigma band and complete causal 5/20-native-session drifts. Obtain the first valid recorded opening midpoint with both legs in the first six seconds of the native session. | Missing macro slots or opening midpoint refuse entry. No M30 bid-open substitution. Incomplete preparation retries after the bounded preparation retry interval. |
| Impulse | At a native 30-minute mark, consume the first eligible fresh two-sided quote. Midpoint must be outside the band; observed displacement from opening midpoint must be ≥48bp and greater than the registered cost reserve. | No inside-band, weak-impulse or cost-infeasible arm. |
| Macro and peer | Both prior-session drifts agree with the chosen side. A fresh paired midpoint relative to its own session origin must not oppose. | Counter-trend impulses veto. Missing/stale peers are explicitly unavailable and optional under the owner's wording. |
| Armed | Persist the arm before updating memory. Freeze the extreme observed at this mark and half-retracement level. No market order is sent at the impulse. | Crossing the origin invalidates; entering the last 30 minutes disarms. Later extremes do not move the frozen level. |
| Retest | On a strictly later fresh quote, long midpoint ≤ retracement but > origin; short midpoint ≥ retracement but < origin. Persist consumption before awaiting admission. | Same-quote entries and restored consumed triggers are refused. |
| Dispatch | Use current ask/bid, calculate structural risk, then apply existing account/risk/quantity/margin/ownership gates. Immediately before writing, recheck quote, retest, peer, cost, account revision, native price-grid match and deadline. | A moved executable price cannot reuse old structural geometry or size. Deadline is at most five seconds, shortened by remaining entry-window time. |
| Open | Broker receives relative −1R stop and full-position +1.38R target. At executable +1R, request entry-price stop, preserving TP. | No continuous Chandelier tightening or TP1 partial runner for new-policy positions. Session-end close and existing retry/ownership handling remain. |

Risk is `abs(executable_entry - opening_midpoint) + execution_cost_reserve`, rounded outward to the broker price grid. The gross target distance is `1.38 × original_R`, rounded outward. Original admitted/sent R remains immutable through fills and reconciliation. A market fill can differ from its dispatch quote, so exact origin placement after real fill slippage is not guaranteed; that discrepancy is retained as execution evidence, not repaired by widening risk after a fill.

Breakeven is based on actual broker entry/VWAP and executable exit side. Where VWAP falls between price ticks, the requested stop is the first non-losing price tick: upwards for a long, downwards for a short, less than one tick from VWAP. It is never activated before +1R. Entry-price breakeven is gross-flat; fees, financing and stop execution can still produce a net loss.

## §4. Restart, old positions and late evidence

An arm is account/symbol/session/registration scoped and stored in existing append-only decision records. Restore verifies its identity, midpoint arithmetic, impulse magnitude, direction, mark and causal macro context. A consumed trigger is durably cleared before account or broker awaits. An uncertain transmission is reconciled rather than resent. Existing per-session order accounting remains shared across registrations, so upgrading does not grant a second recorded entry in the same session.

Only newly recorded opening plans bearing the new production policy use the two-stage exit. Historical opening plans retain their original R, partial-close allocation, TP and Chandelier rules. They can still be managed after their symbol is retired from new entries. External/manual lifecycle changes continue to invalidate management authority.

New cost vetoes apply to the new registration. Earlier vetoes based on band half-width are not imported into the new observed-impulse policy. Earlier outcomes are not imported into the new Kelly sample. Historical stop-loss reviews retain the opening registration and apply its rules; new reviews require the new arm and final dispatch evidence rather than interpreting a pullback as an invalid breakout.

## §5. Cost and evidence limits

The existing primary reserve remains **24bp per side**, plus prepared spread and recorded commission inputs. No 2bp/3.74bp live admission substitution is introduced. Existing class-level commission fallback remains identifiable as a fallback; it is not a newly verified symbol-specific round-trip cost model.

The 260 statement-matched historical positions contain 36 wins and net P&L of −$15,357.91. Their labels are 251 pure S3, seven unlabelled, one S1 and one mixed S1/unlabelled. Therefore, the full 224 losses cannot be assigned to one old S3 version. Phase 2's breakout comparison had 0/3 forced-window wins on that small qualified sample; it is not proof that every alternative has an edge. See the [exact legacy archive](DEPRECATED_OLD_STRATEGY3_PEAK_BREAKOUT_DO_NOT_USE.md).

Native session generation currently does not provide authoritative exchange-holiday/short-session coverage. Missing M30 slots, including legitimate holiday gaps, remain explicit preparation refusals. The system does not guess them away. Crypto's final prior-session M30 bar cannot be final at T−20/T−10: pre-open status stays incomplete until a causal live-session finalisation read. Neither this report nor the homepage should claim all eight symbols are ready merely because their names are configured.

## §6. Telemetry and shared replay

| Event | Durable evidence |
| --- | --- |
| Arm / veto | Registration, policy, native session, causal drifts, frozen impulse quote and leg times, opening midpoint source, cost and peer status |
| Retest | Millisecond event time, bid/ask/mid, original arm, durable consumed state |
| Dispatch | Opening plan plus `ENTRY_VALIDATED_BEFORE_SEND` evidence from the connection writer, joined by entry key; the decision quote is not relabelled as the dispatch quote |
| Fill / amendment | Raw broker event, order/position identity, fill time, actual price/volume, planned R, requested stop, outcome and quote context |
| Broker exit | Raw execution plus latest quote leg times and received timestamp; nearest stored SL/TP remains a proxy label, not proof of server trigger provenance |

Pure live maths is in [macro_pullback.rs](../crates/jiaoyi-core/src/strategy3/macro_pullback.rs). The bounded [JSON replay adapter](../crates/jiaoyi-core/examples/macro_pullback_replay.rs) shares arm/retest/geometry/breakeven helpers and rejects protected or sealed dependencies. It models executable bid/ask stops, limit-price targets, next-quote-effective breakeven, native-session-end execution and explicit end-of-cache censoring. It does not simulate broker latency, account margin or every session's search for new arms, and does not certify input receipt provenance. No historical returns were generated or selected for this production change.

## §7. Verification

| Check | Local measured result and scope |
| --- | --- |
| Shared pure production rules | 7 passing core tests: whitelist, causal macro slots, impulse/retest, long/short exit parity, censoring and fractional-VWAP breakeven |
| New bounded JSON replay | 1 passing adapter test; the other required research examples add 3 passing tests |
| Broker final guard | 6 passing focused tests, including delayed writer checks, fresh/stale peers and changed executable price refusal |
| S3 integration | 78 passing cases, including the separately run late-resolution test; one existing manual benchmark remains ignored |
| S1 compatibility | 44 passing integration tests; S3-only core discovery does not grant S1 new entry authority |
| Stop-loss review | 17 passing tests; legacy registration fixtures remain explicit; missing new-policy evidence remains unknown |
| Source-quote dispatch budget | Passing queued-event/session-cutoff regression: elapsed quote time cannot restart the five-second budget |
| Loader resolution | Corrected focused test passes: all core names are attempted, missing/ambiguous names remain explicit; no aliases or new S1 settings |
| Research Python | 105 tests run: 104 passed, one existing skip |
| Engineering Python | 191 tests passed |
| Formatting / Clippy / full Rust workspace | Checked locally; complete commit-specific workspace execution is also required by the PR's `check` job |

The first full workspace run identified two compatibility assumptions: the loader expected the old unresolved-symbol count, and S1's historical noncore conversion fixture was unintentionally excluded by an overly broad new-data guard. The resolution fixture now explicitly verifies all core names; the S1 guard is scoped to new core data, preserving existing noncore behaviour. Both affected targets were rerun successfully. CI independently runs the complete workspace, including all new cases.

The repository's required GitHub checks are `check`, `docker` and `engineering`. Their exact commit-specific results and the squash-merge SHA are retained in the PR and final owner review block. Railway deployment identity and post-deploy health are checked separately; merging a PR alone is not evidence of activation.

## §8. Release acceptance and follow-through

- Before merge: all required CI checks must report success for the current PR commit; empty or pending checks are not success.
- After deployment: verify the active trading service commit, new registration/policy and eight configured core names. Check account-loop progress and preparation/refusal reasons without placing a verification trade.
- During real sessions: trace impulse, arm, trigger, final guard, fill, amendment and exit by account/symbol/entry key. Distinguish no qualifying setup, missing evidence and dispatch refusal from a stalled actor.
- At the first fully closed new-policy position: reconcile broker money, original R, actual SL/TP and execution costs. At 20 qualified closes per strategy/account: report net win count, realised payoff, expectancy and drawdown. Do not combine old and new policy outcomes into a claimed pass.
- A technical rollback must not silently restore retired peak entry. The owner controls pauses and any restoration of retired trading rules; existing-position exits retain their authority.

The original 12-gate RFC's economic and external-evidence blockers are not waived by this production instruction. Proven engineering behaviour, owner exposure authority and financial qualification remain separately reported.
