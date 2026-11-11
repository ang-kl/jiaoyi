# Overnight NY review and Wednesday Asia preparation — №18,422

№18,422 §§1–11 · Prepared 2026-11-11T07:59:51+08:00 · by <Codex (GPT-6), effort: not exposed>. Requested cutoff: 11 Nov 07:50 SGT. Current books and health were sampled afterwards and are timestamped separately. Read-only production audit.

## TL;DR

Five overnight S3 demo closes still total +$411.55 (three ETH targets, two BTC session-end losses). No new S3 entry or close appeared after №18,421 through 07:50. Closed S3 cash since the specified 10 Nov 09:42 start is +$1,869.61 across 20 positions. The latest-20 win rate remains 15% live and 40% on each demo: no progress since the 06:19 report and no financial PASS.

Asian preparation is not full entry readiness: SpotCrude first quotes arrived about 7.0 seconds after 07:00, outside the six-second opening observation window. All four Crude original references remain missing. Fourteen of 20 current Asian account-symbol references are valid; two additional live metals are withheld by human ownership. The 08:00 crypto preparation is 8/8 complete; its future opening prices and 12:00 readiness cannot be certified from this pre-open snapshot.

The two 02:38 BTC attempts retained a prior-window arm by explicit policy: crypto arms may survive four hours across rollover. This is not a stale-arm violation. A separate code interaction deserves repair review: retaining the old Day for a carried arm can prevent that symbol from supplying its current-window peer reference. Three ETH pair-unknown refusals are directly recorded; three later consumed triggers still lack individual outcome records. No production repair or risk/exit change is deployed in this audit.

## §1. Scope, source quality and deployment

Window: 10 Nov 19:45 inclusive to 11 Nov 07:50 exclusive SGT (UTC 11:45–23:50 on 10 Nov). The post-PR #214 comparison starts at 10 Nov 09:42 SGT, exactly as requested; elapsed time is 22 hours 08 minutes, not yet 24 hours. It counts positions closing after that start, including positions opened earlier. It is not a pure cohort of entries made under the new policy.

Trading and engineering remain at the PR #214 engineering release ef04699334c423e5b1794101189977832d4cb1bf; the active policy is s3_macro_pullback_owner_18409_v1. The original 1R breakeven, 1.38R target and recorded native-window deadlines remain. All four actors are connected and unpaused. Read-only PostgreSQL receipts, existing quotes, health, owner book and deployment state support this audit. No new broker backtest collection, model trading decision, Telegram message or test order was made.

The unchanged five-trade executable-ask summaries from №18,421 are reused with hashes after comparing their cash, fill and quantity evidence with the fresh read. Continuous true MFE remains unknown. Full historical human amendment logs and complete unused-admission history remain unavailable. Source snapshots are not falsely timestamped 07:50.

## §2. Account balances, float, margin and ownership

| Account / leverage | Mark SGT | Balance / float / equity USD | Observed margin level | Open S3 / S1 / external; pending |
| --- | --- | --- | --- | --- |
| 43002148 / 1:200 | 11 Nov 07:52:44.996 | 501.84 / 3,584.33 / 4,086.17 | 851.65% | 0 / 0 / 3; 5 |
| 46130058 / 1:200 | 11 Nov 07:53:44.275 | 25,548.62 / -31.48 / 25,517.14 | 51220.57% | 0 / 2 / 0; 0 |
| 47790949 / 1:25 | 11 Nov 07:53:44.394 | 25,008.23 / -24.92 / 24,983.31 | 6272.41% | 0 / 2 / 0; 0 |
| 49236051 / 1:200 | 11 Nov 07:53:44.381 | 820.41 / 0.00 / 820.41 | No open margin; ratio undefined | 0 / 0 / 0; 0 |

Equity is balance plus broker net float, excluding unverified credit and potential closing commission. Margin ratios come from the separately timestamped owner-book observer, not the account mark whose aggregation can be unknown. The strategy-position column contains only recorded S1 positions; no S2 position was observed. Demo funds are separate from live capital; no combined investable-portfolio balance is claimed.

| Live external object | Kind / side / lots | SL / TP |
| --- | --- | --- |
| 649992251 / JPN225 | position / buy / 0.4 | 56571.3 / None |
| 649772867 / JPN225 | position / buy / 1.2 | 56450.7 / None |
| 637075128 / NatGas | position / buy / 0.15 | 2.327 / 3.875 |
| 400465199 / XAUUSD | pending_entry / buy / 0.1 | 3877.3 / 4339.75 |
| 400871507 / JPN225 | pending_entry / buy / 0.5 | 59366.5 / 60851.5 |
| 398403207 / NatGas | pending_entry / buy / 0.2 | 2.381 / 4.275 |
| 400083120 / JPN225 | pending_entry / buy / 1.5 | 55679.5 / None |
| 399322019 / NatGas | pending_entry / buy / 0.1 | 2.423 / 3.095 |

Three Laws: all inspected S3 trade/protection receipts carry jiaoyi-s3; there is no live bot intent in the audit; XAUUSD external pending ownership reserves the live metals pair; NatGas and JPN225 remain permanently reserved. NVDA.US is protected by external ownership and its exclusion from the core universe. Recorded bot interference is zero, while a universal 100% historical amendment certificate remains Not Verifiable.

External NVDA position 652921796 closed at 23:55:01.638 SGT for −$49.43 (gross −48.93, swap −0.22, closing commission −0.28), separately from S3. Live now has three external positions and five pending entries. The retained NY-close cash-flow receipt shows zero external deposit/withdrawal count and net amount over its own interval from the prior close to 06:00:45. Complete cash-flow coverage for 06:00:45–07:50 is not available. A stable balance alone does not prove the absence of gross cash flows.

Current live margin of about 851.65% exceeds 250%; current pending and stress-margin projections are unknown, so the older 246.62% projection is not reused as current truth. Price-only live stop-loss exposure is about $4,994.27 against $4,086.17 equity; full cost-inclusive risk is unknown. No authority to amend human orders is inferred. A live S1 AUDUSD attempt at 23:50 was explicitly refused at 218.22% projected margin; this does not establish an S3 margin refusal.

## §3. Complete overnight S3 ledger

| Account / position | Symbol / side / units | Entry → exit SGT | Entry / initial SL / TP / exit | Exit / net USD / net R |
| --- | --- | --- | --- | --- |
| 47790949 / 261063163 | BTCUSD / short / 0.28 | 10 Nov 19:49:25.236 → 10 Nov 20:00:01.073 | 74339.11 / 74870.02 / 73606.45 / 74362.96 | native_session_end / -6.68 / -0.044936 |
| 46130058 / 261063162 | BTCUSD / short / 0.28 | 10 Nov 19:49:25.216 → 10 Nov 20:00:00.977 | 74339.11 / 74873.39 / 73601.8 / 74362.96 | native_session_end / -6.68 / -0.044653 |
| 49236051 / 261128944 | ETHUSD / short / 0.12 | 10 Nov 21:30:18.131 → 10 Nov 23:42:12.979 | 2066.67 / 2105.45 / 2013.15 / 2012.61 | broker_protective_fill_at_or_better_than_tp / +6.49 / +1.394619 |
| 47790949 / 261128943 | ETHUSD / short / 3.83 | 10 Nov 21:30:18.071 → 10 Nov 23:42:12.978 | 2066.67 / 2105.45 / 2013.15 / 2012.61 | broker_protective_fill_at_or_better_than_tp / +207.05 / +1.394019 |
| 46130058 / 261128942 | ETHUSD / short / 3.91 | 10 Nov 21:30:18.108 → 10 Nov 23:42:12.966 | 2066.67 / 2105.45 / 2013.15 / 2012.61 | broker_protective_fill_at_or_better_than_tp / +211.37 / +1.393987 |

All five opened and closed inside the audit; no carried-in S3 position or still-open S3 position remains. No S3 fill occurred after the 22:30 NY open. BTC closed about 10½ minutes after entry because 20:00 was its recorded native deadline, without reaching +1R. ETH closed at 2012.61, better than TP 2013.15. There are three broker-TP, two session-timeout, zero breakeven-stop and zero initial-stop exits in this window.

| Account / symbol | Observed MFE R / true continuous MFE | First +1R → acknowledged BE | Quote count / max gap | Confirmation→guard / order→fill |
| --- | --- | --- | --- | --- |
| 47790949 / BTCUSD | 0.130549 / unknown | Not reached | 2730 / 2.731s | 21.070ms / 488ms |
| 46130058 / BTCUSD | 0.129726 / unknown | Not reached | 2730 / 2.724s | 22.363ms / 465ms |
| 49236051 / ETHUSD | 1.389634 / unknown | 461.3ms | 32531 / 3.106s | 18.603ms / 585ms |
| 47790949 / ETHUSD | 1.389634 / unknown | 462.9ms | 32502 / 3.149s | 13.910ms / 519ms |
| 46130058 / ETHUSD | 1.389634 / unknown | 459.6ms | 32502 / 3.109s | 19.716ms / 558ms |

ETH original risk was 38.78 per unit; three BE amendments set SL to the entry price of 2066.67 and retained TP. Acknowledgements followed the first retained +1R asks by 459.6–462.9 ms. The actual closing fill implies 1.3940 gross R versus 1.3896 retained-quote MFE; the differences between broker fills and stored quotes are explicit. Five closing receipts record zero commission, swap and conversion fees; entry and exit rows are not double-counted.

## §4. Cumulative cash and progress towards the goal

| Account | Window closes / net USD | Since 10 Nov 09:42 closes / net USD | Latest 20 wins / net win rate | Change vs №18,421 | R-based PF / realised payoff |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 0 / +0.00 | 0 / +0.00 | 3/20 / 15% | +0 pp | 0.594 / 3.367 |
| 46130058 | 2 / +204.69 | 7 / +1027.02 | 8/20 / 40% | +0 pp | 1.085 / 1.627 |
| 47790949 | 2 / +200.37 | 7 / +816.95 | 8/20 / 40% | +0 pp | 0.961 / 1.442 |
| 49236051 | 1 / +6.49 | 6 / +25.64 | 8/20 / 40% | +0 pp | 1.089 / 1.634 |

Combined S3 closing cash: window +$411.55; specified post-PR period +$1,869.61. Full 10 Nov SGT cash of +$1,662.66 covers a different window that includes earlier losses. The latest 20 needs at least 15 wins (75%) to exceed 73%; demos remain seven wins short in the rolling count, and live remains twelve short. There is no improvement since №18,421. Replicated demo signals and mixed older policies are not independent qualification. The configured gross 1.38R target, realised payoff and profit factor are different metrics.

## §5. Veto funnel, parser integrity and crypto carry

| Stage / diagnostic | Measured records | Interpretation |
| --- | --- | --- |
| New arms / qualified marks | 31 / 31 | Account-symbol records, not independent opportunities. |
| Consumed triggers / persisted opening intents | 15 / 9 | Nine intents: five fills and four final peer refusals. |
| Pre-intent refusals / unattributed consumptions | 3 / 3 | 00:35 ETH pair-unknown versus 01:39 ETH without individual terminal records. |
| Pair skips / disarms | 54 / 3 | Diagnostics, not unique lost trades. |
| Deep invalidations / touch records | 14 / 320 | Touches are not refusals. |
| Macro-veto records | 503 | Repeated diagnostics; not dispatched orders. |
| Unknown/invalid persisted final guards | 0 | Only the persisted cohort; does not certify six pre-intent outcomes. |
| Maximum confirmed-turn→guard | 22.363 ms | Five filled entries; not background preparation or socket duration. |

Two NAS100 intents at 21:25 were refused because US500 held only 19.163 bps, below 24 bps. Two BTC intents at 02:38 were refused because ETH opposed the short impulse. No broker rejection, partial fill or uncertain send appears in those nine persisted intents. The three 00:35 ETH attempts were refused before intent creation with pair_risk_reservation_unknown; the later three 01:39 consumptions remain individually unattributed.

Independent review corrected an initial suspicion: BTC arms formed at 23:46 from the 20:00–00:00 source window were intentionally allowed to survive four hours, until about 03:46. Registration №18,392 explicitly permits crossing a crypto boundary and recording the active execution-window close separately. The 02:38 attempts with 04:00 exit deadlines follow that rule. Do not discard their evidence or label the carry itself a defect.

The source-confirmed interaction is different: strategy3.rs prepare() preserves the old Day while an arm carries, but peer_evidence() rejects a Day whose session is no longer trading. One cache therefore cannot simultaneously serve the original arm and a current-window peer. Three carried BTC arms overlap both ETH cohorts, supporting this explanation for missing peer context. Exact historical actor-memory branches were not retained; the second cohort is not upgraded to three proven refusals. Decisions::note also suppresses unchanged reasons, obscuring repeated outcomes.

## §6. Empirical NY-period capital sizing

The NY civil-time interval is 22:30–05:59 SGT; crypto rows below occurred during that period but retain their native crypto session rules. Minimum risk-only equity equals rounded broker-minimum units × recorded stop distance × recorded quote-to-USD conversion ÷ 0.006. This is arithmetic for saved candidates, not a certificate that every other admission gate passes.

| Symbol | Saved NY candidate floor at 0.6% USD | Evidence / result |
| --- | --- | --- |
| BTCUSD | 1,855.17 | Two saved 02:38 final-peer-refused plans; intentional carried arms, no fill. |
| ETHUSD | 19.49 | Three saved 00:35 pre-intent pair-unknown refusal geometries. |
| US500 | Not verifiable | No saved NY candidate geometry. |
| NAS100 | Not verifiable | The 21:25 saved refusal predates NY; do not relabel it. |
| XAUUSD | Not verifiable | No saved NY candidate geometry. |
| XAGUSD | Not verifiable | No saved NY candidate geometry. |
| SpotCrude | Not verifiable | No saved NY candidate geometry. |
| SpotBrent | Not verifiable | No saved NY candidate geometry. |

For the small demo, the 00:35 ETH plan was 0.42 units with $4.9098 planned risk and 1941.34% projected margin: preparation passed. The refusal was missing peer context. For 47790949, the 02:38 BTC preparation passed with 2852.03% projected margin; the final peer gate refused it. No persisted S3 minimum-volume or margin refusal diagnostic was observed for either account during this interval. The complete count across all stages remains null because warm-admission failures are not durably enumerated.

Original-arm sensitivity is separate: 16 NY arm snapshots imply BTC $513.29–$1,418.55 and ETH $17.27–$29.64 under the stated conversion and initial-geometry assumptions. Later running extremes, spreads and prepared bounds changed the recorded candidates. Forty-five saved geometries match the native rounding arithmetic; that does not make a hypothetical initial-arm calculation the actual rejected quantity. Higher leverage lowers margin demand but not stop-loss dollars. Live still uses its existing one-minimum/10% ceiling, so the requested 0.6% table is not a replacement live rule.

## §7. Wednesday Asian and crypto readiness

| Window / group | Prepared inputs | Usable opening references | Conclusion at this capture |
| --- | --- | --- | --- |
| 07:00 Asian: 5 symbols × 4 | 20/20 complete | 14/20 | Four Crude gaps; two live metals excluded by ownership. |
| SpotCrude 07:00 | 4/4 complete | 0/4 | No bid/ask within first six seconds; first quote 7.023–7.081 seconds after open. |
| US500/NAS100 07:00 | 8/8 complete | 8/8 | Actual references present. |
| XAUUSD/XAGUSD | 8/8 complete | 6/8 | Broker opening observation starts 07:01 by explicit adjustment; live pair withheld. |
| SpotBrent 09:00 | 4/4 complete | Future | 09:00 native open has not occurred. |
| Current crypto 04:00 | 8/8 complete | 8/8 | Original quotes exactly preserved from №18,421. |
| Next crypto 08:00 | 8/8 complete | Future at pre-open capture | Verify actual opening separately; this report does not move its cutoff. |
| Crypto 12:00 | Not yet exposed | Future | No failure declared before its preparation window. |

A bounded SQL read confirms zero SpotCrude ticks during 07:00:00–07:00:06 on all four accounts, then 242–243 ticks per account over the first ten minutes. First ticks arrived at 07:00:07.022973–07:00:07.080577. The six-second opening gate therefore has no admissible original price. Macro drift context itself is available; the generic macro_preparation_unavailable display masks the specific opening-reference failure. No later quote is silently substituted.

The 07:01 metals reference is expected: production_sessions.rs shifts the observation start by one minute for those native instruments. It is not a repeat of the previous 31-minute readiness defect. No redeployment occurred, so the report does not claim a newly tested restart restoration. Carried and current quote identities are compared where the sessions match; future prices and missing Crude references cannot be certified as preserved.

## §8. S1 and worker status

S1 opened two AUDUSD shorts at 23:50 on 46130058 and 47790949 and recorded 21 stop amendments during the audit. Both retain AUS200 plus AUDUSD; live has no bot position. S1 fills and external broker cash are kept separate from the S3 goal ledger.

Worker snapshot, 11 Nov 07:53:46.804 SGT: state idle, heartbeat 0.3 s, sessions 29/36, remaining 7. Activity reason: owner_engineering_window_closed; queued 104. The finite №18,393 monitor remains completed at 12/12, not a new unattended monitoring promise. Automatic public-health assessment does not establish the absence of the context or evidence gaps found here.

## §9. Proposed minimal follow-up and acceptance checks

| Finding | Proposed follow-up; no change deployed | Acceptance evidence |
| --- | --- | --- |
| Crude original-opening quote unavailable | First isolate source recording/arrival latency versus the chosen six-second contract. Any widened observation rule needs an explicit versioned decision; retain the actual first quote timestamp. | 1. Boundary test at 6 s / 7 s; 2. true missing-quote case stays blocked; 3. replay impact and no later-price relabelling. |
| Carried arm/current-peer context coupling | Separate immutable arm-source context from independently current peer context. Preserve intentional four-hour carry, 24 bp peer confirmation, ownership and pair occupancy guards. | 1. Carry across rollover remains valid; 2. current peer reference usable; 3. stale/opposing/missing peer still refuses. |
| Consumed-trigger outcome gap | Record a bounded terminal outcome per consumption; aggregate repetitive idle diagnostics separately. | Exactly one attributable result per consumed trigger, including repeated pre-intent refusal. |
| BTC short remaining horizon | Research only: assess time remaining before session-end exit across independent dates and costs. | No deployment based on two small losses or reintroduction of the rejected profit lock. |

The first two findings concern execution context and observability, not proof of a new alpha edge. This read-only directive does not authorise a trading-parameter change. Preserve proven fixes and the Three Laws. Time is money: pursue these concrete failures with focused checks, rather than spending the session allowance merely to create activity.

## §10. Verification and limitations

Cash, matched opening/closing quantities, original SL/TP, labels, decoded final guards, five quote paths and three BE acknowledgements were reconciled. Window and post-PR totals were computed from distinct closed positions. Current reference identities were compared, with full preparation deliberately separated from usable opening quotes. Source hashes and independent review evidence are included in JSON. Required check, docker and engineering CI checks gate the documentation-only merge.

Unknowns remain explicit: three individual post-consumption outcomes; the exact historical in-memory peer branch; complete warm-admission failure counts; missing per-symbol NY candidate geometry; continuous true MFE; full historical human amendment certification; and cash-flow history after the retained NY-close interval. No claim of 100% safety, guaranteed opportunity capture or goal attainment is made. Current snapshots after 07:50 do not extend the activity cutoff.

## §11. Instructions for Gemini

Read this MD or HTML, then the companion JSON and compact summary on main. Recompute +$411.55 window cash and +$1,869.61 since 10 Nov 09:42. Check that the latter covers 22 h 08 m and may include earlier entries. Review the source-backed Crude seven-second-versus-six-second evidence and the intended crypto carry/current-peer cache interaction. Do not mislabel prior-window arms as expired or treat conditional sizing floors as actual fills.

Challenge each proposed change using the focused checks in §9. Keep confirmed facts, code mechanisms and unrecorded branches distinct. Preserve exact-label human protection, risk/margin limits and original exit rules. Return findings with №18,422 and paragraph references. The task publishes evidence and proposals; it does not place trades, alter production or silently resume the completed monitor.
