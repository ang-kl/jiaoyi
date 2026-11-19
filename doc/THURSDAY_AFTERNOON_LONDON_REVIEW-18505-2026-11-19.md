# Thursday Afternoon Asia, Crypto and London-Open Review — №18,505

**Activity:** [19 November 2026 08:13:00, 19 November 2026 16:24:00) Asia/Singapore. **Read-only production, database and broker audit.**

[Measured JSON and evidence](THURSDAY_AFTERNOON_LONDON_REVIEW-18505-2026-11-19.json) · [Standalone HTML](THURSDAY_AFTERNOON_LONDON_REVIEW-18505-2026-11-19.html)

## §1. Assessment

- **Window closed S3 cash: −$297.00 from 2 closes**: 0 positive-net wins, 2 losses and 0 exact-zero outcomes. **0 S3 positions were open at the activity cutoff.**
- **Thursday 00:00→16:24 S3 cash: −$459.99. Post-PR #214 cumulative closing cash: $2,028.10.** Latest-20 counts are 43002148 3/20, 46130058 6/20, 47790949 6/20, 49236051 12/20. No account meets 15/20 (75%). Financial qualification remains false.
- **4 post-close tracks: 3 complete, 1 partial.** The two carried rows from PR #242 are updated without recounting their closing cash. Initial-stop and breakeven-stop target crossings remain separate.
- The new closes are two account copies of an Asian Silver short, both stopped at their initial SL; neither reached the +1.00R BE activation. They are correlated executions, not two independent market experiments.
- Account money, margin, external cash and S1 ownership are reported separately from S3 in §3. Recorded opening quotes, usable actor references, and human/S1 pair locks are distinguished in §7.
- The captured active production commit is `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`. This audit changes only the three requested documentation artifacts; it sends no order, changes no live setting and performs no Railway deployment.

## §2. Frozen clock, PR #242 anchor and cash reconciliation

The first clock read was **19 November 08:24:33 UTC / 16:24:33 SGT**. The activity cutoff is therefore **16:24:00 SGT**, with a separately frozen **16:24:40 SGT post-close quote horizon**. The activity interval is **8 hours 11 minutes**. Account money, runtime and health snapshots captured afterward carry their own timestamps; they are not retroactive cutoff marks.

The anchor is [№18,502 JSON](THURSDAY_MORNING_24H_REVIEW-18502-2026-11-19.json), **PR #242**, merge `af09d427397c17a56af323442ffb558377783e15`, SHA256 `eacf3ebf6ae94786b60ca6688ede5682583ddd0b0bd88deb2cb1650858e55673`. It ended at **19 November 08:13 SGT**, with Thursday-to-cutoff S3 cash **−$162.99**, post-PR #214 cumulative S3 cash **+$2,325.10**, and **zero open S3 positions**. Its unfinished tracks were **264496289 (US500)** and **264505974 (XAGUSD)**. They are excursion follow-ups only, not new cash in this window.


| Account | Window closes | Window S3 USD | Thu to cutoff S3 USD | Post-PR214 S3 USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | -0.01 | 0 |
| 46130058 | 1 | -142.50 | -239.35 | 895.84 | 0 |
| 47790949 | 1 | -154.50 | -226.51 | 1,099.96 | 0 |
| 49236051 | 0 | 0.00 | 5.87 | 32.31 | 0 |


The exact bridges are **−$162.99 − $297.00 = −$459.99 Thursday-to-cutoff**, and **$2,325.10 − $297.00 = $2,028.10 cumulative**. These are broker closing-cash intervals, not entry-cohort returns attributable solely to a software release. Historical mixed-entry cohort limitations remain disclosed in the cumulative series.

Cash uses unique account/deal IDs, recorded money digits and signed gross/swap/commission/conversion components. Prior follow-up cash is excluded. Open floating P&L, external closes, S1 closes and deposits are not added to S3 realised performance. Retained history coverage and completeness flags remain in JSON; missing evidence is not converted to a verified zero.

## §3. Account snapshots, live cash bridge and strategy ownership


### Account money, inventory and margin

All four accounts were connected and unpaused in the separately captured health snapshot. Primary account money marks were captured around **19 November 16:26:16 SGT**, after the **16:24:00 SGT** activity cutoff. Runtime margin assessments were captured around **16:27 SGT**. They are not a single atomic or retrospective cutoff snapshot.

| Account | Money mark SGT | Balance USD | Net float USD | Equity USD | Current margin % | Pending projected % | Pending stressed % | S3/S1/S2/External | Pending entries |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 2026-11-19T16:26:16.371000+08:00 | -332.90 | 1,923.84 | 1,590.94 | 321.18 | 74.87 | unknown / not assessed | 0/0/0/4 | 1 |
| 46130058 | 2026-11-19T16:26:16.025000+08:00 | 25,184.35 | 231.87 | 25,416.22 | 10,278.79 | unknown / not assessed | unknown / not assessed | 0/6/0/0 | 0 |
| 47790949 | 2026-11-19T16:26:16.308000+08:00 | 25,080.80 | 233.75 | 25,314.55 | 1,595.96 | unknown / not assessed | unknown / not assessed | 0/5/0/0 | 0 |
| 49236051 | 2026-11-19T16:26:16.051000+08:00 | 827.08 | -0.56 | 826.52 | 95,002.30 | unknown / not assessed | unknown / not assessed | 0/1/0/0 | 0 |

Live current assessed margin is **321.18%**, now **above** the 250% new-entry floor, compared with 209.23% at the earlier audit. However, its pending-entry forecast is **74.87%**, below the same floor. Pending stressed margin remains **unknown** because the remaining **NatGas 1.0-lot buy limit #402627798 at 2.532 has no SL**. Improved current equity does not establish unrestricted S3 admission: the pending-book and ownership checks remain separate, and a missing-stop forecast fails closed. This audit does not assert that every live setup reached the final margin guard.

Live has **four external positions, no S1/S2/S3 position, and one pending external entry**. The two JPN225 positions remain 0.4 and 0.6 lots; NVDA.US remains 4.0 lots. The previous Gold long closed and a separate external Gold order filled, as reconciled below. Demo 461 retains six S1 positions (AUDUSD, EURUSD, NZDUSD, USDCAD, GBPUSD, NatGas); demo 477 retains the same five FX positions without NatGas; demo 492 retains its S1 ETHUSD short #262796763. No S1 position closed in this audit window.

| Account | Position | Symbol | Side | Lots | Entry | Current SL | Ownership |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | buy | 0.4 | 57126.25 | 56284.2 | external |
| 43002148 | 649772867 | JPN225 | buy | 0.6 | 57340.1 | 56323.4 | external |
| 43002148 | 657180931 | NVDA.US | buy | 4.0 | 160.825 | 90.82 | external |
| 43002148 | 660682875 | XAUUSD | buy | 0.1 | 3698.47 | 3652.39 | external |
| 46130058 | 261231902 | AUDUSD | sell | 0.09 | 0.68521 | 0.68129 | s1 |
| 46130058 | 261754023 | EURUSD | sell | 0.07 | 1.11005 | 1.11479 | s1 |
| 46130058 | 262578251 | NZDUSD | sell | 0.1 | 0.55271 | 0.56198 | s1 |
| 46130058 | 263178401 | USDCAD | buy | 0.11 | 1.43006 | 1.41672 | s1 |
| 46130058 | 264480035 | GBPUSD | sell | 0.07 | 1.31362 | 1.33387 | s1 |
| 46130058 | 264518049 | NatGas | buy | 0.03 | 3.264 | 2.906 | s1 |
| 47790949 | 261231903 | AUDUSD | sell | 0.09 | 0.68526 | 0.68124 | s1 |
| 47790949 | 261754015 | EURUSD | sell | 0.07 | 1.1101 | 1.11474 | s1 |
| 47790949 | 262976216 | NZDUSD | sell | 0.1 | 0.55202 | 0.56193 | s1 |
| 47790949 | 263178410 | USDCAD | buy | 0.11 | 1.43006 | 1.41677 | s1 |
| 47790949 | 264480037 | GBPUSD | sell | 0.07 | 1.31367 | 1.33382 | s1 |
| 49236051 | 262796763 | ETHUSD | sell | 0.01 | 1746.87 | 2026.76 | s1 |

### Exact live cash bridge and external Gold replacement

**−$177.19 + $0.00 external cashflows − $155.71 external Gold close = −$332.90**. The residual is exactly **$0.00**. The net loss is **−$132.90 gross − $22.11 swap − $0.70 commission**. It is external-account cash, not an S3 loss.

| Position | Deal | Executed SGT | Entry → exit | Native closed volume | Gross USD | Swap USD | Commission USD | Net USD |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 660692549 | 349035009 | 2026-11-19T10:57:19.900000+08:00 | 3723.34 → 3710.05 | 1000 | -132.90 | -22.11 | -0.70 | -155.71 |

Gold **#660692549** closed at **10:57:19.900 SGT** at **3710.05**, against its original recorded entry **3723.34**. A different external position, **#660682875**, opened at **11:08:21.138 SGT**, **0.1 lot at 3698.47**, from order **#402890774**. Its current SL is **3652.39**. The opening changes inventory and unrealized P&L; it is not another realised cash loss.

A dedicated bounded read-only **cashflow-history request** completed at **2026-11-19T16:28:36.959840+08:00** for exactly `[08:13:00,16:24:00) SGT` and returned **zero deposit/withdrawal rows**. Zero cashflows is therefore supported by the account-history response, not merely inferred from a matching cash balance or absent events. No broker market-history request, subscription, or order was made.

### Human protection and source completeness

**Zero live program intents and zero live S3 execution receipts** were recorded in this window. All live positions and the pending NatGas entry have external ownership. Health shows `yield_to_human_position_or_order` on live Gold and the sibling Gold pair reservation on Silver. S3 has no management authority over these positions; the compiled NatGas/JPN225 reservation and label policies were not changed.

The live streaming `trading_events` query returned no rows for this interval, while retained broker deal history independently contains the external Gold close and replacement entry. Consequently, **zero recorded S3 interference** is the supported finding; the specific broker/manual trigger for the Gold close is **not independently established by an execution-event receipt**. A fill near the prior stop alone is insufficient to label it a stop-out, manual action, or bot close. No new JPN225 closing deal occurred in this interval.

| Symbol | Current recorded guard | Observed at UTC |
| --- | --- | --- |
| JPN225 | Blocked: human_reserved_symbol | 2026-11-19T08:27:02.591873253Z |
| NatGas | Blocked: human_reserved_symbol | 2026-11-19T08:26:54.929156170Z |
| XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD | 2026-11-19T08:27:03.100323385Z |
| XAUUSD | Blocked: yield_to_human_position_or_order | 2026-11-19T08:27:03.220558034Z |

### Live per-position floating P&L at its own timestamp

The dedicated account read returned per-position P&L at **2026-11-19T16:26:13.123950+08:00**. Book and balance were unchanged before/after the read. Its net float is **$1,926.23**, producing equity **$1,593.33** from cash **−$332.90**. The primary account table has a slightly later price mark; do not mix the two snapshots.

| External position | Symbol | Lots | Entry | SL | Gross float USD | Net float USD |
| --- | --- | --- | --- | --- | --- | --- |
| 649992251 | JPN225 | 0.4 | 57126.25 | 56284.2 | 693.69 | 673.69 |
| 649772867 | JPN225 | 0.6 | 57340.1 | 56323.4 | 960.21 | 930.23 |
| 657180931 | NVDA.US | 4.0 | 160.825 | 90.82 | -40.90 | -41.54 |
| 660682875 | XAUUSD | 0.1 | 3698.47 | 3652.39 | 364.20 | 363.85 |

One extra zero-P&L broker position ID matches the remaining pending NatGas entry and is excluded from open-position float. No credit or prospective closing commission is invented. The broker P&L proof used five account-data reads plus two authentication frames; the separate cashflow proof used one account-history read plus two authentication frames. **Six focused verification groups pass**, covering the exact cash bridge, unique deals, recorded command isolation, stable P&L/book joins, connected/unpaused accounts, and external live ownership. Exact timestamps, guard receipts, source hashes and read-only request receipts accompany this section in JSON.


## §4. Complete S3 ledger, protection and latest-20 performance

The table covers every S3 position closed, newly opened or still open within the exact activity interval. Current closes comprise **2 initial stops**. Net-cash outcomes are 0 wins, 2 losses and 0 zeroes. Retained account history outside the interval is used only for attribution and cumulative/latest-20 calculations. The original price R is the actual fill-to-initial-stop distance; original USD risk uses filled units and recorded conversion. Net-positive cash defines a net win; a BE-classified exit can be zero or negative after execution.


| Account / position | Symbol / side | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit class | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 264679976 | XAGUSD / sell | 30000 / 0.0600 | 19 Nov 11:26:47.324000 / 44.84900 | 19 Nov 12:11:25.840000 / 45.32400 | 45.32500 / 44.19200 | initial stop | -142.50 / -0.997899 |
| 47790949 / 264679989 | XAGUSD / sell | 30000 / 0.0600 | 19 Nov 11:26:48.758000 / 44.83700 | 19 Nov 13:06:33.674000 / 45.35200 | 45.34800 / 44.13100 | initial stop | -154.50 / -1.007828 |

| Position | Held MFE / MAE R | BE amendment receipt SGT | Exit deadline SGT | Deadline policy |
| --- | --- | --- | --- | --- |
| 264679976 | 0.021008 / 1.010504 | None | 19 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264679989 | 0.000000 / 1.000000 | None | 19 Nov 15:00:00.000000 | native_preclose_v18455_120s |


Held excursions are observed executable-side movements from entry while the position remained open, bounded at the cutoff for open positions. They are not extrapolated across missing quotes. A protective stop can fill at a better price than its stop level; classification follows broker order/protection provenance rather than requiring a fill worse than the stop. Recorded BE amendments and native deadline policies are kept with each position.


| Account | Latest20 wins / closes | Win rate | Rolling Net R | Payoff ratio (R) | Profit factor (R) | Assessment SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | -5.967682 | 3.4277 | 0.6049 | 19 Nov 16:21:12.821244 |
| 46130058 | 6/20 | 30.00% | -2.451545 | 1.5630 | 0.7214 | 19 Nov 16:19:20.869828 |
| 47790949 | 6/20 | 30.00% | -3.082635 | 1.4253 | 0.6578 | 19 Nov 16:21:21.039158 |
| 49236051 | 12/20 | 60.00% | 6.521042 | 1.4542 | 2.1814 | 19 Nov 16:22:34.362216 |


The target is **15/20 (75%)**, alongside the original 1:1.38 planning goal. Rolling net R, payoff and profit factor include realised outcomes and can move differently from the win count as older trades leave the sample. Account copies of one market setup are correlated. Live's legacy rolling window is not a current-policy performance proof; see the independent-verification limitations in JSON. A single audit interval does not qualify a changed entry/exit policy.

## §5. Execution/veto funnel and telemetry integrity


| Metric | Measured count / value |
| --- | --- |
| new_arms | 21 |
| consumed_triggers | 8 |
| terminal_outcomes | 8 |
| pre_intent_refusals | 0 |
| persisted_opening_intents | 8 |
| entries_dispatched | 2 |
| terminal_refusal_receipts | 6 |
| deep_retracement_invalidations | 14 |
| pair_reservation_diagnostics | 11 |
| consumed_triggers_without_individual_outcome | 0 |
| unknown_entry_guard_receipts | 0 |
| confirmation_to_guard_max_ms | 16.303 |

| Terminal refusal reason | Count |
| --- | --- |
| ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction | 6 |


The full consumed-trigger→opening-intent→terminal-outcome joins are retained in JSON. **8 consumed triggers have 8 attributable outcomes: 2 dispatched entries and 6 refusals.** There are **8 persisted opening intents** and **0 pre-intent refusals**. A pre-intent refusal still belongs to its consumed trigger; it is not a missing receipt. Missing individual outcomes are **0**, unknown guard receipts **0**. Confirmation-to-guard timing measures local confirmation through the final guard, not broker send-to-fill or network latency.

The 11 pair-reservation rows are diagnostics, not that many distinct consumed-trigger refusals. Deduplicated idle/pre-arm diagnostics are not an exhaustive hypothetical-trade census. The 08:00 Crypto and 07:00 Asian windows started before the handoff; only receipts timestamped at or after 08:13 enter this activity funnel, avoiding double-counting PR #242.


| Account | Symbol | Gate in separately timed current snapshot |
| --- | --- | --- |
| 43002148 | BTCUSD | Waiting: opening impulse below 48 bps |
| 43002148 | ETHUSD | Waiting: opening impulse below 48 bps |
| 43002148 | NAS100 | Waiting: opening impulse below 48 bps |
| 43002148 | SpotBrent | Waiting: opening impulse below 48 bps |
| 43002148 | SpotCrude | Waiting: opening impulse below 48 bps |
| 43002148 | US500 | Waiting: opening impulse below 48 bps |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Waiting: opening impulse below 48 bps |
| 46130058 | ETHUSD | Waiting: opening impulse below 48 bps |
| 46130058 | NAS100 | Waiting: opening impulse below 48 bps |
| 46130058 | SpotBrent | Waiting: opening impulse below 48 bps |
| 46130058 | SpotCrude | Waiting: opening impulse below 48 bps |
| 46130058 | US500 | Waiting: opening impulse below 48 bps |
| 46130058 | XAGUSD | Vetoed: peer XAUUSD < 24 bps or opposing (-5.28 signed bps); mandatory at dispatch |
| 46130058 | XAUUSD | Waiting: opening impulse below 48 bps |
| 47790949 | BTCUSD | Waiting: opening impulse below 48 bps |
| 47790949 | ETHUSD | Waiting: opening impulse below 48 bps |
| 47790949 | NAS100 | Waiting: opening impulse below 48 bps |
| 47790949 | SpotBrent | Waiting: opening impulse below 48 bps |
| 47790949 | SpotCrude | Waiting: opening impulse below 48 bps |
| 47790949 | US500 | Waiting: opening impulse below 48 bps |
| 47790949 | XAGUSD | Vetoed: peer XAUUSD < 24 bps or opposing (-5.28 signed bps); mandatory at dispatch |
| 47790949 | XAUUSD | Waiting: opening impulse below 48 bps |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Waiting: opening impulse below 48 bps |
| 49236051 | SpotBrent | Waiting: opening impulse below 48 bps |
| 49236051 | SpotCrude | Waiting: opening impulse below 48 bps |
| 49236051 | US500 | Waiting: opening impulse below 48 bps |
| 49236051 | XAGUSD | Vetoed: peer XAUUSD < 24 bps or opposing (-5.28 signed bps); mandatory at dispatch |
| 49236051 | XAUUSD | Waiting: opening impulse below 48 bps |


## §6. Mandatory four-hour post-close excursion audit

All tracks stop at the earlier of exit+4h and **19 November 16:24:40 SGT**. Price/pip/R movement is signed in the original trade direction from the exit fill. Long liquidation uses bid; short liquidation uses ask. Original absolute TP and SL crossings are recorded separately. The explicit broker conventions used are BTC/ETH/Gold 0.1, Brent/Crude/Silver 0.01, and index pip sizes from the retained symbol specification. Each row states its pip size.


| Position / symbol / scope | Pip size | Exit / original SL / TP | Hours / status | Max favourable price / pips / R | Max adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 264679976 / XAGUSD / current | 0.01 | 45.32400 / 45.32500 / 44.19200 | 4.000000 COMPLETE | 0.16000 / 16.00 / 0.336134 | 0.32600 / 32.60 / 0.684874 | -10.30 / -0.216387 | -10.00 / -0.210084 | -0.60 / -0.012605 | False / True |
| 264679989 / XAGUSD / current | 0.01 | 45.35200 / 45.34800 / 44.13100 | 3.301757 PARTIAL | 0.10100 / 10.10 / 0.197652 | 0.29800 / 29.80 / 0.583170 | -7.00 / -0.136986 | -20.80 / -0.407045 | not_elapsed_at_capture | False / True |
| 264496289 / US500 / prior | 1 | 7,318.70000 / 7,359.60000 / 7,250.80000 | 4.000000 COMPLETE | 0.50000 / 0.50 / 0.010941 | 60.10000 / 60.10 / 1.315098 | stale_asof_quote | -35.90 / -0.785558 | -42.40 / -0.927790 | False / True |
| 264505974 / XAGUSD / prior | 0.01 | 45.06700 / 45.88000 / 44.20400 | 4.000000 COMPLETE | 0.02200 / 2.20 / 0.031250 | 0.84100 / 84.10 / 1.194602 | stale_asof_quote | -41.00 / -0.582386 | -51.50 / -0.731534 | False / True |

| Scope | Rows | Complete | Partial | Initial stops later reaching original TP | BE stops later reaching original TP |
| --- | --- | --- | --- | --- | --- |
| Current activity | 2 | 1 | 1 | 0 | 0 |
| Prior follow-ups | 2 | 2 | 0 | 0 | 0 |
| Combined path audit | 4 | 3 | 1 | 0 | 0 |


The two prior US500/Silver 05:57 tracks are followed to their complete 09:57 endpoints without recounting their closing cash. At the 16:24:40 price horizon, a close after 12:24:40 cannot yet have four complete hours. Each row uses its exact elapsed clock. A clock-complete track can still contain the daily maintenance gap. Missing/stale 1h/2h/4h quotes remain unavailable; no interpolation or hypothetical fill is substituted. **Clock-complete does not mean uninterrupted quote coverage.**

A later TP crossing after a stop is path evidence, not realised profit or proof that retaining the original trade would have been safe. Its original SL may already have been crossed. Initial-stop and BE-stop counts are disjoint; zero-cash BE exits are not positive-net wins. The current exit rules remain unchanged.

## §7. Session openings, London DST and Friday readiness


### Recorded native openings versus usable actor references

The four requested openings have **44/44 recorded two-sided quotes within the versioned 15-second observation window**. Recorder coverage does not establish that every slot was admitted or armed. Historical session/assessment receipts are frozen before **16:24 SGT**; the current actor reference and lock snapshot below was captured separately at **16:27:44.553717 SGT**.

| Thursday session | Recorded native quotes | Exact actor quote receipts | Midpoint-only historical corroboration | No matching reference proof | Current ownership/pair locks |
| --- | --- | --- | --- | --- | --- |
| 09:00 Brent | 4/4 | 0 | 4 | 0 | 0 |
| 12:00 Crypto | 8/8 | 0 | 6 | 2 | 2 |
| 15:00 London | 24/24 | 22 | 0 | 2 | 2 |
| 16:00 Crypto | 8/8 | 7 | 0 | 1 | 2 |

The historical **09:00 Brent midpoint is corroborated for all four accounts** by later within-session band assessments. This proves the native opening price used in calculation but does not independently recover the entire actor bid/ask receipt. The six 12:00 Crypto midpoints are similarly corroborated. Demo 49236051's S1 ETH position and ETH/BTC pair reservation explain its current two-slot Crypto lock; the missing historical references are not counted as successful capture.

At **15:00 London**, 22 current actor references match the recorded first quotes exactly. Live Gold and Silver are the two locked slots: external Gold ownership blocks Gold, and the Gold/Silver pair reservation blocks Silver. At **16:00 Crypto**, seven actor references match exactly, but only six slots are free of the current ownership/pair locks: demo 49236051 BTC has a valid captured reference while remaining pair-blocked by S1 ETH; its ETH slot remains ownership-blocked without a matching reference. Reference count and unlocked-slot count therefore differ, and neither is a guarantee of a qualified trade.

Only five of the 29 full quote matches are also present in a persisted session receipt before the activity cutoff; the remaining 24 are corroborated by the later health snapshot of actor state. The current reference uses the original boundary timestamps and prices; no later price is substituted for an opening quote. Exact actor recovery timing is not inferred from the health snapshot.

**Early refusal versus later state:** Live SpotCrude recorded `opening_reference_unavailable` at **15:00:07.575629 SGT**, with causal macro context already OK. The later actor snapshot contains the exact first quote at **15:00:00.227341 SGT**, bid **83.643**, ask **83.671**, midpoint **83.657**, and currently reports an impulse-below-48bp waiting state. Thus the historical refusal is retained, while a permanent current opening-reference failure is not asserted. The precise recovery timestamp is not established by a persisted follow-up receipt. For the earlier Live Silver Asian discrepancy from №18,502, the original refusal remains in retained evidence; current London Silver is independently pair-locked by human Gold. No unobserved recovery of the old Asian reference is claimed.

### Preparation coverage and London late-endpoint limits

| Thursday session | Base prepared | Macro prepared | Entire coefficient table complete | Remaining limitation |
| --- | --- | --- | --- | --- |
| 09:00 Brent | 4/4 | 4/4 | 4/4 | Opening proof detail distinguished above |
| 12:00 Crypto | 8/8 | 8/8 | 8/8 | Ownership/pair locks separate from preparation |
| 15:00 London | 24/24 | 24/24 | 0/24 | 420m and 450m: 13/14 eligible endpoints |
| 16:00 Crypto | 8/8 | 8/8 | 8/8 | Seven current references; six unlocked slots |
| 20:00 Crypto | 8/8 | 8/8 | 8/8 | Future opening quote not yet observable at cutoff |
| 22:30 New York | 24/24 | 24/24 | 24/24 | Future opening quote not yet observable at cutoff |

All preparation counts use the latest qualifying receipt **before 16:24 SGT**. London's four Brent base receipts that were absent at the 08:13 anchor are now present. All **24/24 London base and macro preparations pass**, and early-session coefficients are available. **0/24 complete full tables does not mean London trading is disabled:** the missing marks are specifically **420 and 450 minutes**, affecting the late portion of the session. Every London receipt independently reports `13 same-slot endpoints;14 needed` for those two marks.

The deployed calendar reproduces **13 eligible winter-length weekdays, 2–18 November**. Older summer-regime London sessions ended at 21:30 SGT; their bars cannot supply the later endpoints because the native gate excludes endpoints at or after the old session close. No missing eligible calendar date is identified.

**Friday 20 November 15:00 SGT remains the earliest calendar opportunity for 14/14.** This is conditional on valid Thursday closed bars and fresh Friday preparation. There are **zero Friday preparation receipts at this cutoff**, so full self-healing is not yet verified. No historical bars were requested, fabricated or mutated; production policy and calendar code were not changed.

All 44 quote slots, per-account preparation/lock receipts, current reference timestamps, calendar reconciliation, source hashes, and focused checks are retained in the companion JSON. Queries were bounded read-only transactions; no broker market-data requests were made for this section.


## §8. Assessment and next handoff

The two new Silver shorts recorded only 0.021008R and 0.000000R favourable excursion before their initial stops; neither received a breakeven amendment. Their losses occurred before the baseline +1.00R BE stage. These two account executions share a market path and do not constitute an independent qualification sample.

Execution hygiene, ownership safeguards and strategy performance are separate findings. Complete per-trigger accounting does not demonstrate a predictive edge. External cash and independently owned S1 activity are not S3 returns. The audit makes no entry, exit, risk or deployment change.

The next activity window begins exactly **19 November 16:24:00 SGT**. No open S3 positions carry forward. Continue only the incomplete excursion IDs listed below, preserving original risk and fills. Friday 20 November 15:00 London 14/14 remains conditional until the required Thursday closed bars and fresh Friday preparation receipts exist.


| Incomplete position | Symbol | Current elapsed hours | Four-hour endpoint SGT |
| --- | --- | --- | --- |
| 264679989 | XAGUSD | 3.301757 | 19 Nov 17:06:33.674000 |


## §9. Evidence, verification and Gemini handoff

Sources are retained broker closing deals, labelled S3/S1 orders and fills, account cashflow receipts, current owner book/margin assessments, bounded read-only broker P&L, trigger/intent/outcome joins, saved native opening/preparation receipts, and bounded copies of existing PostgreSQL quotes. Exact source hashes, read-only queries, calculation scripts and tests are included in JSON. No new historical market data was requested; no broker order, live setting, protected source or Railway deployment was changed by this audit.

Only the three requested Markdown, JSON and HTML files under `doc/` are publication changes. Required CI is `check`, `docker`, `engineering`. The PR records the actual merge SHA and checks; post-merge deployment and actor continuity are verified separately by the publishing agent. A green documentation PR is not financial qualification.

**Gemini:** begin with §§1–2 and preserve PR #242's 08:13 handoff. Reconcile window, Thursday and cumulative S3 cash; keep live deposits, human external closes and S1 activity separate. Inspect the full ledger, open-at-cutoff marks, complete prior excursions and new partial tracks. Distinguish recorded boundary quotes from operational opening references and locks. Review source completeness and separately timed money snapshots before proposing any change; do not treat Friday's conditional DST recovery as already verified.


| Tick export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-XAGUSD-2026-11-19-032641.csv.gz | 37666 | b08e4f60885f08a29a031e6234f7536de3f9e0d3ba0d53afe74b71e3d8225c28 |
| 46130058-XAGUSD-2026-11-19-072641.csv.gz | 4978 | f8b09181bd904a5609d45d2837fda944d21fda99e1e0c9db90f4689e8bfeb103 |
| 47790949-XAGUSD-2026-11-18-215655.csv.gz | 25224 | 58c2bf14e8f9e7beb77f93cd4ce44ec5537cd86f2b1be5aac5286d01826480e7 |
| 47790949-XAGUSD-2026-11-19-015655.csv.gz | 13 | e4af6ae71a3846ada06709696800ef43b9b47fb842239948a973ef829238dc19 |
| 47790949-XAGUSD-2026-11-19-032642.csv.gz | 37662 | 86437bcf2cab4e062915318510822e99007a5c708fdc65980c8fd913ddd9c287 |
| 47790949-XAGUSD-2026-11-19-072642.csv.gz | 6936 | 222a368392da1c7d11468b5e4e068066bacf210520bcd8c93ace6e49bd63f187 |
| 49236051-US500-2026-11-18-215655.csv.gz | 20232 | 4eedf2107af9427011f2b08257ea755c222ab18dc4040ee6e69cb0f9821dda74 |
| 49236051-US500-2026-11-19-015655.csv.gz | 15 | 9a78044e05c28ff253c64bd1417152905c4358d7d447f110c12b4e296215f965 |

| Frozen input | SHA256 |
| --- | --- |
| activity.json | c0751ba7ea676ae357cb00ef0e495e843d6edea73fbe59e6afc0a2e19044660e |
| anchor.json | eacf3ebf6ae94786b60ca6688ede5682583ddd0b0bd88deb2cb1650858e55673 |
| capture.json | 7066751328b9a547a714a931dad4bc9bc71f47b778019ecaae7888bc96ac2f72 |
| cumulative.json | 9cbe17ab1761a4384ee3023c4e4610283e0f905a62f3ac4aecdd1ec9545adf64 |
| day.json | 305c29ed4edf5bb510751908b9c349ce36c3bb39c6f1e4e852cac9f3eec261f4 |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| evidence.json | 90adee657c6794df9c6953b67bf232641534ebe54f1a07df6fba1a385a681343 |
| health.json | b649651e51d17638f0eb3366d255094474605351f6bff2d9bc503c43f272ff74 |
| history-proof.json | 4f70d723639a2f4a7f418be46c288e6497340a10ecb96a81eb093cfad9a8946d |
| history-reconciliation.json | fe34590426a2cfeaf34cf85fb9d9ccd764614834f6c4193840a1d4f67fb1aa80 |
| monitor.json | 44b3e626b257c80108ac994bf53a4ba4b5b313f7f4b92294456137854e6e979e |
| preflight.json | a028755728fd76ca4325fb4c9e1a6978d2a0257b064ed0c4382d56dd3c628519 |
| quote-copy-receipts.json | cdd703223909b1a573f2ff2d8beb6d0967eb015cccfec04e585c49f7db6a5a5a |
| runtime.json | c77d5d286339d0fb79da2dbe5d057d4379501243e9f1f5b224375e6ba90811df |
