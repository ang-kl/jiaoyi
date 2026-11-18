# Wednesday London preparation and development-only breakeven ablation

№18,490 · 18 November 2026 · Read-only production evidence plus offline research · Prepared by Codex.

**London base preparation is now 24/24; full tables remain 0/24 because only 12 prior London sessions are long enough for the 420/450-minute marks. The exact PR #214 Tier-1 manifest has no crypto and does not implement the active owner18409 entry policy. Its seven legacy trades show no exit or expectancy change with either +0.03R or +0.05R cushions. This does not qualify a production change.**

## §1. Scope, timestamps and result boundaries

Production readiness was read at **2026-11-18T14:19:57.124509+08:00**. Account health was observed separately at **18 November 14:26:18.158855 SGT**. These are current operational checks; the historical ablation uses only the frozen development inputs described below.

The requested active-policy/crypto comparison cannot be completed on this exact Tier-1 manifest: it contains **35 traditional sessions, zero crypto sessions**, and a different historical entry engine. This report completes the live diagnosis and an explicitly labelled **legacy, entry-fixed exit ablation**. It does not relabel those seven historical wins as active-policy performance, substitute Tier 2, inspect other development dates, or open the sealed validation holdout.

Only the two requested files under `doc/` are publication changes. Analysis scripts are private scratch files outside the repository; their text, plan and hashes are embedded in the JSON for reproducibility. No Rust/Python trading code, settings, risk rules, broker requests/orders or Railway deployments were changed.

## §2. Current production readiness

| 18 November session (SGT) | Slots | Base ready | Full tables | Macro ready |
| --- | --- | --- | --- | --- |
| 09:00 | 4 | 4 | 4 | 4 |
| 12:00 | 8 | 8 | 8 | 8 |
| 15:00 | 24 | 24 | 0 | 24 |
| 16:00 | 8 | 8 | 8 | 8 |

**09:00 Brent is 4/4 prepared. 12:00 Crypto is 8/8 base and full. London 15:00 is 24/24 base and macro, but 0/24 full; each table lacks exactly 420 and 450 minutes. 16:00 Crypto is 8/8 prepared.** Base preparation is not permission to enter: the real opening quote, ownership, macro/peer, sizing, risk and margin guards still apply. The 15:00 opening had not occurred at this capture time.

Earlier №18,484 showed 20/24 London base receipts because Brent was beyond the current/next preparation horizon. Those four receipts are now present. All 24 London receipts contain `source_error=false` and both late marks report exactly `12 same-slot endpoints;14 needed`.

| Account | London receipt IDs / symbols | Base / full |
| --- | --- | --- |
| 43002148 | NAS100=347713, SpotBrent=350618, SpotCrude=347730, US500=347734, XAGUSD=347726, XAUUSD=347718 | 6/6 · 0/6 |
| 46130058 | NAS100=347714, SpotBrent=350625, SpotCrude=347733, US500=347737, XAGUSD=347728, XAUUSD=347720 | 6/6 · 0/6 |
| 47790949 | NAS100=347715, SpotBrent=350623, SpotCrude=347731, US500=347736, XAGUSD=347727, XAUUSD=347719 | 6/6 · 0/6 |
| 49236051 | NAS100=347721, SpotBrent=350630, SpotCrude=347735, US500=347738, XAGUSD=347732, XAUUSD=347729 | 6/6 · 0/6 |

## §3. Exact cause of the London lookback shortage

`load_day` obtains closed M30 preparation through the causal cutoff, then calls `production_sessions::sigma_table`. The latter uses the same prior Asia/London/NY slot, at most 30 prior sessions within 45 calendar days. Before looking up an endpoint bar, it rejects `bar_time >= prior_session.close`. Each coefficient still requires 14 eligible values.

London always begins at 07:00 UTC (15:00 SGT). Its end is New York 09:30. US DST ended on **1 November 2026**: before that, London ended at 13:30 UTC (21:30 SGT), lasting **390 minutes**; afterward it ends at 14:30 UTC (22:30 SGT), lasting **450 minutes**.

| Prior dates | London close SGT | 420-minute bar start | 450-minute bar start | Eligibility |
| --- | --- | --- | --- | --- |
| 2–17 Nov: 12 weekdays | 22:30 | 21:30 | 22:00 | Both starts precede close; all 12 counted |
| 30 Oct and 29 Oct: nearest two older weekdays | 21:30 | 21:30 | 22:00 | 420 starts at close; 450 starts after close; both rejected |
| Other older weekdays in the 30-session list | 21:30 | 21:30 | 22:00 | Same structural exclusion |

The exact 12 eligible dates are **2, 3, 4, 5, 6, 9, 10, 11, 12, 13, 16 and 17 November**. Every account/symbol receipt reports those twelve endpoints. October 29 and 30 are present in `sessions_used`, proving their opening bars were available; their late endpoints are **calendar-ineligible**, not two proven missing DB bars. The eligible-calendar maximum equals the measured count (12), so no missing eligible date is needed to explain the deficit.

The public database schema exposes `daily_bars`, not a raw M30 bar table. The inspected `m30_bars` implementation obtains preparation from the broker and the worker persists derived sigma/macro receipts. This audit did not request fresh bars, and does not assert that raw late-October M30 prices are absent everywhere. Their presence would not make them eligible under the present London-slot rule.

**Impact:** coefficients at 30…390 minutes are available. This does not block the whole London session. Interpolation after 390 minutes—after **21:30 SGT until 22:30 SGT**—requires the missing marks. `Prepared.ready()` only requires the first coefficient plus base inputs, so a partial table counts as base-ready and `should_attempt()` does not refetch it. Refetching today cannot create earlier winter-length sessions.

With valid completed bars for 18 and 19 November, a newly prepared **20 November 15:00** session would be the earliest calendar-sufficient London window (14 winter-length predecessors). This is a conditional calendar forecast, not verified future readiness. A lasting DST-transition remedy needs an explicitly versioned coefficient-sampling policy and transition tests; silently borrowing post-London NY bars or reducing 14 to 12 would change the strategy. No such change is included here.

## §4. Exact historical manifest and policy compatibility

| Symbol | Qualified Tier-1 episodes | Admitted legacy trades |
| --- | --- | --- |
| SpotBrent | 8 | 1 |
| SpotCrude | 8 | 4 |
| US500 | 9 | 2 |
| XAUUSD | 10 | 0 |
| BTCUSD | 0 | N/A — absent |
| ETHUSD | 0 | N/A — absent |
| NAS100 / XAGUSD | 0 | N/A — absent |

All 35 episodes are from **4–8 November 2024** and **7–11 April 2025**. The 45-day preparation and continuation timestamp envelopes were checked before opening the allowlisted normalized files, then every contained bar/tick timestamp was checked. Every historical dependency is before 24 October 2025; none touches 19 August–5 September 2025. No 2026 live quote enters the historical outcomes.

The exact cell is **Tier 1, 48bp impulse floor, 3.74bp/side admission, `arm_c_macro_filtered_pullback`**. Native replay reproduced **35/35 complete outputs** against Phase 3, and the 14 strict/forced admitted rows directly reproduce PR #214’s entry, exit and net R. The other 28 no-entry sessions remain in the denominator. Seven trades occur on five reused April dates; correlated oil trades are not independent trials.

| Dimension | PR214 historical cell | Active owner18409 policy |
| --- | --- | --- |
| Impulse / entry | Frozen opening-30m range; half-hour checks; fixed 50% retest | Production sigma bands; observed running extreme; 50–61.8% touch then 8% turn |
| Peer | Causal M30 direction context | Fresh contemporaneous peer hold at least 24bp, plus direction checks |
| Resolution | Bounded 4h tick window; strict censoring or forced boundary | Native production traditional deadlines; crypto arm carry rules |
| Book / volume | Independent episode, fixed native volume | Account risk, margin, human ownership, concurrent pair caps |
| Crypto coverage | Zero Tier-1 crypto inputs | BTC and ETH four-hour entry windows |

**`active_policy_tested=false`.** Matching +1.00R BE / +1.38R TP does not establish entry, calendar or account-engine parity. Full active-policy asset-class performance is therefore **unavailable**, not zero and not 7/7. A new production-compatible replay specification and adequate data are prerequisites to that separate question.

## §5. Frozen breakeven experiment and cost interpretation

Entry timestamp, direction, fill, original R, native volume, target and saved deadline remain fixed. The three exit arms trigger at exactly **+1.00R** and propose stops at **entry + direction × {0, 0.03, 0.05} × original R**. There is no 0.80R trigger. Positive offsets are rounded toward entry to the whole broker price tick; actual offsets are recorded per trade.

A proposal is applied only on a strictly later quote while the proposed stop remains valid. The existing server stop/target and deadline are checked first. An active stop gaps at the recorded executable bid (long) / ask (short), not at an invented guaranteed stop price. The adapter preserves PR214 snapshot/stale-side/gap flags for baseline parity. This is a next-quote acknowledgement scenario, not measured amendment latency or a guarantee of continuous execution.

Recorded bid/ask spread is included once in entry and exit. Additional reserves are **3.74bp per side** and **24bp per side stress**, with admission frozen at 3.74 in both. Per trade:

```text
gross_R = direction × (exit − entry) / original_R_price
net_R = gross_R − [(entry + exit) × reserve_bps / 10,000 + assumed_fee_per_unit] / original_R_price
scenario_USD = net_R × original_R_price × (native_volume / 100)
```

PR214’s commission input is **zero by assumption**. Historical account-specific commissions, financing and amendment slippage distributions are not verified here. The USD column is the original fixed-native-unit replay convention, not broker cash, not 0.6% account sizing and not a reconstructed portfolio. JSON records the extra round-trip fee capacity before each outcome ceases to be positive; no missing fee is invented.

**A +0.03R or +0.05R price stop does not ensure net breakeven.** Ignoring rounding, gaps and extra fees, an approximately unchanged entry/exit price incurs about `2 × reserve_bps / stop_width_bps` R in reserves. At 3.74bp/side, widths of 80 and 120bp cost roughly **0.0935R and 0.0623R**, exceeding either proposed cushion. Ideal minimum widths are about **249.3bp for +0.03R** and **149.6bp for +0.05R**; at 24bp/side they rise to approximately **1,600bp and 960bp**. These are arithmetic thresholds, not fitted trading rules or tested crypto results.

## §6. Asset partitions and performance

**Legacy exit-only diagnostic, not active-policy performance.** Positive net R defines a win. Drawdown is the chronologically exit-ordered cumulative closed-trade R curve, starting at zero; it excludes floating/account correlation effects. Rolling20 uses overlapping windows of 20 closed trades and requires at least 15 wins. Fewer than 20 gives N/A, never a pass.

| Universe / resolution | Episodes | Closed / entries | Net wins | Mean net R (3.74 / 24bp) | Payoff / PF | Max DD R | Rolling20 pass |
| --- | --- | --- | --- | --- | --- | --- | --- |
| All available traditional, forced | 35 | 7 / 7 | 7 / 7 | 1.204630 / 0.419593 | N/A: no losses | 0.000000 | N/A: 7 < 20 |
| Traditional only, forced | 35 | 7 / 7 | 7 / 7 | 1.204630 / 0.419593 | N/A: no losses | 0.000000 | N/A: 7 < 20 |
| All available traditional, strict | 35 | 6 / 7; 1 censored | 6 / 6 | 1.234595 / 0.441394 | N/A: no losses | 0.000000 | N/A: 6 < 20 |
| 4h crypto: all widths | 0 | N/A | N/A | N/A | N/A | N/A | N/A: no data |
| 4h crypto: width <80bp | 0 | N/A | N/A | N/A | N/A | N/A | N/A: no data |
| 4h crypto: 80 ≤ width <120bp | 0 | N/A | N/A | N/A | N/A | N/A | N/A: no data |
| 4h crypto: width ≥120bp | 0 | N/A | N/A | N/A | N/A | N/A | N/A: no data |

“All available” equals “traditional only” because crypto is absent; these are the same observations, not independent confirmations. The requested traditional-plus-crypto full universe is not measured. Payoff ratio and profit factor are undefined with no losing trades, not an invented finite value or proof of safety.

## §7. Cushion comparison and trade-by-trade evidence

| BE stop offset / trigger | Closed / wins (forced) | Mean R 3.74bp | Mean R 24bp | Scenario USD 3.74 / 24bp | Negative→positive flips | Baseline TP runners cut | Mean ΔR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| +0.00R / +1.00R | 7 / 7 | 1.204630 | 0.419593 | 10.264884 / 4.798392 | 0 | 0 | 0.000000 |
| +0.03R / +1.00R | 7 / 7 | 1.204630 | 0.419593 | 10.264884 / 4.798392 | 0 | 0 | 0.000000 |
| +0.05R / +1.00R | 7 / 7 | 1.204630 | 0.419593 | 10.264884 / 4.798392 | 0 | 0 | 0.000000 |

The identical comparison applies to traditional-only. Strict outcomes remain **six targets and one unresolved path**; forced outcomes add one positive four-hour boundary close. All seven paths proposed/applied the +1R-triggered stop, but none returned to any of the three protected stop levels before its recorded resolution. Thus both candidates change **zero fills**, with **zero rescued losses, zero extra early stops and zero expectancy change** at both cost reserves. There are **zero baseline breakeven-stop exits** and zero nonpositive +1R-MFE resolved trades in this cell; it cannot measure rescue of the live slippage-negative failure mode.

| Episode / date | Symbol / direction | Entry / original R | Stop width bp | Native volume | Forced exit | Net R 3.74 / 24bp | Actual offset R .03 / .05 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `77b10f0f75d5383bb011c4b2` / 2025-04-07 | SpotCrude / short | 59.706 / 0.25 | 41.872 | 100 | broker_take_profit | 1.201876 / 0.236957 | 0.028000 / 0.048000 |
| `2b977b08318352c8c79f2cbc` / 2025-04-08 | SpotCrude / short | 61.23 / 0.336 | 54.875 | 100 | session_end | 1.024839 / 0.288786 | 0.029762 / 0.047619 |
| `5b35427f494f9de75bb0ddd1` / 2025-04-08 | US500 / short | 5193.8 / 43.0 | 82.791 | 10 | broker_take_profit | 1.291564 / 0.804938 | 0.027907 / 0.048837 |
| `b8444e851d499207d0449237` / 2025-04-09 | SpotCrude / short | 57.974 / 0.239 | 41.225 | 100 | broker_take_profit | 1.199828 / 0.219736 | 0.029289 / 0.046025 |
| `5ff6b8a3b787709ae2dcc9a0` / 2025-04-10 | US500 / short | 5315.6 / 23.8 | 44.774 | 10 | broker_take_profit | 1.215808 / 0.313617 | 0.029412 / 0.046218 |
| `ce5a75c11b08e52a205e5350` / 2025-04-11 | SpotCrude / short | 60.702 / 0.331 | 54.529 | 100 | broker_take_profit | 1.244006 / 0.503708 | 0.027190 / 0.048338 |
| `70e3acf0ef22a39ed2354106` / 2025-04-11 | SpotBrent / short | 64.17 / 0.378 | 58.906 | 100 | broker_take_profit | 1.254487 / 0.569410 | 0.029101 / 0.047619 |

The 48bp **impulse admission** floor is not the same as initial stop-width bps after a pullback. Some traditional stop widths are below 48bp; that does not create crypto evidence. No winners, losers or cost-admission variants were pooled from other cells to enlarge the sample.

## §8. Verification, production protection and limitations

Frozen plan SHA256: `9cec40225edb3eb94c0cd87467c31a42e412f25358a49a0bb5241d0258aa932c`. Native binary SHA256: `538e080a75e36ac85f6b3999fab7d1d1a66b219b01a3b6f4e597452c3e9852e6`. Each of the 35 normalized input hashes matches the frozen Phase-3 result. All 35 full native outputs reproduce exactly; all 14 strict/forced admitted baseline paths and net-R values match PR214 directly.

Eight focused tests pass: exact +1R activation in both directions; later-quote amendment and executable gap fill; no retroactive rescue; a synthetic path where a cushion cuts a runner; old bracket priority and price-grid rounding; DST endpoint eligibility; holdout/cutoff exclusion; and metric/cost/native-parity arithmetic. Synthetic paths are tests only and contribute zero research trades. Source provenance, queries, scripts, tests, all no-entry reasons, 24 calendar traces and all metric cells are in JSON.

Railway Trading remains on **`65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`**, active deployment `3d8b18d5-4d9f-4007-8fff-22eb25d5ea73`, image `sha256:42609036a809db4674fdc007804d6a73ce15369fbf1e6f356981ba47ddd86d6c`. At the separate health capture, **4/4 accounts were connected and unpaused**, with policy `s3_macro_pullback_owner_18409_v1`. An initial health read timed out; the later response succeeded. No health state is inferred from a timed-out request.

The three limitations governing the conclusion are: no crypto denominator; no exact active-policy/account simulation; and no verified historical full round-trip cost/acknowledgement model. This repeatedly used April development sample is not independent validation. The sealed holdout stays unopened. Zero simulated drawdown across seven selected-by-existing-entry trades is not a portfolio risk guarantee.

## §9. Assessment and next work

Keep the current production policy unchanged. The missing late-London coefficients are explained by the DST calendar; the evidence does not justify repeated historical fetches as a repair. Monitor newly prepared session receipts as the eligible history reaches 14. For a durable remedy, first specify how unequal seasonal session lengths should be sampled and test the spring/fall transitions; changing that rule requires separate review.

Neither crypto exclusion nor a breakeven cushion is qualified by this manifest. The next research prerequisite is a frozen, active-policy-compatible replay with fresh peer quotes, full native deadlines, explicit pair/account assumptions and verified cost evidence, plus genuinely represented crypto stop-width buckets. If broader cached development inputs are authorised later, enumerate and hash that distinct manifest before outcomes are read. Do not quietly add Tier 2, mix in recent live losses, or use the protected holdout to rescue this comparison. Only after those coverage gates pass can the asset filter and BE offset answer the intended question.

**Financial pass: false. Active-policy evaluation: not completed because the exact requested historical scope lacks the necessary policy and crypto coverage. Legacy exit diagnostic and live readiness audit: completed.**

## §10. Files and independent review

- [Machine-readable report and reproducibility evidence](WEDNESDAY_LONDON_PREP_AND_DEV_ABLATION-18490-2026-11-18.json)
- [PR214 baseline report](READINESS_PARSER_AND_PROFIT_LOCK_REPAIR-18412-2026-11-10.md)
- [№18,484 morning readiness reference](WEDNESDAY_MORNING_NY_ASIA_REVIEW-18484-2026-11-18.md)
- [Phase-3 frozen result](../docs/rebuild_phase3_coverage_and_pullback_results_18360.json)
- [Production sigma/calendar source](https://github.com/ang-kl/jiaoyi/blob/65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2/crates/jiaoyi-core/src/strategy3/production_sessions.rs#L317)
- [Production entry policy source](https://github.com/ang-kl/jiaoyi/blob/65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2/crates/jiaoyi-core/src/strategy3/macro_pullback.rs)

Gemini: read §§1–4 before interpreting the numbers in §§6–7. Verify `active_policy_tested=false`, the zero crypto denominator, 35 native parity receipts, 14 direct PR214 comparisons and the eight tests in JSON. Inspect each of the 24 London calendar traces: twelve eligible dates, no missing eligible dates, and October 29–30 explicitly excluded by session close. Review the two-file PR/CI and merge commit for publication evidence; a green documentation PR is not financial qualification.
