# Thursday mid-day Asian and crypto review; London preparation — №18,435

**As of 12 November 2026, 12:55 SGT.** Activity interval: **04:50 inclusive to 12:55 exclusive**, continuing from [№18,433](OVERNIGHT_NY_REVIEW_AND_POST_CLOSE_EXCURSION-18433-2026-11-12.md). Read-only production audit.

## §1. TL;DR

- **Morning S3 realised cash: −$42.85**, from four ETH shorts, one per account. Three net losses and one cash-flat close; all exited at the existing **08:00 session deadline**, not a stop or breakeven trigger. No S3 position remained open; S3 floating P&L is **$0**.
- **Thursday 00:00–12:55 SGT S3 realised cash: +$337.40**, including the two overnight silver targets (+$380.25). Do not confuse this day total with the morning window.
- **SpotCrude opening capture succeeded 4/4.** Recorded first quotes arrived **7.115–7.180 seconds** after 07:00, outside the old six-second limit and inside the repaired 15-second policy. Runtime references match those exact recorded quotes.
- **A telemetry gap remains:** initial `s3_session` refusal records can remain after a successful in-memory retry. Reading only those rows would incorrectly report an opening failure. Source inspection shows the retry updates `Day` and returns without replacing that initial session receipt. No production repair was made during this audit.
- **14 consumed triggers → 14 persisted intents → 14 individual outcomes:** four fills, four opposing-peer refusals and six below-24bp peer refusals. No unmatched outcome and no unknown persisted entry-guard receipt.
- **London base preparation: 24/24; full sigma tables: 0/24.** Every London table has valid 30–390-minute marks, but 420/450-minute marks have only **8 endpoints vs 14 required**. This is a late-session coverage gap, not a blanket block on the 15:00 opening. Crypto 16:00 preparation is **8/8 complete**.
- Both previously partial silver follow-ups have now elapsed. The first ends during the maintenance break: its exact +4h price is **unavailable**, not fabricated. The second's +4h move is **+33.3 pips / +0.8833R** in the original short direction. All four morning ETH follow-ups completed without touching their original TP or SL in recorded quotes.
- **Live human protection:** zero recorded bot interference. Human-position stops changed and one pending order disappeared; initiating external actors are not proven. A new live GER40 position is explicitly **S1-owned**, not human or S3.
- **Financial pass remains false.** Latest20 wins are 3/20 live and 8/20 on each demo. Production remains `aebf404dedf89dbcdb8cee78c95a582eea042d51`; no trading code, risk/exit parameter or deployment change.

## §2. Evidence boundaries and account snapshot

The server clock was 12:55:10 SGT at audit start. The event interval is `[2026-11-11T20:50Z, 2026-11-12T04:55Z)`. Account marks were observed at 12:54:52–12:54:53 SGT, book inventory at about 12:54:51, and health at 12:56:10.919. Those are bounded observations, not a fictional atomic 12:55 account mark. Successful stored history refreshes after the cutoff cover the full ledger interval. Read-only queries used existing account records and ticks; no new broker history request was issued.

Equity is balance plus recorded net unrealised P&L, excluding unverified credit and future closing charges. The nonempty-book snapshot leaves aggregate broker margin unverified. The separate owner-assessment margin percentages below are labelled projections; they are not substituted for an atomic broker margin statement. An empty book has no meaningful margin percentage.


| Account | Balance USD | Float USD | Equity USD | Assessed margin % | S3 / S1 / S2 / human positions | Pending |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 501.83 | 4,378.68 | 4,880.51 | 987.98 | 0 / 1 / 0 / 3 | 4 |
| 46130058 | 25,113.06 | 39.02 | 25,152.08 | 24,304.03 | 0 / 4 / 0 / 0 | 0 |
| 47790949 | 24,565.02 | 46.00 | 24,611.02 | 2,973.51 | 0 / 4 / 0 / 0 | 0 |
| 49236051 | 801.96 | 0.00 | 801.96 | Unverified / N/A | 0 / 0 / 0 / 0 | 0 |


Open inventory and pending entries:


| Account | ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | Current SL | Current TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Unverified / N/A |
| 43002148 | 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Unverified / N/A |
| 43002148 | 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.80000 | 3.87500 |
| 43002148 | 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,656.40000 | Unverified / N/A |
| 43002148 | 400465199 | XAUUSD | human_external / pending_entry | buy | 1000 / 0.1000 | 3,960.28000 | 3,877.30000 | 4,339.75000 |
| 43002148 | 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 | 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 | 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Unverified / N/A |
| 46130058 | 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,822.00000 | Unverified / N/A |
| 46130058 | 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69925 | Unverified / N/A |
| 46130058 | 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12736 | Unverified / N/A |
| 46130058 | 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,648.90000 | Unverified / N/A |
| 47790949 | 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,822.00000 | Unverified / N/A |
| 47790949 | 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69920 | Unverified / N/A |
| 47790949 | 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12731 | Unverified / N/A |
| 47790949 | 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,648.90000 | Unverified / N/A |


## §3. Three Laws, external changes and live guard reasons

All morning S3 fill/protection lifecycles bear `jiaoyi-s3`. Live GER40 position **656513085**, opened at 10:10:37.686 SGT, bears `jiaoyi-s1`; its recorded trailing amendments belong to S1. Its P&L and inventory are excluded from S3. Broker SWAP events on the three human positions are accounting events, not bot stop amendments or closing deals.

No recorded bot intent targets the prior human objects, and no bot amendment/close event is linked to them. **Recorded interference = 0**, within retained evidence. This does not prove the initiator of every outside broker-book change.


| Human position | Symbol | Earlier SL | Current SL | Quantity changed? |
| --- | --- | --- | --- | --- |
| 649992251 | JPN225 | 56,571.300 | 59,000.000 | No |
| 649772867 | JPN225 | 56,450.700 | 58,500.000 | No |
| 637075128 | NatGas | 2.327 | 2.800 | No |


Pending JPN225 order **400937857** is absent from the current book. No related bot command or closing deal is recorded; cancellation, expiry or another external disposition is **Not Verifiable** from these retained records. The other four human pending entries remain. No human/external closing cash was recorded in this interval (**$0**). Human-position floating changes and broker swaps must not be counted as S3 realised cash.

NatGas and JPN225 remain permanently human-reserved under S3. Human gold pending entry **400465199** continues to lock XAUUSD dynamically; the silver sibling remains pair-reserved. There are 23 gold yield diagnostics and 17 live silver pair diagnostics. Other live S3 consumed attempts were refused once each for `ENTRY_PEER_CONFLICT` (SpotCrude) and `ENTRY_PEER_HOLD_BELOW_24BPS` (BTCUSD); neither reached the broker.

Live gates in the post-cutoff health snapshot (not additional window activity):


| Symbol | Exact gate | Message |
| --- | --- | --- |
| BTCUSD | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| ETHUSD | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| NAS100 | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| SpotBrent | prepared_admission_unavailable | Preparing: background entry admission is not ready yet |
| SpotCrude | prepared_admission_unavailable | Preparing: background entry admission is not ready yet |
| US500 | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| XAUUSD | yield_to_human_position_or_order | Blocked: yield_to_human_position_or_order |


The separate S1 AUDUSD/EURUSD attempts had margin-reserve refusals. Those S1 guard records are not unexplained S3 failures. Current margin, pending projections and stressed margin are different tests; none authorises amendments to human positions.

## §4. Complete morning S3 ledger

All four positions are **short ETHUSD**. They opened at 07:32–07:33 SGT from the carried 04:00 crypto window and closed at its stored 08:00 deadline. Entry time did not start a new four-hour holding period. Native volume is hundredths of a unit, with lot size obtained from each account's contract record.


| Account / position | Native volume / lots | Entry SGT | Entry price | Exit SGT | Exit price | Initial SL / TP | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 261899916 | 15 / 0.1500 | 12 Nov 07:32:38.943 | 2,054.23000 | 12 Nov 08:00:01.107 | 2,054.22000 | 2,085.18000 / 2,011.51000 | 0.00 / 0.000000 |
| 43002148 / 656440634 | 1 / 0.0100 | 12 Nov 07:32:46.987 | 2,052.93000 | 12 Nov 08:00:01.158 | 2,054.18000 | 2,084.33000 / 2,009.59000 | -0.01 / -0.031847 |
| 46130058 / 261899942 | 487 / 4.8700 | 12 Nov 07:32:47.835 | 2,050.81000 | 12 Nov 08:00:01.220 | 2,054.22000 | 2,081.76000 / 2,008.09000 | -16.61 / -0.110200 |
| 47790949 / 261900088 | 476 / 4.7600 | 12 Nov 07:33:27.379 | 2,050.50000 | 12 Nov 08:00:01.259 | 2,056.01000 | 2,081.45000 / 2,007.78000 | -26.23 / -0.178045 |



| Position | Initial R price / cash USD | Observed MFE R | Observed MAE R | BE amendment | Exit classification | Confirmation → guard ms |
| --- | --- | --- | --- | --- | --- | --- |
| 261899916 | 30.9500 / 4.6425 | 0.245557 | 0.218094 | None: +1R not reached | native_session_end | 30.338 |
| 656440634 | 31.4000 / 0.3140 | 0.200637 | 0.256369 | None: +1R not reached | native_session_end | 21.648 |
| 261899942 | 30.9500 / 150.7265 | 0.135057 | 0.328595 | None: +1R not reached | native_session_end | 17.981 |
| 261900088 | 30.9500 / 147.3220 | 0.125040 | 0.338611 | None: +1R not reached | native_session_end | 17.669 |


No morning trade reached +1.00R: measured favourable excursions ranged from **0.1250R to 0.2456R**. No breakeven amendment was recorded. The small demo's **$0.00** cash close is a broker money-precision flat result, not a breakeven-stop exit; it is not counted as a net win. Initial SL/TP remained in place until the native deadline closure.

For continuity, the two overnight silver positions had their entry-price stop amendments at **12 Nov 00:52:55.959 SGT** (261769291) and **02:01:51.348 SGT** (261769290), then exited at their original targets. Those amendments are outside the morning activity interval and were not counted as new morning actions.

## §5. Window cash, full-day cash and latest20


| Account | Morning closes | Morning net USD | 00:00–12:55 net USD | Open S3 float USD | Latest20 wins | Realised payoff | Net-R profit factor |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 1 | -0.01 | -0.01 | 0.00 | 3/20 | 3.4277 | 0.6049 |
| 46130058 | 1 | -16.61 | 174.64 | 0.00 | 8/20 | 1.6198 | 1.0799 |
| 47790949 | 1 | -26.23 | 162.77 | 0.00 | 8/20 | 1.4044 | 0.9363 |
| 49236051 | 1 | 0.00 | 0.00 | 0.00 | 8/20 | 1.5016 | 1.0921 |


Totals: **−$42.85 morning; +$337.40 day-to-cutoff; $0 open S3 float**. The day includes the two silver closes before 04:50. There are six day closes, four in this audit interval. No S1 or human closing cash is mixed into those S3 totals.

At 20 trades the owner threshold requires **15/20 (75%)**, the first attainable integer count meeting 73%. All accounts remain below that win threshold. A realised payoff above 1.38 does not independently satisfy the combined goal. The live latest20 now includes the new ETH close alongside older trades; the four accounts are not independent replications, and overlapping rolling windows are not out-of-sample validation.

## §6. Four-hour post-close excursions: complete clocks and market gaps

Post-close movement is anchored to the **actual exit fill**, signed positively for the original short direction. Extrema are favourable/adverse magnitudes; R uses the original initial risk distance. Pip sizes are broker-defined: ETHUSD **0.1**, XAGUSD **0.01**. Every calculation uses individual recorded ask updates (liquidation side for a short), not chart candles or simulated paths. Quote timestamps are recorder receipt times.

All six four-hour clocks have elapsed. However, silver **261769291** ends at **06:48:21.915 SGT during the maintenance break**. Its last quote is 2,962.703 seconds before that endpoint. The +4h price is therefore null. Its observed extrema over the four-hour interval are still reported. Silver **261769290** ends after the 07:01 native metals reopening; its quote stream contains a **3,721.402-second gap** spanning maintenance. This is not presented as a continuously quoted path.


| Position / symbol | Exit +4h SGT | Favourable price / pips / R | Adverse price / pips / R | Original TP / SL observed |
| --- | --- | --- | --- | --- |
| 261769291 / XAGUSD | 12 Nov 06:48:21.915 | 0.2410 / 24.1000 / 0.7850 | 0.0390 / 3.9000 / 0.1270 | True / False |
| 261769290 / XAGUSD | 12 Nov 07:18:03.600 | 0.3330 / 33.3000 / 0.8833 | 0.1500 / 15.0000 / 0.3979 | True / False |
| 261899916 / ETHUSD | 12 Nov 12:00:01.107 | 10.4400 / 104.4000 / 0.3373 | 11.3300 / 113.3000 / 0.3661 | False / False |
| 656440634 / ETHUSD | 12 Nov 12:00:01.158 | 10.4000 / 104.0000 / 0.3312 | 11.3800 / 113.8000 / 0.3624 | False / False |
| 261899942 / ETHUSD | 12 Nov 12:00:01.220 | 10.4400 / 104.4000 / 0.3373 | 11.3300 / 113.3000 / 0.3661 | False / False |
| 261900088 / ETHUSD | 12 Nov 12:00:01.259 | 12.2300 / 122.3000 / 0.3952 | 9.5400 / 95.4000 / 0.3082 | False / False |


Both silver positions had already realised their TP. Later quotes at/beyond that level are additional movement after a successful exit, not rescued losses. None of the four morning ETH paths reached its original TP or initial SL in the four hours after the deadline close. Those observations do not establish a profitable alternative holding rule.

## §7. Net changes after one, two and four hours

Cells show **favourable-direction price / pips / R** from the exit. For these shorts, positive means a lower ask. JSON also includes the raw signed price change (`quote minus exit`) and the actual endpoint quote. Endpoints use the last update at/before the exact horizon only when its age is at most six seconds; no interpolation or stale carry-forward.


| Position | +1h | +2h | +4h |
| --- | --- | --- | --- |
| 261769291 | 0.0050 / 0.5000 / 0.0163 | 0.0460 / 4.6000 / 0.1498 | stale_asof_quote |
| 261769290 | -0.0230 / -2.3000 / -0.0610 | 0.0340 / 3.4000 / 0.0902 | 0.3330 / 33.3000 / 0.8833 |
| 261899916 | -6.0500 / -60.5000 / -0.1955 | 5.1700 / 51.7000 / 0.1670 | -0.0800 / -0.8000 / -0.0026 |
| 656440634 | -6.0900 / -60.9000 / -0.1939 | 5.1300 / 51.3000 / 0.1634 | -0.1200 / -1.2000 / -0.0038 |
| 261899942 | -6.0600 / -60.6000 / -0.1958 | 5.3600 / 53.6000 / 0.1732 | -0.1900 / -1.9000 / -0.0061 |
| 261900088 | -4.2600 / -42.6000 / -0.1376 | 7.1500 / 71.5000 / 0.2310 | 1.6000 / 16.0000 / 0.0517 |


Interpretation: the second silver short continued another **0.8833R** favourably by +4h; the first's maximum observed extension was **0.7850R**, with its endpoint unavailable. Morning ETH showed modest two-way movement after the deadline, far short of its original target. Extending the hold simply because a four-hour clock can be drawn after exit is not a tested edge. Potential costs, overnight charges, gaps and fill availability would need explicit modelling in any separate counterfactual.

## §8. Native opening evidence and the reporting discrepancy

The versioned native calendar defines **metals observation from 07:01 SGT**, while their Asian session is labelled 07:00. The audit uses that compiled one-minute offset rather than mislabelling a valid 07:01 quote as late. Other requested openings use their scheduled times directly.

All **40 requested account-symbol-window slots** have a recorded first two-sided quote within their native observation window. **38 match operational evidence**: 22/24 traditional account-symbol slots (two live metals intentionally withheld by human/pair protection) and **16/16 crypto slots** across 08:00 and 12:00. Stored broker quotes alone do not prove an actor used them; operational matching was checked separately.


| Account | Symbol | Session open SGT | Observation start SGT | First quote delay s | Midpoint | Operational match / evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | NAS100 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 5.918949 | 28,144.85000 | Yes / runtime_actor_opening_reference |
| 43002148 | BTCUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.146734 | 73,936.58000 | Yes / stored_session_receipt |
| 43002148 | BTCUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.143058 | 73,703.03000 | Yes / runtime_actor_opening_reference |
| 49236051 | SpotBrent | 12 Nov 09:00:00.000 | 12 Nov 09:00:00.000 | 1.361543 | 94.77000 | Yes / runtime_actor_opening_reference |
| 43002148 | XAUUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.532081 | 3,993.53500 | Withheld: live human/pair gate |
| 43002148 | ETHUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.167281 | 2,054.51000 | Yes / stored_session_receipt |
| 43002148 | ETHUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.597534 | 2,051.98000 | Yes / runtime_actor_opening_reference |
| 43002148 | XAGUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.532025 | 51.71550 | Withheld: live human/pair gate |
| 43002148 | US500 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 0.866090 | 7,590.65000 | Yes / runtime_actor_opening_reference |
| 49236051 | NAS100 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 5.936090 | 28,144.85000 | Yes / runtime_actor_opening_reference |
| 49236051 | US500 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 0.909307 | 7,590.65000 | Yes / runtime_actor_opening_reference |
| 49236051 | XAGUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.573506 | 51.71550 | Yes / runtime_actor_opening_reference |
| 49236051 | BTCUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.028910 | 73,937.81000 | Yes / stored_session_receipt |
| 49236051 | BTCUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.231186 | 73,703.03000 | Yes / runtime_actor_opening_reference |
| 49236051 | SpotCrude | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 7.152215 | 86.73800 | Yes / runtime_actor_opening_reference |
| 49236051 | ETHUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.196763 | 2,054.51000 | Yes / stored_session_receipt |
| 49236051 | ETHUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.610324 | 2,051.98000 | Yes / runtime_actor_opening_reference |
| 43002148 | SpotCrude | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 7.115228 | 86.73800 | Yes / runtime_actor_opening_reference |
| 49236051 | XAUUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.573511 | 3,993.53500 | Yes / runtime_actor_opening_reference |
| 43002148 | SpotBrent | 12 Nov 09:00:00.000 | 12 Nov 09:00:00.000 | 1.349915 | 94.77000 | Yes / runtime_actor_opening_reference |
| 46130058 | NAS100 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 5.983302 | 28,144.85000 | Yes / runtime_actor_opening_reference |
| 46130058 | US500 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 0.857577 | 7,590.65000 | Yes / runtime_actor_opening_reference |
| 47790949 | XAUUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.630003 | 3,993.53500 | Yes / runtime_actor_opening_reference |
| 47790949 | BTCUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.025039 | 73,937.81000 | Yes / stored_session_receipt |
| 47790949 | BTCUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.228910 | 73,703.03000 | Yes / runtime_actor_opening_reference |
| 46130058 | XAUUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.551319 | 3,993.53000 | Yes / runtime_actor_opening_reference |
| 46130058 | SpotBrent | 12 Nov 09:00:00.000 | 12 Nov 09:00:00.000 | 1.373895 | 94.77000 | Yes / runtime_actor_opening_reference |
| 46130058 | ETHUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.189702 | 2,054.51000 | Yes / stored_session_receipt |
| 46130058 | ETHUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.593107 | 2,051.98000 | Yes / runtime_actor_opening_reference |
| 47790949 | XAGUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.630001 | 51.71550 | Yes / runtime_actor_opening_reference |
| 47790949 | US500 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 0.868472 | 7,590.65000 | Yes / runtime_actor_opening_reference |
| 47790949 | NAS100 | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 5.927566 | 28,144.85000 | Yes / runtime_actor_opening_reference |
| 47790949 | SpotCrude | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 7.152919 | 86.73800 | Yes / runtime_actor_opening_reference |
| 46130058 | SpotCrude | 12 Nov 07:00:00.000 | 12 Nov 07:00:00.000 | 7.179580 | 86.73800 | Yes / runtime_actor_opening_reference |
| 47790949 | SpotBrent | 12 Nov 09:00:00.000 | 12 Nov 09:00:00.000 | 1.384209 | 94.77000 | Yes / runtime_actor_opening_reference |
| 46130058 | XAGUSD | 12 Nov 07:00:00.000 | 12 Nov 07:01:00.000 | 0.551318 | 51.71550 | Yes / runtime_actor_opening_reference |
| 46130058 | BTCUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.025093 | 73,937.81000 | Yes / stored_session_receipt |
| 46130058 | BTCUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.189294 | 73,703.03000 | Yes / runtime_actor_opening_reference |
| 47790949 | ETHUSD | 12 Nov 08:00:00.000 | 12 Nov 08:00:00.000 | 0.191616 | 2,054.51000 | Yes / stored_session_receipt |
| 47790949 | ETHUSD | 12 Nov 12:00:00.000 | 12 Nov 12:00:00.000 | 0.614029 | 2,051.98000 | Yes / runtime_actor_opening_reference |


**SpotCrude's first post-PR-#219 rollover passed.** Its quotes have bid **86.717**, ask **86.759**, midpoint **86.738**, at 07:00:07.115228 / .179580 / .152919 / .152215 SGT across the four accounts. Each actor retains the exact timestamp and `native_open_first_quote_v18425_15s` policy. These are original opening-window quotes, not later mid-session substitutions.

The initial durable `s3_session` receipt nevertheless says opening unavailable. In `strategy3.rs`, the bounded opening retry updates `day.open`, `day.opening_quote` and `prepared_at`, then returns without a new session receipt. Runtime evidence plus the stored tick reconstruction resolves that apparent contradiction. Several other openings show the same stale initial diagnostic; four Brent session rows were absent from the queried durable opening set, while all four running actors held the verified 09:00 reference.

**Follow-up engineering recommendation, not executed:** persist an attributable success/recovery receipt after the retry, preserving both the original failure and the recovered quote provenance. A regression test should cover initial read before recorder flush, recovery within the immutable observation window and rejection of later quotes. The opening policy itself worked in this observed rollover; this audit does not justify extending its time window again.

## §9. Veto funnel and telemetry integrity


| Metric | Value | Interpretation |
| --- | --- | --- |
| New arms | 42 | Account-symbol arm receipts |
| Consumed triggers | 14 | Distinct consumed decision IDs |
| Persisted opening intents | 14 | Four filled, ten refused before send |
| Entry dispatches / first fills | 4 | Distinct new S3 positions |
| Opposing-peer refusals | 4 | Final persisted-intent gate |
| Peer below 24bp refusals | 6 | Final persisted-intent gate |
| Deep retracement invalidations | 15 | >61.8% state transitions |
| Pair reservation diagnostics | 29 | Repeated diagnostic records, not 29 distinct risk reservations |
| Consumed without individual outcome | 0 | Exact consumed-ID → outcome-ID joins |
| Unknown entry-guard receipts | 0 | Persisted-intent parser scope |
| Maximum confirmation → guard | 30.338 ms | Four successful entries; excludes broker fill time |



| Consumed ID | Outcome ID | Account / symbol | Time SGT | Status / reason |
| --- | --- | --- | --- | --- |
| 104753 | 104756 | 49236051 / ETHUSD | 12 Nov 07:32:38.339 | filled / Filled { position_id: Some(261899916), price: Some(2054.23), volume: Some(15) } |
| 104761 | 104765 | 43002148 / ETHUSD | 12 Nov 07:32:46.414 | filled / Filled { position_id: Some(656440634), price: Some(2052.93), volume: Some(1) } |
| 104762 | 104774 | 46130058 / ETHUSD | 12 Nov 07:32:47.222 | filled / Filled { position_id: Some(261899942), price: Some(2050.81), volume: Some(487) } |
| 104785 | 104788 | 47790949 / ETHUSD | 12 Nov 07:33:26.792 | filled / Filled { position_id: Some(261900088), price: Some(2050.5), volume: Some(476) } |
| 106170 | 106178 | 46130058 / SpotCrude | 12 Nov 09:00:23.954 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 106171 | 106179 | 47790949 / SpotCrude | 12 Nov 09:00:23.954 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 106172 | 106177 | 49236051 / SpotCrude | 12 Nov 09:00:23.954 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 106173 | 106181 | 43002148 / SpotCrude | 12 Nov 09:00:23.991 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 106756 | 106759 | 49236051 / BTCUSD | 12 Nov 09:36:08.499 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 106757 | 106761 | 47790949 / BTCUSD | 12 Nov 09:36:08.513 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 106762 | 106764 | 46130058 / BTCUSD | 12 Nov 09:36:09.511 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 106765 | 106767 | 43002148 / BTCUSD | 12 Nov 09:36:11.997 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 108150 | 108152 | 46130058 / XAUUSD | 12 Nov 11:09:04.341 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 108157 | 108159 | 47790949 / XAUUSD | 12 Nov 11:09:16.318 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |


Idle and pre-admission diagnostics remain deduplicated. Their counts are not an exhaustive per-tick rejection census. The individual-outcome assertion applies specifically to the 14 consumed triggers, including repeated refusal reasons. The operational evidence shows activity and deliberate refusals, rather than an account-wide idle loop.

## §10. London 15:00 and crypto 16:00 readiness

All four account actors are connected, unpaused and on the same production policy. The health snapshot shows their loops progressing. No S3 position or unresolved open attempt is reported. This is current liveness evidence, not a guarantee of future fills.


| Window | Base inputs | Complete sigma tables | Exact qualification |
| --- | --- | --- | --- |
| 15:00 London traditional | 24/24 | 0/24 | 30–390-minute marks valid; 420/450 unavailable on all 24 tables |
| 16:00 Crypto | 8/8 | 8/8 | No missing marks or macro/source errors |


All preparations have contract metadata, spread and commission readiness, causal macro context and no source error. London marks **420 and 450** report exactly `8 same-slot endpoints;14 needed`. At a 15:00 start these correspond to 22:00 and 22:30 SGT, near the NY handover. The observed insufficiency is established; its full causal attribution to calendar alignment or historical availability has not been proven by this audit. No replacement history was requested and no threshold was relaxed.

Thus the **base opening preparation passes**, while complete late-session sigma coverage does not. A later trade requiring an unavailable band must remain vetoed. The new 15:00/16:00 opening quotes are not yet observable at 12:55. Fresh quote, macro, peer, pullback, human ownership, pair, size and margin checks still apply at dispatch.

## §11. Checks, limits and unchanged production

Focused verification covers receipt SHA256 and row counts, broker cash/volume/label reconciliation, long/short and pip arithmetic, cutoff/freshness handling, exact trigger joins, and operational quotes matched to the native-window source. Fourteen quote exports contain **278,808 rows**; one export is correctly empty during silver maintenance. All six follow-up clocks are elapsed, with one explicitly unpriced endpoint. No missing evidence is replaced with zero.

Current snapshots, initial durable session diagnostics, runtime recovered references and recorded ticks are separate sources and are labelled separately. Zero recorded human interference is not a claim that every external action is attributable. The missing human pending order and human stop changes remain visible rather than being described as unchanged inventory.

The branch changes only this report's Markdown, HTML and JSON files in `doc/`. Required CI: `check`, `docker`, `engineering`. Railway deployment identity before publication: **c0fedaca-0370-4cec-9234-f59f8c7cafce**, production commit **aebf404dedf89dbcdb8cee78c95a582eea042d51**, image **sha256:631e668c1ca138d5810f22fdab1ba27f3b7f4f34c6a28fa544c7ad09c2a9fad3**. Docs are outside the deployment watch patterns. Deployment identity and actor continuity are checked again after the docs merge; actual PR/merge evidence is returned in the final review block.

No strategy code, owner settings, risk percentage, margin floor, pair cap, breakeven threshold, TP, live trading deployment or human order was changed. **Financial pass: false.**

## §12. Instructions for Gemini and evidence index

Read this report and its JSON together at the merged commit. Check `ledger` for raw cash receipts, guards, initial protections, breakeven amendment times, every post-close endpoint and data gap. Inspect `opening_audit` before interpreting the stale durable refusal records: it contains both operational references and independently reconstructed first quotes. Review `preparations` for the exact 420/450 errors, and `human_protection` for the changed stops and absent pending ID.

Do not turn a time-based close into a breakeven stop, count account copies as independent experiments, count a past target touch as a rescued loser, equate base preparation with all marks being available, or fill in silver's maintenance-window endpoint. Potential next work is the retry-success telemetry receipt and a bounded investigation of late London mark coverage; neither change was authorised or implemented in this read-only audit.

The JSON includes inspectable calculations and source queries. Raw stored quote regeneration requires authorised database access; hashes verify identity, not independent correctness of private source data.


| Quote segment | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-XAGUSD-2026-11-11-144010.csv.gz | 25716 | 5733e9a6557707da9eb5c32aadcb79b6c102f7f0786c51ef5f6db771eb33ea42 |
| 46130058-XAGUSD-2026-11-11-184010.csv.gz | 13541 | a5c26eb9a4ba85e41c34859998577f32ce44d417bd0267b2c0e91ebf799e636f |
| 46130058-XAGUSD-2026-11-11-224010.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAGUSD-2026-11-11-144010.csv.gz | 25718 | d1089328563e66e607a42d2c449c58f4a3f32bfedd632bfa6eed64d7610153f6 |
| 47790949-XAGUSD-2026-11-11-184010.csv.gz | 13542 | b14a60068cddd9845b49f8e924cbad6057abf07c396ef713ed263ff4bbf24bc3 |
| 47790949-XAGUSD-2026-11-11-224010.csv.gz | 1900 | efd14c43743b61cae8e401524a61020c74f35190c6a160e83e2670c860f91edd |
| 49236051-ETHUSD-2026-11-11-233232.csv.gz | 45565 | 43ddd54a3d7d812cfcf184307a744b1504f4edba6d746f63b141af9a565eaf94 |
| 49236051-ETHUSD-2026-11-11-033232.csv.gz | 4048 | c3adfccac7685682e4584ee938358785064d2e4b991e56d8825f4309681959cf |
| 43002148-ETHUSD-2026-11-11-233240.csv.gz | 45713 | 3002b4b8cb502c188a37736497a8f7cd9f377146ada6cbbcd3a6a1cfb2d173a0 |
| 43002148-ETHUSD-2026-11-11-033240.csv.gz | 4035 | aefa806a6bd297116811c932e3676ff484f73a3a541df46d95656bdea7f81e2c |
| 46130058-ETHUSD-2026-11-11-233241.csv.gz | 45546 | b864bbbbfc70e2037b13bce576b04b348c99b633e5f5d6ef2e08abbecda8d0fc |
| 46130058-ETHUSD-2026-11-11-033241.csv.gz | 4026 | 44d58738d67fab71aea1a8879b467cf275cf37a4efe5c2cbf16e0512ec578253 |
| 47790949-ETHUSD-2026-11-11-233321.csv.gz | 45535 | 41e6bcd3cf4b4ad49844c536876b6d1e42cafa8acc37d727093878c633de6ea8 |
| 47790949-ETHUSD-2026-11-11-033321.csv.gz | 3923 | 672d44ae9eee8daf23a2dfcf7655bd0ceff5b010da72c403ff62405ee2ddc25c |



| Source | SHA256 |
| --- | --- |
| opening-ticks.json | ec1a85f4040b5102052c04b4c8cae2bd859c5a9db1d786c7b6854c63a7491fb9 |
| preflight.json | 2c0385f593f419f5170aa527bc5afbc8aa0a6436c406b7329f6787c5af35734b |
| monitor.json | 16ccf58ee83455fbcbdc65d5aa19975495b2283d11931337b74dd515d16d6948 |
| health.json | 2a6e4acc4c00ea9f07075ca205a10749c56f0a7483008e311c2fcc1c70ed98cc |
| activity.json | 32b9677f89756d3f6557024b3113a6a2dc4747237c7a6e55c35d2652fe5ddcbf |
| evidence.json | cf2fdb131922b8a6895c154242929ce19e6464ba097b4195c593064a3907ad21 |
| deployment-before.json | 6cab90a4e75ee1ed9ad74d89b5cb42a8c91d062e0a39cbe9e231c04b4049604a |
| runtime.json | 37910239a2fe1d9b8c44eebe7ad4473f9da562f65d437896a6b6de97e2757760 |
