# Friday pre-dawn New York audit and post-close excursions — №18,446

**Cutoff: 13 November 2026, 04:55 SGT.** Activity **[12 November 19:45, 13 November 04:55) SGT**, continuing from [№18,444](THURSDAY_LONDON_REVIEW_AND_NY_PREP-18444-2026-11-12.md). Post-close quote horizon: **05:04 SGT**. Later account, health and inventory snapshots are separately timestamped. Read-only audit; production trading unchanged.

## §1. TL;DR

- **Overnight S3 realised cash: +$819.82** across eight demo closes: **four TP wins, three small positive breakeven-stop fills, one deadline loss**. Seven net wins out of eight is a cash classification; it is not seven target completions.
- **All four carried positions resolved**: the two gold shorts hit TP; the two crude shorts exited at breakeven stops with small positive fills. Six new demo entries produced four more closes and **two open XAGUSD shorts**.
- **Full 12 November SGT closed S3 cash: +$919.13** across 21 closes. **13 November 00:00–04:55: −$0.74**, one ETH deadline close just after midnight. No live S3 entry/close in the overnight window.
- **Live external closing cash: −$1,408.70**, separate from S3. The original manual gold position **652821184 closed at 22:57:04 SGT for −$920.60**. It is no longer open. Current gold **657187868** is a different external position, increased after the activity cutoff. No recorded bot intent or owned amendment/close targeted the audited external inventory.
- **All ten requested follow-ups now cover 4.000 hours**: eight overnight closes plus the two prior BTC target exits. All 30 one/two/four-hour endpoints have quotes within six seconds. Maximum recorded quote-update gaps are disclosed.
- Both crude BE exits were followed by original-stop crossings without a TP touch. The later BTC BE exit **262172983** subsequently touched its original TP at **22:11:31 SGT** without an observed original-stop crossing. The ETH deadline close did not touch TP and subsequently crossed its original stop. These differing paths do not justify changing exits from this sample alone.
- **10 consumed triggers → 10 persisted intents → 10 individual outcomes**, comprising six fills and four peer refusals. Missing individual outcomes and unknown guard receipts are zero. Max confirmation-to-guard latency: **21.843 ms**.
- **Friday preparation: Asian 07:00 20/20, crypto 08:00 8/8, Brent 09:00 4/4**, with all required marks complete. Four actors connected and unpaused. Future opening quotes are not yet observable.
- **Financial pass remains false**: latest20 net wins are **3/20 live; 10/20, 10/20, 7/20 demos**. The 73% goal requires at least 15/20, together with the payoff requirement; one profitable window is not independent validation.

## §2. Sources, boundaries and account snapshots

The execution clock was 05:04 SGT. Activity is fixed at the requested half-open interval; post-close quotes use the separately frozen 05:04 horizon. Monetary marks are around **05:04:53**, health **05:07:44**, inventory **05:09:51**, and its price assessment around **05:10:13 SGT**. These are bounded current observations, not an atomic 04:55 reconstruction. Successful stored history receipts after the cutoff cover the closing-deal tail. The audit copied recorded PostgreSQL data using read-only queries and read existing owner/health endpoints; it initiated no broker request.

Production policy is `s3_macro_pullback_owner_18409_v1`, commit `aebf404dedf89dbcdb8cee78c95a582eea042d51`. Monetary equity is recorded balance plus broker net unrealised P&L; credit and potential closing commission are not invented. The monetary source has no verified aggregate broker margin level for nonempty books. The margin values below are separately timed owner-assessment projections.


| Account | Balance USD | Broker account float USD | Equity USD | Assessed margin % | S3 / S1 / S2 / external positions | Pending entries |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | -906.87 | 4,592.97 | 3,686.10 | 358.93 | 0 / 2 / 0 / 5 | 5 |
| 46130058 | 25,303.42 | 48.64 | 25,352.06 | 2,970.62 | 1 / 4 / 0 / 0 | 0 |
| 47790949 | 24,957.39 | 56.04 | 25,013.43 | 1,722.11 | 1 / 4 / 0 / 0 | 0 |
| 49236051 | 800.22 | 0.00 | 800.22 | Not verifiable / N/A | 0 / 0 / 0 / 0 | 0 |


Live balance is **negative $906.87**, while equity remains positive because account-wide floating P&L is **+$4,592.97**. The five recorded external closing deals total −$1,408.70, arithmetically reconciling the prior $501.83 balance to −$906.87. This is a balance reconciliation, not a claim that every external cash-flow category was independently fetched. S3 closed cash is zero on live during this window.

Live's current assessed margin buffer is **358.93%**; the pending-entry projection is **173.03%**, below the existing 250% threshold. The stressed pending projection is unverified. These are present projections, not proof that margin caused every earlier non-entry. No order or protection was modified by this audit.

Complete current inventory follows. Null marks and cost components remain unavailable rather than being filled with zero.


| Account | Position/order ID | Symbol | Owner / kind | Side | Native volume / lots | Entry / trigger | Current SL | Current TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Not verifiable / N/A |
| 43002148 | 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Not verifiable / N/A |
| 43002148 | 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.91300 | 3.87500 |
| 43002148 | 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,802.90000 | Not verifiable / N/A |
| 43002148 | 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,826.30000 | Not verifiable / N/A |
| 43002148 | 657180931 | NVDA.US | human_external / position | buy | 200 / 2.0000 | 165.22000 | 149.66000 | Not verifiable / N/A |
| 43002148 | 657187868 | XAUUSD | human_external / position | buy | 2500 / 0.2500 | 3,869.03800 | 3,847.97000 | 4,146.31000 |
| 43002148 | 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Not verifiable / N/A |
| 43002148 | 401488805 | XAUUSD | human_external / pending_entry | buy | 1000 / 0.1000 | 3,813.63000 | 3,786.80000 | Not verifiable / N/A |
| 43002148 | 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 5,052.00000 | 4,925.90000 | 5,904.00000 |
| 43002148 | 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 | 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Not verifiable / N/A |
| 46130058 | 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,826.30000 | Not verifiable / N/A |
| 46130058 | 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69560 | Not verifiable / N/A |
| 46130058 | 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12530 | Not verifiable / N/A |
| 46130058 | 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,795.40000 | Not verifiable / N/A |
| 46130058 | 262253774 | XAGUSD | s3 / position | sell | 30000 / 0.0600 | 50.01200 | 50.51400 | 49.31900 |
| 47790949 | 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,826.30000 | Not verifiable / N/A |
| 47790949 | 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69555 | Not verifiable / N/A |
| 47790949 | 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12525 | Not verifiable / N/A |
| 47790949 | 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,795.40000 | Not verifiable / N/A |
| 47790949 | 262253752 | XAGUSD | s3 / position | sell | 25000 / 0.0500 | 50.01600 | 50.58800 | 49.22600 |


## §3. Human protection, external cash and live S1

The two JPN225 positions and NatGas remain external, with observed stops **59,000**, **58,500** and **2.913** unchanged from the prior report. Original manual gold order **400465199** links to position **652821184**, entry 0.1 lot at 3,959.95. That position closed overnight; it must not be described as still intact/open. No bot intent, S3 ownership or owned amendment/close receipt links to that external close. The retained bot journal does not establish whether an external close was a human click, broker protection execution or another external initiator.

The external closing ledger is kept at **deal grain**, not deduplicated by position ID:


| Deal / position | Symbol | Executed SGT | Deal side / filled native volume | Close entry price | Exit price | Gross USD | Commission + swap + conversion USD | Net close USD |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 347799722 / 656730071 | US500 | 12 Nov 22:31:06.110 | SELL / 800 | 7,562.60000 | 7,563.70000 | 8.80 | 0.00 | 8.80 |
| 347817583 / 652821184 | XAUUSD | 12 Nov 22:57:04.361 | SELL / 1000 | 3,959.95000 | 3,867.96000 | -919.90 | -0.70 | -920.60 |
| 347817671 / 657043459 | XAUUSD | 12 Nov 22:57:06.952 | SELL / 1000 | 3,906.99000 | 3,867.46000 | -395.30 | -0.70 | -396.00 |
| 347827377 / 657129197 | XAUUSD | 12 Nov 23:14:28.533 | SELL / 2000 | 3,866.94000 | 3,855.71000 | -112.30 | -0.70 | -113.00 |
| 347832108 / 657129197 | XAUUSD | 12 Nov 23:23:25.278 | BUY / 1000 | 3,855.71000 | 3,854.43000 | 12.80 | -0.70 | 12.10 |


Position **657129197** has two different closing deals: a SELL of 2,000 native units with a 1,000-unit closed component at 23:14:28, then a BUY close at 23:23:25 using 3,855.71 as close-entry price. This is consistent with reversal/offset mechanics; treating it as one unchanged long or dropping the second deal would corrupt the cash ledger. Broker initiator attribution is not verified. The JSON retains filled and closed volumes and both deal IDs.

Current external holdings also include **NVDA.US 657180931**, two units at 165.22, and gold **657187868**, now 0.25 lot at weighted entry 3,869.038. Gold additions at **04:59:18.604** and **05:01:42.905 SGT** are after the 04:55 activity cutoff; they affect the later book and margin snapshot, not overnight activity counts. Five external pending entries are listed in §2, including a separate gold order and Cocoa. Current gold locks live XAUUSD and reserves XAGUSD under the human/pair rules.

**Recorded bot interference: zero.** The audit checks retained bot intents against prior and current external IDs, including external positions already closed, and checks owned execution receipts. This is narrower than proving every external broker modification's origin. Law 1 ownership isolation remains visible; Law 2 gold/silver locking is explicit in the current gates; Law 3 NatGas/JPN225 reservation remains unchanged. No production protection rule was edited.

Live S1 **GER40 656513085** and **AUS200 656795267** remain open. No new live S1 fill or trailing amendment is recorded in the overnight interval. Their trailing history is therefore carried forward, not misreported as new overnight activity:


| Live S1 position | Entry | Initial SL | Latest recorded SL | Recorded replacements since entry | Last replacement SGT | In-window replacements |
| --- | --- | --- | --- | --- | --- | --- |
| 656513085 | 26,809.0 | 25,589.7 | 25,802.9 | 50 | 12 Nov 17:40:06.388 | 0 |
| 656795267 | 9,165.3 | 8,822.1 | 8,826.3 | 7 | 12 Nov 17:49:09.445 | 0 |


The JSON contains every retained live S1 replacement with owner label and timestamp. GER40's 25,802.9 and AUS200's 8,826.3 are still below their long entries; they are not breakeven stops. Demo S1 amendments are separate from the live S1 and S3 ledgers.

## §4. Overnight S3 ledger and all four carryovers

Four carried positions plus six new entries give **ten overnight positions**: eight closed and two open. The two older BTC rows appear only in the follow-up section and are excluded from this window's cash. All audited S3 positions are shorts; labels and opening intents identify them as `jiaoyi-s3`. Initial risk remains fixed to actual fill and initial broker SL, including after a BE amendment.


| Account / position | Carryover? / symbol | Side / native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 262101929 | Yes / XAUUSD | sell / 500 / 0.0500 | 12 Nov 18:07:13.954 / 3,942.25000 | 12 Nov 21:33:10.560 / 3,903.31000 | 3,970.38000 / 3,903.43000 | 194.70 / 1.384287 |
| 47790949 / 262101991 | Yes / XAUUSD | sell / 500 / 0.0500 | 12 Nov 18:07:16.701 / 3,942.25000 | 12 Nov 21:33:09.162 / 3,903.43000 | 3,970.36000 / 3,903.45000 | 193.74 / 1.378442 |
| 49236051 / 262132524 | Yes / SpotCrude | sell / 700 / 0.0700 | 12 Nov 19:16:35.196 / 86.23900 | 12 Nov 21:07:56.721 / 86.21700 | 86.88600 / 85.34600 | 0.15 / 0.033120 |
| 46130058 / 262132606 | Yes / SpotCrude | sell / 23000 / 2.3000 | 12 Nov 19:16:43.244 / 86.22900 | 12 Nov 21:07:56.744 / 86.21700 | 86.87600 / 85.33600 | 2.76 / 0.018547 |
| 47790949 / 262172983 | New / BTCUSD | sell / 41 / 0.4100 | 12 Nov 21:08:42.153 / 72,637.94000 | 12 Nov 21:21:58.525 / 72,634.10000 | 72,996.88000 / 72,142.60000 | 1.57 / 0.010668 |
| 49236051 / 262174895 | New / ETHUSD | sell / 18 / 0.1800 | 12 Nov 21:12:38.724 / 2,001.34000 | 13 Nov 00:00:01.416 / 2,005.44000 | 2,026.78000 / 1,966.23000 | -0.74 / -0.161600 |
| 46130058 / 262188472 | New / ETHUSD | sell / 1180 / 11.8000 | 12 Nov 21:43:05.047 / 2,011.44000 | 12 Nov 22:19:38.561 / 1,993.37000 | 2,024.20000 / 1,993.83000 | 213.23 / 1.416171 |
| 47790949 / 262213604 | New / ETHUSD | sell / 1021 / 10.2100 | 12 Nov 22:35:47.497 / 2,010.61000 | 12 Nov 23:03:34.736 / 1,989.61000 | 2,025.17000 / 1,990.51000 | 214.41 / 1.442308 |
| 47790949 / 262253752 | New / XAGUSD | sell / 25000 / 0.0500 | 13 Nov 00:01:03.112 / 50.01600 | Open / unavailable / Not verifiable / N/A | 50.58800 / 49.22600 | Not verifiable / N/A / Not verifiable / N/A |
| 46130058 / 262253774 | New / XAGUSD | sell / 30000 / 0.0600 | 13 Nov 00:01:06.420 / 50.01200 | Open / unavailable / Not verifiable / N/A | 50.51400 / 49.31900 | Not verifiable / N/A / Not verifiable / N/A |



| Position | Original R price / initial USD risk | Observed MFE R | Observed MAE R | Entry-price BE amendment SGT | Exit classification | Frozen native deadline SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 262101929 | 28.13000 / 140.6500 | 1.382865 | 0.482403 | 12 Nov 21:24:16.430 | broker_protective_fill_at_or_better_than_tp | 12 Nov 22:30:00.000 |
| 262101991 | 28.11000 / 140.5500 | 1.381003 | 0.481679 | 12 Nov 21:24:16.127 | broker_protective_fill_at_or_better_than_tp | 12 Nov 22:30:00.000 |
| 262132524 | 0.64700 / 4.5290 | 1.069552 | 0.352396 | 12 Nov 21:07:36.620 | breakeven_stop_with_fill_slippage | 12 Nov 22:30:00.000 |
| 262132606 | 0.64700 / 148.8100 | 1.054096 | 0.367852 | 12 Nov 21:07:36.941 | breakeven_stop_with_fill_slippage | 12 Nov 22:30:00.000 |
| 262172983 | 358.94000 / 147.1654 | 1.343428 | 0.249512 | 12 Nov 21:10:02.939 | breakeven_stop_with_fill_slippage | 13 Nov 00:00:00.000 |
| 262174895 | 25.44000 / 4.5792 | 0.792846 | 0.686321 | None recorded | native_session_end | 13 Nov 00:00:00.000 |
| 262188472 | 12.76000 / 150.5680 | 1.412226 | 0.501567 | 12 Nov 22:17:49.710 | broker_protective_fill_at_or_better_than_tp | 13 Nov 00:00:00.000 |
| 262213604 | 14.56000 / 148.6576 | 1.381868 | 0.482830 | 12 Nov 22:44:17.125 | broker_protective_fill_at_or_better_than_tp | 13 Nov 00:00:00.000 |
| 262253752 | 0.57200 / 143.0000 | 0.152098 | 0.578671 | None recorded | open | 13 Nov 05:59:00.000 |
| 262253774 | 0.50200 / 150.6000 | 0.165339 | 0.667331 | None recorded | open | 13 Nov 05:59:00.000 |


The four carryovers are fully reconciled: gold **262101929/262101991** hit targets for **+$388.44** combined; oil **262132524/262132606** closed at amended entry-price stops for **+$2.91**. New BTC **262172983** returned **+$1.57** at its BE stop, and two new ETH targets returned **+$427.64**. Small-demo ETH **262174895** closed at the source-window deadline for **−$0.74**.

Exit geometry is unchanged: +1R entry-price stop activation, +1.38R gross target, native deadline. A BE stop's realised cash can differ from zero because of fill price and costs. Target net R can differ from 1.38 because of execution and charges. Observed quote MFE and broker fill can also differ; an executable quote is not a guaranteed fill.

The two XAGUSD shorts entered around 00:01 SGT with stored deadlines **05:59 SGT**, not four hours after fill. This is the native NY session deadline recorded in their opening orders, not a newly invented six-hour policy. Both remained open at 04:55 and in the later owner book; no entry-price BE amendment was recorded by cutoff. Their observed MFE was below 0.17R.

## §5. Cash periods, current open marks and latest20


| Account | Window closes / wins | Window net USD | Full 12 Nov net USD | 13 Nov to 04:55 net USD | Open S3 | Latest20 wins | Realised payoff | Net-R profit factor |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 / 0 | 0.00 | -0.01 | 0.00 | 0 | 3/20 | 3.4277 | 0.6049 |
| 46130058 | 3 / 3 | 410.69 | 365.00 | 0.00 | 1 | 10/20 | 1.7120 | 1.7120 |
| 47790949 | 3 / 3 | 409.72 | 555.14 | 0.00 | 1 | 10/20 | 1.6567 | 1.6567 |
| 49236051 | 2 / 1 | -0.59 | -1.00 | -0.74 | 0 | 7/20 | 1.4771 | 0.8617 |


Full-day reconciliation: prior 12 November 00:00–19:45 **+$98.57**, plus **+$820.56** closed between 19:45 and midnight, equals **+$919.13**. The **−$0.74** ETH close at 00:00:01.416 belongs to 13 November, making the overnight window **+$819.82**. The two prior BTC wins are already in the earlier +$98.57 and are not counted twice.

The monetary account floats in §2 contain S1/external inventory as well as S3. Verified S3 net unrealised cash is not separately available in the inspected aggregate snapshot. The exact cutoff's executable silver marks give a price-only approximation below, excluding closing charges and future execution:


| Open S3 position | 04:55 executable ask | Quote SGT | Quote age s | Price-only signed R | Price-only USD |
| --- | --- | --- | --- | --- | --- |
| 262253752 | 50.00900 | 13 Nov 04:54:59.737 | 0.262369 | 0.012238 | 1.7500 |
| 262253774 | 50.00900 | 13 Nov 04:54:59.725 | 0.274588 | 0.005976 | 0.9000 |


These two marks total **+$2.65 before costs** at cutoff. They are not the later account-wide float and not realised profits. Latest20 is the last official supervisor record before cutoff, retained with entry keys and timestamps in JSON. At 20 trades, **15/20 = 75%** is the first integer win count meeting 73%. The two larger demos are at 10/20; the small demo is 7/20 and live 3/20. A realised payoff above 1.38 does not remove the win-count requirement. Account copies are correlated. **Financial pass: false.**

## §6. All ten four-hour post-close paths

All ten requested four-hour clocks have elapsed by the 05:04 capture horizon. This comprises **eight overnight closes plus two BTC target exits from №18,444**, with no partial rows. Broker pip sizes are **BTCUSD 0.1, ETHUSD 0.1, XAUUSD 0.1, SpotCrude 0.01**; open silver uses 0.01. All are shorts, so liquidation uses **ask**. Post-close favourable movement is exit fill minus later ask; adverse movement is the opposite. Maxima are nonnegative; checkpoint signs retain direction. R always divides by the original fill-to-stop distance.


| Position / symbol / cohort | Pip size | Hours | Max favourable price / pips / R | Max adverse price / pips / R | Original TP touch / first SGT | Original SL touch / first SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 262076304 / BTCUSD / prior BTC | 0.100 | 4.000 complete | 1,707.28000 / 17,072.80000 / 5.25414 | 230.64000 / 2,306.40000 / 0.70979 | 12 Nov 19:40:43.029 | Not observed |
| 262076346 / BTCUSD / prior BTC | 0.100 | 4.000 complete | 1,725.24000 / 17,252.40000 / 5.30957 | 212.68000 / 2,126.80000 / 0.65454 | 12 Nov 19:40:38.783 | Not observed |
| 262101929 / XAUUSD / overnight | 0.100 | 4.000 complete | 50.87000 / 508.70000 / 1.80839 | 6.26000 / 62.60000 / 0.22254 | 12 Nov 21:33:10.655 | Not observed |
| 262101991 / XAUUSD / overnight | 0.100 | 4.000 complete | 51.02000 / 510.20000 / 1.81501 | 6.11000 / 61.10000 / 0.21736 | 12 Nov 21:33:09.342 | Not observed |
| 262132524 / SpotCrude / overnight | 0.010 | 4.000 complete | 0.11200 / 11.20000 / 0.17311 | 1.84700 / 184.70000 / 2.85471 | Not observed | 12 Nov 21:08:31.521 |
| 262132606 / SpotCrude / overnight | 0.010 | 4.000 complete | 0.11200 / 11.20000 / 0.17311 | 1.84700 / 184.70000 / 2.85471 | Not observed | 12 Nov 21:08:31.503 |
| 262172983 / BTCUSD / overnight | 0.100 | 4.000 complete | 1,490.78000 / 14,907.80000 / 4.15328 | 82.00000 / 820.00000 / 0.22845 | 12 Nov 22:11:31.441 | Not observed |
| 262174895 / ETHUSD / overnight | 0.100 | 4.000 complete | 8.08000 / 80.80000 / 0.31761 | 22.67000 / 226.70000 / 0.89112 | Not observed | 13 Nov 00:06:52.760 |
| 262188472 / ETHUSD / overnight | 0.100 | 4.000 complete | 12.20000 / 122.00000 / 0.95611 | 34.74000 / 347.40000 / 2.72257 | 12 Nov 22:19:38.708 | 13 Nov 00:06:49.562 |
| 262213604 / ETHUSD / overnight | 0.100 | 4.000 complete | 0.00000 / 0.00000 / 0.00000 | 38.50000 / 385.00000 / 2.64423 | 12 Nov 23:03:34.910 | 13 Nov 00:06:49.557 |



| Position | 1h signed price / pips / R | 2h signed price / pips / R | 4h signed price / pips / R | Largest quote-update gap s | End quote age s |
| --- | --- | --- | --- | --- | --- |
| 262076304 | 14.98000 / 149.80000 / 0.04610 | 362.79000 / 3,627.90000 / 1.11648 | 1,450.09000 / 14,500.90000 / 4.46264 | 4.005671 | 0.147980 |
| 262076346 | 31.35000 / 313.50000 / 0.09648 | 378.11000 / 3,781.10000 / 1.16367 | 1,482.55000 / 14,825.50000 / 4.56268 | 3.960271 | 0.127913 |
| 262101929 | 0.61000 / 6.10000 / 0.02169 | 39.60000 / 396.00000 / 1.40775 | 30.23000 / 302.30000 / 1.07465 | 2.468701 | 0.041454 |
| 262101991 | 0.69000 / 6.90000 / 0.02455 | 40.15000 / 401.50000 / 1.42832 | 30.25000 / 302.50000 / 1.07613 | 2.442211 | 0.269544 |
| 262132524 | -0.54300 / -54.30000 / -0.83926 | -0.98000 / -98.00000 / -1.51468 | -1.15000 / -115.00000 / -1.77743 | 8.091963 | 0.383555 |
| 262132606 | -0.54300 / -54.30000 / -0.83926 | -0.98000 / -98.00000 / -1.51468 | -1.15000 / -115.00000 / -1.77743 | 8.085045 | 0.392967 |
| 262172983 | 638.52000 / 6,385.20000 / 1.77890 | 1,199.67000 / 11,996.70000 / 3.34226 | 920.55000 / 9,205.50000 / 2.56463 | 2.896713 | 0.049299 |
| 262174895 | -1.99000 / -19.90000 / -0.07822 | -10.39000 / -103.90000 / -0.40841 | -10.96000 / -109.60000 / -0.43082 | 7.191222 | 0.106292 |
| 262188472 | -7.85000 / -78.50000 / -0.61520 | -20.63000 / -206.30000 / -1.61677 | -23.85000 / -238.50000 / -1.86912 | 5.642623 | 0.149971 |
| 262213604 | -22.02000 / -220.20000 / -1.51236 | -18.22000 / -182.20000 / -1.25137 | -19.53000 / -195.30000 / -1.34135 | 5.642376 | 0.023245 |


All 30 endpoints are supported by a quote at or before the requested time no more than six seconds old. Export segments were hashed and row-count checked. The longest observed update gap is about **8.092 seconds** in oil. Full four-hour elapsed coverage does not certify an uninterrupted tape, and maxima describe retained quotes only. No interpolation or future quote was substituted for an endpoint.

Interpretation by exit type:

1. **Initial stop-loss exits: none in this overnight close cohort.** Oil and BTC were BE-stop exits, not initial-stop losses. The JSON retains the distinct classification.
2. **Oil BE protection preserved capital in these paths.** Both exits were followed roughly 35 seconds later by original-stop crossings, with maximum subsequent adverse movement about **2.855 original R** from the exit. Neither original target was observed within four hours.
3. **BTC BE protection had an opportunity cost on this path.** Position 262172983 reached about **1.343R** observed MFE before its BE close. It later touched original TP at 22:11:31, with no original-stop crossing observed. Subsequent maximum favourable movement from exit was **4.153R**. This one path does not establish that removing BE improves a full portfolio.
4. **The ETH deadline close was not rescued.** Position 262174895 had only **0.793R** observed MFE while open, no BE amendment, and closed for −$0.74. Its subsequent path crossed original SL at 00:06:52 and did not reach original TP within four hours. This differs from the earlier London ETH deadline paths that later touched target.
5. **ETH target exits realised gains before reversal.** Both already-profitable targets were subsequently followed by original-stop crossings around 00:06:49. Counting their post-exit target re-touches as rescued losses would be wrong.
6. **The two prior BTC follow-ups are now complete.** Positions 262076304/262076346 continued favourably after already taking profit, reaching about **5.254R/5.310R** additional observed favourable excursion from exit. No original-stop crossing was observed. Their earlier realised cash remains assigned to №18,444.

Original-level touch tests are not a full hold/re-entry counterfactual. They do not replay intervening BE, account pair occupancy, slippage, financing or later signals. There is no automatic trading recommendation or policy change from these descriptive paths.

## §7. Veto funnel and individual outcome integrity


| Account | New arms | Consumed = intents | Fills | Terminal refusals | Deep-retracement invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 8 | 0 | 0 | 0 | 3 |
| 46130058 | 5 | 4 | 2 | 2 | 1 |
| 47790949 | 9 | 4 | 3 | 1 | 4 |
| 49236051 | 9 | 2 | 1 | 1 | 7 |


Totals: **31 new arms, 10 consumed triggers, 10 persisted opening intents, six fills, 15 >61.8% invalidation receipts and 150 pair-reservation diagnostic rows**. Four final peer refusals comprise **three opposing-peer conflicts** and **one peer below 24 bps**. The peer threshold is enforced at dispatch; an earlier arm does not guarantee a later admissible peer.

Each consumed decision ID joins to one individual terminal outcome and its opening intent. **Consumed triggers without individual outcome = 0; unknown entry-guard receipts = 0.** Max confirmation-to-final-guard latency is **21.843 ms**. Idle diagnostics can be deduplicated, so the 150 rows are not 150 unique trades or every tick's pair refusal.


| Consumed → outcome ID | Account / symbol | Consumed SGT | Terminal status / exact reason |
| --- | --- | --- | --- |
| 118878 → 118880 | 49236051 / ETHUSD | 12 Nov 20:13:06.393 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 118882 → 118884 | 47790949 / ETHUSD | 12 Nov 20:13:19.935 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 119666 → 119671 | 47790949 / BTCUSD | 12 Nov 21:08:41.625 | filled / Filled { position_id: Some(262172983), price: Some(72637.94), volume: Some(41) } |
| 119877 → 119880 | 49236051 / ETHUSD | 12 Nov 21:12:38.203 | filled / Filled { position_id: Some(262174895), price: Some(2001.34), volume: Some(18) } |
| 120195 → 120197 | 46130058 / ETHUSD | 12 Nov 21:31:42.224 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 120381 → 120384 | 46130058 / ETHUSD | 12 Nov 21:43:04.527 | filled / Filled { position_id: Some(262188472), price: Some(2011.44), volume: Some(1180) } |
| 121005 → 121009 | 47790949 / ETHUSD | 12 Nov 22:35:46.971 | filled / Filled { position_id: Some(262213604), price: Some(2010.61), volume: Some(1021) } |
| 123339 → 123343 | 47790949 / XAGUSD | 13 Nov 00:01:02.662 | filled / Filled { position_id: Some(262253752), price: Some(50.016), volume: Some(25000) } |
| 123341 → 123348 | 46130058 / XAGUSD | 13 Nov 00:01:05.912 | filled / Filled { position_id: Some(262253774), price: Some(50.012), volume: Some(30000) } |
| 123424 → 123426 | 46130058 / BTCUSD | 13 Nov 00:08:44.583 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |


Live had **eight arms, three invalidation receipts, zero consumed triggers and zero persisted S3 opening intents**. Arm and invalidation counts are not a closed one-to-one cohort; arms can carry and remain active. Current gates below are a separately timed snapshot and must not be retrospectively assigned to every earlier live non-entry. The retained diagnostics do not uniquely explain every pre-intent path.


| Live symbol | Exact gate | Current observed message |
| --- | --- | --- |
| BTCUSD | peer_hold_below_24bps_or_opposing | Vetoed: peer ETHUSD < 24 bps or opposing (16.08 signed bps); mandatory at dispatch |
| ETHUSD | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| NAS100 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 0.860%, 20d: -4.154%) |
| SpotBrent | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.619%, 20d: -11.370%) |
| SpotCrude | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.911%, 20d: -10.263%) |
| US500 | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| XAUUSD | yield_to_human_position_or_order | Blocked: yield_to_human_position_or_order |


## §8. Overnight opening evidence and Friday readiness

All **48 overnight account-symbol-window slots** have first recorded two-sided quotes within the native opening window. **46 have an exact operational quote/time match** after reconciling current references, stored session/arm evidence and the previous report's verified 20:00 snapshot: crypto 20:00 **8/8**, NY 22:30 **24/24**, crypto 00:00 **6/8**, crypto 04:00 **8/8**.

The remaining two are **00:00 BTCUSD on live 43002148 and demo 46130058**. Fresh recorded opening quotes exist at 00:00:00.055092 and 00:00:00.049066 respectively. One retained session row records an early unavailable-opening state; the other has no matching session receipt. The current actor has already advanced to 04:00. Exact operational use of those midnight references is therefore **not verifiable from retained evidence**, not proof of permanent missing market data or a trading failure. Four initially negative 20:00 rows are contradicted by later stored arm/previous runtime evidence using the exact original quotes; they must not be counted as lasting outages. This preserves the known distinction between first-attempt durable diagnostics and later in-memory recovery.


| Account | Symbol | Session SGT | First quote delay s | Mid | Operational source / exact match |
| --- | --- | --- | --- | --- | --- |
| 43002148 | NAS100 | 12 Nov 22:30:00.000 | 0.189519 | 27,964.55000 | runtime_actor_opening_reference / yes |
| 43002148 | BTCUSD | 12 Nov 20:00:00.000 | 0.099951 | 72,850.76000 | stored_session_receipt / yes |
| 43002148 | BTCUSD | 13 Nov 00:00:00.000 | 0.055092 | 71,650.78000 | Not verifiable: historical operational receipt gap |
| 43002148 | BTCUSD | 13 Nov 04:00:00.000 | 0.052021 | 71,598.53000 | runtime_actor_opening_reference / yes |
| 49236051 | SpotBrent | 12 Nov 22:30:00.000 | 0.009313 | 95.99000 | runtime_actor_opening_reference / yes |
| 43002148 | XAUUSD | 12 Nov 22:30:00.000 | 0.189513 | 3,903.32500 | runtime_actor_opening_reference / yes |
| 43002148 | ETHUSD | 12 Nov 20:00:00.000 | 0.099950 | 2,020.65000 | stored_session_receipt / yes |
| 43002148 | ETHUSD | 13 Nov 00:00:00.000 | 0.055084 | 2,006.88000 | stored_session_receipt / yes |
| 43002148 | ETHUSD | 13 Nov 04:00:00.000 | 0.095310 | 2,014.79000 | runtime_actor_opening_reference / yes |
| 43002148 | XAGUSD | 12 Nov 22:30:00.000 | 0.087953 | 50.40000 | runtime_actor_opening_reference / yes |
| 43002148 | US500 | 12 Nov 22:30:00.000 | 0.122879 | 7,562.10000 | runtime_actor_opening_reference / yes |
| 49236051 | NAS100 | 12 Nov 22:30:00.000 | 0.009307 | 27,960.15000 | runtime_actor_opening_reference / yes |
| 49236051 | US500 | 12 Nov 22:30:00.000 | 0.186620 | 7,562.10000 | runtime_actor_opening_reference / yes |
| 49236051 | XAGUSD | 12 Nov 22:30:00.000 | 0.063702 | 50.40000 | runtime_actor_opening_reference / yes |
| 49236051 | BTCUSD | 12 Nov 20:00:00.000 | 0.121660 | 72,850.76000 | stored_session_receipt / yes |
| 49236051 | BTCUSD | 13 Nov 00:00:00.000 | 0.065098 | 71,650.78000 | stored_session_receipt / yes |
| 49236051 | BTCUSD | 13 Nov 04:00:00.000 | 0.091162 | 71,598.53000 | runtime_actor_opening_reference / yes |
| 49236051 | SpotCrude | 12 Nov 22:30:00.000 | 0.465365 | 86.84800 | runtime_actor_opening_reference / yes |
| 49236051 | ETHUSD | 12 Nov 20:00:00.000 | 0.121657 | 2,020.65000 | stored_arm_opening_quote:119628 / yes |
| 49236051 | ETHUSD | 13 Nov 00:00:00.000 | 0.065094 | 2,006.88000 | stored_session_receipt / yes |
| 49236051 | ETHUSD | 13 Nov 04:00:00.000 | 0.139337 | 2,014.79000 | runtime_actor_opening_reference / yes |
| 43002148 | SpotCrude | 12 Nov 22:30:00.000 | 0.417803 | 86.84800 | runtime_actor_opening_reference / yes |
| 49236051 | XAUUSD | 12 Nov 22:30:00.000 | 0.186621 | 3,903.32500 | runtime_actor_opening_reference / yes |
| 43002148 | SpotBrent | 12 Nov 22:30:00.000 | 0.024640 | 95.99000 | runtime_actor_opening_reference / yes |
| 46130058 | NAS100 | 12 Nov 22:30:00.000 | 0.140755 | 27,964.55000 | runtime_actor_opening_reference / yes |
| 46130058 | US500 | 12 Nov 22:30:00.000 | 0.140753 | 7,562.10000 | runtime_actor_opening_reference / yes |
| 47790949 | XAUUSD | 12 Nov 22:30:00.000 | 0.157431 | 3,903.32500 | runtime_actor_opening_reference / yes |
| 47790949 | BTCUSD | 12 Nov 20:00:00.000 | 0.132751 | 72,850.76000 | stored_session_receipt / yes |
| 47790949 | BTCUSD | 13 Nov 00:00:00.000 | 0.063601 | 71,650.78000 | stored_session_receipt / yes |
| 47790949 | BTCUSD | 13 Nov 04:00:00.000 | 0.051764 | 71,598.53000 | runtime_actor_opening_reference / yes |
| 46130058 | XAUUSD | 12 Nov 22:30:00.000 | 0.140754 | 3,903.32000 | runtime_actor_opening_reference / yes |
| 46130058 | SpotBrent | 12 Nov 22:30:00.000 | 0.020364 | 95.99000 | runtime_actor_opening_reference / yes |
| 46130058 | ETHUSD | 12 Nov 20:00:00.000 | 0.141899 | 2,020.65000 | stored_arm_opening_quote:120200 / yes |
| 46130058 | ETHUSD | 13 Nov 00:00:00.000 | 0.049062 | 2,006.88000 | stored_session_receipt / yes |
| 46130058 | ETHUSD | 13 Nov 04:00:00.000 | 0.135590 | 2,014.79000 | runtime_actor_opening_reference / yes |
| 47790949 | XAGUSD | 12 Nov 22:30:00.000 | 0.043485 | 50.40000 | runtime_actor_opening_reference / yes |
| 47790949 | US500 | 12 Nov 22:30:00.000 | 0.157427 | 7,562.10000 | runtime_actor_opening_reference / yes |
| 47790949 | NAS100 | 12 Nov 22:30:00.000 | 0.157433 | 27,964.55000 | runtime_actor_opening_reference / yes |
| 47790949 | SpotCrude | 12 Nov 22:30:00.000 | 0.487026 | 86.84800 | runtime_actor_opening_reference / yes |
| 46130058 | SpotCrude | 12 Nov 22:30:00.000 | 0.467057 | 86.84800 | runtime_actor_opening_reference / yes |
| 47790949 | SpotBrent | 12 Nov 22:30:00.000 | 0.487020 | 95.97500 | runtime_actor_opening_reference / yes |
| 46130058 | XAGUSD | 12 Nov 22:30:00.000 | 0.081611 | 50.40000 | runtime_actor_opening_reference / yes |
| 46130058 | BTCUSD | 12 Nov 20:00:00.000 | 0.141907 | 72,850.76000 | stored_arm_opening_quote:120775 / yes |
| 46130058 | BTCUSD | 13 Nov 00:00:00.000 | 0.049066 | 71,650.78000 | Not verifiable: historical operational receipt gap |
| 46130058 | BTCUSD | 13 Nov 04:00:00.000 | 0.082782 | 71,598.53000 | runtime_actor_opening_reference / yes |
| 47790949 | ETHUSD | 12 Nov 20:00:00.000 | 0.132746 | 2,020.65000 | stored_arm_opening_quote:119627 / yes |
| 47790949 | ETHUSD | 13 Nov 00:00:00.000 | 0.063597 | 2,006.88000 | stored_session_receipt / yes |
| 47790949 | ETHUSD | 13 Nov 04:00:00.000 | 0.093731 | 2,014.79000 | runtime_actor_opening_reference / yes |


Four actors are connected and unpaused on the unchanged production commit, with session generation 1, no actor quote/execution receiver lag recorded, and zero unresolved opening attempts in the health snapshot. These are liveness observations, not financial acceptance certificates.

Friday preparation has **32/32 complete tables**: **07:00 Asian 20/20** (five traditional symbols per account), **08:00 crypto 8/8**, and **09:00 Brent 4/4**. Metadata, spread/commission preparation, causal macro context and all required marks pass; no preparation source/macro error or missing mark remains in these tables. Future opening references do not exist yet at the audit snapshot, and live human ownership/pair locks still apply. Prepared does not mean dispatch authorised.


| Account | Symbol | Prepared session SGT | Base prepared | All marks prepared | Missing marks |
| --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 43002148 | ETHUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 43002148 | NAS100 | 13 Nov 07:00:00.000 | True | True | 0 |
| 43002148 | SpotBrent | 13 Nov 09:00:00.000 | True | True | 0 |
| 43002148 | SpotCrude | 13 Nov 07:00:00.000 | True | True | 0 |
| 43002148 | US500 | 13 Nov 07:00:00.000 | True | True | 0 |
| 43002148 | XAGUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 43002148 | XAUUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 46130058 | BTCUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 46130058 | ETHUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 46130058 | NAS100 | 13 Nov 07:00:00.000 | True | True | 0 |
| 46130058 | SpotBrent | 13 Nov 09:00:00.000 | True | True | 0 |
| 46130058 | SpotCrude | 13 Nov 07:00:00.000 | True | True | 0 |
| 46130058 | US500 | 13 Nov 07:00:00.000 | True | True | 0 |
| 46130058 | XAGUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 46130058 | XAUUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 47790949 | BTCUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 47790949 | ETHUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 47790949 | NAS100 | 13 Nov 07:00:00.000 | True | True | 0 |
| 47790949 | SpotBrent | 13 Nov 09:00:00.000 | True | True | 0 |
| 47790949 | SpotCrude | 13 Nov 07:00:00.000 | True | True | 0 |
| 47790949 | US500 | 13 Nov 07:00:00.000 | True | True | 0 |
| 47790949 | XAGUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 47790949 | XAUUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 49236051 | BTCUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 49236051 | ETHUSD | 13 Nov 08:00:00.000 | True | True | 0 |
| 49236051 | NAS100 | 13 Nov 07:00:00.000 | True | True | 0 |
| 49236051 | SpotBrent | 13 Nov 09:00:00.000 | True | True | 0 |
| 49236051 | SpotCrude | 13 Nov 07:00:00.000 | True | True | 0 |
| 49236051 | US500 | 13 Nov 07:00:00.000 | True | True | 0 |
| 49236051 | XAGUSD | 13 Nov 07:00:00.000 | True | True | 0 |
| 49236051 | XAUUSD | 13 Nov 07:00:00.000 | True | True | 0 |


## §9. Assessment and bounded next work

The overnight S3 cash recovery is real in the reconciled demo ledger, while live external losses are a separate and larger cash movement. Two larger demos have reached **50% on their latest20**, but that is an overlapping realised window, not proof that every asset class has a durable 50% edge. Three tiny positive BE fills contribute to the seven wins; outcome magnitude and cost matter alongside win count.

The observed exit trade-off is mixed: oil BE and ETH TP/deadline exits avoided later adverse movement, while BTC's BE and earlier TP exits left subsequent favourable movement unclaimed. Selecting only BTC would be hindsight selection. Any later exit research should preserve a frozen cohort, costs, occupancy, native deadlines and independent validation, and include the adverse oil/ETH paths. This directive authorises reporting only; it does not authorise such a policy change.

Next read-only audit should reconcile the two silver positions through their **05:59 SGT** native deadline, capture the actual Friday 07:00/08:00/09:00 first quotes, and distinguish live human-metal locks from unavailable market/preparation data. The two historical midnight BTC operational receipt gaps and per-arm admission attribution remain observability follow-ups; no repair is made here. External gold additions and pending-order margin should remain visibly separated from S3 performance.

## §10. Verification, provenance and release boundary

Four focused analytical tests passed: signed price/pip/R arithmetic and stale-endpoint handling; window/day cash and external deal-grain reconciliation; full follow-up denominators and touch classification; unique trigger outcomes, ownership and preparation evidence. **24 quote-export segments / 787,238 rows** reconcile to their uncompressed SHA256 hashes. Source queries and calculation scripts are embedded in the JSON; the report can be reviewed without the private workspace, though re-extracting original account data requires authorised database access.

Only the three named `doc/` report files change. Required CI is `check`, `docker`, `engineering`; the final release response supplies the merged PR/commit and post-merge deployment verification. The production baseline is deployment `c0fedaca-0370-4cec-9234-f59f8c7cafce`, commit `aebf404dedf89dbcdb8cee78c95a582eea042d51`, image `sha256:631e668c1ca138d5810f22fdab1ba27f3b7f4f34c6a28fa544c7ad09c2a9fad3`. Report paths are outside Trading deployment watch patterns.

No trading Rust/Python code, alpha, risk, 1R BE, 1.38R TP, margin threshold, pair cap, human-protection law, account setting or deployment is changed. `new_broker_requests = 0` refers to this audit; the already-running trading service continues its normal authorised work.

**Gemini reading instructions:** read the merged Markdown and JSON together. Review `ledger`, `post_close_summary`, `human_protection`, `telemetry`, `opening_audit`, `preparations` and `methodology`. Check eight overnight closes plus two prior follow-ups, four carryover resolutions, the midnight day split, external reversal-related deal grain, and seven cash wins versus four TP completions. Do not reinterpret post-close touches as a backtested replacement strategy. The two historical operational opening gaps remain unverified. **Financial pass: false.**


| Quote segment | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 49236051-BTCUSD-2026-11-12-092555.csv.gz | 56463 | b126fd3851b3ee83c19eb55e6de4aa84f5d1c2a36a9dd313936112af4eecde4d |
| 49236051-BTCUSD-2026-11-12-132555.csv.gz | 36654 | 205a453d3186b8f1ea0a84b384280a703d5500eeb7187e3b93e3c1c94f8ebe90 |
| 47790949-BTCUSD-2026-11-12-092556.csv.gz | 56467 | cc5fe27172e60ff9fc8e6ffb15bfe4fa146d22547484604789858d6f74257c67 |
| 47790949-BTCUSD-2026-11-12-132556.csv.gz | 36628 | 2657a52ec9a6c8360ebf9e7d4e54d3c8ae3e56e08d28d6e7490c591f94ad7cfd |
| 46130058-XAUUSD-2026-11-12-100707.csv.gz | 41984 | 271def824de6d672e8d797529d853f5c1080fe697d901d9f754e8113593365f8 |
| 46130058-XAUUSD-2026-11-12-140707.csv.gz | 45423 | a857c66bf934b037f42f3d8901af92b097fbfa2cd5d157425dcd683207c26027 |
| 47790949-XAUUSD-2026-11-12-100710.csv.gz | 42015 | 40a639464d2f355660bad43d74268c699ff63dcd37b7a4188e7cb06d430063a5 |
| 47790949-XAUUSD-2026-11-12-140710.csv.gz | 45413 | 19544c3fa33e5f75d5c92837142a6647103f6e69fe09a4a1faa86c117e55944c |
| 49236051-SpotCrude-2026-11-12-111629.csv.gz | 25161 | fcb4c54ba6edd7521b30444e13072df661736b186e0e82bb58ab0865043980f3 |
| 49236051-SpotCrude-2026-11-12-151629.csv.gz | 12528 | c6b5b4bed042055337135a8673426584f5b0381dc20147108ae012046991dbd9 |
| 46130058-SpotCrude-2026-11-12-111637.csv.gz | 25141 | e677706003271ba22f78a2cfa8e4afe8f5a540c9557815099a3d5b219dcac8a2 |
| 46130058-SpotCrude-2026-11-12-151637.csv.gz | 12529 | 99e6b9c65aee737fe293edb45d3af609da8654db4dd3e33ff929238d384f6151 |
| 47790949-BTCUSD-2026-11-12-130836.csv.gz | 65708 | c166e8c5b4e5396336ac6e695bc6af32f6640f36fb2f05d9b1343af6ff6a098d |
| 47790949-BTCUSD-2026-11-12-170836.csv.gz | 3360 | 287f89e8680ed94fdf37e763ca549221e428f853028e597b7087cb6d9fb48446 |
| 49236051-ETHUSD-2026-11-12-131232.csv.gz | 56753 | ea42d55488ea0253845a005d6a92781bafc7311637a087c16c3cf8d862e00443 |
| 49236051-ETHUSD-2026-11-12-171232.csv.gz | 32603 | 1ade30a723c15e6f1e57f75227cda414519a0ca7d6fb001c2729783fda6904f0 |
| 46130058-ETHUSD-2026-11-12-134259.csv.gz | 55810 | 42abcc229f273bd589a35fdab40dc39a595a8864faecdf4cb30077a52f968acb |
| 46130058-ETHUSD-2026-11-12-174259.csv.gz | 7174 | e5d30659e55332a89686b258d2285e46ad778f166a96028f84e74612187838c9 |
| 47790949-ETHUSD-2026-11-12-143541.csv.gz | 53489 | 0a69e63e6789dedf39a45678b424da4bdc434c06ffe5b9bce7d82a3fb37cbf18 |
| 47790949-ETHUSD-2026-11-12-183541.csv.gz | 5629 | 9a84f68e648db48ca47d65bd5ab2eedaaada23bd77b23466887c6f4205fe8ba6 |
| 47790949-XAGUSD-2026-11-12-160057.csv.gz | 29083 | 6f6b8a1e6bc3b90edc48c5de3b92815aa5e950c3843bf72b20a6f172586b2816 |
| 47790949-XAGUSD-2026-11-12-200057.csv.gz | 6050 | 4f394a933ebd5792fa7707204730748afe8ab28ff4d4e2c09ac09b7e0d481b37 |
| 46130058-XAGUSD-2026-11-12-160100.csv.gz | 29120 | ad2863be1ad527c4ba2e629f9e7d5ae7eff5eff4328012f94e362a51920b41a1 |
| 46130058-XAGUSD-2026-11-12-200100.csv.gz | 6053 | 70b567b177d05c31a36ee92990b1f778a337513fefe04e0d3115f5033fc91c24 |



| Source | SHA256 |
| --- | --- |
| opening-ticks.json | 09b6ee41c0adc59108aac86060ca07be607366a2e0dd2b246278b7020432f323 |
| preflight.json | f927c51a73acb90aae675bba7846a114c702773d69a0a57b4c6aa32068fc1b56 |
| monitor.json | 7f7cc276f22b4fd24d6bf37f4df5f0972f56f922a0104e8d855de2763075740c |
| human.json | 8d4f26ae69a85130f9d6eecfb602bced3f19f8cf33da0d1ef029b336df6857d2 |
| health.json | 9fbe9b69feba27befde7755b7ebfdc5e9743f51de5a6b4c116597c480fe75a71 |
| activity.json | abeb9cfef8031efccec670324af312c7d67094cb5b0caba88e1c3de3948a1b5c |
| evidence.json | e7b0dad3acb55e8dc9850ff218b44de50efac4cc42899d3157e438dffee2ce88 |
| deployment-before.json | 6cab90a4e75ee1ed9ad74d89b5cb42a8c91d062e0a39cbe9e231c04b4049604a |
| runtime.json | 4a82983569dc88eb043837d978f254dd2ade7962e5eba2e5a093aef2547ea9c3 |
