# Phase 4 — Arm C trade anatomy and one frozen hypothesis

№18,362 §§1–12 · 08-11-2026 10:27:35 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed>

## TLDR

**Tier 1's seven wins are mostly completed targets:** 6 strict +1.38R target hits and 1 profitable forced four-hour close. There are 0 breakeven-stop exits. All seven are short trades concentrated in five dates in the April 2025 cohort. This supports a specific development hypothesis, not a proven general edge.

**Tier 2's six wins are more dependent on the clock and cost assumption:** 3 target hits, 3 profitable forced closes and 2 losses. All eight become losses at 24 bp/side stress pricing. A target hit is therefore not sufficient evidence of positive net expectancy.

**The three vetoed paths are hypothetical B trades, not executed C trades.** All three are unresolved under strict four-hour bracket accounting and negative at forced close. Own 5/20-session direction vetoes all 3; effective peer-only vetoes: 0. Their mean avoided counterfactual return is -0.838747R under the 24 bp stress scenario.

**Tier-2 upgrade inventory:** 167 episodes; none ready locally. 18 need only missing intervals outside the retained request catalog; 132 need gaps within previously requested/reserved intervals or calendars reconciled; 17 need sealed preparation and cannot be upgraded under the present seal. No episode is after 24 October 2025.

**Frozen candidate:** `s3_macro_pullback_v1`, the full-M30 48 bp impulse / 3.74 bp admission rule with primary 24 bp stress evaluation. The opening-range 28 bp variant is not silently substituted. Four metadata-selected historical weeks reserve 20 dates × 8 symbols = 160 tick jobs, plus at most 256 deduplicated M30 segment jobs. None is executed. The November week needs explicit holiday/short-session verification before these ceilings become executable jobs.

**Goal progress:** 18 previously simulated paths audited; 0 new independent observations; 0 new broker requests; 0 account-goal qualification. The original 73% / 1:1.38 latest-20 goal per strategy/account and interim 50% per-class research screen remain unchanged. RFC gates fully passed: 2/12, inherited bounded engineering passes G2/G3; freezing a proposal does not pass the financial gates.

## §1. Evidence, terminology and the question answered

This audit reads Phase 3 at commit `145f9154426ba355d95cb2413f9d4a634a5051ef` (PR #193), its immutable native-output receipts, the Phase 1 manifest and verified cached quote files. It does not rerun new strategy arms or inspect new market dates. All 18 entries and exits are reconciled to recorded executable-side quotes; target fills are the replay engine's target-level assumption once the recorded quote crosses that level. These are broker-data **simulated paths**, not actual cTrader execution deals.

For each row, gross R is signed `(exit-entry)/initial_r`; cost-scenario net R subtracts `(entry+exit)*bps/10000/initial_r`. Recorded bid/ask spread is already included once. Decimal arithmetic independently checks both cost columns. Commission, swap, conversion, actual dispatch latency, queue fills and full round-trip slippage remain unverified. The current contract grid is a recorded scenario, not a verified historical contract specification.

Strict target means the target resolved inside the observed window, with outward grid rounding around gross 1.38R. A `forced_window_close` uses the separate boundary-close calculation after native session open+4h, not four hours after entry. A stop moved to entry is distinct from a trade actually exiting at that stop; a gap through an amended entry stop can still lose money. The audit classifies actual exit state rather than guessing from positive/negative P&L.

| Group | Trades | Strict targets | BE-stop exits | Initial stops | Forced closes | Profitable forced closes at 3.74 bp | Wins at 3.74 bp | Wins at 24 bp | Mean R at 3.74 bp | Mean R at 24 bp | Distinct dates |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1_7_of_7 | 7 | 6 | 0 | 0 | 1 | 1 | 7 | 7 | 1.20463 | 0.419593 | 5 |
| tier2_6_of_8 | 8 | 3 | 0 | 1 | 4 | 3 | 6 | 0 | 0.279986 | -0.940631 | 7 |
| vetoed_3 | 3 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | -0.345236 | -0.838747 | 3 |

## §2. Tier 1 — complete seven-trade Arm C ledger

The six strict targets are the stronger part of this result. The remaining winner is a marked-to-market close, not another observed target completion. The three represented instruments include two correlated oil contracts. The short-only concentration in one selected macro week prevents extrapolation to long trades, normal-volatility periods, metals, crypto or live accounts.

| Episode ID | Symbol | Date | Direction | Impulse R bp | 5-session drift | 20-session drift | Peer status | Entry UTC | Exit UTC | Exit class | Net R at 3.74 bp | Net R at 24 bp |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 70e3acf0ef22a39ed2354106 | SpotBrent | 2025-04-11 | short | 56.7487 | -8.00% | -9.69% | non_conflicting | 2025-04-11T10:03:37Z | 2025-04-11T10:51:42.127Z | strict_target_1_38r | 1.25449 | 0.56941 |
| 77b10f0f75d5383bb011c4b2 | SpotCrude | 2025-04-07 | short | 48.2385 | -9.55% | -6.07% | non_conflicting | 2025-04-07T09:09:32.322Z | 2025-04-07T09:21:41.276Z | strict_target_1_38r | 1.20188 | 0.236957 |
| 2b977b08318352c8c79f2cbc | SpotCrude | 2025-04-08 | short | 48.0801 | -15.14% | -7.76% | non_conflicting | 2025-04-08T10:21:36.559Z | 2025-04-08T11:00:00.219Z | forced_window_close | 1.02484 | 0.288786 |
| b8444e851d499207d0449237 | SpotCrude | 2025-04-09 | short | 51.7444 | -17.59% | -11.51% | non_conflicting | 2025-04-09T09:09:34.220Z | 2025-04-09T09:28:50.574Z | strict_target_1_38r | 1.19983 | 0.219736 |
| ce5a75c11b08e52a205e5350 | SpotCrude | 2025-04-11 | short | 60.4986 | -8.39% | -10.13% | non_conflicting | 2025-04-11T10:03:36.606Z | 2025-04-11T10:50:18.330Z | strict_target_1_38r | 1.24401 | 0.503708 |
| 5b35427f494f9de75bb0ddd1 | US500 | 2025-04-08 | short | 70.6152 | -9.61% | -9.73% | absent | 2025-04-08T15:40:52.170Z | 2025-04-08T16:23:00.163Z | strict_target_1_38r | 1.29156 | 0.804938 |
| 5ff6b8a3b787709ae2dcc9a0 | US500 | 2025-04-10 | short | 64.5844 | -0.49% | -2.70% | absent | 2025-04-10T14:31:27.206Z | 2025-04-10T14:58:15.046Z | strict_target_1_38r | 1.21581 | 0.313617 |

## §3. Tier 2 — complete eight-trade Arm C ledger

The two losses comprise one initial-stop exit and one negative forced close. Three other forced closes are profitable only under the lower cost assumption. The narrowest winner is SpotBrent on 2025-10-07: break-even symmetric slippage is approximately **3.803149 bp/side**, only 0.063149 bp/side above the assumed 3.74. An omitted fee or slightly worse execution can reverse it.

The three target hits also turn negative under 24 bp stress because their risk distance is small relative to the stated reserve. This is direct evidence of cost fragility, not an argument that costs should be reduced until results pass.

| Episode ID | Symbol | Date | Direction | Impulse R bp | 5-session drift | 20-session drift | Peer status | Entry UTC | Exit UTC | Exit class | Net R at 3.74 bp | Net R at 24 bp |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| e2f8731ee1bbbd8ebec301bb | BTCUSD | 2025-05-12 | long | 38.0711 | 7.50% | 19.00% | absent | 2025-05-12T01:21:01.902Z | 2025-05-12T01:34:54.288Z | initial_stop_loss | -1.27968 | -2.55704 |
| daaa576e731c68edb339d1a8 | BTCUSD | 2025-10-08 | long | 41.3967 | 0.63% | 4.17% | absent | 2025-10-08T01:16:24.227Z | 2025-10-08T02:21:59.944Z | strict_target_1_38r | 1.15435 | -0.0681349 |
| 362375ec6c45fb8d7a2b7d3f | SpotBrent | 2025-10-07 | short | 35.1574 | -1.95% | -1.49% | non_conflicting | 2025-10-07T10:03:15.212Z | 2025-10-07T11:00:00.440Z | forced_window_close | 0.0027674 | -0.885093 |
| 5aca717f89d879a30446d2ed | SpotCrude | 2025-10-07 | short | 34.0583 | -2.36% | -1.85% | non_conflicting | 2025-10-07T10:53:36.730Z | 2025-10-07T11:00:00.434Z | forced_window_close | 0.236858 | -0.866446 |
| 714dcd4ea643c7abb6c67ecd | US500 | 2023-01-24 | long | 30.3611 | 0.49% | 4.56% | absent | 2023-01-24T17:01:07.816Z | 2023-01-24T18:30:00.811Z | forced_window_close | 0.408516 | -1.00561 |
| f8992619aab40fdf9fa815cf | US500 | 2023-01-27 | long | 46.1985 | 3.74% | 6.28% | absent | 2023-01-27T15:46:46.666Z | 2023-01-27T17:16:17.598Z | strict_target_1_38r | 1.16473 | -0.0385635 |
| 0c8d1df35ca48a63714fb422 | US500 | 2024-11-04 | short | 32.9478 | -2.07% | -0.12% | absent | 2024-11-04T17:28:23.969Z | 2024-11-04T18:30:00.215Z | forced_window_close | -0.608762 | -2.06672 |
| 2c7fe293cc545089e134e08c | US500 | 2025-02-12 | long | 40.4261 | 0.76% | 2.58% | absent | 2025-02-12T16:12:41.579Z | 2025-02-12T17:00:57.222Z | strict_target_1_38r | 1.1611 | -0.0374353 |

## §4. Three B paths rejected by Arm C

All 3 effective vetoes come from the own 5/20-session trend test, evaluated before the peer test. 1 path(s) would additionally conflict with the peer's direction; that is not counted a second time as an effective peer veto. Missing context is not responsible for these three decisions. C entered none of them. The avoided mean is a conditional retrospective difference, not realised profit or proof of future filtering accuracy.

| Episode ID | Symbol | Date | Direction | Impulse R bp | 5-session drift | 20-session drift | Peer status | Entry UTC | Exit UTC | Exit class | Net R at 3.74 bp | Net R at 24 bp |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 8d44e2c3e3ecabd890265eb0 | SpotCrude | 2025-05-15 | short | 55.9837 | 7.53% | 0.21% | disagrees | 2025-05-15T10:14:34.071Z | 2025-05-15T11:00:00.086Z | forced_window_close | -0.329546 | -0.793489 |
| 40cc399289d0861550320dd8 | US500 | 2023-01-26 | short | 56.7715 | 2.89% | 5.47% | absent | 2023-01-26T16:48:09.553Z | 2023-01-26T18:30:00.598Z | forced_window_close | -0.271589 | -0.763822 |
| 4d12cf15cd8e08baa46e7d29 | US500 | 2025-05-12 | short | 55.8746 | 0.26% | 3.66% | absent | 2025-05-12T14:53:47.467Z | 2025-05-12T17:30:02.211Z | forced_window_close | -0.434574 | -0.95893 |

## §5. Price geometry, costs and exit mechanics

| Group / episode | Symbol | Fill | Exit | Initial risk price | Gross target distance | Actual gross R | Symmetric break-even cost bp/side | Stop amendments applied | Raw exit reason | Continuity caveat |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1_7_of_7 / 70e3acf0ef22a39ed2354106 | SpotBrent | 64.17 | 63.648 | 0.378 | 0.522 | 1.38095 | 40.8393 | 1 | broker_take_profit | gap_before_quote |
| tier1_7_of_7 / 77b10f0f75d5383bb011c4b2 | SpotCrude | 59.706 | 59.361 | 0.25 | 0.345 | 1.38 | 28.9753 | 1 | broker_take_profit | gap_before_quote |
| tier1_7_of_7 / 2b977b08318352c8c79f2cbc | SpotCrude | 61.23 | 60.84 | 0.336 | 0.464 | 1.16071 | 31.9489 | 1 | session_end | gap_before_quote |
| tier1_7_of_7 / b8444e851d499207d0449237 | SpotCrude | 57.974 | 57.644 | 0.239 | 0.33 | 1.38075 | 28.5423 | 1 | broker_take_profit | gap_before_quote |
| tier1_7_of_7 / ce5a75c11b08e52a205e5350 | SpotCrude | 60.702 | 60.245 | 0.331 | 0.457 | 1.38066 | 37.7851 | 1 | broker_take_profit | gap_before_quote |
| tier1_7_of_7 / 5b35427f494f9de75bb0ddd1 | US500 | 5193.8 | 5134.4 | 43 | 59.4 | 1.3814 | 57.5124 | 1 | broker_take_profit | gap_before_quote |
| tier1_7_of_7 / 5ff6b8a3b787709ae2dcc9a0 | US500 | 5315.6 | 5282.7 | 23.8 | 32.9 | 1.38235 | 31.0427 | 1 | broker_take_profit | gap_before_quote |
| tier2_6_of_8 / e2f8731ee1bbbd8ebec301bb | BTCUSD | 104357 | 104012 | 330.49 | 456.08 | -1.04387 | -16.5566 | 0 | gap_through_stop | gap_before_quote |
| tier2_6_of_8 / daaa576e731c68edb339d1a8 | BTCUSD | 121680 | 122238 | 404.24 | 557.86 | 1.38002 | 22.8708 | 1 | broker_take_profit | gap_before_quote |
| tier2_6_of_8 / 362375ec6c45fb8d7a2b7d3f | SpotBrent | 65.76 | 65.71 | 0.3 | 0.414 | 0.166667 | 3.80315 | 0 | session_end | gap_before_quote |
| tier2_6_of_8 / 5aca717f89d879a30446d2ed | SpotCrude | 61.859 | 61.759 | 0.227 | 0.314 | 0.440529 | 8.08944 | 0 | session_end | gap_before_quote |
| tier2_6_of_8 / 714dcd4ea643c7abb6c67ecd | US500 | 4009.6 | 4017.3 | 11.5 | 15.9 | 0.669565 | 9.59274 | 0 | session_end | gap_before_quote |
| tier2_6_of_8 / f8992619aab40fdf9fa815cf | US500 | 4058.9 | 4077.9 | 13.7 | 19 | 1.38686 | 23.3507 | 1 | broker_take_profit | gap_before_quote |
| tier2_6_of_8 / 0c8d1df35ca48a63714fb422 | US500 | 5718.3 | 5723.7 | 15.9 | 22 | -0.339623 | -4.71945 | 0 | session_end | gap_before_quote |
| tier2_6_of_8 / 2c7fe293cc545089e134e08c | US500 | 6020 | 6048.2 | 20.4 | 28.2 | 1.38235 | 23.3672 | 1 | broker_take_profit | gap_before_quote |
| vetoed_3 / 8d44e2c3e3ecabd890265eb0 | SpotCrude | 60.962 | 61.092 | 0.533 | 0.736 | -0.243902 | -10.651 | 0 | session_end | gap_before_quote |
| vetoed_3 / 40cc399289d0861550320dd8 | US500 | 4030.1 | 4036.1 | 33.2 | 45.9 | -0.180723 | -7.43845 | 0 | session_end | gap_before_quote |
| vetoed_3 / 4d12cf15cd8e08baa46e7d29 | US500 | 5815.7 | 5830.9 | 45 | 62.1 | -0.337778 | -13.051 | 0 | session_end | gap_before_quote |

The same fixed admission reserve also contributes to B/C's structural risk distance. Consequently, lower-cost admission changes both selection and risk/target geometry; fixed-entry repricing isolates accounting only after those entries are chosen. This known coupling is preserved for a faithful candidate replication. Separating risk geometry from cost would require a different registered version, not a silent amendment labelled the same strategy.

Every selected path carries recorded continuity caveats. A quote matching a modelled fill proves price-path consistency, not guaranteed executable liquidity. No profitable forced close is relabelled as a strict target; no censored path is turned into a zero-return observation.

## §6. What can be upgraded from Tier 2?

| Mutually exclusive upgrade disposition | Episodes |
| --- | --- |
| needs_spent_gap_or_calendar_reconciliation | 132 |
| only_unspent_history_missing | 18 |
| protected_45day_dependency | 17 |

| Symbol | Tier 2 episodes | Partial local prep | No local prep in lookback | Ready now | Only unspent gaps | Spent/reserved/calendar gaps | Protected prep |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude | 30 | 26 | 4 | 0 | 0 | 26 | 4 |
| SpotBrent | 30 | 26 | 4 | 0 | 0 | 27 | 3 |
| US500 | 42 | 38 | 4 | 0 | 1 | 36 | 5 |
| NAS100 | 7 | 4 | 3 | 0 | 7 | 0 | 0 |
| XAUUSD | 32 | 32 | 0 | 0 | 0 | 32 | 0 |
| XAGUSD | 6 | 3 | 3 | 0 | 6 | 0 | 0 |
| BTCUSD | 18 | 16 | 2 | 0 | 2 | 11 | 5 |
| ETHUSD | 2 | 0 | 2 | 0 | 2 | 0 | 0 |

All 167 symbols/episode combinations have some local M30 files for their symbol somewhere in the catalog. That is weaker than having the required history for a particular date: 145 have partial usable preparation and 22 have none inside the required lookback. No complete 45-day preparation exists for these 167 definitions under the existing rules.

The January 2024 NAS100/XAGUSD collector requested only the four-hour intraday M30 job window. Its embedded bars must not be treated as a spent 45-day preparation interval. Correctly reading that collector moves nine cases from the initial conservative gap category into the unspent-history category. A regression test now protects this distinction. Other inferred envelopes remain explicitly conservative; definitive broker-global request history is not available.

**Smallest concrete missing slice:** US500 4 November 2024 has 401 of 403 required slots. Its missing range is `[{"from": "2024-09-20T13:30:00Z", "m30_slots": 2, "to": "2024-09-20T14:30:00Z"}]`. The collection boundary was shifted by the later UTC session opening across DST, leaving the earlier preparation day's first hour outside the requested range. This is a specific collection-boundary diagnosis, not evidence that all incomplete M30 history is broken. The future plan now starts preparation at the earlier of the 45-day UTC boundary and the dated native session opens, so it includes this hour across DST. A focused regression covers that boundary. No refetch or code change to the trading strategy is performed.

The 17 protected cases occur on October 2025 labels whose 45-day context reaches the sealed 19 August–5 September interval. They cannot be upgraded by simply collecting the missing bars. The 132 spent/reserved-gap cases need source/decoder or dated-calendar reconciliation before any further request. Holidays and maintenance may legitimately have no bar; counting coded weekdays alone is not an authoritative market calendar. The 18 potential unspent repairs would still be reused development data, and their new Tier1 signals/outcomes are unknown until qualified; existing Tier2 profits do not transfer automatically.

## §7. One frozen candidate economic hypothesis

| Frozen component | Specification |
| --- | --- |
| id | s3_macro_pullback_v1 |
| economic_hypothesis | Conditional on same-sign5/20-session price trends and a same-direction historical sigma-band impulse, a50% retracement can offer a better remaining-session continuation opportunity than immediately chasing the impulse. Persistent order-flow/rebalancing is a proposed mechanism, not observed participant evidence; the macro input is technical price context only. |
| falsification | Reject a general edge if independently reserved sessions fail positive stress-cost expectancy, if incremental C-versus-B returns vanish, or if apparent gains depend on one week/class, cheap unverified costs, censoring or data abstention. |
| primary_model | Phase 3 tier1,48bp threshold,3.74bp/side admission; exactly preserve the tested full-M30 rule. |
| source_mode | Full45-calendar-day coded-session M30 qualification; no Tier2 opening-range fallback. Historical broker calendars and contracts remain prerequisites, not inferred from missing bars. |
| impulse | Existing native14-session sigma and half-hour marks; first admitted impulse; R>=48bp at decision and later fresh fill. Full spread+2*price*3.74bp C<R durable admission veto. Strictly later fresh executable dispatch within5s; no hindsight peak. |
| entry | Freeze impulse midpoint; fixed50% retracement towards fresh opening midpoint, executable side touch on a later quote, reject origin overshoot. One attempt/session; no retry after macro veto or no retest. |
| macro | At session open, last5 and20 completed coded-session drifts must strictly agree with impulse direction. Same-account same-date peer, when present, must have complete causal context and neither drift opposing. Missing peer context blocks; absent peer allowed and separately reported, matching Phase 3. |
| planned_peer_policy | All8 symbols are scheduled, so peers should be present. A failed planned peer is data failure, not permission to recategorise it as absent; no outcome-conditioned peer availability. |
| initial_r | Ceil-to-grid(abs(entry-open_mid)+touch_spread+2*entry*3.74/10000), preserving cost-dependent tested geometry. This coupling is disclosed; a risk-independent redesign would be a different candidate. |
| exit | Initial-1R stop; entry-price stop proposal at+1R applied on strictly later valid quote; gross+1.38R target rounded outward. Existing bracket priority. Primary horizon ends at native session open+4h with fresh<=5s forced close; separate strict bracket-exit anatomy. No72h continuation in the confirmatory score. |
| sizing | Fixed native volumes from manifest for quote-path comparison only; no account Kelly/margin simulation or change to live risk limits. |

The hypothesised cause is persistence in a pre-existing directional flow after a temporary counter-move. Institutional rebalancing is a possible mechanism, **not measured evidence in this cache**. The observable forecast is continuation after a same-direction impulse and retracement during the remaining session. It is falsifiable by negative independent stress-cost expectancy, failure against the fixed B control, concentration in one event/class or sensitivity to unverified execution assumptions.

The three admitted research arms are not three candidates to optimise after testing. Only C is the candidate. A and B are unchanged controls; no-trade is a zero-return reference. Planned peers are deliberately scheduled uniformly. A failed planned peer is a data failure, not a convenient transition to the peer-absent policy that helped some earlier US500 results.

## §8. Frozen independent-historical evaluation design

| Week beginning | Scheduled dates | Prep envelope begins UTC | Envelope ends UTC | Tick jobs ceiling | M30 segment ceiling |
| --- | --- | --- | --- | --- | --- |
| 2023-03-20 | 2023-03-20, 2023-03-21, 2023-03-22, 2023-03-23, 2023-03-24 | 2023-02-03T00:00:00Z | 2023-03-25T00:00:00Z | 40 | 64 |
| 2023-05-22 | 2023-05-22, 2023-05-23, 2023-05-24, 2023-05-25, 2023-05-26 | 2023-04-07T00:00:00Z | 2023-05-27T00:00:00Z | 40 | 64 |
| 2023-09-18 | 2023-09-18, 2023-09-19, 2023-09-20, 2023-09-21, 2023-09-22 | 2023-08-04T00:00:00Z | 2023-09-23T00:00:00Z | 40 | 64 |
| 2023-11-20 | 2023-11-20, 2023-11-21, 2023-11-22, 2023-11-23, 2023-11-24 | 2023-10-06T00:00:00Z | 2023-11-25T00:00:00Z | 40 | 64 |

Scan Monday-start weeks chronologically from2023-01-02; five weekday labels/week; choose first4 with entire45-day prep+label bundle outside retained price/request intervals, protected dependencies, prior plan label dates and other selected bundles. Metadata only, no returns used. Reserve all8core symbols uniformly.

The retained registry contains 2279 deduplicated interval records and 100 date reservations from 89 metadata files. Tick labels and 45-day preparation bundles do not overlap retained market/request intervals, the fixed protected interval or each other. Selection used metadata, not price/return results.

**Limits of independence:** 530 historical references were unavailable in the earlier inventory. The overlap claim is therefore against the retained catalog, not universal broker/chat history. These 2023 dates also precede later development samples; this is frozen historical replication, not chronological forward validation. G9 remains partial. Before execution, reconcile current global reservations and unavailable references where their bounds are uncertain; abort on an overlap rather than quietly relabelling a reused date.

The four-week/eight-symbol scope is the minimum number of weeks requested while preserving the existing core universe. It is a **maximum of 160 tick jobs and 256 M30 jobs**, not 416 raw broker messages: tick pagination can require multiple messages and must retain existing per-job caps. M30 segments are deduplicated across the five dates and use interval-only mode without count=500. The plan does not launch a collector, consume an engineering-model session allowance or create a paid API call.

**Calendar checkpoint:** The mechanically selected November2023 week requires particular verification around23–24November (US Thanksgiving period). Historical CFD trading hours remain unverified;20 dates/160 tick jobs are ceilings before the calendar gate, not promises of20 full market sessions. Holidays/shortened sessions must be classified using dated authoritative broker/venue evidence before any outcome is revealed. Exclusions stay in the denominator, and dates are not replaced after looking at performance. In particular, November 23–24 must not be assumed to be two ordinary US cash sessions.

Primary score: forced 4h partial-cost netR at 24 bp/side plus recorded spread, with strict exit anatomy reported separately. Secondary 3.74 bp is a fixed sensitivity, not measured round-trip cost. Unknown fees and financing block financial claims. Candidate risk/entry parameters do not change if later cost calibration is inconvenient.

Each class: at least20 priced closed candidate trades across at least3 of4 weeks, net win-rate>=50%, positive mean stress netR, and positive C-minus-B netR per eligible opportunity. Otherwise fail or insufficient evidence; aggregate pooling cannot qualify a class. The73%/1:1.38 account goal remains unchanged and is never claimed from this pilot.

Report per-week metrics and descriptive95% Wilson win-rate intervals with dependence caveat; four selected calendar blocks are insufficient for robust population confidence or live promotion. No multiple-candidate winner selection.

Exactly one candidate and fixed controls; publish all eligible/missing/censored outcomes after the complete batch. No optional stopping at20wins, no automatic extra weeks, no threshold/cost/timing retuning on these dates.

Four weeks may produce too few candidate closes, especially by class. That result is insufficient evidence, not permission to keep sampling until a target hit rate appears. The account-level 73% goal needs actual independently verified account trading outcomes; it is not certified by this pilot even if its descriptive win percentage is high.

## §9. Acceptance gates and next authorised boundary

| RFC gate | Disposition | Evidence / remaining requirement |
| --- | --- | --- |
| G1 | partial | Source/code/calendar/contract/cost identities linked; historical authority remains unverified. |
| G2 | passed | Inherited verified159/35 parity; replay code unchanged in Phase 4, Phase 3 hashes retained. |
| G3 | passed | Offline manifest workflow, zero broker requests, deterministic audit arithmetic and complete denominators. |
| G4 | partial | Exact167 gap map; protected and spent gaps cannot be silently filled. Future full paths not collected. |
| G5 | open | Dated historical holidays, maintenance and contract validity required before collection/qualification. |
| G6 | open | Matched broker round-trip deals, fees, swaps and conversion not available. |
| G7 | open | No out-of-sample calibrated round-trip cost error bounds;3.74bp remains assumption. |
| G8 | partial | 18 cached native paths and recorded quote prices reconciled; real execution/restart parity unproved. |
| G9 | partial | Single candidate and locally unspent four-week plan frozen. Missing historical references and future collection/execution checks prevent universal independence certification. |
| G10 | not_passed | Selected historical paths are development evidence; no independent result generated. |
| G11 | not_performed | No broker shadow/canary evaluation. |
| G12 | not_demonstrated | No per-strategy/account operational goal qualification. |

Phase 4 completes the forensic audit and freezes the hypothesis/plan. It does not complete the eight-phase rebuild or authorise fresh collection/live deployment. Before a later execution phase, resolve the unspent-history audit bounds, calendars/contracts and round-trip execution costs; then collect exactly the reserved qualified inputs once and evaluate the frozen candidate/controls together. Preserve current proven implementation and risk rules.

## §10. Verification and reproducibility

- Audit JSON SHA256:`2581b38797f917f78e7b09a4716fa38104cf12ffda2bf20a18416c8c9d05275a`.
- Scope SHA256:`0a41a266b50cea12c760c2d8e1d5963869d69ceefbc6c9d25df7f1d76de5e1f4`; builder SHA256:`36db9c3c3669f3b723f349590042d5fb57e78779aa13210167dd48dbb06eaea7`.
- 18 cached paths; 36 independent decimal net-cost checks; 36 recorded-side price checks; 18 native output receipts reconciled.
- 160 future tick/preparation dependency checks; no broker requests.
- Five focused regression tests cover cost arithmetic/exit classification, independent-week overlap, gate inflation the January intraday-M30 boundary and the future DST preparation boundary. The report verification artifact records full research-suite and layout results.
- No Rust source or live trading rule changed. Phase 3 parity evidence is inherited and hash-linked; Phase 4 does not pretend to have generated new backtest observations.

```text
python3 ops/research/phase4_forensics.py
python3 -m unittest discover -s ops/research -p test_phase4_forensics.py
python3 ops/research/phase4_report.py
```

Reproduction needs the immutable Phase 1 cache and Phase 3 native receipts at the frozen paths, plus the existing calendar metadata binary. A GitHub-only checkout can read all delivered ledgers/plans but cannot recreate raw quote checks without those cached files. Missing or modified receipts fail closed. The verification JSON is separate from the deterministic measured audit.

## §11. Complete 167-episode upgrade register

| Episode | Symbol | Date | Required prep slots | Local prep slots | Missing prep slots | Opening M30 present | Missing within spent/reserved intervals | Missing outside retained intervals | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 001f989aba50039f6db56506 | US500 | 2024-07-24 | 416 | 399 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 00aba56cdad194a6a6e1630d | XAUUSD | 2023-07-28 | 858 | 852 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 00b72c8130b8d25638b1b9b8 | NAS100 | 2025-07-14 | 403 | 0 | 403 | False | 0 | 404 | only_unspent_history_missing |
| 0223cba302b7e83b96407428 | SpotBrent | 2025-07-21 | 806 | 796 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 04ea9a8aed2ded217967b9a0 | US500 | 2024-07-23 | 403 | 386 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 068cb592754e6491f5762257 | US500 | 2025-07-14 | 403 | 321 | 82 | True | 17 | 65 | needs_spent_gap_or_calendar_reconciliation |
| 0c8d1df35ca48a63714fb422 | US500 | 2024-11-04 | 403 | 401 | 2 | True | 0 | 2 | only_unspent_history_missing |
| 0d8adf49267297042d98d06a | SpotBrent | 2025-10-06 | 806 | 520 | 286 | True | 286 | 0 | protected_45day_dependency |
| 10d1ae4831eddd0539ba98b8 | XAUUSD | 2024-01-23 | 806 | 753 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 11a6a8c22334e78217761ee7 | SpotCrude | 2025-02-13 | 858 | 831 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 11bdabc3c84d460a73a8eb56 | XAUUSD | 2024-01-22 | 806 | 753 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 134bb1b1c5cae3963b7a4c6c | SpotCrude | 2025-10-06 | 806 | 520 | 286 | True | 286 | 0 | protected_45day_dependency |
| 136bcb5c0c607b3895112788 | US500 | 2025-07-07 | 403 | 256 | 147 | True | 17 | 130 | needs_spent_gap_or_calendar_reconciliation |
| 13d1b94f806804837755c3fb | SpotCrude | 2024-07-25 | 858 | 850 | 8 | True | 8 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 14cfdcad3095953593d732bf | ETHUSD | 2025-07-07 | 2160 | 0 | 2160 | False | 0 | 2161 | only_unspent_history_missing |
| 18c4d4b660a470dcab36d784 | XAUUSD | 2025-02-13 | 858 | 831 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 196614c16843b2ea7ef2c9a0 | XAUUSD | 2025-07-23 | 832 | 823 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 1a897af29e15f58be0c69494 | SpotBrent | 2024-07-24 | 832 | 820 | 12 | True | 12 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 1d8414a63d83d92b96f7d5c8 | XAUUSD | 2024-07-23 | 806 | 800 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 2148bb57b5270748d1dc5857 | US500 | 2025-02-10 | 403 | 369 | 34 | True | 34 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 2549b6349e4927934cea6b9a | NAS100 | 2024-01-26 | 429 | 32 | 397 | True | 0 | 397 | only_unspent_history_missing |
| 258b031ea15e984f9aa6c571 | SpotBrent | 2025-07-25 | 858 | 848 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 25a29bafe6cd3fdda3e5faa4 | XAUUSD | 2025-07-25 | 858 | 849 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 25e98df1d7dd285726528311 | US500 | 2025-10-09 | 429 | 299 | 130 | True | 130 | 0 | protected_45day_dependency |
| 260d8b5043d33a0c4948f9e4 | SpotCrude | 2025-02-10 | 806 | 779 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 2c7fe293cc545089e134e08c | US500 | 2025-02-12 | 416 | 382 | 34 | True | 34 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 2fae307746859cdb6463521e | XAUUSD | 2024-07-24 | 832 | 826 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 2fb13c0d341a2a7c4d93d305 | US500 | 2025-02-11 | 403 | 369 | 34 | True | 34 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 301cc40952795f0f5982c6b9 | XAUUSD | 2023-07-27 | 858 | 852 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 310539d714446e4a4c51da87 | XAGUSD | 2025-07-07 | 806 | 0 | 806 | False | 0 | 807 | only_unspent_history_missing |
| 3525cfcc7489cf8cd70fb13a | US500 | 2025-07-23 | 416 | 399 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 355ca1c07adee0d1aeb493fd | SpotCrude | 2023-01-24 | 806 | 753 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 35aec47a684766f6889db8e4 | US500 | 2025-07-25 | 429 | 412 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 35ba9e2c232645825deb093d | SpotCrude | 2025-07-22 | 806 | 797 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 362375ec6c45fb8d7a2b7d3f | SpotBrent | 2025-10-07 | 806 | 546 | 260 | True | 260 | 0 | protected_45day_dependency |
| 39f5596f1015e6d5706ba25b | BTCUSD | 2025-10-06 | 2160 | 1440 | 720 | True | 720 | 0 | protected_45day_dependency |
| 3ddc5b55e115b2e2e1936c1c | SpotBrent | 2025-05-12 | 806 | 780 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 40cc399289d0861550320dd8 | US500 | 2023-01-26 | 429 | 403 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 413acab1546b60736c1ea52d | SpotBrent | 2024-01-24 | 832 | 0 | 832 | False | 833 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4350fd4ccc6e7fa2e964e43c | BTCUSD | 2025-05-13 | 2160 | 2121 | 39 | True | 39 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 43e781706c514672a6a96738 | US500 | 2023-07-27 | 429 | 412 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 43eeb5e76c9e47a95bccd800 | US500 | 2025-05-16 | 429 | 416 | 13 | True | 13 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4688021d87094d090147d2bf | XAGUSD | 2024-01-25 | 858 | 24 | 834 | True | 0 | 834 | only_unspent_history_missing |
| 46b1da2b93e2935f42ee3164 | BTCUSD | 2024-01-24 | 2160 | 1701 | 459 | True | 459 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4818610971895f5197695336 | XAUUSD | 2024-07-22 | 806 | 800 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4a422a139782982730a96c44 | BTCUSD | 2024-07-25 | 2160 | 2104 | 56 | True | 56 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4ad806629abdf9d7d75ac14e | XAUUSD | 2025-02-10 | 806 | 779 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4bf29f97bbe50d6359520302 | XAUUSD | 2023-07-26 | 832 | 826 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4c63892002b69ae4559435bd | BTCUSD | 2024-01-23 | 2160 | 1653 | 507 | True | 507 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4d12cf15cd8e08baa46e7d29 | US500 | 2025-05-12 | 403 | 390 | 13 | True | 13 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4fe286efdb366d24671e8115 | US500 | 2024-01-22 | 403 | 0 | 403 | False | 404 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 4fe7cb2650e4b5c284d9356d | SpotBrent | 2025-02-13 | 858 | 832 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 5204aa8a50ac25f4b1d6d4bb | XAUUSD | 2024-01-25 | 858 | 805 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 5294e3691e86e4143000b304 | SpotCrude | 2025-10-09 | 858 | 598 | 260 | True | 260 | 0 | protected_45day_dependency |
| 550647596835f77c4cbbcfda | SpotBrent | 2024-01-26 | 858 | 0 | 858 | False | 859 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 5aca717f89d879a30446d2ed | SpotCrude | 2025-10-07 | 806 | 546 | 260 | True | 260 | 0 | protected_45day_dependency |
| 5acc5528cd945dc8e3fc1e27 | US500 | 2023-07-24 | 403 | 386 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 5aff841b20cdc59bd1507911 | SpotCrude | 2025-07-25 | 858 | 849 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 5da65ba1cd0c09e8b1e69062 | SpotBrent | 2023-07-27 | 858 | 848 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 5faad815c6cd7484d9d45885 | BTCUSD | 2025-10-07 | 2160 | 1488 | 672 | True | 672 | 0 | protected_45day_dependency |
| 6681fcc669deff45d1f60f88 | US500 | 2024-07-22 | 403 | 386 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 67e0c555d803a502cc15d6dd | US500 | 2025-10-08 | 416 | 286 | 130 | True | 130 | 0 | protected_45day_dependency |
| 68f8dac888fe05b1d594711a | XAUUSD | 2025-07-14 | 806 | 667 | 139 | True | 9 | 130 | needs_spent_gap_or_calendar_reconciliation |
| 6e18297f6eeed480dc9234ba | SpotBrent | 2023-01-24 | 806 | 728 | 78 | True | 78 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 6ebdfc6f362294e0e607bf79 | SpotBrent | 2024-07-23 | 806 | 794 | 12 | True | 12 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 6fcf6013c2d5308631e99a7d | XAUUSD | 2023-01-25 | 832 | 780 | 52 | True | 52 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7048b77cb373429eafea7db7 | NAS100 | 2024-01-24 | 416 | 16 | 400 | True | 0 | 400 | only_unspent_history_missing |
| 70d6f75e60e33e934c60a925 | XAUUSD | 2025-07-21 | 806 | 797 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7106ba337a214ad20426e39f | XAUUSD | 2025-07-07 | 806 | 537 | 269 | True | 9 | 260 | needs_spent_gap_or_calendar_reconciliation |
| 714dcd4ea643c7abb6c67ecd | US500 | 2023-01-24 | 403 | 377 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 71c12956d026bd69a24d715f | NAS100 | 2025-07-07 | 403 | 0 | 403 | False | 0 | 404 | only_unspent_history_missing |
| 736039437cfe8703c32341ae | SpotCrude | 2023-01-27 | 858 | 805 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 743d69735dec5f4d37cf225b | ETHUSD | 2025-07-14 | 2160 | 0 | 2160 | False | 0 | 2161 | only_unspent_history_missing |
| 74608bc5524e70321ed4bed1 | NAS100 | 2024-01-23 | 403 | 8 | 395 | True | 0 | 395 | only_unspent_history_missing |
| 7515dfa76df9ce2dd2f5c809 | US500 | 2023-07-26 | 416 | 399 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 758fe8143b87051535048c5d | XAUUSD | 2025-02-11 | 806 | 779 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7781d2ba777facd710ca1457 | US500 | 2025-05-15 | 429 | 416 | 13 | True | 13 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 78212982822d191a812f095b | XAUUSD | 2024-07-25 | 858 | 852 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7b4c0272869b56392daed4d2 | XAGUSD | 2024-01-24 | 832 | 16 | 816 | True | 0 | 816 | only_unspent_history_missing |
| 7b74fe5bafc42e70f1ae1521 | US500 | 2025-02-14 | 429 | 395 | 34 | True | 34 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7c936f35c2bb48764b26236a | SpotCrude | 2025-10-08 | 832 | 572 | 260 | True | 260 | 0 | protected_45day_dependency |
| 7d29c206fcaad3e409105de3 | SpotBrent | 2023-01-23 | 806 | 728 | 78 | True | 78 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7e2b39cab272c70c3468188b | BTCUSD | 2025-10-09 | 2160 | 1584 | 576 | True | 576 | 0 | protected_45day_dependency |
| 7e4329e54fa5d8930de487f8 | XAUUSD | 2023-07-24 | 806 | 800 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 7f35dcccb70ef17e64457117 | US500 | 2023-01-25 | 416 | 390 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8157378e5e5a500e0590fb57 | SpotCrude | 2025-07-23 | 832 | 823 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8199d9cbcec7df48ffc053bd | BTCUSD | 2024-07-22 | 2160 | 2076 | 84 | True | 84 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 82256abfa13bcc7222b6f93d | SpotCrude | 2023-07-27 | 858 | 852 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 835d49e718917e037ce90b18 | SpotBrent | 2025-07-07 | 806 | 536 | 270 | True | 10 | 260 | needs_spent_gap_or_calendar_reconciliation |
| 83bd140279a8e96c93fb6105 | BTCUSD | 2024-01-26 | 2160 | 1797 | 363 | True | 363 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 83dde1f2c6ae2b80589433bf | SpotCrude | 2024-01-26 | 858 | 0 | 858 | False | 859 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 845b548701e7627dc3f53be3 | US500 | 2024-01-23 | 403 | 0 | 403 | False | 404 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8507537493997a182445c464 | SpotCrude | 2024-01-23 | 806 | 0 | 806 | False | 807 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 86d05af4f69296fddf2b69d9 | SpotCrude | 2024-07-24 | 832 | 824 | 8 | True | 8 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 876f6148e2ef9692e251c46d | US500 | 2023-01-23 | 403 | 377 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8970de420891b9d784ed058b | SpotCrude | 2025-07-14 | 806 | 667 | 139 | True | 9 | 130 | needs_spent_gap_or_calendar_reconciliation |
| 8a9060211292afa3d94ff18b | BTCUSD | 2025-05-16 | 2160 | 2125 | 35 | True | 35 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8c5697c68cd1fc74ace69625 | SpotBrent | 2025-07-24 | 858 | 848 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8cc480b98b6fa8a6e114d91d | XAUUSD | 2024-01-26 | 858 | 805 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8d44e2c3e3ecabd890265eb0 | SpotCrude | 2025-05-15 | 858 | 832 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 8f401fb18724fa4ecb2f8a36 | SpotCrude | 2023-07-24 | 806 | 800 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 9546b2e4fa6800370e6b787f | US500 | 2025-10-06 | 403 | 260 | 143 | True | 143 | 0 | protected_45day_dependency |
| 96de88c9fded8e055ca84165 | US500 | 2025-07-22 | 403 | 386 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 97d0fcd243c70f3d16342c6b | SpotCrude | 2025-07-21 | 806 | 797 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 9856b173f9625f9053df2d6a | US500 | 2024-07-26 | 429 | 412 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 9beae18a5071ebb3f315a848 | SpotBrent | 2023-07-25 | 806 | 796 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 9ce2dd036808138bac5852eb | BTCUSD | 2025-05-15 | 2160 | 2125 | 35 | True | 35 | 0 | needs_spent_gap_or_calendar_reconciliation |
| 9d1d88c1aeae55f5d7850090 | SpotBrent | 2023-01-26 | 858 | 780 | 78 | True | 78 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a21704982a48b223406411f6 | US500 | 2023-07-28 | 429 | 412 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a3281b32cf61af3a41b4fb3a | BTCUSD | 2025-07-07 | 2160 | 0 | 2160 | False | 0 | 2161 | only_unspent_history_missing |
| a3305367b536943b295e9831 | XAUUSD | 2023-01-23 | 806 | 754 | 52 | True | 52 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a3f3c4991fe881e8160596d7 | SpotCrude | 2025-07-07 | 806 | 537 | 269 | True | 9 | 260 | needs_spent_gap_or_calendar_reconciliation |
| a51722a99e186dedf667cb58 | SpotBrent | 2024-01-22 | 806 | 0 | 806 | False | 807 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a5ac3d675d4a755b21393c6f | XAUUSD | 2023-07-25 | 806 | 800 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a6bc4b04896b9807e1b6e9c6 | SpotBrent | 2023-01-27 | 858 | 780 | 78 | True | 78 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a7b42dc8ec6b6738c775ba17 | SpotCrude | 2023-07-25 | 806 | 800 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| a7db221453c42363d5851c6b | XAGUSD | 2024-01-23 | 806 | 8 | 798 | True | 0 | 798 | only_unspent_history_missing |
| aae2661c0700eeb400008256 | BTCUSD | 2025-10-10 | 2160 | 1632 | 528 | True | 528 | 0 | protected_45day_dependency |
| ab6aca1489429fe387e656d0 | SpotCrude | 2024-01-22 | 806 | 0 | 806 | False | 807 | 0 | needs_spent_gap_or_calendar_reconciliation |
| ae497d730dbeacf465872233 | BTCUSD | 2025-07-14 | 2160 | 0 | 2160 | False | 0 | 2161 | only_unspent_history_missing |
| affe109b180bfeb08d170ec7 | SpotBrent | 2025-05-15 | 858 | 832 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| b25d14035008c3a9fb364d7b | XAUUSD | 2023-01-26 | 858 | 806 | 52 | True | 52 | 0 | needs_spent_gap_or_calendar_reconciliation |
| b29542347402bab6234839d6 | SpotCrude | 2025-05-12 | 806 | 780 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| b50a68dfbdb04a30f12e9a60 | SpotCrude | 2023-01-25 | 832 | 779 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| b51bc946e3e8272f970a1779 | XAUUSD | 2025-07-24 | 858 | 849 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| b6074168c8eb0dc0e2f8ae9a | SpotBrent | 2024-07-25 | 858 | 846 | 12 | True | 12 | 0 | needs_spent_gap_or_calendar_reconciliation |
| baa63ee6bb8b431a9d217332 | SpotCrude | 2025-07-24 | 858 | 849 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| bdce29dd8539285849b9e070 | XAUUSD | 2025-07-22 | 806 | 797 | 9 | True | 9 | 0 | needs_spent_gap_or_calendar_reconciliation |
| bf44e6b3bcb58bb2906fe359 | BTCUSD | 2024-01-25 | 2160 | 1749 | 411 | True | 411 | 0 | needs_spent_gap_or_calendar_reconciliation |
| c1bf02ca4edb813e89f28dd7 | US500 | 2025-05-14 | 416 | 403 | 13 | True | 13 | 0 | needs_spent_gap_or_calendar_reconciliation |
| c3e3b322f89aa16d477fe5d7 | SpotCrude | 2023-01-23 | 806 | 753 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| c47e497770890fb64a6335ad | US500 | 2025-10-10 | 429 | 312 | 117 | True | 117 | 0 | protected_45day_dependency |
| c4de64f1ffa0ef8e3de8fc0a | SpotBrent | 2025-07-22 | 806 | 796 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| c544f73b7b6de1593d3c133a | US500 | 2024-01-25 | 429 | 0 | 429 | False | 430 | 0 | needs_spent_gap_or_calendar_reconciliation |
| c8420ab56a4df29d24c8d1b5 | SpotBrent | 2023-07-26 | 832 | 822 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| c905518f0136641e39bad9c5 | SpotBrent | 2025-10-09 | 858 | 598 | 260 | True | 260 | 0 | protected_45day_dependency |
| cf4371537a3ed53252478781 | US500 | 2025-05-13 | 403 | 390 | 13 | True | 13 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d01543c086951e45fc2662f6 | XAUUSD | 2023-01-24 | 806 | 754 | 52 | True | 52 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d1bc6b75ea34848580a14441 | SpotCrude | 2024-01-25 | 858 | 0 | 858 | False | 859 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d2dea2acc6abeda4dd84ac5c | SpotBrent | 2023-07-24 | 806 | 796 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d35cbc2e2f670f3769850370 | XAUUSD | 2024-01-24 | 832 | 779 | 53 | True | 53 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d5f172ae04b389f87a33926f | XAUUSD | 2025-02-14 | 858 | 831 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d7c8c21dd5d0ad94aed42504 | SpotBrent | 2025-02-11 | 806 | 780 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d81b55b30354e68499a01af1 | US500 | 2025-02-13 | 429 | 395 | 34 | True | 34 | 0 | needs_spent_gap_or_calendar_reconciliation |
| d85383ce4fc5cc9e6b2a3788 | SpotBrent | 2023-01-25 | 832 | 754 | 78 | True | 78 | 0 | needs_spent_gap_or_calendar_reconciliation |
| da807a1142c70b322b781e37 | US500 | 2025-10-07 | 403 | 273 | 130 | True | 130 | 0 | protected_45day_dependency |
| daa7c84bfb25caac4549cc96 | US500 | 2023-07-25 | 403 | 386 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| daaa576e731c68edb339d1a8 | BTCUSD | 2025-10-08 | 2160 | 1536 | 624 | True | 624 | 0 | protected_45day_dependency |
| dae3cca41d4e71510881f096 | NAS100 | 2024-01-25 | 429 | 24 | 405 | True | 0 | 405 | only_unspent_history_missing |
| dd362c3ee13db2ddcbd9cf99 | SpotCrude | 2023-07-26 | 832 | 826 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e08a05e0094d27338df76a67 | XAUUSD | 2025-02-12 | 832 | 805 | 27 | True | 27 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e2f8731ee1bbbd8ebec301bb | BTCUSD | 2025-05-12 | 2160 | 2119 | 41 | True | 41 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e37467317fb2c0e7cbde2e15 | US500 | 2025-07-24 | 429 | 412 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e48e81f1c2a54ffeea4d2c00 | SpotCrude | 2024-07-23 | 806 | 798 | 8 | True | 8 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e56cc14a79f4db75fde05223 | US500 | 2024-01-26 | 429 | 0 | 429 | False | 430 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e5c3746472e635be94b42589 | XAUUSD | 2023-01-27 | 858 | 806 | 52 | True | 52 | 0 | needs_spent_gap_or_calendar_reconciliation |
| e876f67aef231b91b3c5c5a4 | SpotBrent | 2025-07-23 | 832 | 822 | 10 | True | 10 | 0 | needs_spent_gap_or_calendar_reconciliation |
| ea65ff19fabbc0109ea7c967 | BTCUSD | 2025-05-14 | 2160 | 2121 | 39 | True | 39 | 0 | needs_spent_gap_or_calendar_reconciliation |
| ed2e5b33c6527114311316f0 | XAGUSD | 2025-07-14 | 806 | 0 | 806 | False | 0 | 807 | only_unspent_history_missing |
| f2ddfe7b0570504c76b992ab | SpotBrent | 2024-01-25 | 858 | 0 | 858 | False | 859 | 0 | needs_spent_gap_or_calendar_reconciliation |
| f31451969f167fdb421b9195 | XAUUSD | 2024-07-26 | 858 | 852 | 6 | True | 6 | 0 | needs_spent_gap_or_calendar_reconciliation |
| f3ec49598c955968c2cdec80 | NAS100 | 2024-01-22 | 403 | 0 | 403 | True | 0 | 403 | only_unspent_history_missing |
| f3fb5435535d98dde263ca12 | US500 | 2025-07-21 | 403 | 386 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |
| f5b1803cf39d6edeb396f06c | SpotBrent | 2025-02-10 | 806 | 780 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| f8992619aab40fdf9fa815cf | US500 | 2023-01-27 | 429 | 403 | 26 | True | 26 | 0 | needs_spent_gap_or_calendar_reconciliation |
| fd538dee2dfee62bbdfef31a | XAGUSD | 2024-01-22 | 806 | 0 | 806 | True | 0 | 806 | only_unspent_history_missing |
| ff90f42b6db284ffa5fee37a | US500 | 2024-07-25 | 429 | 412 | 17 | True | 17 | 0 | needs_spent_gap_or_calendar_reconciliation |

Exact missing M30 runs, source paths, reservations and planned request intervals are retained in the audit JSON. No missing interval is represented as successfully collected.

## §12. Receipt register and instruction for Gemini

| Group / episode | Tick receipt SHA256 | Native output SHA256 |
| --- | --- | --- |
| tier1_7_of_7 / 70e3acf0ef22a39ed2354106 | acff5e90d2efd99da0435944edfa9f5ffd6bedbee21a821ba104a1283af713f8 | 790b5bd0907b94e6ce566be8321a73f9a6eb6f62a3096f4a6831f4ba69470258 |
| tier1_7_of_7 / 77b10f0f75d5383bb011c4b2 | 987cb0e0a519b08c3badd6b1504c7de8653e8b81e2a22009333d679816659a94 | e093a560a27488009a076ea812e88eb95b70be082a536264d59daa851c1c7b1c |
| tier1_7_of_7 / 2b977b08318352c8c79f2cbc | 272715beb972799b23e09991e766e60bfbc31c65a98c91672b2ae9ce94afeecf | d5f215afac1d55748addb384a9af5c59c2cafaa81618d2c4e6aa9247eade2273 |
| tier1_7_of_7 / b8444e851d499207d0449237 | d74c6f509737425855971e69a4645f1c4d53a46c629a8e51bc8de8ff8d3f98a5 | 5df22b5bf26732c04e32203c6eb50f33cc45b01ed30ff62ee2555b4b43ad3efd |
| tier1_7_of_7 / ce5a75c11b08e52a205e5350 | 2098bd4c155b2aa42c393e5f623be9ea20961905142e289f0190072999fa2903 | c58c15c7a8cb169662290e6f2212de49c682994a5e958ac9617e9ffb2ca95974 |
| tier1_7_of_7 / 5b35427f494f9de75bb0ddd1 | 8f14641b7f92c7f54a1cea1e5a2af3b5df9d0dfc360d87bce65e7398cf1fb4a5 | 2b14411bc1e30d85694a4fe127568fa44c65d1b8c6cd5da6655ae8fc2a5922f0 |
| tier1_7_of_7 / 5ff6b8a3b787709ae2dcc9a0 | 08b8d4507c71c9730b0a8ce8f76f31c2f3162ba0eb8aaa01a671efcffb058289 | 14bb6986f58e94cd414f1dcd3cca3cfaaf9262986c64b234cf0beeadb055cd3e |
| tier2_6_of_8 / e2f8731ee1bbbd8ebec301bb | 44d2bc43268ae8f9a3a12275248d04a7109183b50b5f6215da6340643b0104eb | a0ff2699e66d6567c7672ab1df24bc1bf1cb38fbff36337c74fb28fe0982c38b |
| tier2_6_of_8 / daaa576e731c68edb339d1a8 | e8168f91c6fe750c5b0f3635583b9f36c8d8696f093ae8fc1cef643db7b26a36 | b35fd1e9d37fb3fdeedac021f2daa5e0a459684dfdfd9484d326840aa11c22e1 |
| tier2_6_of_8 / 362375ec6c45fb8d7a2b7d3f | dfafce1eed0f81666d908013b34069d819388f4fe2755a21bde7ec5a72c1915f | 176964580708a63fc3e3d9a193282f4a2b43ca58ef89be97209078fc71af5e22 |
| tier2_6_of_8 / 5aca717f89d879a30446d2ed | 3ff78383c62605fcc7ad0ad3de1333ffe373a3a3dc871cb7192daca1a90b83da | 1a034f6cb613369328e7b69cce48b65e3b1440ce6320975d968bb7ea9f5c7264 |
| tier2_6_of_8 / 714dcd4ea643c7abb6c67ecd | a563d9625e25eb37569f730ed9addb02f8ca1452eb05a2c191cf7d276872b6c8 | d3130c30a2111a3552a48a2254da5762604c1ff5c24416ce16f7aded2cd57d08 |
| tier2_6_of_8 / f8992619aab40fdf9fa815cf | 6bdecda65792a4cb1738670d62b09ecc78b835789b9856cc6569ef8ff8ff3da4 | b9633150aee3b7719b640577c6d55339d8cf7cb37a8af87ad29cf36b5dc19631 |
| tier2_6_of_8 / 0c8d1df35ca48a63714fb422 | 7f96f1152cd42979472d511943bc60c7329b92ccb4c73b5be2ce5f72d79537ba | b70e41aa9fca493f83f3b02da2f120a66c71c8dae1bf7c6448d2c145505fb4bb |
| tier2_6_of_8 / 2c7fe293cc545089e134e08c | 23cdf061a359c61d4d322fdc28ee5562d2b8a2ccb90c2a6708d7dd775cc6c279 | 636b6464d3fb318ffae287a761a6fd6ea678a79b61c5cc3f12296b664eeb7705 |
| vetoed_3 / 8d44e2c3e3ecabd890265eb0 | 098f1261d1e6aa8d5c7ee6d965edc3a064021da5c4880344ed483c01a6c9c6cf | bea5e41f526b120f5a3bbb38a4b4ec92f920f3b4983ec3101c64c8091d4ccaa9 |
| vetoed_3 / 40cc399289d0861550320dd8 | b211aa26537b6c1cfbb9f95b990b5cb95585abf69c4738e5eec481fcc5e048df | 2383b190ecced334dc7ccd08a79b202524b7fdbea5b6918de51522e110c2e9ba |
| vetoed_3 / 4d12cf15cd8e08baa46e7d29 | b3ef3ee6e5564c6228f0b8148dcbe33501e4b16bb8fca1408cc41e7432f8148c | 330fea0498b73fc23dc425a23b8b7c053eee44ac729579146b9ebff1f9082a4f |

Read this report, `docs/rebuild_phase4_forensic_audit_and_hypothesis_freeze_18362.json`, `docs/rebuild_phase4_scope_18362.json` and `docs/rebuild_phase4_verification_18362.json` alongside Phase 3. Check the 18 ledger rows, target-versus-clock anatomy, effective versus secondary peer veto, exact upgrade categories, cheap-cost fragility and four-week dependency reservations. Challenge causal/economic claims and execution prerequisites separately from CI success. Do not promote these selected wins or mark planned dates as executed. The original goals, serial/paragraph numbering, SGT timestamps, preserving proven fixes and “Time is Money” remain the invariants.
