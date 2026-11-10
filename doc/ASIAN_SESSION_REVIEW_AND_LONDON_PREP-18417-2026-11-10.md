# Asian session review and London preparation — №18,417

№18,417 §§1–10 · Prepared 2026-11-10T14:39:49+08:00 · by <Codex (GPT-6), effort: not exposed>. Activity window: 10 November 2026, 09:42 inclusive to 14:15 exclusive SGT. Production: ef04699334c423e5b1794101189977832d4cb1bf.

## TL;DR

Jiaoyi was trading. Three demo ETHUSD shorts closed for a combined +$412.88; three new demo SpotCrude shorts filled and remained open in the 14:17 snapshot. Four other S3 entry intents were refused before send by fresh peer checks. All four account actors were connected and unpaused. Live account 43002148 had zero bot opening intents in the requested interval and five human/external positions in the later book.

The goal is still unmet: latest-20 S3 wins are 3, 4, 4 and 5 across live 43002148 and demos 46130058, 47790949, 49236051. Two demos improved by 5 percentage points from №18,414; the other two did not improve. The three ETH wins are copies of one signal, not three independent demonstrations of an edge.

London opens at 15:00 SGT, not 15:30. All 24 traditional account-symbol combinations have base preparation, but all lack the 420- and 450-minute volatility coefficients. Early-session marks are available; new unarmed impulses after 21:30 can be blocked by those missing coefficients. The eight crypto combinations have full preparation for their separate 16:00 window. Future opening prices cannot be certified before those opens.

No recorded bot interference with the live human book was found. A complete claim that every human SL/TP stayed unchanged is not verifiable from these records. The live book’s price-only stop exposure exceeds the observed equity estimate; existing margin guards are therefore material. This audit makes no trading, configuration, deployment or human-order change.

## §1. Scope, source times and running release

| Evidence | Time / scope | What is established |
| --- | --- | --- |
| Trading activity | 09:42–14:15 SGT; [01:42,06:15) UTC | Persisted decisions, intents, fills, broker execution events and closing deal cash. |
| Health / owner book | Health request 14:17:34–14:17:42; books around 14:16:42; owner assessment 14:17:34 | Later operational snapshot; not an exact broker book at 14:15. |
| Account marks | 14:19:44–14:19:45 SGT | Balance, float and equity shown separately from interval realised cash. |
| Retrospective deal history | Four successful refreshes through 14:21:42–43 SGT | Each fetched interval covers the audit cutoff; outcomes after 14:15 are excluded. |
| Engineering worker | 14:27:22 SGT | Heartbeat, budget, queue and monitor lifecycle; no worker setting changed. |

Both services report successful active deployments bound to ef046993. Trading deployment: 8dcc60be-052d-481f-a967-98c8ca936715; engineering deployment: c203668c-36a9-476f-b00f-5e317dc65776. The actual trading policy is s3_macro_pullback_owner_18409_v1. The engineering release in PR #214 did not rename that policy or deploy the rejected early profit lock.

Baseline exits remain entry-price breakeven after +1R, a +1.38R gross target and the recorded native-session deadline. Crypto windows are four hours; traditional windows follow the production calendar. “Four hours after every entry” would misdescribe the current production exits.

| Account | Connected / owner paused | Exit checks since actor start | Most recent completed exit check, SGT |
| --- | --- | --- | --- |
| 43002148 | True / False | 5929 | 14:17:40 |
| 46130058 | True / False | 5926 | 14:17:41 |
| 47790949 | True / False | 5950 | 14:17:41 |
| 49236051 | True / False | 5948 | 14:17:41 |

Actors started at about 09:21:41 SGT and had run for 4.934 hours at health completion. This is actor uptime, not a measured container-uptime field. All 32 core quotes had bid/ask/timestamps; their ages at health-response completion ranged from 8.188 to 8.366 seconds. A snapshot and cumulative counter do not certify uninterrupted subsecond execution.

## §2. Account reconciliation and open exposure

| Account | Balance USD | Float USD | Equity USD* | S3 realised in interval | Positions: S3 / S1 / human | Pending entries |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 LIVE | 668.04 | 3,680.63 | 4,348.67 | 0.00 | 0 / 0 / 5 | 4 |
| 46130058 DEMO | 24,726.90 | 42.59 | 24,769.49 | 205.30 | 1 / 1 / 0 | 0 |
| 47790949 DEMO | 24,392.28 | 48.15 | 24,440.43 | 201.00 | 1 / 1 / 0 | 0 |
| 49236051 DEMO | 801.35 | 1.44 | 802.79 | 6.58 | 1 / 0 / 0 | 0 |

* Equity is balance plus recorded broker net unrealised P&L, with credit/bonus treatment and prospective closing costs unverified. Marks above are from 14:19, not from the 14:15 cutoff. Demo funds and live funds are kept separate. Float is not realised bot profit.

Demo subtotal: balance $49,920.53, float $92.18, equity $50,012.71; interval closing cash +$412.88. No other account closing cash was recorded in the requested interval. S1 made 34 stop-amendment intents, 17 on each of demos 46130058 and 47790949; their AUS200 positions are separate from S3.

| Account | Observed gross leverage | Price-only mark-to-stop exposure USD | Full exit-cost open risk |
| --- | --- | --- | --- |
| 43002148 | 19.793× | 5,153.16 | Not verifiable |
| 46130058 | 1.107× | 325.28 | Not verifiable |
| 47790949 | 1.111× | 323.20 | Not verifiable |
| 49236051 | 0.859× | 5.72 | Not verifiable |

The live price-only mark-to-stop projection is $5,153.16 against estimated equity of $4,348.67. It excludes gaps, slippage and unquoted closing costs and is not a guaranteed maximum loss. The observer reported current margin about 525.08%, pending-entry projected margin 235.51%, and a negative stress projection. The specific S3 Brent admission cache separately reported 236.51%, below the unchanged 250% floor. These are different, timestamped projections, not contradictory current-margin values.

The nearest retained balance reads before the interval are from 09:21, not 09:42. Live’s older $716.16 and later $668.04 must not be presented as a $48.12 trading loss inside this audit window: intervening cashflows and an exact starting balance are not reconciled. Exact equity-return percentages, MTD/YTD and VaR are outside the evidence supplied here.

## §3. Trade ledger and whether the retained exits worked

All three closed positions were ETHUSD shorts opened around 09:00:31, before the requested interval. Each had a broker-acknowledged stop amendment to its original entry price, then a protective closing fill at or better than its initial take-profit. Opening and closing native quantities reconcile exactly; recorded closing commission, swap and conversion fee are all zero.

| Demo account / position | Entry → exit price | Initial SL / TP | BE amendment SGT | Close SGT | Net USD / original R |
| --- | --- | --- | --- | --- | --- |
| 46130058 / 260838310 | 2125.70 → 2107.37 | 2138.83 / 2107.58 | 11:25:48 | 11:41:54 | +205.30 / +1.3961R |
| 47790949 / 260838304 | 2124.28 → 2106.09 | 2137.41 / 2106.16 | 11:37:37 | 11:43:17 | +201.00 / +1.3854R |
| 49236051 / 260838301 | 2124.32 → 2106.05 | 2137.45 / 2106.20 | 11:37:37 | 11:43:17 | +6.58 / +1.3921R |

cTrader encodes these as STOP_LOSS_TAKE_PROFIT closing orders, not a distinct “take-profit-only” enum. The closing price relative to the original TP supports the target conclusion. R uses actual initial stop distance and filled units; improved fills and cash rounding explain small differences from exactly +1.38R. This verifies these three lifecycles, not the profitability of the strategy in general.

| Demo account / position | New position | Entry / SL / TP | Initial price risk USD | Persisted order SGT | Recorded exit deadline |
| --- | --- | --- | --- | --- | --- |
| 46130058 / 260862445 | SpotCrude short 2.74 lots | 86.338 / 86.873 / 85.599 | 146.59 | 09:48:00 | 15:00:00 SGT |
| 47790949 / 260862442 | SpotCrude short 2.71 lots | 86.338 / 86.873 / 85.599 | 144.99 | 09:48:00 | 15:00:00 SGT |
| 49236051 / 260862431 | SpotCrude short 0.08 lots | 86.358 / 86.893 / 85.619 | 4.28 | 09:47:59 | 15:00:00 SGT |

All three Crude trades remained open in the 14:17 book. Their arms were created at 09:38:55, before the 09:42 audit start; fills occurred around 09:48. They therefore must not be attributed to the ten newly recorded arms inside the window. All broker receipts use label jiaoyi-s3. The final guards verify a fresh peer move of at least 24 bp and a turn of at least 8% of the impulse; initial SL/TP are present.

| Measured stage | Observed result | Interpretation |
| --- | --- | --- |
| Confirmed turn → final writer guard | 22.271–23.371 ms; 3 samples | Includes wait until final guard; not socket-write or broker-fill latency. |
| Touch → final writer guard | 13.805–14.413 seconds; 3 samples | Includes waiting for the required turn; do not label this entire period an execution stall. |
| Recorded order send → fill | 557 / 610 / 644 ms; median 610 ms | Broker-fill receipt timing for the three new entries. |
| Background admission worker duration | Not instrumented / null | The supplied 7.5 ms placeholder is unsupported. |

The updated receipt parser accepts all three new confirmed-turn guards: zero unknown/invalid guard receipts, zero uncertain attempts, zero partial fills and zero broker rejections in this entry cohort. This supports the P1 monitoring repair on observed traffic; it does not provide a universal latency guarantee.

## §4. Setups, refusals and why the live account was inactive

| Measure in 09:42–14:15 | Count | Meaning |
| --- | --- | --- |
| New qualified impulses / newly armed account-symbol records | 10 / 10 | Live 4; each demo 2. Account-scoped records, not ten independent market signals. |
| Persisted S3 opening intents / confirmed new fills | 7 / 3 | Three inherited Crude arms filled; four other intents were refused before send. |
| Peer refusals at final send guard | 4 | NAS100 ×2: peer below 24 bp; BTCUSD ×2: opposing peer. Demos 46130058 and 47790949. |
| Pair-reservation diagnostics / arm disarms | 54 / 3 | 54 skip rows and 3 disarm rows; not 57 distinct profitable opportunities. |
| Macro-regime diagnostic records | 250 | Logged rejection/state observations, not all evaluated quotes. |
| Deep retracement invalidations | 7 | Arms invalidated beyond 61.8%; not counted as broker errors. |

| Live symbol(s), at 14:17 | Observed state / reason |
| --- | --- |
| SpotBrent | Armed context, but no prepared admission: projected margin 236.51% < 250% floor. |
| SpotCrude, NAS100 | Invalidated: retracement exceeded 61.8% without turn confirmation. |
| XAUUSD | Yield to a human/external pending order. |
| XAGUSD | Pair reservation held by XAUUSD. |
| US500, BTCUSD, ETHUSD | Waiting for a qualified opening impulse; current display says below 48 bp. |

Each demo’s held Crude position reserves its oil pair and blocks a simultaneous Brent leg. Demo 49236051 also retained minimum-volume-risk-budget refusals for some candidates and a current BTC peer veto. Zero cached prepared admissions at this snapshot is not proof of a frozen worker: preparation is conditional on qualification, ownership, capacity and fresh evidence. The UI reports these states and does not control whether Rust trades.

## §5. Three Laws and the human book

| Law / check | Observed evidence | Conclusion |
| --- | --- | --- |
| Exact S3 label ownership | All recorded new S3 fills and ETH protection events carry jiaoyi-s3; source guards require exact ownership. | Pass for the inspected S3 receipts. |
| Yield to human positions or pending entries | Live XAUUSD is locked by a human pending entry; XAGUSD also yields through pair reservation. | Active lock is visible; no live bot intent recorded in the interval. |
| Permanent S3 reservations | Compiled HUMAN_RESERVED_SYMBOLS contains NatGas and JPN225. | No S3 entry or management authority for these symbols. |
| NVDA.US | Human/external ownership; outside the eight-symbol S3 core. | Protected by ownership; NVDA.US is not falsely described as a third permanent reservation. |

| Human/external position | Side / volume | Recorded SL | Recorded TP |
| --- | --- | --- | --- |
| JPN225 / 649992251 | buy / 0.4 lots | 56571.3 | Not set |
| JPN225 / 649772867 | buy / 1.2 lots | 56450.7 | Not set |
| NatGas / 637075128 | buy / 0.15 lots | 2.327 | 3.875 |
| NVDA.US / 652921796 | buy / 7.0 lots | 167.79 | Not set |
| JPN225 / 653631770 | sell / 0.5 lots | 60856.3 | 59131.6 |

All five human/external positions have a recorded SL field. NVDA’s quote was stale for an executable protection check, so SL presence must not be upgraded to a fresh-price protection PASS. Several human trades have no fixed TP; that is an observation, not authority for the bot to add one. Four human pending entries are present: XAUUSD, JPN225 and two NatGas orders; their full observed protections appear in the JSON.

No live bot opening, amendment or closing intent, and no live program execution event, was recorded during the interval. That supports zero recorded bot interference. It cannot prove every manual/broker SL/TP change over the whole interval: an exact complete 09:42 broker book and an exhaustive external amendment history are unavailable. Accordingly human_positions_intact is null, and the report rejects the supplied “100% PASS” placeholder.

## §6. London readiness: what is ready and what remains blocked

| Group | Next open SGT | Preparation start / deadline | Base preparation | All coefficients ready | Opening references at 14:17 |
| --- | --- | --- | --- | --- | --- |
| SpotCrude, SpotBrent, US500, NAS100, XAUUSD, XAGUSD ×4 accounts | 15:00 | 14:40 / 14:50 | 24/24 | 0/24; missing 420 and 450 min | Future opening prices not yet observable. |
| BTCUSD, ETHUSD ×4 accounts | 16:00 | 15:40 / 15:50 | 8/8 | 8/8 | Next-window opening prices not yet observable. |

Base preparation covers symbol metadata, current cost inputs, macro context and early-session volatility. It is not a complete-history flag or an unconditional order permission. All 24 traditional rows explicitly show prepared_partial_coefficients/history_ready=false. For their 48 missing coefficient cells, the stored reason is “6 same-slot endpoints;14 needed”. All other 30-minute marks through 390 minutes are available.

The code and calendar explain the shared pattern. New York’s 1 November DST change extended the London window from 390 to 450 minutes. Only six prior weekday London windows in the causal history reach the new late marks. production_sessions::sigma_table refuses endpoints beyond each historical session close and requires fourteen observations. band::sigma_at interpolates between adjacent marks, so immediately after minute 390 (21:30 SGT), an unarmed new impulse needs the absent 420-minute coefficient. At minute 420 and later, the missing 420/450 values also prevent a new band assessment. This is a real late-session coverage limitation, not evidence that the 15:00 opening is blocked.

Existing armed setups and owned-position exits have their own recorded state and guards; a missing new-entry band does not authorise abandoning protective exits. No observation in this audit justifies fabricating late marks, weakening the fourteen-session minimum, or widening trading authority. A later engineering task can evaluate whether the existing 45-day boundary should remain the intended seasonal-history rule.

Opening-reference recovery is observed for current eligible sessions, including original six-second quotes; that does not certify a future opening. Ownership-locked live metals deliberately lack current S3 opening references. Cached prepared_at timestamps can reflect rebuilding an already-recorded session and must not, alone, be called a fresh opening-readiness delay. Demo ETH preparations for the noon window are recorded at approximately 12:00:00–12:00:02, consistent with the PR #214 readiness repair working before a current M30 bar closes.

## §7. Exact progress towards the project goal

The invariant remains 73% net wins and 1:1.38 risk:reward over the latest twenty completed trades, separately for each strategy/account. Twenty-trade windows have five-percentage-point increments; at least 15/20 wins are needed to exceed 73%. Realised payoff and R-based profit factor are separate from the configured gross TP.

| S3 account | Latest 20 | Change vs №18,414 | Gap to 73% | Wins short of 15/20 | Realised payoff | R-based PF | Net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 (15%) | +0 pp | 58 pp | 12 | 3.367 | 0.594 | -6.240 |
| 46130058 | 4/20 (20%) | +5 pp | 53 pp | 11 | 1.043 | 0.261 | -9.413 |
| 47790949 | 4/20 (20%) | +5 pp | 53 pp | 11 | 1.245 | 0.311 | -7.733 |
| 49236051 | 5/20 (25%) | +0 pp | 48 pp | 10 | 1.387 | 0.462 | -6.684 |

All four stored assessments have an empty missing-reasons list and remain below_target. They include older policy trades and therefore do not independently qualify owner_18409. The three profitable ETH positions produce 3/3 wins in this interval but only one replicated economic signal. No S1 financial qualification is inferred from two open S1 positions. financial_pass=false.

## §8. Engineering service, finite monitor and logs

At 14:27:22 the engineering worker was responsive and idle, with a 4.8-second-old heartbeat, 14 of 36 daily model sessions used and 22 remaining. Its reason was owner_engineering_window_closed, not allowance exhaustion. The latest job, 902, completed with no_change; the next scheduled poll was 14:51:16. Public-health/goal-window polling does not establish full account data quality or an absence of defects.

The queue reported 98 items, six pending reviews and two old owner-blocked release records. There were zero pending verifications. Queue count does not mean 98 tasks are currently eligible to edit production. This report does not clear those reviews or claim that no engineering backlog remains.

| Pending review IDs | Recorded reason |
| --- | --- |
| 211, 222 | unresolved_review |
| 250 | structured_source_linked_defect_required |
| 581 | incident_job_identity_mismatch |
| 604, 830 | incident_batch_identity_mismatch |
| Release records 88 / 133 (PRs 81 / 127) | owner_blocked; release_identity_changed; old records requiring separate reconciliation |

The finite 24-hour run 18393-20261109, started 9 November 11:28:32 SGT, is completed: 12/12 checkpoints, zero missed checkpoints, zero model calls and zero Telegram messages. No further checkpoint is scheduled by that finished run. The Rust account actors, protective exits and normal supervision remain separate running services. This audit does not silently extend the monitor.

For the requested interval, Railway’s warning/error filter returned zero trading-service lines and one engineering line at 11:44:02 SGT: “pg_ctl: another server might be running; trying to start server anyway”. The surrounding retrieved lines do not establish its cause or resolution. The later worker heartbeat and completed job show subsequent responsiveness; no trading deployment restart or trade impact is demonstrated by that message. Absence of warning lines is not a proof of no defects.

## §9. Findings and the next three checks

| Check | Required evidence | Current conclusion |
| --- | --- | --- |
| 1. Native opening readiness | At/after 15:00, original opening quotes, causal preparation and exact reasons for any withheld admission; separately check crypto at 16:00. | Future event; not passed in a 14:15 report. Base London preparation is ready, late marks are not. |
| 2. First genuine lifecycle / remaining Crude exits | Broker SL/TP, writer guard, fill, owned-position deadline and realised cash; account-specific ownership and margin retained. | Three Crude entries are verified; their subsequent outcomes are outside this cutoff. Three earlier ETH exits reconcile. |
| 3. Economics and evidence quality | New closed-trade ledger, policy attribution, latest-20 per account, and complete cash/quantity/history reconciliation. | Current all-account S3 goal fails. Two demos improve, insufficient evidence for a new edge claim. |

Owner attention is warranted for live whole-book stop exposure and pending-order margin. Retain human control; do not cancel or amend those orders from this report. The late London coefficient issue and engineering review backlog are explicit follow-ups, not permission to change trading rules. There is no evidence here that bypassing a peer, retracement or margin refusal would improve net results.

No environment publication is needed for this report. No broker backtest collection, verification trade, code deployment, allowance change or Telegram send was made. Time is money: preserve verified fixes and investigate only a named remaining defect or measured trading loss before proposing another change.

## §10. Verification, reproducibility and instructions for another AI

Validation independently recomputed the three closing cash amounts, reconciled entry/exit volumes, decoded broker protection events, checked the exact BE price and TP-side fills, and verified three entry guard proofs. Counts reconcile 7 intents = 3 fills + 4 confirmed refusals; all four later successful history windows cover the requested interval. Preparation reconciles 32 base-ready rows = 24 partial traditional + 8 complete crypto, with exactly 48 missing late cells. Focused arithmetic/provenance checks and desktop/mobile HTML rendering accompany this report; repository CI gates publication.

SQL was read-only repeatable-read with a 12-second statement timeout, 300 ms lock timeout and rollback. The later history receipts have complete=false because the normal recent seven-day refresh deliberately writes that flag; status=ok/error=null and the fetched range establish this narrower interval’s coverage. It is not an assertion of complete account history from inception. Source request status freshness=not_verified is preserved; actual quote, book and fetch timestamps are reported rather than upgrading that metadata silently.

Read this report first, then the companion JSON. It contains account marks, closing money components, trade/guard/protection receipts, decision counts, current gates, preparation states, history ranges, worker state and SHA256 source hashes. The short -gemini-review.json file supplies the measured replacement for the owner’s example JSON. Raw private source captures are not needed to recompute the published cash and quantity arithmetic; hashes identify those captures without publishing credentials or raw database dumps.

Gemini review task: challenge the cutoff separation, recompute $412.88 from the three closing-deal money components, distinguish replicated signals from independent observations, confirm the unchanged production policy and inspect the seasonal 420/450-minute coefficient limitation. Do not turn current float into realised S3 profit, count diagnostic rows as unique trades, claim human history is universally unchanged, or claim that future opening references are already available. Use actual source times and return findings with №18,417 and section references.

Source code anchors at ef046993: crates/jiaoyi/src/strategy3.rs (LABEL, HUMAN_RESERVED_SYMBOLS, human_symbol_locks, load_day and recorded exits); crates/jiaoyi-core/src/strategy3/production_sessions.rs (windows_on, sigma_table, band_at); crates/jiaoyi-core/src/strategy3/band.rs (sigma_at); crates/jiaoyi/src/history.rs (recent deal refresh); ops/engineering/monitoring/dispatch_evidence.py and sql_evidence.py (receipt parsing and cash/coverage).

Files: ASIAN_SESSION_REVIEW_AND_LONDON_PREP-18417-2026-11-10.md; ASIAN_SESSION_REVIEW_AND_LONDON_PREP-18417-2026-11-10.html; ASIAN_SESSION_REVIEW_AND_LONDON_PREP-18417-2026-11-10.json; ASIAN_SESSION_REVIEW_AND_LONDON_PREP-18417-2026-11-10-gemini-review.json. This is a measured audit, not a financial PASS or a new production policy.
