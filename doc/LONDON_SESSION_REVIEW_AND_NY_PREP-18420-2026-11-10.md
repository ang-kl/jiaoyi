# London session review, sizing and New York preparation — №18,420

№18,420 §§1–12 · Prepared 2026-11-10T20:04:12+08:00 · by <Codex (GPT-6), effort: not exposed>. Activity: 10 November 2026, 14:15 inclusive to 19:45 exclusive SGT. Production ef04699334c423e5b1794101189977832d4cb1bf.

## TL;DR

Jiaoyi traded: 12 demo S3 positions closed for +$1,045.18 in this window, including all three carried-in Crude positions. Nine new entries filled; three other opening intents were refused by the final peer check. No S3 position remained in the 19:45 operational snapshot. All four account actors were connected and unpaused.

The latest-20 S3 net win rate is now 35% on each demo, versus 20%, 20% and 25% in №18,417. Live remains 15%. The goal is 73% with 1:1.38 risk:reward, separately per strategy/account; financial_pass=false. Today’s profitable correlated demo trades do not establish a durable edge.

Account 47790949 is the approximately $24,812, 1:25 demo; account 49236051 is the approximately $814, 1:200 demo. The latter currently reports a BTC minimum-volume risk-budget refusal. A conditional reconstruction needs $884.85 equity for that minimum BTC stop risk at 0.6%; greater leverage does not reduce that cash risk. An exact capital figure clearing every setup today is not verifiable.

Upcoming NY preparation is complete for 24/24 traditional account-symbol combinations. Crypto has a separate 20:00 window with 8/8 prepared. Future opening prices are unavailable before the opens. The current London session still lacks its late 420/450-minute coefficients. No production, environment, human-order or strategy change is part of this report.

## §1. Scope, timing and deployment

| Evidence | Time / scope | Interpretation |
| --- | --- | --- |
| Activity | 14:15–19:45 SGT; [06:15,11:45) UTC | Persisted intents, fills, protection events and broker closing cash. |
| Full day to cutoff | 00:00–19:45 SGT | 21 S3 closes; not a completed whole-day return. |
| Health / owner book | Health approximately 19:45:21–19:45:32; book around 19:44:42 | Operational snapshot, not an atomic book at 19:45:00. |
| Account marks | 19:44:44 SGT | Separate balance/float/equity snapshot. |
| History and source recheck | 19:50:38.641 / 19:57:40.503 SGT | Later reads retain the original cutoff; no additional pre-cutoff deal/order/event IDs. |

Trading deployment 8dcc60be-052d-481f-a967-98c8ca936715 and engineering deployment c203668c-36a9-476f-b00f-5e317dc65776 remain successful on the PR #214 release. Actual active policy is s3_macro_pullback_owner_18409_v1; it was not renamed by the infrastructure repair. The baseline is +1R favourable movement to an entry-price stop and +1.38R gross TP, with the recorded native-session deadline. Traditional sessions are not universally four hours after entry.

The production cost allowance remains 3.74 bp per side, explicitly owner-selected rather than a verified round-trip calibration. Realised cash below comes from broker deal components; the hypothetical allowance is not subtracted again from those actual fills. The allowance still affects admission and funded stop geometry.

The source queries use read-only transactions, bounded timeouts and rollback. Recent seven-day history fetches deliberately carry complete=false; successful status, error=null and their requested ranges establish this interval’s coverage, not full history since account inception. Current source-status freshness metadata is preserved in JSON alongside actual timestamps.

## §2. Account balance, float, realised cash and open book

| Account | Leverage read | Balance USD | Float USD | Equity USD* | S3 window net | S3 day-to-cutoff net | Open S3 / S1 / external |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 LIVE | 1:200 | 551.27 | 3,389.34 | 3,940.61 | 0.00 | 0.00 | 0 / 0 / 4 |
| 46130058 DEMO | 1:200 | 25,343.93 | -1.62 | 25,342.31 | 617.03 | 716.92 | 0 / 1 / 0 |
| 47790949 DEMO | 1:25 | 24,807.86 | 4.40 | 24,812.26 | 415.58 | 517.51 | 0 / 1 / 0 |
| 49236051 DEMO | 1:200 | 813.92 | 0.00 | 813.92 | 12.57 | 16.68 | 0 / 0 / 0 |

* Equity here is recorded balance plus broker net unrealised P&L, excluding unverified credit and potential closing charges. Live and demo funds are not pooled. Window S3 cash is +$1,045.18; full SGT-day-to-cutoff S3 cash is +$1,251.11. The open AUS200 positions belong to S1 and do not belong in the S3 denominator.

Live had no S3 entry or S3 close, but three external/unattributed JPN225 closing deals booked −$164.89: −$48.12 and −$37.68 on position 653631770, then −$79.09 on position 653855547. These are outside bot S3 ownership. The live balance is $551.27 and float is +$3,389.34; positive float must not hide realised losses or be presented as bot profit.

| Live external closing deal / position | Close SGT | Net USD |
| --- | --- | --- |
| 347147540 / 653631770 | 14:18:05.561 | -48.12 |
| 347148695 / 653631770 | 14:30:26.163 | -37.68 |
| 347196767 / 653855547 | 18:31:17.215 | -79.09 |

Exact account percentage returns, MTD/YTD and deposit-adjusted performance are not asserted: the required exact starting equity and all cashflows are not reconciled by this bounded audit.

## §3. Complete window S3 trade ledger

All twelve positions were shorts. All have exact opening/closing quantity reconciliation, broker stop/target acknowledgements and zero recorded commission, swap and conversion fee. The three Crude positions opened around 09:48 are the carryovers from №18,417; the other nine entries are new in this interval.

| Demo / position | Symbol; native units | Entry SGT → exit SGT | Fill entry → exit | Initial SL / TP | Exit classification | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 260862431 | SpotCrude; 8 | 09:48:00.052 → 14:43:31.041 | 86.358 → 85.594 | 86.893 / 85.619 | broker_protective_fill_at_or_better_than_tp | +6.11 / +1.4276 |
| 47790949 / 260862442 | SpotCrude; 271 | 09:48:00.720 → 14:43:31.021 | 86.338 → 85.594 | 86.873 / 85.599 | broker_protective_fill_at_or_better_than_tp | +201.62 / +1.3906 |
| 46130058 / 260862445 | SpotCrude; 274 | 09:48:00.757 → 14:43:31.042 | 86.338 → 85.594 | 86.873 / 85.599 | broker_protective_fill_at_or_better_than_tp | +203.86 / +1.3907 |
| 49236051 / 260963913 | SpotBrent; 10 | 15:42:58.657 → 16:39:24.515 | 93.685 → 93.687 | 94.15 / 93.043 | breakeven_stop_with_fill_slippage | -0.02 / -0.0043 |
| 47790949 / 260963925 | SpotBrent; 317 | 15:43:00.416 → 16:39:24.124 | 93.679 → 93.677 | 94.144 / 93.037 | breakeven_stop_with_fill_slippage | +0.63 / +0.0043 |
| 46130058 / 260971498 | SpotCrude; 406 | 16:14:51.447 → 16:32:51.533 | 85.577 → 85.064 | 85.945 / 85.069 | broker_protective_fill_at_or_better_than_tp | +208.28 / +1.3940 |
| 46130058 / 260982208 | SpotBrent; 193 | 17:00:20.884 → 17:45:46.880 | 93.471 → 92.391 | 94.251 / 92.394 | broker_protective_fill_at_or_better_than_tp | +208.44 / +1.3846 |
| 47790949 / 260982991 | SpotCrude; 328 | 17:01:38.186 → 17:34:27.948 | 85.476 → 84.817 | 85.925 / 84.856 | broker_protective_fill_at_or_better_than_tp | +216.15 / +1.4677 |
| 49236051 / 260982989 | SpotCrude; 10 | 17:01:38.195 → 17:34:27.882 | 85.476 → 84.817 | 85.93 / 84.849 | broker_protective_fill_at_or_better_than_tp | +6.59 / +1.4515 |
| 46130058 / 260983375 | ETHUSD; 9.59 | 17:02:32.625 → 19:15:47.725 | 2101.69 → 2102.06 | 2117.41 / 2079.99 | breakeven_stop_with_fill_slippage | -3.55 / -0.0235 |
| 47790949 / 260983374 | ETHUSD; 9.39 | 17:02:32.687 → 19:15:47.639 | 2101.69 → 2101.99 | 2117.41 / 2079.99 | breakeven_stop_with_fill_slippage | -2.82 / -0.0191 |
| 49236051 / 260983373 | ETHUSD; 0.3 | 17:02:32.688 → 19:15:47.665 | 2101.69 → 2102.06 | 2117.41 / 2079.99 | breakeven_stop_with_fill_slippage | -0.11 / -0.0233 |

Seven protective fills were at/beyond the TP; five were exits near the amended entry stop. cTrader’s protective order enum is STOP_LOSS_TAKE_PROFIT, so the target/stop distinction uses the recorded stop/TP and actual fill price. Entry-price stops produced four small net losses and one small gain; they do not guarantee net breakeven. Window win counts are 3/4, 3/4 and 2/4 across the three demos.

## §4. Favourable excursion, protection timing and giveback

| Account / position | Quote-only peak R | Highest documented R incl. closing fill | BE ack SGT | First recorded +1R quote → ack | Largest retained quote gap |
| --- | --- | --- | --- | --- | --- |
| 49236051 / 260862431 | 1.4280 | 1.4280 | 14:30:15.060 | 661.5 ms | 31.263 s |
| 47790949 / 260862442 | 1.3290 | 1.3907 | 14:30:29.056 | 484.1 ms | 31.196 s |
| 46130058 / 260862445 | 1.3907 | 1.3907 | 14:30:29.044 | 511.9 ms | 31.185 s |
| 49236051 / 260963913 | 1.3634 | 1.3634 | 16:33:24.492 | 681.0 ms | 9.891 s |
| 47790949 / 260963925 | 1.3505 | 1.3505 | 16:33:31.042 | 683.5 ms | 9.899 s |
| 46130058 / 260971498 | 1.3940 | 1.3940 | 16:30:19.190 | 694.6 ms | 7.073 s |
| 46130058 / 260982208 | 1.3846 | 1.3846 | 17:34:29.441 | 513.3 ms | 9.082 s |
| 47790949 / 260982991 | 1.4076 | 1.4677 | 17:33:21.450 | 534.1 ms | 6.527 s |
| 49236051 / 260982989 | 1.3921 | 1.4515 | 17:33:31.988 | 603.1 ms | 6.529 s |
| 46130058 / 260983375 | 1.2767 | 1.2767 | 18:27:45.099 | 458.9 ms | 5.962 s |
| 47790949 / 260983374 | 1.2767 | 1.2767 | 18:27:45.101 | 499.7 ms | 5.979 s |
| 49236051 / 260983373 | 1.2767 | 1.2767 | 18:27:45.132 | 532.5 ms | 5.983 s |

MFE uses the executable ask for shorts and the original funded stop distance. These are maxima of retained evidence, not a complete continuous broker path. In particular, carryover Crude 260862442 has quote-only peak 1.3290R, but its actual closing fill documents 1.3907R. Its quote stream therefore cannot prove the exact true peak. The JSON retains both values, tick counts, entry/exit boundary gaps and timestamps; true_continuous_peak_r remains null.

All twelve broker stop amendments equal the original entry and follow the first retained +1R quote. Their observed quote-to-ack delays range 458.9–694.6 ms. The nine new confirmations reached the final writer guard in 13.324–26.633 ms; recorded send-to-fill is 511–709 ms. These bounded samples do not measure every tick, socket-write latency or unused background preparation.

The three new ETH shorts reached a retained peak of 1.2767R and returned to the entry stop; net cash was −$6.48 across demos. The first two Brent shorts reached about 1.35–1.36R, then returned near entry for a combined +$0.61. This is baseline giveback behaviour. It does not show that the stop amendment was late, and it does not overturn the earlier backtest evidence that rejected the 0.80R→0.50R profit-lock candidate.

## §5. Entry activity, refusals and why accounts differed

| Window measure | Count | Scope |
| --- | --- | --- |
| New qualified/armed account-symbol records | 49 | Replicated account records, not independent alpha observations. |
| Opening intents / new fills / final peer refusals | 12 / 9 / 3 | Three BTC peer conflicts at 16:49:09 SGT, one per demo; refused before send. |
| Pair skips / pair disarms | 49 / 5 | Diagnostic records, not unique missed opportunities. |
| Macro veto diagnostics | 261 | Recorded evaluations, not every quote. |
| Deep-retracement invalidations | 23 | Beyond 61.8%; not a broker rejection. |
| Pullback touch records | 298 | Waiting-state observations; not 298 failures. |
| Unknown guard / uncertain / partial / broker-rejected entries | 0 / 0 / 0 / 0 | Twelve persisted opening intents reconciled. |
| All-stage unique min-lot / margin blocks | Not verifiable | Unused background preparation refusals are not durably counted. |

Account 46130058 captured a Crude TP at 16:32 and a later Brent TP at 17:45. Accounts 47790949 and 49236051 first captured a Brent entry-stop exit at 16:39, then a Crude TP at 17:34. Pair reservations, separate turn confirmations and different timing explain the different oil sequences. No evidence shows that account 47790949 lost a fill in this cohort because of its 1:25 margin setting. Small fill differences also explain why its Brent exit is +$0.63 while the small demo’s is −$0.02.

| Account group / symbol | 19:45 observed gate |
| --- | --- |
| Live oil | Both Crude and Brent invalidated after >61.8% retracement. |
| Live metals | XAUUSD yields to a human pending entry; XAGUSD is blocked by that pair reservation. |
| Live BTC / ETH | Background admission preparing; no stored refusal in this snapshot, and touch not yet recorded. Duration and cause cannot be inferred from one state. |
| All US500 / NAS100 | Waiting: opening impulse below 48 bp. |
| Demo oil / ETH | One entry already recorded per symbol/window. |
| Demo metals | 5d/20d macro signs disagree; explicit veto. |
| Demos 46130058 / 47790949 BTC | Armed, awaiting pullback/turn; no touch recorded. |
| Demo 49236051 BTC | Last prepared-admission refusal: min_volume_exceeds_trade_risk_budget; touch also not yet recorded. |

## §6. Minimum-volume risk versus margin: capital arithmetic

A demo minimum trade must fit the normal 0.6% trade-risk budget. The deployed code does not round a minimum order through the former 10% demo escape. The separate live minimum-position rule remains unchanged; the 0.6% capital comparisons here are the requested demo scenario. Account leverage changes margin requirements, not stop-loss dollars.

For broker-native volume v in hundredths of a unit, stop distance R_price and quote-to-deposit conversion fx: risk_cash = (v/100) × R_price × fx; risk-only minimum equity = risk_cash / 0.006. Round the minimum volume upward to its legal step. For an isolated flat 1:200 illustration only, margin = minimum notional/200 and a 250% floor needs equity ≥2.5×margin. Real admission also considers the existing book, pending orders, stress reserve, symbol margin tiers, freshness, ownership, cost, macro and peer gates.

| Actual new fills on 47790949 | Projected margin level | Stress margin level | 250% floor |
| --- | --- | --- | --- |
| SpotBrent / 260963925 | 1836.00% | 1814.76% | Passed |
| SpotCrude / 260982991 | 1931.23% | 1909.16% | Passed |
| ETHUSD / 260983374 | 1088.97% | 1069.20% | Passed |

The normal broker leverage reads show 47790949 at 1:25 and the other three accounts at 1:200, including live. These current receipts supersede older statement filename labels. The approximately $24.8k account is not the smallest account. Its observed admission margins were well above 250%; the two 1:200 demos have different equity and books, so leverage alone is not a controlled explanation for their P&L.

| Small-demo BTC conditional reconstruction | Value |
| --- | --- |
| Evidence time / account | 19:45:21.346 SGT / 49236051 |
| Minimum size | 0.01 BTC (native volume 1) |
| Current equity / 0.6% cash budget | $813.92 / $4.88352 |
| Reconstructed entry bound / R distance | 74307.12 / 530.91 USD/BTC |
| Minimum initial stop-risk dollars | $5.3091 |
| Required equity for this risk alone | $884.85 |
| Difference from observed equity | $70.93 |

This example combines the current recorded quote and running extreme with the most recent recorded zero commission component and USD conversion=1. The exact failed cache preparation inputs were not retained. Thus $884.85 is a conditional arithmetic result, not a verified rejected-order invoice, a deposit recommendation or a promise that a trade would trigger. Even at that equity, a missing touch, peer veto or another guard can still prevent entry.

| Symbol | Original arm records today | Risk-only equity range at minimum size, USD |
| --- | --- | --- |
| BTCUSD | 25 | 495.14–998.90 |
| ETHUSD | 18 | 17.92–25.52 |
| NAS100 | 8 | 1721.67–2456.67 |
| SpotBrent | 31 | 65.34–185.00 |
| SpotCrude | 24 | 58.67–136.00 |

These 106 records are the original arm snapshots; entry-bound, cost and native-grid geometry are reconstructed without altering Rust. Exact geometry was checked against all 21 actual filled setups and matched 21/21. The largest original-arm sensitivity is $2,456.67 for NAS100, assuming one isolated minimum position and quote/USD=1. The largest risk-only threshold among the 21 actual filled setups is $169.50. These answer different restricted questions. Later running extremes/spreads and unavailable refusal/book history mean neither is the exact minimum equity to clear every setup today. That requested universal field, and the all-day unique block count, are null.

## §7. Human protection and live whole-book exposure

| Protection law | Evidence | Conclusion |
| --- | --- | --- |
| Exact label ownership | Inspected S3 entry/protection events carry jiaoyi-s3; no live bot intent or live program execution event inside the window. | No recorded bot interference. |
| Yield to human position or pending entry | Live XAUUSD human pending entry holds XAUUSD and its XAGUSD pair. | Observed lock active. |
| Permanent human symbols | Compiled NatGas and JPN225 reservations remain. | No S3 authority. |
| NVDA.US | External ownership and outside S3 core. | Ownership protection; not an invented third permanent reservation. |

| Human/external position | Side / lots | SL | TP |
| --- | --- | --- | --- |
| JPN225 / 649992251 | buy / 0.4 | 56571.3 | Not set |
| JPN225 / 649772867 | buy / 1.2 | 56450.7 | Not set |
| NatGas / 637075128 | buy / 0.15 | 2.327 | 3.875 |
| NVDA.US / 652921796 | buy / 7.0 | 167.79 | Not set |

Four external positions and four pending entries remain in the snapshot. Two JPN225 short lifecycles changed externally during the audit, so human_positions_intact is not set to true. No complete broker amendment history certifies every human SL/TP unchanged. NVDA’s executable quote/protection check is unknown at this snapshot; a present stop field is not a fresh-price protection certificate.

The read-only observer reports current margin 544.48%, pending-entry projected margin 227.34% and stress projection -160.75%. Pending projections are below the 250% new-entry floor even though current margin is above it. They are forecasts for the observed external book, not permission to cancel human orders or proof that a particular S3 candidate was refused. Full exit-cost risk remains unverified.

## §8. NY 22:30 and separate crypto readiness

| Session group | Open SGT | Start / deadline | Prepared | Future opening references at audit |
| --- | --- | --- | --- | --- |
| Six traditional symbols × four accounts | 22:30 (09:30 New York) | 22:10 / 22:20 | 24/24 full coefficients + macro + base inputs | 0/24: future quote not observable |
| BTC / ETH × four accounts | 20:00 | 19:40 / 19:50 | 8/8 complete | 0/8: future quote not observable |
| Current London traditional window | 15:00 to 22:30 | Already active | 24/24 base; 0/24 complete late coefficients | 22/24 current references; two live metals withheld by human/pair locks |
| Current crypto window | 16:00 to 20:00 | Already active | 8/8 complete | 8/8 current references |

Current eligible London references retain the original first-six-second opening quotes. Their recorded preparation timestamps range 15:00:00.426–15:00:11.503 SGT; current crypto is 16:00:00.164–16:00:02.437. This observed opening cohort does not reproduce the former 31-minute readiness defect. Twelve transient missing-opening-quote diagnostics were later resolved; no later market price is substituted as the opening reference.

The remaining London coverage limitation is unchanged: 420/450-minute coefficients have six same-slot historical endpoints, below fourteen required after the seasonal NY-clock change. New unarmed band assessments after 21:30 can be blocked until the next window. Existing armed state and owned exits remain governed separately. NY itself has complete prepared coefficients; this report does not fabricate late London values or alter that historical minimum.

Prepared inputs are not an unconditional trade permission. NY must still establish its own opening quote, then pass human/pair reservations, impulse, macro, pullback/turn, peer, sizing and margin checks. At this audit there are no open S3 positions; live human XAUUSD still reserves the metals pair. Quote evidence exists for all 32 current core combinations, but a snapshot is not a promise of future freshness.

## §9. Exact progress towards the invariant goal

| S3 account | Latest 20 net wins | Change vs №18,417 | Gap to 73% | Wins short of 15/20 | Realised payoff | R-based PF | Net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 (15%) | +0 pp | 58 pp | 12 | 3.367 | 0.594 | -6.240 |
| 46130058 | 7/20 (35%) | +15 pp | 38 pp | 8 | 1.454 | 0.783 | -2.079 |
| 47790949 | 7/20 (35%) | +15 pp | 38 pp | 8 | 1.163 | 0.626 | -3.797 |
| 49236051 | 7/20 (35%) | +10 pp | 38 pp | 8 | 1.568 | 0.844 | -1.592 |

Twenty-trade win rates advance in five-point increments, so at least 15/20 net wins are needed to exceed 73%. All four remain below_target. These rolling windows contain older policies, and the demo trades share underlying signals. Neither one profitable window nor overlapping windows demonstrate consistency. The +1.38R configured gross target, realised payoff and R-based profit factor are different quantities. No S1 qualification is inferred.

## §10. Engineering worker and finite monitor

At 19:46:06 SGT the worker heartbeat was 1.2 seconds old; 30/36 daily sessions had been used, six remained. It was idle with owner_engineering_window_closed; job 926 had completed no_change at 19:43:13. The queue had 102 records, six pending reviews, two old owner-blocked releases and no pending verification. These are unresolved work states, not proof of an execution freeze or authority to edit production during this audit.

The earlier 24-hour monitor is completed: 12/12 checkpoints, no missed checkpoints, no model calls and no Telegram messages for that monitor. It has no next checkpoint. The Rust trading actors and routine engineering assessment are separate; this report neither extends the finite monitor nor silently changes its allowance. No incident report was sent to Telegram.

## §11. Findings and the next three checks

| Check | Evidence needed | Present conclusion |
| --- | --- | --- |
| 1. Opening readiness | Actual 20:00 crypto / 22:30 NY opening quotes and causal preparation, with ownership exceptions explicit. | Preparation passed before open; future opening prices were not yet observable at cutoff. |
| 2. Next genuine trade lifecycle | Entry guard, broker brackets, first +1R quote, amendment acknowledgement, close cash/quantity, account risk and margin. | This window reconciles nine new entries and twelve complete exits; preserve the proven infrastructure repairs. |
| 3. Economics and sizing evidence | Per-account latest-20 and policy attribution; for an exact future sizing audit, durable deduplicated admission transitions with geometry/equity/margin. | Goal still fails; unused admission history is a measurement gap, not grounds to bypass risk or force trades. |

The main remaining trading question is whether entries and retained exit geometry produce an independently validated net edge. Current ETH/Brent giveback is a concrete research observation; any revised exit proposal should be frozen and backtested with costs before deployment. The previously rejected early profit lock remains rejected. No observed fill in this audit calls for a rushed leverage, stop, ownership or peer-rule change.

Time is money: retain the verified fixes, separate missing evidence from a confirmed defect, and avoid spending sessions to manufacture activity. No environment publication is needed for a read-only report. Production, broker orders, risk limits and deployment remain unchanged.

## §12. Verification and instructions for Gemini

The companion JSON includes all 21 day-to-cutoff S3 lifecycles, the twelve window quote/protection summaries, decoded cash receipts, final guards, account books, source hashes, 106 original-arm sensitivity rows and source SQL. Focused verification checks cash, filled quantities, exact labels, cutoffs, recorded BE amendments, 21/21 geometry parity, count reconciliation, null handling and future/current session separation. Desktop/mobile HTML is checked for readable tables and no page-width overflow. Required check/docker/engineering CI gates the report-only PR.

Read the MD or HTML first, then the full JSON and short -gemini-review.json. Recompute +$1,045.18 and +$1,251.11; compare the 12 exits against original risk and closing fills; challenge MFE gaps and cost assumptions; distinguish conditional $884.85/$2,456.67 sizing examples from the unavailable all-day capital certificate. Verify 24 NY rows separately from eight crypto rows. Return findings with №18,420 and section references; do not count replicated demo trades as independent proof or invent missing refusal counts.

Source anchors at ef046993: crates/jiaoyi/src/strategy3.rs (size, Three Laws, load_day, exits); crates/jiaoyi/src/strategy3/fast_entry.rs (prepared admission and dispatch); crates/jiaoyi-core/src/strategy3/macro_pullback.rs (entry-bound rounding, geometry, breakeven); production_sessions.rs (calendar/preparation); ops/engineering/monitoring/dispatch_evidence.py and sql_evidence.py (guard and cash/coverage parsing). Source hashes identify private reads; credentials and redundant raw binary payloads are excluded.
