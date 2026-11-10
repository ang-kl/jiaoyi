# Overnight session review and Asian preparation — №18,421

№18,421 §§1–12 · Prepared 2026-11-11T06:33:32+08:00 · by <Codex (GPT-6), effort: not exposed>. Continues №18,420 from 10 November 19:45 through 11 November 06:19 SGT. Operational snapshots follow at approximately 06:20–06:21; cash and decisions use the exact cutoff.

## TL;DR

Five S3 demo trades closed for +$411.55: three ETH targets earned $424.91 and two BTC session-end exits lost $13.36. All four actors were connected and unpaused; no S3 position remained at the operational snapshot. The latest-20 net win rate improved from 35% to 40% on each demo, while live remained 15%. The original 73% / 1:1.38 goal is still unmet.

No new S3 entry filled during the NY window starting 22:30, or after midnight. Assessment continued: there were 31 new arms across the full audit window and 15 consumed triggers. Nine became persisted intents (five fills, four peer refusals); three were refused before intent because peer identity/opening context was unknown; three later consumed triggers lack individual refusal evidence. That final gap deserves targeted engineering investigation, rather than a blanket claim that all non-trades were correct.

The BTC entries had only about 10½ minutes before the mandatory 20:00 session close. ETH instead reached +1R, received acknowledged breakeven stops, and hit its target. These observations support reviewing time-to-session-close and peer readiness; they do not justify resurrecting the rejected early-profit-lock candidate or bypassing guards. This is a read-only audit with no production, order, risk, environment or deployment change.

## §1. Evidence boundary and production identity

| Item | Verified scope |
| --- | --- |
| Activity window | 10 Nov 19:45 inclusive → 11 Nov 06:19 exclusive SGT; UTC 11:45 → 22:19 on 10 Nov. |
| Production commit | ef04699334c423e5b1794101189977832d4cb1bf, unchanged from PR214 engineering release. |
| Active policy | s3_macro_pullback_owner_18409_v1; +1R → entry-price stop, +1.38R target, recorded native-session deadline. |
| Deployment | Trading 8dcc60be… and engineering c203668c… SUCCESS; no release undertaken. |
| Sources | Read-only PostgreSQL transactions, existing broker history/receipts, /health, owner account observer, worker state and existing executable quotes. |
| Reconciliation | 26 complete S3 lifecycles for 10 Nov; five belong to this audit window. Four account histories reported complete. |
| Limits | No fresh backtest collection, parameter experiment, external market narrative or discretionary test trade. Later book snapshots are not invented exact 06:19 valuations. |

## §2. Account balances, float and realised trading

| Account / leverage | Balance USD | Float USD | Equity estimate USD | Window S3 closes / wins | Window S3 net USD | Open inventory |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 1:200 | 501.84 | 3,394.58 | 3,896.42 | 0 / 0 | 0.00 | S3 0; S1 0; external 3; pending 5 |
| 46130058 / 1:200 | 25,548.62 | -32.59 | 25,516.03 | 2 / 1 | 204.69 | S3 0; S1 2; external 0; pending 0 |
| 47790949 / 1:25 | 25,008.23 | -26.03 | 24,982.20 | 2 / 1 | 200.37 | S3 0; S1 2; external 0; pending 0 |
| 49236051 / 1:200 | 820.41 | 0.00 | 820.41 | 1 / 1 | 6.49 | S3 0; S1 0; external 0; pending 0 |

Marks were observed at about 06:20:44–06:20:45 SGT. Equity is balance plus broker net unrealised P&L, excluding unverified credit and potential closing commission. The four accounts include one live and three demos; their dollar amounts must not be represented as one investable portfolio. Exact window percentage returns are not supplied without a matching opening mark and flow reconciliation.

| Account | Full 10 Nov S3 net USD / closes / wins | All-strategy broker closing cash, 10 Nov SGT |
| --- | --- | --- |
| 43002148 | 0.00 / 0 / 0 | -214.32 |
| 46130058 | 921.61 / 9 / 6 | 926.23 |
| 47790949 | 717.88 / 9 / 6 | 672.31 |
| 49236051 | 23.17 / 8 / 5 | 23.17 |

Full 10 November S3: 26 closes, 17 wins and +$1,662.66, including the earlier +$1,251.11 through 19:45. All-strategy closing cash differs: it includes S1 and external trades. There were no recorded closing deals from 11 Nov 00:00 to 06:19 SGT. This daily cash result is not a return, an all-history recovery certificate or a current-policy independent win rate.

## §3. Exact five-trade ledger and exit behaviour

| Account / position | Symbol / side / units | Entry → exit SGT | Fill → close | Original SL / TP | Exit classification | Net USD |
| --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 261063163 | BTCUSD short / 0.28 | 10 Nov 19:49:25.236 → 10 Nov 20:00:01.073 | 74339.11 → 74362.96 | 74870.02 / 73606.45 | native_session_end | -6.68 |
| 46130058 / 261063162 | BTCUSD short / 0.28 | 10 Nov 19:49:25.216 → 10 Nov 20:00:00.977 | 74339.11 → 74362.96 | 74873.39 / 73601.8 | native_session_end | -6.68 |
| 49236051 / 261128944 | ETHUSD short / 0.12 | 10 Nov 21:30:18.131 → 10 Nov 23:42:12.979 | 2066.67 → 2012.61 | 2105.45 / 2013.15 | broker_protective_fill_at_or_better_than_tp | 6.49 |
| 47790949 / 261128943 | ETHUSD short / 3.83 | 10 Nov 21:30:18.071 → 10 Nov 23:42:12.978 | 2066.67 → 2012.61 | 2105.45 / 2013.15 | broker_protective_fill_at_or_better_than_tp | 207.05 |
| 46130058 / 261128942 | ETHUSD short / 3.91 | 10 Nov 21:30:18.108 → 10 Nov 23:42:12.966 | 2066.67 → 2012.61 | 2105.45 / 2013.15 | broker_protective_fill_at_or_better_than_tp | 211.37 |

BTC: both entries filled at 19:49:25; their recorded exit deadline was 20:00. Session-end fills arrived at 20:00:00.977 and 20:00:01.073, consistent with the existing native-window rule. Both lost $6.68. Their best recorded favourable excursion was only about +0.13R, so the +1R breakeven condition was never observed. This was a short available holding horizon, not an observed late-stop or ignored +1R trigger.

ETH: three shorts entered at 21:30:18 and closed at 23:42:12.966–.979. Original risk was $38.78 per unit; broker closing price 2012.61 was better than TP 2013.15. Recorded maximum adverse excursion reached about −0.744R before the target. The three trades share the same underlying signal: count three account outcomes, not three independent market discoveries.

| Account / symbol | Recorded peak R | Worst recorded R | Quotes / max gap | First +1R SGT → BE acknowledgement | Confirmed turn → guard / order → fill |
| --- | --- | --- | --- | --- | --- |
| 47790949 / BTCUSD | 0.130549 | -0.084402 | 2,730 / 2.731s | Not reached / no BE amendment expected | 21.070ms / 488ms |
| 46130058 / BTCUSD | 0.129726 | -0.083870 | 2,730 / 2.724s | Not reached / no BE amendment expected | 22.363ms / 465ms |
| 49236051 / ETHUSD | 1.389634 | -0.744198 | 32,531 / 3.106s | 10 Nov 23:40:28.173 → 461.3ms | 18.603ms / 585ms |
| 47790949 / ETHUSD | 1.389634 | -0.744198 | 32,502 / 3.149s | 10 Nov 23:40:28.170 → 462.9ms | 13.910ms / 519ms |
| 46130058 / ETHUSD | 1.389634 | -0.744198 | 32,502 / 3.109s | 10 Nov 23:40:28.175 → 459.6ms | 19.716ms / 558ms |

All three ETH amendments set SL exactly to entry 2066.67 and retained TP 2013.15; acknowledgement followed the first retained +1R ask by 459.6–462.9 ms. Recorded quote gaps were at most 3.149 seconds, with no gap over six seconds. The actual ETH exit implies +1.3940 gross R versus +1.3896R observed on retained asks: broker fills and retained quotes differ, so continuous true MFE remains unknown. Recorded closing commission, swap and conversion fee for these five S3 closes were zero; this is receipt accounting, not a claim of universally cost-free trading.

## §4. Why there were no later S3 fills

| Stage / diagnostic | Count | Interpretation |
| --- | --- | --- |
| New qualified / armed account-symbol records | 31 / 31 | Not 31 independent instruments or completed trades. |
| Consumed pullback triggers | 15 | Arm removed before awaited admission, protecting against duplicate/uncertain sends. |
| Persisted opening intents | 9 | Five fills and four final peer refusals. |
| Direct pre-intent pair-unknown refusals | 3 | ETH, three demos, 11 Nov 00:35:39. |
| Later consumed triggers without individual reason | 3 | ETH, three demos, 11 Nov 01:39:28. |
| Pair diagnostic records | 54 skips + 3 disarms | Do not sum as unique lost trading opportunities. |
| Deep-retracement invalidations / touch records | 14 / 320 | Touches are state updates; invalidation protects against >61.8% retracement. |
| Macro-veto records | 503 | Repeated account/time diagnostics, not 503 denied orders. |

At 21:25 the two NAS100 intents failed because US500 held only 19.163 bps in the proposed direction, below the required 24 bps. At 02:38 the two BTC intents failed because ETH opposed the selected impulse (peer midpoint 2043.82 versus opening 2022.26 for a proposed short). All four have explicit final refusal receipts; no broker order was sent. Nine arms and eight consumed triggers occurred after midnight, demonstrating ongoing assessment despite zero fills.

Three ETH attempts at 00:35:39 explicitly failed with pair_risk_reservation_unknown before durable intent creation. In compiled entry_guard.rs this reason means missing peer identity/origin or invalid self-peer. It does not establish that BTC occupied the account. peer_evidence() depends on a valid, currently trading peer Day and opening reference. The 47790949 BTC new-window session diagnostic appears at 02:38 despite a retained original 00:00 quote; that supports inspecting peer-context readiness, but log timing alone cannot prove exactly when every actor held valid peer state.

The second three ETH triggers at 01:39:28 have no matching individual persisted intent or refusal reason. Decisions::note suppresses the same reason indefinitely for an account/symbol/kind until that reason changes. A repeated pair-unknown refusal could therefore be hidden; several earlier-return paths also exist. The report leaves the three outcomes unattributed instead of converting that plausible explanation into a verified result. Zero invalid guards among nine persisted intents does not mean zero problems among all fifteen consumed triggers.

## §5. Sizing, margin and other strategy activity

| Conditional example | Risk at broker minimum | Risk-only equity at 0.6% | Limit of inference |
| --- | --- | --- | --- |
| BTC, actual 47790949 geometry | 0.01 × 530.91 = $5.3091 | $884.85 | Reconstructs this filled setup; not an exact absent small-account refusal. |
| BTC, actual 46130058 geometry | 0.01 × 534.28 = $5.3428 | $890.47 | Different rounding/prepared bound; preserve account-specific geometry. |
| ETH, all three filled geometries | 0.01 × 38.78 = $0.3878 | $64.64 | Risk floor only; margin, total risk and peer rules remain separate. |
| Small demo actual ETH entry | 0.12 × 38.78 = $4.6536 | Within 0.6% of pre-trade $813.92 | Closed +$6.49; current balance $820.41. |

47790949 is the roughly $25,000 1:25 demo; 49236051 is the roughly $820 1:200 demo. Higher leverage reduces margin demand but does not reduce dollars lost at the stop. The previous audit observed a small-demo BTC minimum-volume risk refusal; this audit does not invent a complete new count of all background sizing refusals. Exact minimum capital for every rejected setup remains unknown without the corresponding saved geometry, equity, conversion and account book.

S1 independently opened AUDUSD shorts at 23:50:06 on 46130058 and 47790949, 0.09 lot each, and recorded 21 stop amendments during the window. Both accounts retain an older S1 AUS200 position plus AUDUSD. A live S1 AUDUSD attempt was refused at projected margin about 218.22%, below the existing 250% floor. That is an S1 admission result, not an S3 or broker execution failure.

## §6. Human protection and live exposure

| Law / event | Evidence and meaning |
| --- | --- |
| Label ownership | S3 entry/protection receipts carry jiaoyi-s3. No live program intent appears in the window. |
| Yield to human | Live XAUUSD external pending entry continues to reserve XAUUSD and its XAGUSD pair. |
| Permanent reservation | NatGas and JPN225 remain human-reserved. NVDA is protected by ownership/outside-core scope, not an invented additional permanent reservation. |
| External NVDA close | Position 652921796 closed 23:55:01.638 SGT: gross −$48.93, swap −$0.22, close commission −$0.28; net −$49.43. |
| Current external book | Three open human/external positions and five pending entries, versus four/four previously. Human book was not unchanged. |
| Interference evidence | No recorded bot interference; no complete broker amendment history proving every human SL/TP unchanged. |

| Live external object | Kind / side / lots | SL | TP |
| --- | --- | --- | --- |
| 649992251 / JPN225 | position / buy / 0.4 | 56571.3 | Not set |
| 649772867 / JPN225 | position / buy / 1.2 | 56450.7 | Not set |
| 637075128 / NatGas | position / buy / 0.15 | 2.327 | 3.875 |
| 400465199 / XAUUSD | pending_entry / buy / 0.1 | 3877.3 | 4339.75 |
| 400871507 / JPN225 | pending_entry / buy / 0.5 | 59366.5 | 60851.5 |
| 398403207 / NatGas | pending_entry / buy / 0.2 | 2.381 | 4.275 |
| 400083120 / JPN225 | pending_entry / buy / 1.5 | 55679.5 | Not set |
| 399322019 / NatGas | pending_entry / buy / 0.1 | 2.423 | 3.095 |

Current margin projection was 812.55%; observed pending entries projected 246.62%, below 250%. Stress projection is unknown because complete marks/conversion/protection evidence is unavailable. The live mark estimates price-only stop loss exposure of about $4,804.29 versus equity $3,896.42; full cost-inclusive open risk is unknown. These are external-book observations, not authority to amend or cancel human trades.

## §7. NY opening verification and next Asian preparations

The NY window retained 22 of 24 traditional opening references; the two live metal references were withheld under human/pair locks. Available references use original first-six-second opening quotes. The current 04:00 crypto window has 8/8 references and preparation timestamps 04:00:00.413–04:00:02.697. A latest preparation timestamp is not always first readiness: 47790949 refreshed several NY contexts at 23:57, while earlier session and assessment records already exist between 22:30 and 22:59. That refresh is not evidence of an 87-minute initial outage.

| Next native session group | Opening SGT | Preflight start / deadline SGT | Base + full coefficients + macro | Opening reference at 06:19 |
| --- | --- | --- | --- | --- |
| US500, NAS100, SpotCrude, XAUUSD, XAGUSD × 4 | 07:00 | 06:40 / 06:50 | 20/20 | Future; not yet observable |
| BTCUSD, ETHUSD × 4 | 08:00 | 07:40 / 07:50 | 8/8 | Future; not yet observable |
| SpotBrent × 4 | 09:00 | 08:40 / 08:50 | 4/4 | Future; not yet observable |

All 32 next-session preparation rows are complete at the snapshot. SpotBrent has its own 09:00 native opening; do not promise that all six traditional symbols open at 07:00. Traditional quotes around the snapshot reflect their closed/maintenance period, whereas crypto quotes continue. Future execution still requires a fresh opening quote and all ownership, pair, macro, impulse, pullback/turn, peer, cost, sizing and margin checks. The audit does not claim future 32/32 opening prices or unconditional permission to trade.

## §8. Progress against the invariant target

| S3 account | Latest 20 wins | Change vs №18,420 | Gap to 73% / wins short of 15 | Realised payoff | R-based PF | Net R |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 (15%) | +0 pp | 58 pp / 12 | 3.367 | 0.594 | -6.240 |
| 46130058 | 8/20 (40%) | +5 pp | 33 pp / 7 | 1.627 | 1.085 | 0.695 |
| 47790949 | 8/20 (40%) | +5 pp | 33 pp / 7 | 1.442 | 0.961 | -0.312 |
| 49236051 | 8/20 (40%) | +5 pp | 33 pp / 7 | 1.634 | 1.089 | 0.820 |

At least 15/20 net wins (75%) are needed to exceed a 73% latest-20 threshold. Each demo improved by one net winning outcome in its rolling count, but remains seven wins short of 15; live remains twelve short. These are changing rolling windows, not a forecast of how many next trades will suffice. The samples contain older policies and correlated demo copies. Configured gross target 1.38R, realised payoff and profit factor are distinct. Financial PASS remains false; no S1 qualification is inferred.

## §9. Engineering worker and monitor status

At 06:21:33 SGT the engineering heartbeat was 2.7 seconds old. The worker was idle under owner_engineering_window_closed; 27/36 daily sessions were used and nine remained. Latest job 968 completed no_change at 05:30; the public assessment at 06:03 found unchanged evidence; next poll was due 06:32:58. Its 104 queued records, six pending reviews and two old owner-blocked releases are unresolved states, not evidence that every issue is repaired.

The earlier finite 24-hour monitor completed 12/12 checkpoints with zero missed checkpoints and no next checkpoint. Rust actors continue separately. The worker’s automatic assessment scope is public health and strategy assessment windows; that scope does not establish full-account source quality or detect every pre-intent evidence gap. This audit neither resumes a finite monitor nor changes its allowance, owner window or Telegram settings.

## §10. Findings and the next four checks

| Priority / check | Present evidence | Proposed next action / acceptance |
| --- | --- | --- |
| 1. Peer readiness and trigger attribution | 3 direct pair-unknown refusals; 3 later consumed ETH triggers without individual reason. | Inspect peer Day/opening restoration before trigger consumption. Preserve fail-closed ownership/pair checks. Require one durable outcome per consumed trigger and a focused stale-peer/repeated-reason regression before any repair release. |
| 2. Session-horizon economics | Two BTC entries had only ~10½ minutes left; both closed −$6.68 without reaching +1R. | Freeze a minimum-time-remaining hypothesis only if broader evidence supports it; compare baseline on separate dates with explicit costs. No assumption that holding past the deadline wins. |
| 3. Upcoming opening readiness | 32/32 next-session preparation rows; future opens unavailable. | Observe actual 07:00/08:00/09:00 first quotes, causal context and active ownership exceptions. Do not misclassify later refresh timestamps as initial failures. |
| 4. Next real lifecycle and goal accounting | ETH guard/fill/BE/TP/cash verified; latest20 demos40%, live15%. | Reconcile the next genuine trade and per-account rolling outcomes. Preserve proven readiness/parser fixes and rejected-profit-lock decision; qualify an edge independently. |

No code fix was deployed in this reporting task. The highest-value engineering follow-up is the observed peer-context/outcome-attribution gap, not another generic stop adjustment. Time is money: retain proven fixes, pursue source-linked failures, and avoid extra sessions merely to create activity. “It’s not the will to win that matters—everyone has that. It’s the will to prepare to win that matters.”

## §11. Verification and residual uncertainty

Focused report validation reconciles all five quantities and net cash to broker receipts, all 26 daily S3 closes to +$1,662.66, exact cutoff membership, labels, final guard outcomes, three entry-price stop acknowledgements, 15→9+3+3 trigger accounting, independent latest-20 scope, future preparation counts and null handling. Desktop/mobile HTML is checked for readable scrollable tables and no page-width overflow. The report-only PR must pass check, docker and engineering before merge.

Unresolved: the exact reason for three later consumed triggers; the causal timeline of missing peer Day/origin; complete unused-admission refusal history; exact continuous MFE; full historical human amendment certification; and independent evidence of the strategy’s net edge. No complete-data or no-bugs claim is made. No environment publication is required for this report.

## §12. Instructions for Gemini

Read this report, the companion .json and -gemini-review.json on main. Compare with №18,420 and the PR214 engineering release. Recompute the five-trade +$411.55 and full-day S3 +$1,662.66; separate S1 and external cash. Challenge the six consumed/no-intent paths: source shows pair-unknown and indefinite diagnostic deduplication, but does not prove all six share a cause. Check BTC time-to-deadline without assuming an alternative exit would win. Verify three ETH BE acknowledgements and distinguish retained-quote MFE from broker fills.

Inspect source at ef046993: strategy3.rs Decisions::note, peer_evidence and entry trigger consumption; strategy3/fast_entry.rs dispatch_prepared; jiaoyi-ctrader entry_guard.rs validate_pair_occupancy and trading.rs pre-intent checks. Return findings with №18,421 and section references, separating confirmed defect, plausible cause and missing evidence. Propose only the smallest evidence-backed engineering change; retain the Three Laws, risk limits, pair/peer rules and rejected profit-lock decision. No inferred permission to change live strategy from this report alone.
