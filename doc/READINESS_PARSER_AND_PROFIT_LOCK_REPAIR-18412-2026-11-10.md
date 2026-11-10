# TLDR — Repairs verified; earlier profit lock does not qualify for combined deployment

№18,412 §§1–12 · 2026-11-10T08:13:06+08:00 SGT · by <Codex (GPT-6), effort: not exposed> · PR214 · Base main 11d5137.

The readiness, dispatch-parser and atomic assessment repairs are implemented. The candidate improves the selected overnight paths but does not repair the preceding daytime losses, and its historical results must pass the same frozen comparison. This report does not claim that the five release checks passed. Production remains unchanged.

Historical cost-adjusted regressions: Tier1 at48bp impulse /3.74bp admission falls from1.2046R to0.5950R per trade; at24bp stress it flips from+0.4196R to−0.1913R. Tier2 at28bp /3.74bp admission falls from0.2800R to0.0846R. The candidate is rejected for deployment under these tested assumptions.

Goal: 73% net wins over the latest 20 closed trades per strategy/account, with the 1:1.38 planned reward/risk target. This run contributes zero new qualifying live trades. It does not demonstrate progress in realised win rate or consistent profitability.

## §1. Exact scope, frozen rules and source discipline

The plan was written and SHA256-recorded before collection of recorded overnight PostgreSQL quotes. No new broker history/tick calls were made. The experiment holds each entry, direction, volume, original risk and target fixed. It changes only the exit thresholds: +0.65R proposes an entry-price stop; +0.80R proposes a +0.50R stop; the existing gross target remains +1.38R. The frozen baseline moves to entry only after +1R.

Every amendment takes effect only on a strictly later quote if the proposed stop remains valid. Existing server stop/target checks occur first; a gap fills at the recorded executable price. The half-R stop rounds towards entry on the price grid. This is an optimistic next-quote acknowledgement assumption, not measured broker amendment latency or a guaranteed profit. No candidate exit rule is wired into live management.

Observed cohort: 15 daytime trades from №18,406 and six overnight trades ending at the approved 10 Nov 06:49 SGT report cutoff. Each is three demo accounts copying a setup: five daytime and two overnight signal episodes, not 21 independent trials. The live account has no S3 trade in these scoped cohorts.

Historical development: Phase3 35 Tier1 and 167 Tier2 sessions, retaining each original 28/48bp threshold and 2/3.74/24bp admission variant. These variants overlap and cannot be summed as independent trades. Stage1 has 40 scheduled episodes: 24 evaluated, six quote-source/coverage exclusions and ten preparation-blocked crypto episodes. The protected 19 Aug–5 Sep 2025 holdout stays unopened.

## §2. Engineering repairs and verification

| Repair | Change | Acceptance evidence |
| --- | --- | --- |
| P1 readiness | Verified cached preparation bypasses the unnecessary current-M30 request. Otherwise only closed, causal preparation bars are read. Original first-six-second two-sided quotes remain mandatory. | PostgreSQL/fake-broker restart test: all eight symbols ready at open+5s, no forming M30 bars and no repeated history requests. Missing quote and budget guards pass. |
| P1 dispatch evidence | Explicit registry for recorded touch, bounded-touch and confirmed-turn versions; matching schema, side and timestamps required. Touch latency and confirmation latency are separate. | 15/15 historical 18392 and 6/6 current 18409 guards accepted; zero invalid. Unknown versions, mismatched proof, reversed time, wrong side and missing proof rejected. |
| P2 assessment | One read-only repeatable-read transaction covers outcomes, settings and history; database-clock cutoff established with the snapshot. | Concurrent history/settings/fill changes remain invisible until the next assessment; stale, failed, future or missing evidence still yields unknown. Existing 15 goal integration tests pass. |

The old missing-opening test removed only one fixture stream, which could collide with the wall clock at a session boundary. It now removes every quote in the opening interval. The holiday test now represents an absent opening quote rather than treating an unfinished M30 candle as proof of a holiday. Trading calendar, ownership, margin, risk, pair reservation and entry rules are unchanged.

## §3. The proposed structural explanation versus actual fills

| Account | Symbol | Fill | Opening midpoint | Recorded impulse extreme | Original R | R to old extreme |
| --- | --- | --- | --- | --- | --- | --- |
| 46130058 | ETHUSD | 2131.99 | 2144.56 | 2124.6 | 17.27 | 0.4279 |
| 49236051 | ETHUSD | 2132.02 | 2144.56 | 2124.6 | 17.27 | 0.4296 |
| 47790949 | ETHUSD | 2132.02 | 2144.56 | 2124.6 | 17.27 | 0.4296 |
| 47790949 | SpotCrude | 86.359 | 87.3015 | 85.81200000000001 | 1.017 | 0.5379 |
| 46130058 | SpotCrude | 86.256 | 87.3015 | 85.81200000000001 | 1.002 | 0.4431 |
| 49236051 | SpotCrude | 86.328 | 87.3015 | 85.81200000000001 | 1.002 | 0.5150 |

For an exact 50% retracement and R=half-impulse+cost reserve, the old extreme is reached at half-impulse/(half-impulse+cost reserve) R. That fraction is not a universal 0.80R or 0.92R. The actual confirmed-turn executable fills above imply only 0.428–0.538R to the saved extreme. The market subsequently traded beyond those saved extremes. The ETH best recorded ask was 2114.88; oil 85.451, not the directive’s 85.80. A fixed 0.65/0.80R rule is therefore a separate empirical exit hypothesis, not a correction mathematically compelled by the original impulse geometry.

## §4. Actual cash and paired quote-replay cash

| Cohort | Actual broker cash USD | Baseline replay USD | Candidate replay USD | Replay improvement USD |
| --- | --- | --- | --- | --- |
| daytime_15 | -788.68 | -795.38 | -795.38 | 0.0 |
| overnight_6 | -206.95 | -206.31 | 273.46 | 479.77 |

Cash scenarios use the recorded volume and price scale, plus the recorded zero commission/swap/conversion charges for these particular closed deals. They include quoted spread through executable entry/exit prices, but no additional hypothetical slippage reserve. Actual filled cash is reconciled to each broker deal. The reported replay improvement compares replay with replay; it is not money earned in the accounts.

| Cohort / additional bps per side | Baseline | Candidate |
| --- | --- | --- |
| daytime_15 / 0.0 | 3W/12L/0flat; mean -0.5360R; PF 0.3400 (defined); DD 12.1821R | 3W/12L/0flat; mean -0.5360R; PF 0.3400 (defined); DD 12.1821R |
| daytime_15 / 3.74 | 3W/12L/0flat; mean -0.7097R; PF 0.2592 (defined); DD 14.3689R | 3W/12L/0flat; mean -0.7097R; PF 0.2592 (defined); DD 14.3689R |
| daytime_15 / 24.0 | 3W/12L/0flat; mean -1.6504R; PF 0.0556 (defined); DD 26.2148R | 3W/12L/0flat; mean -1.6504R; PF 0.0556 (defined); DD 26.2148R |
| overnight_6 / 0.0 | 3W/3L/0flat; mean -0.3488R; PF 0.3043 (defined); DD 3.0080R | 6W/0L/0flat; mean 0.4535R; PF Not measurable (no_losses); DD 0.0000R |
| overnight_6 / 3.74 | 3W/3L/0flat; mean -0.4271R; PF 0.1995 (defined); DD 3.2015R | 6W/0L/0flat; mean 0.3754R; PF Not measurable (no_losses); DD 0.0000R |
| overnight_6 / 24.0 | 0W/6L/0flat; mean -0.8516R; PF 0.0000 (defined); DD 5.1096R | 3W/3L/0flat; mean -0.0474R; PF 0.3688 (defined); DD 0.4508R |

The 24bp stress reserve can turn an entry-price breakeven or positive gross stop into a net loss. Costs are applied as (entry+exit)×bps/10000/originalR; spread is not charged twice. These sensitivities are assumptions, not a calibrated claim about future round-trip costs. Maximum drawdown here is from the chronological closed-trade R curve, not floating portfolio drawdown; account-specific summaries remain in JSON.

## §5. All21 observed positions — actual versus estimated exits

| Cohort | Account | Symbol | Position | Actual USD | Base est.USD | New est.USD | Base grossR | New grossR | New exit | UTC exit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| daytime_15 | 46130058 | ETHUSD | 260303252 | 200.47 | 198.32 | 198.32 | 1.3807 | 1.3807 | broker_take_profit | 2026-11-09T06:31:52.717972Z |
| daytime_15 | 47790949 | ETHUSD | 260303253 | 205.14 | 204.62 | 204.62 | 1.3807 | 1.3807 | broker_take_profit | 2026-11-09T06:31:52.642739Z |
| daytime_15 | 49236051 | ETHUSD | 260303251 | 6.76 | 6.61 | 6.61 | 1.3807 | 1.3807 | broker_take_profit | 2026-11-09T06:31:52.717658Z |
| daytime_15 | 49236051 | SpotCrude | 260368215 | -4.84 | -4.92 | -4.92 | -1.0302 | -1.0302 | gap_through_stop | 2026-11-09T07:58:20.610728Z |
| daytime_15 | 47790949 | SpotCrude | 260368223 | -151.22 | -151.22 | -151.22 | -1.0134 | -1.0134 | gap_through_stop | 2026-11-09T07:58:20.095364Z |
| daytime_15 | 46130058 | SpotCrude | 260368257 | -154.45 | -157.18 | -157.18 | -1.0777 | -1.0777 | gap_through_stop | 2026-11-09T07:58:20.500623Z |
| daytime_15 | 49236051 | SpotBrent | 260368312 | -4.96 | -4.96 | -4.96 | -1.0122 | -1.0122 | gap_through_stop | 2026-11-09T08:02:03.852872Z |
| daytime_15 | 46130058 | SpotBrent | 260368313 | -144.34 | -147.61 | -147.61 | -1.0143 | -1.0143 | gap_through_stop | 2026-11-09T08:02:04.680472Z |
| daytime_15 | 47790949 | SpotBrent | 260368311 | -150.78 | -150.78 | -150.78 | -1.0122 | -1.0122 | gap_through_stop | 2026-11-09T08:02:03.865383Z |
| daytime_15 | 49236051 | US500 | 260383619 | -2.77 | -2.77 | -2.77 | -1.0000 | -1.0000 | stop | 2026-11-09T08:43:24.081491Z |
| daytime_15 | 47790949 | US500 | 260383621 | -146.81 | -146.81 | -146.81 | -1.0036 | -1.0036 | gap_through_stop | 2026-11-09T08:43:24.082379Z |
| daytime_15 | 46130058 | US500 | 260383624 | -145.08 | -144.04 | -144.04 | -1.0036 | -1.0036 | gap_through_stop | 2026-11-09T08:43:23.880733Z |
| daytime_15 | 47790949 | NAS100 | 260389633 | -142.8 | -142.24 | -142.24 | -1.0049 | -1.0049 | gap_through_stop | 2026-11-09T08:43:21.339548Z |
| daytime_15 | 49236051 | NAS100 | 260389632 | -10.2 | -10.16 | -10.16 | -1.0049 | -1.0049 | gap_through_stop | 2026-11-09T08:43:21.334873Z |
| daytime_15 | 46130058 | NAS100 | 260389631 | -142.8 | -142.24 | -142.24 | -1.0049 | -1.0049 | gap_through_stop | 2026-11-09T08:43:21.340196Z |
| overnight_6 | 46130058 | ETHUSD | 260655209 | 44.97 | 44.89 | 64.98 | 0.3040 | 0.4401 | gap_through_stop | 2026-11-09T17:47:59.718709Z |
| overnight_6 | 49236051 | ETHUSD | 260655210 | 1.43 | 1.43 | 2.06 | 0.3057 | 0.4418 | gap_through_stop | 2026-11-09T17:47:59.689630Z |
| overnight_6 | 47790949 | ETHUSD | 260655217 | 44.65 | 44.56 | 64.4 | 0.3057 | 0.4418 | gap_through_stop | 2026-11-09T17:47:59.689478Z |
| overnight_6 | 47790949 | SpotCrude | 260741238 | -143.72 | -145.43 | 67.35 | -1.0000 | 0.4631 | gap_through_stop | 2026-11-09T19:42:36.315557Z |
| overnight_6 | 46130058 | SpotCrude | 260741239 | -150.38 | -147.73 | 72.91 | -1.0030 | 0.4950 | gap_through_stop | 2026-11-09T19:42:34.824487Z |
| overnight_6 | 49236051 | SpotCrude | 260741495 | -3.9 | -4.03 | 1.76 | -1.0050 | 0.4391 | gap_through_stop | 2026-11-09T19:42:36.212245Z |

## §6. Reconciliation, baseline parity and data limits

Broker entry/close identity, quantity, fill prices and cash reconcile for21/21 positions. Exact quote-replay fill price plus timestamp matches 0/21. Broker execution and recorded quote streams are different observations: queueing, server-side stops, spread, rounding and slippage prevent exact equality. The runner reports price/time residuals rather than forcing its baseline to equal broker cash.

| Account | Symbol | Position | Baseline−broker price | Baseline−broker time ms | Recorded intervals>6s |
| --- | --- | --- | --- | --- | --- |
| 46130058 | ETHUSD | 260303252 | 0.1700 | -29.03 | 1 |
| 47790949 | ETHUSD | 260303253 | 0.0400 | -94.26 | 1 |
| 49236051 | ETHUSD | 260303251 | 0.3500 | -81.34 | 1 |
| 49236051 | SpotCrude | 260368215 | 0.0070 | -46.27 | 25 |
| 47790949 | SpotCrude | 260368223 | 0.0000 | -102.64 | 25 |
| 46130058 | SpotCrude | 260368257 | 0.0070 | -97.38 | 25 |
| 49236051 | SpotBrent | 260368312 | 0.0000 | -146.13 | 69 |
| 46130058 | SpotBrent | 260368313 | 0.0110 | -70.53 | 70 |
| 47790949 | SpotBrent | 260368311 | 0.0000 | -89.62 | 70 |
| 49236051 | US500 | 260383619 | 0.0000 | -110.51 | 0 |
| 47790949 | US500 | 260383621 | 0.0000 | -163.62 | 0 |
| 46130058 | US500 | 260383624 | -0.2000 | -122.27 | 0 |
| 47790949 | NAS100 | 260389633 | -0.4000 | -133.45 | 0 |
| 49236051 | NAS100 | 260389632 | -0.4000 | -128.13 | 0 |
| 46130058 | NAS100 | 260389631 | -0.4000 | -83.80 | 0 |
| 46130058 | ETHUSD | 260655209 | 0.0100 | -952.97 | 2 |
| 49236051 | ETHUSD | 260655210 | 0.0100 | -906.65 | 2 |
| 47790949 | ETHUSD | 260655217 | 0.0100 | -881.02 | 2 |
| 47790949 | SpotCrude | 260741238 | 0.0120 | -96.61 | 107 |
| 46130058 | SpotCrude | 260741239 | -0.0180 | -103.97 | 106 |
| 49236051 | SpotCrude | 260741495 | 0.0320 | -152.64 | 108 |

The unchanged historical native baseline reproduced 1212/1212 Phase3 variant outputs exactly across202 distinct sessions; the exit adapter reproduces every admitted ArmC strict and forced outcome exactly before applying the candidate. Stage1 reproduces 24/24 outputs and still has zero candidate entries. This is computational parity, not proof of continuous executable ticks. Quiet intervals and stale-side omissions remain disclosed; no missing price path is invented.

## §7. Historical comparison — keep tiers and entry admission distinct

| Tier / threshold / admission / resolution | Additional bps/side | Baseline | Candidate |
| --- | --- | --- | --- |
| tier1/floor=28/admission=24/forced | 3.74 | 2W/0L/0flat; mean 1.0284R; PF Not measurable (no_losses); DD 0.0000R | 2W/0L/0flat; mean 1.0284R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=28/admission=24/forced | 24.0 | 2W/0L/0flat; mean 0.6533R; PF Not measurable (no_losses); DD 0.0000R | 2W/0L/0flat; mean 0.6533R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=28/admission=24/strict | 3.74 | 1W/0L/0flat; mean 1.3206R; PF Not measurable (no_losses); DD 0.0000R | 1W/0L/0flat; mean 1.3206R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=28/admission=24/strict | 24.0 | 1W/0L/0flat; mean 0.9950R; PF Not measurable (no_losses); DD 0.0000R | 1W/0L/0flat; mean 0.9950R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=28/admission=2/forced | 3.74 | 6W/3L/0flat; mean 0.2764R; PF 1.6352 (defined); DD 2.6018R | 6W/3L/0flat; mean -0.0288R; PF 0.9339 (defined); DD 2.6018R |
| tier1/floor=28/admission=2/forced | 24.0 | 4W/5L/0flat; mean -0.7004R; PF 0.2889 (defined); DD 6.7472R | 1W/8L/0flat; mean -1.0061R; PF 0.0702 (defined); DD 9.0553R |
| tier1/floor=28/admission=2/strict | 3.74 | 5W/3L/0flat; mean 0.2891R; PF 1.5905 (defined); DD 2.6018R | 5W/3L/0flat; mean -0.0543R; PF 0.8892 (defined); DD 2.6018R |
| tier1/floor=28/admission=2/strict | 24.0 | 4W/4L/0flat; mean -0.7534R; PF 0.2982 (defined); DD 6.4714R | 1W/7L/0flat; mean -1.0974R; PF 0.0722 (defined); DD 8.7795R |
| tier1/floor=28/admission=3.74/forced | 3.74 | 6W/3L/0flat; mean 0.4008R; PF 2.3006 (defined); DD 2.5708R | 7W/2L/0flat; mean 0.1576R; PF 1.5519 (defined); DD 2.5708R |
| tier1/floor=28/admission=3.74/forced | 24.0 | 4W/5L/0flat; mean -0.4860R; PF 0.3829 (defined); DD 5.5629R | 1W/8L/0flat; mean -0.7297R; PF 0.0984 (defined); DD 6.5671R |
| tier1/floor=28/admission=3.74/strict | 3.74 | 5W/3L/0flat; mean 0.4299R; PF 2.2399 (defined); DD 2.5708R | 6W/2L/0flat; mean 0.1563R; PF 1.4864 (defined); DD 2.5708R |
| tier1/floor=28/admission=3.74/strict | 24.0 | 4W/4L/0flat; mean -0.5136R; PF 0.3978 (defined); DD 5.2976R | 1W/7L/0flat; mean -0.7877R; PF 0.1021 (defined); DD 6.3018R |
| tier1/floor=48/admission=24/forced | 3.74 | 2W/0L/0flat; mean 1.0284R; PF Not measurable (no_losses); DD 0.0000R | 2W/0L/0flat; mean 1.0284R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=24/forced | 24.0 | 2W/0L/0flat; mean 0.6533R; PF Not measurable (no_losses); DD 0.0000R | 2W/0L/0flat; mean 0.6533R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=24/strict | 3.74 | 1W/0L/0flat; mean 1.3206R; PF Not measurable (no_losses); DD 0.0000R | 1W/0L/0flat; mean 1.3206R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=24/strict | 24.0 | 1W/0L/0flat; mean 0.9950R; PF Not measurable (no_losses); DD 0.0000R | 1W/0L/0flat; mean 0.9950R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=2/forced | 3.74 | 7W/0L/0flat; mean 1.2058R; PF Not measurable (no_losses); DD 0.0000R | 7W/0L/0flat; mean 0.5833R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=2/forced | 24.0 | 7W/0L/0flat; mean 0.3613R; PF Not measurable (no_losses); DD 0.0000R | 2W/5L/0flat; mean -0.2624R; PF 0.1568 (defined); DD 1.8367R |
| tier1/floor=48/admission=2/strict | 3.74 | 6W/0L/0flat; mean 1.2245R; PF Not measurable (no_losses); DD 0.0000R | 7W/0L/0flat; mean 0.5833R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=2/strict | 24.0 | 6W/0L/0flat; mean 0.3702R; PF Not measurable (no_losses); DD 0.0000R | 2W/5L/0flat; mean -0.2624R; PF 0.1568 (defined); DD 1.8367R |
| tier1/floor=48/admission=3.74/forced | 3.74 | 7W/0L/0flat; mean 1.2046R; PF Not measurable (no_losses); DD 0.0000R | 7W/0L/0flat; mean 0.5950R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=3.74/forced | 24.0 | 7W/0L/0flat; mean 0.4196R; PF Not measurable (no_losses); DD 0.0000R | 2W/5L/0flat; mean -0.1913R; PF 0.2544 (defined); DD 1.5757R |
| tier1/floor=48/admission=3.74/strict | 3.74 | 6W/0L/0flat; mean 1.2346R; PF Not measurable (no_losses); DD 0.0000R | 7W/0L/0flat; mean 0.5950R; PF Not measurable (no_losses); DD 0.0000R |
| tier1/floor=48/admission=3.74/strict | 24.0 | 6W/0L/0flat; mean 0.4414R; PF Not measurable (no_losses); DD 0.0000R | 2W/5L/0flat; mean -0.1913R; PF 0.2544 (defined); DD 1.5757R |
| tier2/floor=28/admission=2/forced | 3.74 | 6W/2L/0flat; mean 0.2651R; PF 2.0583 (defined); DD 1.3076R | 6W/2L/0flat; mean 0.1224R; PF 1.4884 (defined); DD 1.3076R |
| tier2/floor=28/admission=2/forced | 24.0 | 0W/8L/0flat; mean -1.1001R; PF 0.0000 (defined); DD 8.8009R | 0W/8L/0flat; mean -1.2426R; PF 0.0000 (defined); DD 9.9405R |
| tier2/floor=28/admission=2/strict | 3.74 | 3W/1L/0flat; mean 0.5228R; PF 2.5993 (defined); DD 1.3076R | 4W/1L/0flat; mean 0.2820R; PF 2.0783 (defined); DD 1.3076R |
| tier2/floor=28/admission=2/strict | 24.0 | 0W/4L/0flat; mean -0.8463R; PF 0.0000 (defined); DD 3.3851R | 0W/5L/0flat; mean -1.1317R; PF 0.0000 (defined); DD 5.6584R |
| tier2/floor=28/admission=3.74/forced | 3.74 | 6W/2L/0flat; mean 0.2800R; PF 2.1861 (defined); DD 1.2797R | 5W/3L/0flat; mean 0.0846R; PF 1.3135 (defined); DD 1.2797R |
| tier2/floor=28/admission=3.74/forced | 24.0 | 0W/8L/0flat; mean -0.9406R; PF 0.0000 (defined); DD 7.5250R | 0W/8L/0flat; mean -1.1356R; PF 0.0000 (defined); DD 9.0852R |
| tier2/floor=28/admission=3.74/strict | 3.74 | 3W/1L/0flat; mean 0.5501R; PF 2.7196 (defined); DD 1.2797R | 3W/2L/0flat; mean 0.2091R; PF 1.6750 (defined); DD 1.2797R |
| tier2/floor=28/admission=3.74/strict | 24.0 | 0W/4L/0flat; mean -0.6753R; PF 0.0000 (defined); DD 2.7012R | 0W/5L/0flat; mean -1.0534R; PF 0.0000 (defined); DD 5.2669R |

Only cells containing an admitted ArmC trade appear above; all no-entry denominators remain in the1212 baseline parity records. Strict rows retain unresolved trades separately; forced rows close on the first eligible boundary quote. Repeated variants and strict/forced outcomes must not be pooled. Stage1 provides no exit-policy validation because it has no ArmC entries. There is no new independent unseen result in this run.

## §8. Failure prevention and five release checks

| Check | Status | Evidence |
| --- | --- | --- |
| 1 | passed_in_tests | all eight symbols restore causal preparation/original quotes and prepare at open+5s; absent opening quotes still refuse; runtime deployment not yet tested |
| 2 | passed_in_tests | 15/15 older and 6/6 current guards accepted; invalid=0. Concurrent writes remain invisible within one snapshot, visible on the next; real missing history remains unknown |
| 3 | partial_exact_fill_parity_not_met | 21/21 broker cash/quantity/price receipts reconcile; 0/21 exact quote-replay fill/time matches. Historical native outputs match 1212 variants and 24 Stage1 episodes. |
| 4 | failed_historical_regressions | No pooling across tiers, cost admissions or policy eras. New independent unseen qualification is absent. |
| 5 | release_held_CI_tracked_on_PR214 | PR214 draft; owner conditioned combined release on all five checks. No production change or verification trade. |

The combined change is held under the owner’s explicit “if and only if all five checks pass” condition. A green compiler or CI cannot convert a failed financial comparison into a pass. The tested engineering repairs can be reviewed separately from the rejected/unqualified exit experiment; that separate release decision is not implied by this report.

## §9. Recommended next action

Keep the current live exit rule while reviewing an engineering-only release of the readiness/parser/assessment repairs. Retain this frozen result, including any losing cells. Do not optimise0.65/0.80/0.50 repeatedly on these same dates. An additional exit hypothesis would need an economic justification, fixed acceptance criteria, executable amendment constraints and genuinely separate evaluation dates before a live-policy decision.

The daytime losses were not improved by earlier profit protection: those losers did not reach the proposed protective thresholds before stopping. Entry quality and context remain unresolved. Tightening protection alone does not create a predictive edge or establish the account-level73% goal. The latest 20 window is assessed per account; these retrospective quotes are not appended as live wins.

## §10. Reproduction and audit trail

Plan: docs/readiness_profit_lock_plan_18412.json. Native adapter: crates/jiaoyi-core/examples/profit_lock_replay.rs. Runner: ops/research/profit_lock_18412.py. Run PYTHONPATH=ops/research python3 ops/research/profit_lock_18412.py with the verified private caches present. JSON contains source, input/output and binary SHA256 values, all 21position comparisons, every historical admitted exit and1212 baseline parity receipts. No tokens or raw market cache are published.

Source code commit and final CI/release status are recorded on PR214. Reports are versioned in doc/; CODEX_Log adds a new append-only entry. The requested§LOG152 already exists, so §LOG176 is used. GROK_Log latest №17,703 was checked before choosing the serial.

## §11. Invariants

| Invariant | Result |
| --- | --- |
| Original73% /1:1.38/latest20per strategy/account | Preserved; not achieved |
| Traceability | №18,412,paragraphs,SGTtimestamp,modelbyline |
| Humanownership/reservedNatGas,JPN225/risk/margin/pairlimits | Production rules unchanged |
| Realdata,explicitcosts,zerohard-codedreturns | Passed; all outcomes computed |
| Brokercollection/protectedvalidation/Telegram | 0new broker requests;holdoutclosed;nomessages |
| Timeismoney | Threeevidencedrepairs;onefrozenexitcandidate;notuningloop |

## §12. Files for independent review

GitHub PR: https://github.com/ang-kl/jiaoyi/pull/214. Read the Markdown first, then inspect the JSON cells and receipts. HTML is a standalone readable copy. These are draft-branch deliverables until the release decision is resolved; do not describe them as merged or deployed.

