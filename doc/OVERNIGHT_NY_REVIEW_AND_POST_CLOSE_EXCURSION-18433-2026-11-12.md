# Overnight New York audit and post-close excursion review — №18,433

As of **12 November 2026, 04:50 SGT**. Activity: **11 November 19:45 inclusive to 12 November 04:50 exclusive**. Read-only audit; three report files only.

## §1. TL;DR

- **+$89.47 overnight S3 net cash from five closed positions:** ETH shorts lost **$290.78**; silver shorts made **$380.25**. Two wins, three losses; all five were demo positions. No S3 position remained open. Wednesday S3 closes through the cutoff totalled **−$854.38**; this positive window did not reverse the day's loss.
- **No stopped trade was rescued to its original TP in the nine elapsed four-hour recorded paths.** All nine revisited or exceeded their original stop. The afternoon BTC shorts recovered after exit, but did not reach their original targets; oil and ETH moved materially further against their shorts.
- **Two silver follow-ups are incomplete:** only 2h01m38s and 1h31m56s had elapsed by 04:50. Both had already realised their original target; subsequent quotes at/beyond that target are not rescued losing trades.
- **Telemetry reconciles:** 31 new arms, 15 consumed triggers, 15 individual outcomes, five fills and ten final peer refusals. Zero unmatched consumed triggers and zero unknown persisted entry-guard receipts.
- **4/4 actors connected and unpaused**, with zero recorded live-account bot interference. Live gold remains human-locked; its silver sibling remains pair-blocked.
- **Preparation passed for the actual native schedule:** 20/20 account-symbol slots at 07:00, 8/8 crypto slots at 08:00, and 4/4 SpotBrent slots at **09:00 SGT**. Future opening prices cannot yet be verified. Prepared historical inputs do not guarantee a qualifying setup or admissible size.
- **No trading deployment change.** Observed production remains `aebf404dedf89dbcdb8cee78c95a582eea042d51`; financial acceptance remains **false**.

## §2. Cutoff, provenance and what “exact” means

The first clock check was 04:54:52 SGT. SQL reads used read-only transactions and retained existing PostgreSQL evidence. Recorded history synchronisations subsequently covered the full audit interval. No historical market request, order, amendment, cancellation, strategy edit, configuration change or deployment was initiated by this audit.

The activity interval is `[2026-11-11T11:45Z, 2026-11-11T20:50Z)`. Account balance/float marks were observed at 04:54:52–04:54:53 SGT; inventory at 04:54:51; health at 04:58:57.882. They are labelled **post-cutoff snapshots**, not reconstructed atomic 04:50 marks. The ledger and all excursion calculations stop at the requested cutoff.

The quote copy contains **545,885 rows in 22 hashed segments**. Every receipt hash and row count was checked. Extrema use every recorded liquidation-side update, not chart candles. They are exact for the retained records; quote gaps prevent a claim of continuous tick-complete market coverage. A no-hit result means **not observed in that recorded path**. Endpoints use the last quote at/before the exact horizon, no more than six seconds old; each timestamp and age is in JSON.

The submitted full SHA contains a transposition (`...dedf89bcdb...`). Railway reports `aebf404dedf89dbcdb8cee78c95a582eea042d51`, consistent with the intended `aebf404d` release. The report uses the observed SHA.

## §3. Four-account snapshot and inventory

Margin values below come from the separately timestamped owner-assessment `current_margin_buffer` projection. The account snapshot itself leaves aggregate margin unverified for nonempty books; these percentages are not an atomic broker margin statement. Empty-book margin level is undefined. Equity is balance plus recorded net unrealised P&L, excluding unverified credit and potential future closing charges.


| Account | Balance USD | Float USD | Equity USD | Assessed margin % | S3 / other bot / human positions | Pending | Latest S3 20 wins |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 501.84 | 5,121.87 | 5,623.71 | 1,172.72 | 0 / 0 / 3 | 5 | 3/20 |
| 46130058 | 25,129.67 | 8.74 | 25,138.41 | 24,290.75 | 0 / 4 / 0 | 0 | 8/20 |
| 47790949 | 24,591.25 | 15.72 | 24,606.97 | 2,973.02 | 0 / 4 / 0 | 0 | 8/20 |
| 49236051 | 801.96 | 0.00 | 801.96 | Not verifiable | 0 / 0 / 0 | 0 | 8/20 |


Live latest-20 performance is an older cohort ending 29 October; it is not a test of this overnight release. Demo latest-20 values are overlapping rolling windows, not independent validation.


| Account | Position/order ID | Instrument | Ownership / kind | Side | Native volume / lots | Entry / trigger | Current SL | Current TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | human_or_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 56,571.30000 | Not verifiable |
| 43002148 | 649772867 | JPN225 | human_or_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 56,450.70000 | Not verifiable |
| 43002148 | 637075128 | NatGas | human_or_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.32700 | 3.87500 |
| 43002148 | 400465199 | XAUUSD | human_or_external / pending_entry | buy | 1000 / 0.1000 | 3,960.28000 | 3,877.30000 | 4,339.75000 |
| 43002148 | 400937857 | JPN225 | human_or_external / pending_entry | buy | 5000 / 0.5000 | 59,631.30000 | 59,366.50000 | Not verifiable |
| 43002148 | 398403207 | NatGas | human_or_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 | 400083120 | JPN225 | human_or_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Not verifiable |
| 43002148 | 400920027 | NatGas | human_or_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Not verifiable |
| 46130058 | 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,819.50000 | Not verifiable |
| 46130058 | 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69978 | Not verifiable |
| 46130058 | 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12869 | Not verifiable |
| 46130058 | 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,554.90000 | Not verifiable |
| 47790949 | 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,819.50000 | Not verifiable |
| 47790949 | 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69973 | Not verifiable |
| 47790949 | 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12864 | Not verifiable |
| 47790949 | 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,554.90000 | Not verifiable |


The two larger demos also hold four S1 positions each: AUS200, AUDUSD, EURUSD and GER40. EURUSD and GER40 were new S1 entries during this interval. Those eight open positions must not be described as S3 inactivity, nor their floating P&L treated as S3 realised cash. No S1 closing deal or human/external closing deal was recorded in this audit window.

The live pending-order stress projection is a separate concern: current assessed margin is 1,172.72%, ordinary pending projection 355.01%, but the stored pending stress projection is −168.16% and marked breach. This is an existing advisory assessment of human orders, with incomplete cost evidence; it does not authorise bot changes to those orders.

## §4. Three Laws and live metals reservation

Live account 43002148 has **three human/external positions and five human/external pending entries**. Their IDs, quantities and SL/TP match the previous №18,431 inventory snapshot. This window contains zero live trading intents, zero recorded live trading events and zero live closing deals. Thus **recorded bot interference = 0**, within the retained-evidence scope; unrecorded broker activity is not certified.

- **Label ownership:** all five S3 entry/exit lifecycles are attributable to `jiaoyi-s3`. S1 amendments retain their separate `jiaoyi-s1` ownership. No bot authority over live external objects was observed.
- **Yield to human:** XAUUSD pending entry `400465199` retains its human lock. There were 52 recorded gold yield diagnostics.
- **Pair reservation:** XAGUSD was blocked by the XAUUSD sibling on the live account, with 27 recorded pair diagnostics; the runtime gate confirms the sibling name. These are diagnostics, not 27 separate reservations.
- **Permanent reservation:** NatGas and JPN225 remain human-reserved. Their live positions/orders and protection levels were unchanged. Gold is dynamically human-locked, not permanently reserved.

## §5. Overnight S3 position ledger

All five opened and fully closed inside the activity interval. No S3 position carried into 19:45 or remained open at 04:50. All sides are **short**. Native cTrader volume is hundredths of a unit; lot conversion uses the stored contract lot size, not a cross-asset assumption. Cash is broker closing P&L net of recorded swap, commission and conversion fee.


| Account / position | Symbol | Native volume / lots | Entry SGT | Entry price | Exit SGT | Exit price | Initial SL / TP | Net USD |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 261735093 | ETHUSD | 39 / 0.3900 | 11 Nov 21:04:16.280 | 2,046.77000 | 11 Nov 21:54:54.236 | 2,059.75000 | 2,058.96000 / 2,029.94000 | -5.06 |
| 46130058 / 261735098 | ETHUSD | 1233 / 12.3300 | 11 Nov 21:04:17.524 | 2,046.76000 | 11 Nov 21:54:53.855 | 2,058.47000 | 2,058.95000 / 2,029.93000 | -144.38 |
| 47790949 / 261735099 | ETHUSD | 1207 / 12.0700 | 11 Nov 21:04:17.495 | 2,046.76000 | 11 Nov 21:54:53.843 | 2,058.47000 | 2,058.95000 / 2,029.93000 | -141.34 |
| 46130058 / 261769291 | XAGUSD | 45000 / 0.0900 | 11 Nov 22:40:16.515 | 52.32300 | 12 Nov 02:48:21.915 | 51.89800 | 52.63000 / 51.89900 | +191.25 |
| 47790949 / 261769290 | XAGUSD | 35000 / 0.0700 | 11 Nov 22:40:16.432 | 52.32300 | 12 Nov 03:18:03.600 | 51.78300 | 52.70000 / 51.80200 | +189.00 |



| Account / position | Initial R price | Initial risk USD | Recorded MFE / MAE R | Actual net R | Exit classification | Peer at guard bps | Confirmation → guard ms |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 261735093 | 12.19000 | 4.75 | 0.0353 / 1.0607 | -1.0643 | initial_stop_loss | 28.752 | 18.681 |
| 46130058 / 261735098 | 12.19000 | 150.30 | 0.0345 / 1.0000 | -0.9606 | initial_stop_loss_with_price_improvement | 28.280 | 13.869 |
| 47790949 / 261735099 | 12.19000 | 147.13 | 0.0345 / 1.0000 | -0.9606 | initial_stop_loss_with_price_improvement | 28.280 | 18.270 |
| 46130058 / 261769291 | 0.30700 | 138.15 | 1.3974 / 0.3192 | +1.3844 | broker_protective_fill_at_or_better_than_tp | 25.825 | 14.818 |
| 47790949 / 261769290 | 0.37700 | 131.95 | 1.4297 / 0.2599 | +1.4324 | broker_protective_fill_at_or_better_than_tp | 25.825 | 12.816 |


ETH entries at about 21:04 SGT achieved only **0.034–0.035R** favourable excursion before their 21:54 stops. Two stops filled inside the nominal stop level; broker protective order type 4, lifecycle context and proximity to the recorded stop establish price improvement rather than an unexplained discretionary exit.

Silver entries at 22:40 SGT reached their targets at **02:48 and 03:18 SGT**. Both received a recorded breakeven stop amendment after the baseline threshold. The two accounts had different recorded initial risk distances (0.307 and 0.377), and therefore different targets despite the same entry fill. These were the existing admitted geometries, not an audit modification. Their stored native exit deadline was **05:59 SGT**, not entry+4h; target exits after more than four hours were within the recorded deadline. The four-hour post-close diagnostic does not change that policy.

The small demo did not mirror silver: its first two arms invalidated after >61.8% retracement; a subsequent arm remained subject to admission checks. The post-cutoff runtime snapshot reports `min_volume_exceeds_trade_risk_budget` for XAGUSD. This is a risk-budget block, not evidence that lot size can always be reduced below the broker minimum.


| Account | Closed positions | Wins / losses | Window net USD |
| --- | --- | --- | --- |
| 43002148 | 0 | 0 / 0 | +0.00 |
| 46130058 | 2 | 1 / 1 | +46.87 |
| 47790949 | 2 | 1 / 1 | +47.66 |
| 49236051 | 1 | 0 / 1 | -5.06 |


## §6. Post-close excursion definitions

Each path starts immediately after the broker exit execution timestamp and ends exactly four hours later, clipped to 04:50 SGT. **The anchor is the actual exit fill**, not the entry. For a short, a lower ask is favourable. Favourable/adverse excursions are nonnegative magnitudes; checkpoint changes are signed, positive for movement favourable to the original trade. R is always the original initial price-risk distance.

Stored contract pips: BTCUSD/ETHUSD **0.1 price units per pip**; SpotCrude/SpotBrent/XAGUSD **0.01**. For example, a $100 BTC move is 1,000 broker pips here. The original TP test compares the actual price with the actual original target. A 1.38R favourable move from a losing exit is not the same as recovering to a +1.38R target from entry.

“Held to TP” below is only a price-touch diagnostic that ignores the already-triggered exit. It is not a backtested rule, a fill guarantee or evidence that an unchanged stop could have survived. No stop or target was altered.

## §7. Four-hour maximum movement after every required close


| Position / symbol | Cohort | Window elapsed h | Favourable price / pips / R | Adverse price / pips / R | Original TP observed? | Original SL crossed? |
| --- | --- | --- | --- | --- | --- | --- |
| 261629338 / BTCUSD | afternoon follow-up | 4.000 complete | 676.740 / 6,767.400 / 2.0801 | 58.950 / 589.500 / 0.1812 | Not observed | Yes |
| 261629441 / BTCUSD | afternoon follow-up | 4.000 complete | 691.460 / 6,914.600 / 2.1253 | 60.410 / 604.100 / 0.1857 | Not observed | Yes |
| 261629442 / BTCUSD | afternoon follow-up | 4.000 complete | 691.460 / 6,914.600 / 2.1253 | 60.410 / 604.100 / 0.1857 | Not observed | Yes |
| 261636289 / SpotCrude | afternoon follow-up | 4.000 complete | 0.092 / 9.200 / 0.2054 | 0.954 / 95.400 / 2.1295 | Not observed | Yes |
| 261636412 / SpotBrent | afternoon follow-up | 4.000 complete | 0.076 / 7.600 / 0.1246 | 1.160 / 116.000 / 1.9016 | Not observed | Yes |
| 261636417 / SpotBrent | afternoon follow-up | 4.000 complete | 0.021 / 2.100 / 0.0344 | 1.215 / 121.500 / 1.9918 | Not observed | Yes |
| 261735093 / ETHUSD | overnight | 4.000 complete | 16.830 / 168.300 / 1.3806 | 43.520 / 435.200 / 3.5701 | Not observed | Yes |
| 261735098 / ETHUSD | overnight | 4.000 complete | 15.550 / 155.500 / 1.2756 | 44.800 / 448.000 / 3.6751 | Not observed | Yes |
| 261735099 / ETHUSD | overnight | 4.000 complete | 15.550 / 155.500 / 1.2756 | 44.800 / 448.000 / 3.6751 | Not observed | Yes |
| 261769291 / XAGUSD | overnight | 2.027 PARTIAL | 0.142 / 14.200 / 0.4625 | 0.039 / 3.900 / 0.1270 | Yes, already TP exit | Not observed (partial) |
| 261769290 / XAGUSD | overnight | 1.532 PARTIAL | 0.027 / 2.700 / 0.0716 | 0.150 / 15.000 / 0.3979 | Yes, already TP exit | Not observed (partial) |


Nine completed windows: **0/9 original TP touches; 9/9 original SL crossings**. Both partial silver windows contain quotes at or beyond their already-achieved TP; they must not be counted as rescued stopped trades. The silver four-hour deadlines are **06:48:21.915 and 07:18:03.600 SGT**. Their outstanding horizons remain unmeasured in this 04:50 report.

## §8. Signed changes at one, two and four hours

Each cell is **price / broker pips / original R**, relative to the actual exit fill. No price after the cutoff is used. `Not elapsed` is an unavailable future endpoint, not zero movement.


| Position / symbol | Exit +1h | Exit +2h | Exit +4h |
| --- | --- | --- | --- |
| 261629338 / BTCUSD | +79.330 / +793.300 / +0.2438 | +199.960 / +1,999.600 / +0.6146 | +531.690 / +5,316.900 / +1.6343 |
| 261629441 / BTCUSD | +87.900 / +879.000 / +0.2702 | +256.290 / +2,562.900 / +0.7878 | +638.320 / +6,383.200 / +1.9620 |
| 261629442 / BTCUSD | +87.900 / +879.000 / +0.2702 | +256.290 / +2,562.900 / +0.7878 | +638.320 / +6,383.200 / +1.9620 |
| 261636289 / SpotCrude | -0.616 / -61.600 / -1.3750 | -0.323 / -32.300 / -0.7210 | -0.553 / -55.300 / -1.2344 |
| 261636412 / SpotBrent | -0.698 / -69.800 / -1.1443 | -0.424 / -42.400 / -0.6951 | -0.249 / -24.900 / -0.4082 |
| 261636417 / SpotBrent | -0.783 / -78.300 / -1.2836 | -0.489 / -48.900 / -0.8016 | -0.309 / -30.900 / -0.5066 |
| 261735093 / ETHUSD | +9.370 / +93.700 / +0.7687 | -16.120 / -161.200 / -1.3224 | -34.180 / -341.800 / -2.8039 |
| 261735098 / ETHUSD | +8.150 / +81.500 / +0.6686 | -17.400 / -174.000 / -1.4274 | -35.460 / -354.600 / -2.9089 |
| 261735099 / ETHUSD | +8.130 / +81.300 / +0.6669 | -17.400 / -174.000 / -1.4274 | -35.460 / -354.600 / -2.9089 |
| 261769291 / XAGUSD | +0.005 / +0.500 / +0.0163 | +0.046 / +4.600 / +0.1498 | not_elapsed_at_cutoff |
| 261769290 / XAGUSD | -0.023 / -2.300 / -0.0610 | not_elapsed_at_cutoff | not_elapsed_at_cutoff |


## §9. What the paths support

**BTC: recovery after exit, but not the original target.** Afternoon shorts later moved 2.080–2.125R favourably from their losing exits. After allowing for the loss already incurred at exit, the best subsequent quote was still below the original +1.38R profit objective. All three also traded beyond their original stops during the follow-up. This is a diagnostic of timing and path dependence, not evidence to remove stops.

**Oil: stops prevented additional adverse exposure in this observation.** Crude went another 2.129R against the original short from its exit; Brent went another 1.902–1.992R. Four-hour endpoint changes remained adverse. No original oil target was observed. These paths support the earlier finding that the 17:14 entries failed locally; they do not support simply extending those losses in hope of recovery.

**ETH: weak follow-through at entry, followed by substantial adverse continuation.** Overnight ETH shorts barely became favourable before stopping. The later maximum favourable movement from their exits was 1.276–1.381R, but still did not recover to the original target. The adverse extension reached 3.570–3.675R from exit. Four-hour net movement was approximately −2.80 to −2.91R. A target-recovery claim based only on the post-exit favourable maximum would be mathematically wrong.

**Silver: the existing target captured a realised gain.** The first silver trade had a further observed favourable extension of 0.463R after exit, the second 0.072R; their partial paths also retraced. Neither has a completed four-hour observation in this report. This does not establish that extending all targets would improve expectancy.

These 11 positions are correlated account copies of a small number of market episodes. They are not 11 independent tests. The five overnight positions represent two shared entry ideas, so 2/5 wins and roughly 1.31 cash profit factor cannot establish a durable edge. No new exit parameter is proposed from this sample; the separate №18,432 pure-0.80R BE experiment remains rejected on its own evidence.

## §10. Vetoes, consumed triggers and telemetry integrity


| Metric | Observed value | Denominator / meaning |
| --- | --- | --- |
| New arms | 31 | Account-symbol armed_pullback receipts; existing carried arms can produce later triggers |
| Consumed triggers | 15 | Distinct pullback_trigger_consumed decision IDs |
| Individual terminal outcomes | 15 | Each joins to exactly one consumed ID and one persisted opening intent |
| Persisted S3 opening intents | 15 | All are resolved: five filled and ten refused before sending |
| Entries dispatched / first fills | 5 | Five owned opening fills; protective closing fills are not counted as new entries |
| Final peer refusals | 10 | Nine opposing peers; one same-direction peer below 24 bps |
| Deep retracement invalidations | 10 | Recorded >61.8% invalidation state transitions |
| Pair reservation diagnostics | 115 | 113 skip diagnostics plus two state receipts; not 115 unique risk reservations |
| Macro-veto diagnostics | 716 | Deduplicated/re-emitted diagnostic rows, not 716 executable trades |
| Unmatched consumed triggers | 0 | Exact consumed ID → outcome ID join, including repeated refusal reasons |
| Unknown entry-guard receipts | 0 | Monitor parser found no invalid/unknown persisted-intent guard receipts |
| Largest confirmation → final guard | 18.681 ms | Five successful-entry samples; not a bound on unobserved failures or network fill latency |



| Account | New arms | Consumed / outcomes | Fills | Peer refusals | Deep invalidations | Pair diagnostics |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 6 | 0 / 0 | 0 | 0 | 6 | 27 |
| 46130058 | 8 | 5 / 5 | 2 | 3 | 1 | 44 |
| 47790949 | 8 | 5 / 5 | 2 | 3 | 1 | 44 |
| 49236051 | 9 | 5 / 5 | 1 | 4 | 2 | 0 |


Exact trigger-to-outcome joins:


| Consumed ID | Outcome ID | Account / instrument | Consumed SGT | Terminal status / reason |
| --- | --- | --- | --- | --- |
| 92456 | 92464 | 49236051 / ETHUSD | 11 Nov 20:19:56.455 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 92457 | 92463 | 46130058 / ETHUSD | 11 Nov 20:19:56.455 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 92458 | 92462 | 47790949 / ETHUSD | 11 Nov 20:19:56.459 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 92996 | 93005 | 49236051 / ETHUSD | 11 Nov 21:04:15.811 | filled / Filled { position_id: Some(261735093), price: Some(2046.77), volume: Some(39) } |
| 93000 | 93010 | 46130058 / ETHUSD | 11 Nov 21:04:17.135 | filled / Filled { position_id: Some(261735098), price: Some(2046.76), volume: Some(1233) } |
| 93001 | 93007 | 47790949 / ETHUSD | 11 Nov 21:04:17.143 | filled / Filled { position_id: Some(261735099), price: Some(2046.76), volume: Some(1207) } |
| 94121 | 94123 | 49236051 / BTCUSD | 11 Nov 22:38:02.302 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 94197 | 94206 | 46130058 / XAGUSD | 11 Nov 22:40:15.937 | filled / Filled { position_id: Some(261769291), price: Some(52.323), volume: Some(45000) } |
| 94198 | 94202 | 47790949 / XAGUSD | 11 Nov 22:40:15.942 | filled / Filled { position_id: Some(261769290), price: Some(52.323), volume: Some(35000) } |
| 94312 | 94315 | 47790949 / BTCUSD | 11 Nov 22:48:08.056 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 94313 | 94318 | 49236051 / BTCUSD | 11 Nov 22:48:08.058 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 94314 | 94317 | 46130058 / BTCUSD | 11 Nov 22:48:08.061 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 94669 | 94673 | 46130058 / BTCUSD | 11 Nov 23:12:16.259 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 94670 | 94672 | 49236051 / BTCUSD | 11 Nov 23:12:16.260 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 94671 | 94674 | 47790949 / BTCUSD | 11 Nov 23:12:16.260 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |


Repetitive idle/pre-admission diagnostics remain deduplicated. Their counts are not exhaustive per-tick refusal counts; the monitor explicitly reports all-stage refusal completeness as false. The individual consumed-trigger proof is narrower and passes. The 106 S1 stop amendments in the window belong to S1 and were excluded from S3 counts. Repeated refusal strings did not erase any of the 15 consumed-trigger outcomes.

## §11. Thursday readiness at the actual native openings


| Open SGT | Symbols | Prepared account-symbol slots | Future opening reference |
| --- | --- | --- | --- |
| 07:00 | SpotCrude, US500, NAS100, XAUUSD, XAGUSD | 20/20 | Not yet observable |
| 08:00 | BTCUSD, ETHUSD | 8/8 | Not yet observable |
| 09:00 | SpotBrent | 4/4 | Not yet observable |


All 32 next-session preparations had metadata, spread and commission readiness, zero missing sigma marks, no source error and no macro-context error before the cutoff. Traditional preparation receipts were recorded about 22:30 SGT; crypto at 04:00 SGT. The broker-native SpotBrent opening recorded in all four preparations is 09:00 SGT, so reporting “24/24 traditional symbols ready for 07:00” would conceal that calendar distinction.

The 04:58 health snapshot confirms four connected, unpaused actors, zero unresolved open attempts and zero reported actor quote/execution receiver lag since their recorded connection generation. Existing current-session opening references are present. This does not predict Thursday's first quote: the native-open capture, fresh peer quote, macro, pullback, human ownership, size and margin checks still have to pass at the time of a new trade.

Specific post-cutoff runtime blockers remain visible: live XAUUSD is human-locked and XAGUSD sibling-blocked; small-demo XAGUSD reports minimum volume above its risk budget. Small-demo SpotCrude reports an armed quote too stale/future for preparation, while other Crude actors show preparation not ready. These are current entry-admission diagnostics, distinct from the successful next-session historical-input preparations. They are not silently converted into “all instruments ready to trade”.

## §12. Data quality and verification


| Position | Post-close updates | Largest update gap s | First / tail gap s | 1h quote age s | 2h quote age s | 4h quote age s |
| --- | --- | --- | --- | --- | --- | --- |
| 261629338 | 59895 | 13.891 | 0.050 / 0.164 | 0.013 | 0.203 | 0.164 |
| 261629441 | 59813 | 13.883 | 0.028 / 0.197 | 0.052 | 0.046 | 0.197 |
| 261629442 | 59813 | 13.887 | 0.035 / 0.190 | 0.047 | 0.037 | 0.190 |
| 261636289 | 18473 | 14.567 | 0.294 / 2.513 | 0.043 | 2.745 | 2.513 |
| 261636412 | 13813 | 26.028 | 0.073 / 2.512 | 1.731 | 0.629 | 2.512 |
| 261636417 | 13811 | 26.031 | 0.137 / 1.933 | 0.058 | 2.158 | 1.933 |
| 261735093 | 50841 | 4.243 | 0.112 / 0.822 | 0.006 | 0.016 | 0.822 |
| 261735098 | 50843 | 4.224 | 0.084 / 0.415 | 0.031 | 0.240 | 0.415 |
| 261735099 | 50888 | 4.167 | 0.105 / 0.394 | 0.212 | 0.229 | 0.394 |
| 261769291 | 9817 | 17.862 | 0.100 / 0.542 | 1.731 | 0.296 | Not elapsed |
| 261769290 | 7444 | 17.859 | 0.156 / 0.580 | 0.565 | Not elapsed | Not elapsed |


Every elapsed 1h/2h/4h endpoint has a quote no more than six seconds old. No interpolated endpoint or future price was substituted. Post-close update gaps reached about 26 seconds in oil; in-position quote gaps reached about 21 seconds in silver. These gaps constrain claims about unobserved excursions and fill timing. Recorded same-timestamp price alternatives are retained for extrema; all actual checkpoint timestamps are inspectable in JSON.

Successful seven-day deal refreshes at 04:54 cover the audit start and cutoff on every account. Their `complete=false` field marks a recent mutable refresh, as explicitly set by the recent-poll implementation; it is not a failed history request. Coverage was checked from `status=ok`, no error and the actual requested interval, and reconciled with owned fills and the next inventory snapshot.

Focused checks cover signed long/short movement, broker pip conversion, checkpoint cutoff/freshness, all receipt SHA256/row counts, cash reconciliation, one-to-one trigger outcomes, all 32 preparations and the nine-complete/two-partial split. Browser checks cover desktop and narrow mobile layouts. Required repository CI is `check`, `docker`, `engineering`; final PR and merge evidence is supplied with publication rather than invented in advance.

## §13. Production boundary and next observations

The report branch changes exactly three paths under `doc/`. Railway's Rust deployment watch patterns exclude them. Before publication, Railway showed deployment `c0fedaca-0370-4cec-9234-f59f8c7cafce`, created 11 November 08:54:06 SGT, image `sha256:631e668c1ca138d5810f22fdab1ba27f3b7f4f34c6a28fa544c7ad09c2a9fad3`. The same deployment identity must remain active after the docs merge. No deployment command is used.

The next factual checkpoints are the two incomplete silver horizons at 06:48 and 07:18, the 07:00 native traditional opening, 08:00 crypto and 09:00 Brent. This report does not schedule a new unattended monitor or change execution rules. A subsequent audit can use those newly elapsed records; it must preserve this report's 04:50 cutoff and distinguish new evidence from retrospective changes.

**Financial pass: false.** Operations and traceability have evidence of functioning; the record does not establish independent positive expectancy, the account win-rate target or a justified exit-policy change.

## §14. Gemini review instructions and evidence index

Read this Markdown report, its HTML companion and the JSON at the same repository commit. Start with §§1, 5, 7–11. In JSON inspect `ledger`, `telemetry.trigger_outcome_joins`, `readiness`, `human_protection`, `quote_receipts`, `source_hashes`, `methodology` and `analysis_reproduction`.

Specifically challenge: (1) exit-anchored R versus the original TP level; (2) nine elapsed paths versus two censored silver paths; (3) repeated account copies versus independent market episodes; (4) next-session preparation versus current entry permission; (5) observational post-close prices versus a survivable stop-preserving counterfactual. The three published files are sufficient to inspect every reported value and calculation rule. Independent raw-quote regeneration requires authorised access to the private stored data; hashes alone cannot prove that the raw data are correct.

Private input/source integrity hashes:


| Source file | SHA256 |
| --- | --- |
| preflight.json | c3ed5ac59c2995caf461a6b43b72e8b82ade9d54ddd7842414dc0744937b593a |
| monitor.json | 99c66a59758c1621ac3daa74049926719027bddbb7408c449f2606e8796a1ff1 |
| quote-copy-plan.json | 8d3eda1e24a4774c9498127e8bd60ba3b27b0423a98d5d83751bc26cffb2a5b9 |
| health.json | 4451d83bcd4eab573ca355636278a01e964c99ee323af5fd274d6be6fe1aa382 |
| quote-copy-receipts.json | 64b5e4a06ecf5550425a5a06fc925c8e11538fc4a1d6f45de2fe46406390a686 |
| activity.json | 17d87f1491d08290490d39d214f1cdefae913147909a2aeca2d34a48b3ebf861 |
| evidence.json | 8d850d5d4a002790c8695903a55793ef482c52b7c94a28b4fc286abaa684d722 |
| deployment-before.json | 6cab90a4e75ee1ed9ad74d89b5cb42a8c91d062e0a39cbe9e231c04b4049604a |
| runtime.json | a2dd5c7ad6e521e5cba5a2cb231ed31edc8810782187387f149f81dc6fd0f863 |


Quote export receipts (existing PostgreSQL records only):


| Account / symbol | Window start UTC | Through exclusive UTC | Rows | Uncompressed CSV SHA256 |
| --- | --- | --- | --- | --- |
| 47790949 / BTCUSD | 2026-11-11T09:03:27.033000+00:00 | 2026-11-11T13:03:27.033000+00:00 | 60447 | b278bccb53e757186da909892811281a8c603ef1fa4b6834879a75548e4e0cc5 |
| 47790949 / BTCUSD | 2026-11-11T13:03:27.033000+00:00 | 2026-11-11T14:31:09.756001+00:00 | 21537 | ab8ceeb070f3915f4a287e838174ec7c95d8a93814d9ee2596483ce5ef8e0fc7 |
| 49236051 / BTCUSD | 2026-11-11T09:03:37.091000+00:00 | 2026-11-11T13:03:37.091000+00:00 | 60383 | b6682e8ccce68fc26f3c82905151d8d581d8828fe2e35d8d04867b2f75f61493 |
| 49236051 / BTCUSD | 2026-11-11T13:03:37.091000+00:00 | 2026-11-11T14:32:29.623001+00:00 | 21818 | 9a6f7389fa3bfa6d0d839e2e4584fe0d7c8940cc169a275b1c205df6f9c92a90 |
| 46130058 / BTCUSD | 2026-11-11T09:03:37.076000+00:00 | 2026-11-11T13:03:37.076000+00:00 | 60383 | d2a756d5120dcbe1f47d50d588c6dcae40eabeac83d730d76b61cbe854b7880a |
| 46130058 / BTCUSD | 2026-11-11T13:03:37.076000+00:00 | 2026-11-11T14:32:29.615001+00:00 | 21818 | ee35f00393b7ed5b8944fbbddc48fb97e9ee1af734c20be861773886640e82fb |
| 46130058 / SpotCrude | 2026-11-11T09:14:14.985000+00:00 | 2026-11-11T13:14:14.985000+00:00 | 18729 | d828ec8675691d68046d183c26148a4c0de49983669819b3dbb350786549942e |
| 46130058 / SpotCrude | 2026-11-11T13:14:14.985000+00:00 | 2026-11-11T13:16:39.746001+00:00 | 201 | 575eed5588e9dd62feda037acaeebb2d02b52e376f8c7df9d77f428aeff594c7 |
| 49236051 / SpotBrent | 2026-11-11T09:14:31.517000+00:00 | 2026-11-11T13:14:31.517000+00:00 | 14062 | d7b01c20e511308f3555f49d7f353bd0717d46132f713728270462a26c3717ba |
| 49236051 / SpotBrent | 2026-11-11T13:14:31.517000+00:00 | 2026-11-11T13:18:04.173001+00:00 | 204 | 77be2e6d2c5b69e027d57fd06420e5c9e935188a9d55eb799c13c32be716654d |
| 47790949 / SpotBrent | 2026-11-11T09:14:32.559000+00:00 | 2026-11-11T13:14:32.559000+00:00 | 14048 | 6a0c52acc003ca2254751cf5ce9ac58480fa95023c3fe81f88f632c2abcb9583 |
| 47790949 / SpotBrent | 2026-11-11T13:14:32.559000+00:00 | 2026-11-11T13:18:00.284001+00:00 | 203 | e2de75f7377aa91fdf826c8ec8a6bfb511a1eda8af11075cb948f427f359a44a |
| 49236051 / ETHUSD | 2026-11-11T13:04:10.280000+00:00 | 2026-11-11T17:04:10.280000+00:00 | 49481 | e61a4a2856da82fc97ab7d3fc6dd9e2417d3b012cf6a02ad1f7423ba3cfde099 |
| 49236051 / ETHUSD | 2026-11-11T17:04:10.280000+00:00 | 2026-11-11T17:54:54.236001+00:00 | 10369 | 34638ba08f8a9bde1f9a0ee71ffa63af5b1ed5739b555944c5207a10793a09f5 |
| 46130058 / ETHUSD | 2026-11-11T13:04:11.524000+00:00 | 2026-11-11T17:04:11.524000+00:00 | 49483 | cdda0c3041bef594a726762f595566d3726a4c8b01dd9050f5d18ea4121a3457 |
| 46130058 / ETHUSD | 2026-11-11T17:04:11.524000+00:00 | 2026-11-11T17:54:53.855001+00:00 | 10366 | 22dd9bdd61552f9a58198ab364c89811d300b813bc21a2020bcdb82c384b4ec7 |
| 47790949 / ETHUSD | 2026-11-11T13:04:11.495000+00:00 | 2026-11-11T17:04:11.495000+00:00 | 49528 | 0c54f08fde3317ecf40e769d133c548d6c7cf2cc62bfed6995f97ded8c2d03a4 |
| 47790949 / ETHUSD | 2026-11-11T17:04:11.495000+00:00 | 2026-11-11T17:54:53.843001+00:00 | 10376 | fc491e0f550149bf300f90c5a25a648e52f93c6d7d6261790878300b21128d98 |
| 46130058 / XAGUSD | 2026-11-11T14:40:10.515000+00:00 | 2026-11-11T18:40:10.515000+00:00 | 25716 | 5733e9a6557707da9eb5c32aadcb79b6c102f7f0786c51ef5f6db771eb33ea42 |
| 46130058 / XAGUSD | 2026-11-11T18:40:10.515000+00:00 | 2026-11-11T20:50:00+00:00 | 10507 | 218925d3fe0abc67d1aa4016fb7375621ce517a3d63174dd36fefdae07b03cd3 |
| 47790949 / XAGUSD | 2026-11-11T14:40:10.432000+00:00 | 2026-11-11T18:40:10.432000+00:00 | 25718 | d1089328563e66e607a42d2c449c58f4a3f32bfedd632bfa6eed64d7610153f6 |
| 47790949 / XAGUSD | 2026-11-11T18:40:10.432000+00:00 | 2026-11-11T20:50:00+00:00 | 10508 | 22463122614c32f2e6a6486557dfeb310a89d20f5d0e38b643ed80b67ca31c25 |
