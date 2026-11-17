# Tuesday Asia & London Review — №18,483

**Activity:** [17 Nov 2026 04:45:00,18:00:00) Asia/Singapore. **Read-only production/database/broker audit.** Production commit `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`, policy `s3_macro_pullback_owner_18409_v1`.

[Measured JSON and evidence](TUESDAY_ASIA_LONDON_REVIEW-18483-2026-11-17.json) · [Standalone HTML](TUESDAY_ASIA_LONDON_REVIEW-18483-2026-11-17.html)

## §1. Assessment

- **The 120-second pre-close buffer worked in production:** carried demo Gold shorts 263304831 /263304849 filled at **05:57:01.495 /05:57:01.609 SGT**, well before 05:59 maintenance. Combined realised cash **−$69.92**; no recorded rollover MARKET_CLOSED rejections or repeated session-end close attempts. This verifies execution hygiene, not trading profitability. [1,2]
- **Window S3 cash: −$703.87 from six losing closes**, comprising the two Gold time exits plus four ETH initial stops. No S3 position remains open at 18:00 or in the later owner book. Full Tuesday cash is **−$1,019.91**; post-PR 214 cumulative closing cash **+$1,714.66**. [1,3]
- **Latest 20 wins: live 3/20; demos 6/20,7/20,12/20.** None meets the requested **15/20 (75%)** target. Both large-demo rolling Net R totals are now negative. **Financial pass=false.**
- **Live balance bridge: $27.39 +$0.04 dividend +$200 deposit = $227.43.** No external or S1 close occurred during this activity window. Separately captured live per-position net floating P&L totals **$1,757.47** and equity **$1,984.90** at 18:06:04.446742. [4]
- Live assessed margin was **115.82%**, pending projection **83.17%**, stress **−96.27%** in the earlier owner assessment. These fail the 250% new-entry floor. The one recorded live program action was an owned S1 AUS200 stop amendment; **zero recorded interference on human inventory**. [4]
- **34 arms →9 consumed triggers/intents →4 fills +5 terminal refusals.** Four peer-hold refusals and one pullback-bound refusal; missing individual outcomes and unknown entry-guard receipts are zero. Maximum confirmation-to-guard latency **28.733 ms**. [5]
- **Post-close tracking:** four complete 4 h clocks and two partial clocks of approximately 2.962 h. No original target was crossed in the observed six paths. The Gold 1 h endpoints occur during maintenance and remain unavailable. [6]
- **Opening references:**72 source-quote slots;43 exact operational quote receipts, 21 recovered-midpoint proofs, 8 reference-withheld slots. **London base inputs 24/24, but full sigma tables 0/24:** its 420/450-minute marks have 11 same-slot endpoints, 14 needed. Future 20:00 crypto 8/8 and 22:30 NY 24/24 base/full preparation are available. [7]

## §2. Clock, boundaries and cash reconciliation

The first system clock read was **17 Nov 10:00:14.479943 UTC /18:00:14.479943 SGT**. As it was after 18:00, the directive fixes the exclusive activity cutoff at **18:00:00 SGT**. Post-close prices are separately frozen at **18:00:15 SGT**. Later money, book, broker-P&L and health snapshots retain their own timestamps; they do not extend the trading-cash interval.

This audit continues exactly from [№18,474 /PR #238](TUESDAY_PREDAWN_NY_REVIEW_AND_EXCURSIONS-18474-2026-11-17.md), merge `88ee90cc9d09e322a01aa7e20c18e1b79218df96`. Its 04:45 exclusive end is this audit's inclusive start. There were **zero unfinished prior excursion rows**; the two carried Gold shorts enter this window's closing ledger once. [1,3]


| Account | Window closes | Window S3 USD | Tuesday 00:00→18:00 USD | Post-PR214 S3 USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | -0.01 | 0 |
| 46130058 | 3 | -355.76 | -517.97 | 739.83 | 0 |
| 47790949 | 3 | -348.11 | -501.94 | 948.40 | 0 |
| 49236051 | 0 | 0.00 | 0.00 | 26.44 | 0 |


The arithmetic reconciles exactly: **Tuesday −$316.04 −$703.87 = −$1,019.91**; **cumulative $2,418.53 −$703.87 = $1,714.66**. Closing-deal IDs are unique by account and deal. Cash is broker gross plus signed swap minus absolute allocated closing commission and conversion fee, using recorded money digits and decimal arithmetic. Opening commission is not charged again.

The cumulative interval remains **[10 Nov 09:42,17 Nov 18:00) SGT**, including +$412.88 from three positions entered before its start. It is a mixed-entry closing-cash cohort, not an isolated causal return estimate for the release. The independently repeated activity query returned identical parsed results. Successful retained history fetches extend beyond the cutoff on all accounts. The broker history worker's rolling `complete=false` state is preserved rather than relabelled complete. [3]

## §3. First production execution verification of the 120-second buffer

Both original Gold entry receipts store **Session.close 05:59**, **exit deadline 05:57**, and policy `native_preclose_v18455_120s`. The session close and entry-band calendar were not changed. The exit helper in the deployed commit matches the inspected source byte-for-byte; its SHA256 is in the JSON. [2]


| Account / position | Close trigger SGT | Send SGT | Broker fill SGT | Fill price | 05:57 as-of ask / age | Fill minus deadline ask | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 263304831 | 17 Nov 05:57:00.873461 | 17 Nov 05:57:01.154692 | 17 Nov 05:57:01.495000 | 3,856.62 | 3,856.68 / 0.027242s | -0.06 | -34.88 / -0.245495 |
| 46130058 / 263304849 | 17 Nov 05:57:00.907021 | 17 Nov 05:57:01.203830 | 17 Nov 05:57:01.609000 | 3,856.65 | 3,856.45 / 4.592052s | 0.20 | -35.04 / -0.246344 |

| Position | Send→broker fill ms | Send→local fill receipt ms | Local fill receipt SGT | Original deadline / session close |
| --- | --- | --- | --- | --- |
| 263304831 | 340.308 | 462 | 17 Nov 05:57:01.617582 | 17 Nov 05:57:00.000000 / 17 Nov 05:59:00.000000 |
| 263304849 | 405.170 | 530 | 17 Nov 05:57:01.734654 | 17 Nov 05:57:00.000000 / 17 Nov 05:59:00.000000 |


The exact 05:57 comparison uses each account's last recorded ask at or before the deadline. Its age is **27.242 ms** on 47790949 and **4.592052 s** on 46130058; both pass the six-second endpoint bound. Those differently aged quotes must not be treated as simultaneous broker prices. Relative to the later decision asks 3856.69 /3856.72, both fills improved by 0.07. The deadline comparison and the send-to-fill measurement are separate quantities.

Across **[05:59,07:05) SGT**, the bounded audit found **zero rejection/error or explicit MARKET_CLOSED events**, **zero carried-Gold close intents**, and **zero Strategy 3 session-end attempts**. Both positions had already closed. This is the first observed successful deployment execution of the buffer on these two demo positions; it is not a statement that human Gold on Live 43002148 was closed. That human position remains external and protected.

## §4. Four-account money, margin and open inventory

The demo money marks are around **18:02:16 SGT**. Live money below comes from the separate read-only broker P&L response at **18:06:04.446742**, surrounded by matching book reads and stable balances. Owner book/margin assessment was captured at **17 Nov 18:01:18.683974**; health at **17 Nov 18:01:46.972654**. These are not an atomic 18:00 historical snapshot. [4]


| Account | Money snapshot SGT | Balance USD | Floating USD | Equity USD | Assessed margin % | Pending projected % | Pending stressed % | S3 / S1 / S2 / external | Pending |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 17 Nov 18:06:04.446742 | 227.43 | 1,757.47 | 1,984.90 | 115.82 | 83.17 | -96.27 | 0 / 1 / 0 / 5 | 3 |
| 46130058 | 17 Nov 18:02:16.077000 | 25,261.43 | 46.07 | 25,307.50 | 13,596.50 | Unverified / N/A | Unverified / N/A | 0 / 6 / 0 / 0 | 0 |
| 47790949 | 17 Nov 18:02:16.199000 | 25,139.68 | 46.03 | 25,185.71 | 1,692.01 | Unverified / N/A | Unverified / N/A | 0 / 6 / 0 / 0 | 0 |
| 49236051 | 17 Nov 18:02:16.094000 | 821.21 | -0.64 | 820.57 | 94,320.69 | Unverified / N/A | Unverified / N/A | 0 / 1 / 0 / 0 | 0 |


Live current, pending and stressed assessments all fail the 250% entry floor. The negative stress figure is a model projection, not a negative current equity observation or authority to close positions. Demo pending projections are unavailable because no pending entries are observed; unavailable values are not zeros. Position margins are not summed to invent a broker account margin under its MAX aggregation mode. Passing a margin check also does not override ownership or strategy gates.


| Account / ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | Current SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 56,284.20000 | Unverified / N/A |
| 43002148 / 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 56,323.40000 | Unverified / N/A |
| 43002148 / 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,831.60000 | Unverified / N/A |
| 43002148 / 657180931 | NVDA.US | human_external / position | buy | 400 / 4.0000 | 160.82500 | 149.66000 | Unverified / N/A |
| 43002148 / 658574004 | XAUUSD | human_external / position | buy | 3000 / 0.3000 | 3,841.86000 | 3,830.00000 | Unverified / N/A |
| 43002148 / 658793840 | NatGas | human_external / position | buy | 500000 / 0.5000 | 2.99500 | 2.96200 | Unverified / N/A |
| 43002148 / 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 / 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 / 402061476 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | Unverified / N/A |
| 46130058 / 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,831.70000 | Unverified / N/A |
| 46130058 / 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.68823 | Unverified / N/A |
| 46130058 / 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.11842 | Unverified / N/A |
| 46130058 / 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.60000 | Unverified / N/A |
| 46130058 / 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56289 | Unverified / N/A |
| 46130058 / 263178401 | USDCAD | s1 / position | buy | 1100000 / 0.1100 | 1.43006 | 1.41210 | Unverified / N/A |
| 47790949 / 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,831.70000 | Unverified / N/A |
| 47790949 / 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.68818 | Unverified / N/A |
| 47790949 / 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.11837 | Unverified / N/A |
| 47790949 / 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.60000 | Unverified / N/A |
| 47790949 / 262976216 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55202 | 0.56284 | Unverified / N/A |
| 47790949 / 263178410 | USDCAD | s1 / position | buy | 1100000 / 0.1100 | 1.43006 | 1.41215 | Unverified / N/A |
| 49236051 / 262796763 | ETHUSD | s1 / position | sell | 1 / 0.0100 | 1,746.87000 | 2,027.27000 | Unverified / N/A |


## §5. Live per-position P&L, balance bridge and human protection

The six live position IDs match the owner inventory. Broker P&L returned three additional zero marks allocated to unfilled pending entries; their pending-order linkage was verified and they are excluded from the open-position total. No marks are silently dropped. Gross plus the positions' recorded swap and signed commission reconciles to each net mark. Prospective closing commission is excluded from broker net unrealised P&L. [4]


| Position | Symbol / ownership | Gross float USD | Recorded swap USD | Recorded commission USD | Net float USD |
| --- | --- | --- | --- | --- | --- |
| 649992251 | JPN225 / human_external | 391.61 | -16.97 | 0.00 | 374.64 |
| 649772867 | JPN225 / human_external | 988.84 | -50.87 | 0.00 | 937.97 |
| 656795267 | AUS200 / s1 | -18.58 | -0.60 | 0.00 | -19.18 |
| 657180931 | NVDA.US / human_external | -19.58 | -0.34 | -0.08 | -20.00 |
| 658574004 | XAUUSD / human_external | 558.60 | -23.01 | -1.05 | 534.54 |
| 658793840 | NatGas / human_external | 20.00 | -70.50 | 0.00 | -50.50 |


Total net float is **$1,757.47**; balance **$227.43** plus that float gives **$1,984.90 equity**. In particular, NatGas has **+$20.00 gross price P&L but −$50.50 net**, explained by its recorded −$70.50 swap. Gold's +$558.60 gross becomes +$534.54 after −$23.01 swap and −$1.05 commission. These are observed broker amounts, not estimated financing rates.

The earlier stored live money mark at 18:02:16 showed **$1,813.95 float /$2,041.38 equity**. It is retained in the JSON, separate from the 18:06 per-position snapshot. Their difference is not assigned to an unexplained cash flow; the balance is unchanged and market marks were taken at different times.


| Broker balance operation | Execution SGT | Amount USD | Balance afterward USD |
| --- | --- | --- | --- |
| BALANCE_DEPOSIT_DIVIDENDS | 17 Nov 06:45:00.004000 | 0.04 | 27.43 |
| BALANCE_DEPOSIT | 17 Nov 17:55:42.901000 | 200.00 | 227.43 |


**Exact bridge: $27.39 +$0.04 dividend at 06:45:00.004 +$200 deposit at 17:55:42.901 = $227.43.** There are no external/S1 closing deals or withdrawals in the activity interval. The dividend record supplies a balance-operation classification, not a proven instrument attribution. Neither dividend nor deposit is S3 profit.

**Zero recorded S3/human interference.** The sole live program intent was an owned **S1 AUS200 position 656795267 stop amendment at 06:50:01.464**, taking its SL from 8830.7 to 8831.6. This is separate from S3. Human JPN225 stops changed to 56284.2 /56323.4, and the human NatGas SL to 2.962, without recorded bot intents against those positions; those changes are not attributed to the bot. NatGas/JPN225 remain reserved, external Gold retains its ownership lock and Silver pair constraint, and no live S3 position exists.

The large demos each recorded 61 successful S1 trailing amendments, with their six S1 positions still present. Demo 49236051 retains S1 ETH short 262796763, SL 2027.27, separately from S3; it blocks S3 crypto pair entry. No S1 entry or close is recorded in this activity window. Current S1 and pending inventories are listed in §4.

## §6. Complete S3 ledger and loss anatomy

There are **six active-window S3 positions, all closed**: two carried Gold shorts and four new ETH shorts. Original R is the actual entry-to-initial-SL distance; net R uses actual volume and recorded conversion. There were no BE amendments in this cohort. [1]


| Account / position | Symbol / dir | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Reason | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 263304831 | XAUUSD / sell | 800 / 0.0800 | 17 Nov 02:49:27.938000 / 3,852.33000 | 17 Nov 05:57:01.495000 / 3,856.62000 | 3,870.09000 / 3,827.82000 | session end | -34.88 / -0.245495 |
| 46130058 / 263304849 | XAUUSD / sell | 800 / 0.0800 | 17 Nov 02:49:28.612000 / 3,852.27000 | 17 Nov 05:57:01.609000 / 3,856.65000 | 3,870.05000 / 3,827.73000 | session end | -35.04 / -0.246344 |
| 46130058 / 263416267 | ETHUSD / sell | 1323 / 13.2300 | 17 Nov 08:25:00.645000 / 1,813.15000 | 17 Nov 09:42:01.754000 / 1,825.69000 | 1,824.79000 / 1,797.08000 | initial stop | -165.90 / -1.077292 |
| 47790949 / 263416280 | ETHUSD / sell | 1316 / 13.1600 | 17 Nov 08:25:01.730000 / 1,811.39000 | 17 Nov 09:40:41.498000 / 1,823.50000 | 1,823.03000 / 1,795.32000 | initial stop | -159.37 / -1.040394 |
| 47790949 / 263516658 | ETHUSD / sell | 1021 / 10.2100 | 17 Nov 14:42:32.245000 / 1,817.96000 | 17 Nov 15:02:31.186000 / 1,833.03000 | 1,832.85000 / 1,797.41000 | initial stop | -153.86 / -1.012058 |
| 46130058 / 263516659 | ETHUSD / sell | 1026 / 10.2600 | 17 Nov 14:42:32.286000 / 1,817.95000 | 17 Nov 15:02:31.177000 / 1,833.04000 | 1,832.84000 / 1,797.40000 | initial stop | -154.82 / -1.013410 |

| Position | Held MFE / MAE R | BE amendment | Exit deadline SGT | Stored policy |
| --- | --- | --- | --- | --- |
| 263304831 | 0.189189 / 0.808559 | None observed | 17 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 263304849 | 0.183915 / 0.812711 | None observed | 17 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 263416267 | 0.267182 / 1.011168 | None observed | 17 Nov 12:00:00.000000 | native_preclose_v18455_120s |
| 263416280 | 0.115979 / 1.036942 | None observed | 17 Nov 12:00:00.000000 | native_preclose_v18455_120s |
| 263516658 | 0.000000 / 1.009402 | None observed | 17 Nov 16:00:00.000000 | native_preclose_v18455_120s |
| 263516659 | 0.000000 / 1.010074 | None observed | 17 Nov 16:00:00.000000 | native_preclose_v18455_120s |


**Gold −$69.92** reflects the scheduled close, not a target or BE event. **ETH −$633.95** comprises two 08:25 entries stopped around 09:40–09:42 and two 14:42 entries stopped at 15:02:31. The latter are crypto 12:00-window setups spanning the London open, not new traditional London entries. All four ETH exits are initial stops, with realised losses slightly exceeding 1 R after execution/cost effects. No candidate policy, risk parameter or exit was changed.

This is **0/6 positive-net closes** in the window. Account copies of the same setup are correlated; six losing account positions do not represent six independent market hypotheses. The execution-hygiene repair succeeding does not imply that the signal has a profitable edge. No post-close recovery is added to realised cash.


| Account | Latest20 wins / count | Win rate | Rolling net R | Realised payoff (R) | Profit factor (R) | Assessment SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | -5.967682 | 3.4277 | 0.6049 | 17 Nov 17:55:12.796055 |
| 46130058 | 6/20 | 30.00% | -4.814039 | 1.1802 | 0.5058 | 17 Nov 17:59:20.049650 |
| 47790949 | 7/20 | 35.00% | -3.242063 | 1.2259 | 0.6601 | 17 Nov 17:54:27.258713 |
| 49236051 | 12/20 | 60.00% | 6.204701 | 1.3690 | 2.0534 | 17 Nov 17:56:30.352577 |


The target is explicitly **15/20 (75%)**. Large-demo wins fell 8→6 and 9→7 from №18,474; live 3 and small-demo 12 are unchanged. The rolling windows are account-level S3 history, not a fresh independent test of the current release. **Financial pass=false.**

## §7. Mandatory four-hour post-close pip excursions

The price horizon is **18:00:15 SGT**. Gold and the first ETH pair have complete **4.000 h clocks**; the 15:02:31 ETH pair has approximately **2.962 h**, ending fully at **19:02:31 SGT**. The counts for non-BE initial stops and BE stops are explicitly disjoint. Neither category has an observed original-TP crossing in this audit. [6]

For these six shorts, movement is signed from the actual exit fill using the ask side. Broker pip size is **0.1** for Gold/ETH/BTC and 0.01 for oil; other symbols use their recorded broker `pip_position`, with no FX/index position in this excursion cohort. Favourable/adverse columns are **price / pips / original R**. Original TP/SL tests use absolute original price levels.


| Position / symbol | Exit / initial SL / TP | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Orig TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 263304831 / XAUUSD | 3,856.620 / 3,870.090 / 3,827.820 | 4.000000 COMPLETE | 13.10000 / 131.00 / 0.737613 | 15.79000 / 157.90 / 0.889077 | stale_asof_quote | -73.80 / -0.415541 | -41.80 / -0.235360 | False / True |
| 263304849 / XAUUSD | 3,856.650 / 3,870.050 / 3,827.730 | 4.000000 COMPLETE | 13.10000 / 131.00 / 0.736783 | 15.79000 / 157.90 / 0.888076 | stale_asof_quote | -72.80 / -0.409449 | -41.80 / -0.235096 | False / True |
| 263416267 / ETHUSD | 1,825.690 / 1,824.790 / 1,797.080 | 4.000000 COMPLETE | 16.66000 / 166.60 / 1.431271 | 17.98000 / 179.80 / 1.544674 | 96.40 / 0.828179 | -44.90 / -0.385739 | -93.30 / -0.801546 | False / True |
| 263416280 / ETHUSD | 1,823.500 / 1,823.030 / 1,795.320 | 4.000000 COMPLETE | 14.47000 / 144.70 / 1.243127 | 20.17000 / 201.70 / 1.732818 | 63.70 / 0.547251 | -80.50 / -0.691581 | -139.10 / -1.195017 | False / True |
| 263516658 / ETHUSD | 1,833.030 / 1,832.850 / 1,797.410 | 2.962171 PARTIAL | 28.38000 / 283.80 / 1.905977 | 2.63000 / 26.30 / 0.176629 | 27.50 / 0.184688 | 181.80 / 1.220954 | not_elapsed_at_capture | False / True |
| 263516659 / ETHUSD | 1,833.040 / 1,832.840 / 1,797.400 | 2.962173 PARTIAL | 28.39000 / 283.90 / 1.906649 | 2.62000 / 26.20 / 0.175957 | 27.60 / 0.185359 | 181.90 / 1.221625 | not_elapsed_at_capture | False / True |


**Gold:** four-hour favourable/adverse extrema are approximately 0.737 R /0.889 R from the exit. Original TP was not crossed; original SL was. The 1 h endpoint is 06:57, inside maintenance: the last quote is about 58 minutes old, so signed 1 h movement is **unavailable**, not zero. The 2 h and 4 h endpoints have fresh quotes after reopen.

**First ETH pair:** price recovered approximately 1.431 R /1.243 R from the stop fills but also moved adversely 1.545 R /1.733 R. Neither original target was crossed in its complete four-hour window. **Later ETH pair:** observed recovery reaches about 1.906 R from the stop fill, but this still does not equal the original entry-based target; no TP crossing is observed by 18:00:15. Both original stops were crossed again. Its 4 h endpoints remain null until 19:02:31.

There are 14 fresh elapsed checkpoints, two unavailable Gold maintenance checkpoints, and two unelapsed 4 h checkpoints. The Gold recording gap spans about 62 minutes across the closed market. Complete clock duration does not imply continuous quotes or executability. Exact extrema, endpoint ages, gap counts and crossing timestamps are retained in JSON; no interpolation or hypothetical fills are used.

## §8. Execution and veto funnel


| Account | Arms | Consumed / persisted intents / outcomes | Filled entries | Terminal refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 15 | 0 / 0 / 0 | 0 | 0 | 13 |
| 46130058 | 9 | 4 / 4 / 4 | 2 | 2 | 1 |
| 47790949 | 9 | 5 / 5 / 5 | 2 | 3 | 0 |
| 49236051 | 1 | 0 / 0 / 0 | 0 | 0 | 0 |

| Terminal guard reason | Count |
| --- | --- |
| ENTRY_PULLBACK_BOUND_EXCEEDED: executable price exceeds the funded structural bound | 1 |
| ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction | 4 |


All 9 consumed triggers join to an individual terminal outcome and an opening intent created after consumption and before the outcome. **4 fills +5 refusals =9 outcomes**. Missing individual outcomes and unknown guard receipts are both **zero**. There are 14 >61.8% invalidations and 68 pair-reservation diagnostic records. Repeated stage diagnostics are not additional independent rejected trades. [5]

Maximum confirmation-to-final-guard latency is **28.733 ms**; this excludes broker send-to-fill time and touch/turn waiting. Canonical coverage retains `refusals_all_stages_complete=false`: complete consumed-trigger accounting does not certify an exhaustive census of all deduplicated idle/pre-arm vetoes.


| Account | Symbol | Current separately timed gate |
| --- | --- | --- |
| 43002148 | BTCUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 82.08% below floor 250.00% |
| 43002148 | ETHUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 83.60% below floor 250.00% |
| 43002148 | NAS100 | Vetoed: peer US500 < 24 bps or opposing (20.52 signed bps); mandatory at dispatch |
| 43002148 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.842%, 20d: -13.876%) |
| 43002148 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.602%, 20d: -12.459%) |
| 43002148 | US500 | Latest assessment: impulse_below_48bps (2026-11-17 10:01:16.112243468 UTC) |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 8 |
| 46130058 | ETHUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 8 |
| 46130058 | NAS100 | Vetoed: peer US500 < 24 bps or opposing (20.52 signed bps); mandatory at dispatch |
| 46130058 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.842%, 20d: -13.888%) |
| 46130058 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.602%, 20d: -12.459%) |
| 46130058 | US500 | Latest assessment: impulse_below_48bps (2026-11-17 10:01:15.328409839 UTC) |
| 46130058 | XAGUSD | Waiting: opening impulse below 48 bps |
| 46130058 | XAUUSD | Waiting: opening impulse below 48 bps |
| 47790949 | BTCUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 8 |
| 47790949 | ETHUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 8 |
| 47790949 | NAS100 | Vetoed: peer US500 < 24 bps or opposing (20.52 signed bps); mandatory at dispatch |
| 47790949 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.842%, 20d: -13.888%) |
| 47790949 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.602%, 20d: -12.459%) |
| 47790949 | US500 | Latest assessment: impulse_below_48bps (2026-11-17 10:01:15.327544094 UTC) |
| 47790949 | XAGUSD | Waiting: opening impulse below 48 bps |
| 47790949 | XAUUSD | Waiting: opening impulse below 48 bps |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Vetoed: peer US500 < 24 bps or opposing (20.52 signed bps); mandatory at dispatch |
| 49236051 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.842%, 20d: -13.888%) |
| 49236051 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.602%, 20d: -12.459%) |
| 49236051 | US500 | Latest assessment: impulse_below_48bps (2026-11-17 10:01:15.308544664 UTC) |
| 49236051 | XAGUSD | Waiting: opening impulse below 48 bps |
| 49236051 | XAUUSD | Waiting: opening impulse below 48 bps |


## §9. Session openings and evening preparation

There are **72 requested opening slots**, all with a stored first two-sided quote inside the versioned 15-second observation window. Operational evidence is **43 exact quote receipts +21 recovered midpoint proofs +8 reference-withheld slots**. Ten slots have ownership/pair constraints; two of those retain an exact reference despite blocked entry. Source capture, operational reference and permission to enter are separate measurements. [7]


| Session open SGT | Slots / source quotes | Exact quote receipt | Recovered midpoint | Reference withheld |
| --- | --- | --- | --- | --- |
| 17 Nov 07:00:00.000000 | 20 | 1 | 17 | 2 |
| 17 Nov 08:00:00.000000 | 8 | 7 | 0 | 1 |
| 17 Nov 09:00:00.000000 | 4 | 0 | 4 | 0 |
| 17 Nov 12:00:00.000000 | 8 | 6 | 0 | 2 |
| 17 Nov 15:00:00.000000 | 24 | 22 | 0 | 2 |
| 17 Nov 16:00:00.000000 | 8 | 7 | 0 | 1 |


Asian metals use the versioned **07:01 broker observation start** under the 07:00 session identity. SpotCrude's longest observed opening delay is **6.859847 s**, within 15 s. Tuesday Brent uses **09:00**, matching its non-Monday native calendar; the Monday 07:00 exception is not applied to Tuesday. Recovered midpoint proofs match the first source quote numerically but do not certify full immutable bid/ask timestamp persistence.


| Account | Symbol | Session SGT | Observation start SGT | First recorded quote SGT | Delay s | Reference evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.044803 | 0.044803 | exact_quote_receipt |
| 43002148 | BTCUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.056427 | 0.056427 | exact_quote_receipt |
| 43002148 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.130666 | 0.130666 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.126234 | 0.126234 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.388308 | 0.388308 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.130664 | 0.130664 | exact_quote_receipt |
| 49236051 | BTCUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.054981 | 0.054981 | exact_quote_receipt |
| 49236051 | BTCUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.051311 | 0.051311 | ownership_locked |
| 49236051 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.043538 | 0.043538 | exact_quote_receipt |
| 49236051 | ETHUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.115431 | 0.115431 | ownership_locked |
| 49236051 | ETHUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.417390 | 0.417390 | ownership_locked |
| 49236051 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.105227 | 0.105227 | ownership_locked |
| 47790949 | BTCUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.184902 | 0.184902 | exact_quote_receipt |
| 47790949 | BTCUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.052949 | 0.052949 | exact_quote_receipt |
| 47790949 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.010058 | 0.010058 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.183837 | 0.183837 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.421422 | 0.421422 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.131905 | 0.131905 | exact_quote_receipt |
| 46130058 | BTCUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.018383 | 0.018383 | exact_quote_receipt |
| 46130058 | BTCUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.055085 | 0.055085 | exact_quote_receipt |
| 46130058 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.023301 | 0.023301 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.000000 | 17 Nov 08:00:00.135949 | 0.135949 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.000000 | 17 Nov 12:00:00.420355 | 0.420355 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.136749 | 0.136749 | exact_quote_receipt |
| 43002148 | NAS100 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.550717 | 0.550717 | exact_quote_receipt |
| 49236051 | SpotBrent | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.269565 | 0.269565 | exact_quote_receipt |
| 43002148 | XAUUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.324434 | 0.324434 | ownership_locked |
| 43002148 | XAGUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.219434 | 0.219434 | ownership_locked |
| 43002148 | US500 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.627694 | 0.627694 | exact_quote_receipt |
| 49236051 | NAS100 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.540087 | 0.540087 | exact_quote_receipt |
| 49236051 | US500 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.651692 | 0.651692 | exact_quote_receipt |
| 49236051 | XAGUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.269565 | 0.269565 | exact_quote_receipt |
| 49236051 | SpotCrude | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.348877 | 0.348877 | exact_quote_receipt |
| 43002148 | SpotCrude | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.385023 | 0.385023 | exact_quote_receipt |
| 49236051 | XAUUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.324466 | 0.324466 | exact_quote_receipt |
| 43002148 | SpotBrent | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.219435 | 0.219435 | exact_quote_receipt |
| 46130058 | NAS100 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.550943 | 0.550943 | exact_quote_receipt |
| 46130058 | US500 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.706099 | 0.706099 | exact_quote_receipt |
| 47790949 | XAUUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.340985 | 0.340985 | exact_quote_receipt |
| 46130058 | XAUUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.324057 | 0.324057 | exact_quote_receipt |
| 46130058 | SpotBrent | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.225355 | 0.225355 | exact_quote_receipt |
| 47790949 | XAGUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.246782 | 0.246782 | exact_quote_receipt |
| 47790949 | US500 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.670819 | 0.670819 | exact_quote_receipt |
| 47790949 | SpotCrude | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.425083 | 0.425083 | exact_quote_receipt |
| 46130058 | SpotCrude | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.415913 | 0.415913 | exact_quote_receipt |
| 47790949 | NAS100 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.548863 | 0.548863 | exact_quote_receipt |
| 47790949 | SpotBrent | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.246783 | 0.246783 | exact_quote_receipt |
| 46130058 | XAGUSD | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.000000 | 17 Nov 15:00:00.225354 | 0.225354 | exact_quote_receipt |
| 43002148 | NAS100 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.669367 | 0.669367 | recovered_midpoint_only |
| 43002148 | XAUUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.510469 | 0.510469 | ownership_locked |
| 43002148 | XAGUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.510468 | 0.510468 | ownership_locked |
| 43002148 | US500 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.669366 | 0.669366 | recovered_midpoint_only |
| 49236051 | NAS100 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.667530 | 0.667530 | recovered_midpoint_only |
| 49236051 | US500 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.667529 | 0.667529 | recovered_midpoint_only |
| 49236051 | XAGUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.560864 | 0.560864 | recovered_midpoint_only |
| 49236051 | SpotCrude | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:06.855525 | 6.855525 | recovered_midpoint_only |
| 43002148 | SpotCrude | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:06.830770 | 6.830770 | recovered_midpoint_only |
| 49236051 | XAUUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.560865 | 0.560865 | recovered_midpoint_only |
| 46130058 | NAS100 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.664589 | 0.664589 | recovered_midpoint_only |
| 46130058 | US500 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.664589 | 0.664589 | recovered_midpoint_only |
| 47790949 | XAUUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.516477 | 0.516477 | recovered_midpoint_only |
| 46130058 | XAUUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.510704 | 0.510704 | recovered_midpoint_only |
| 47790949 | XAGUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.516476 | 0.516476 | recovered_midpoint_only |
| 47790949 | US500 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.664181 | 0.664181 | recovered_midpoint_only |
| 47790949 | SpotCrude | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:06.845003 | 6.845003 | recovered_midpoint_only |
| 46130058 | SpotCrude | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:06.859847 | 6.859847 | recovered_midpoint_only |
| 47790949 | NAS100 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.000000 | 17 Nov 07:00:00.664182 | 0.664182 | exact_quote_receipt |
| 46130058 | XAGUSD | 17 Nov 07:00:00.000000 | 17 Nov 07:01:00.000000 | 17 Nov 07:01:00.510703 | 0.510703 | recovered_midpoint_only |
| 49236051 | SpotBrent | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.867289 | 0.867289 | recovered_midpoint_only |
| 43002148 | SpotBrent | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.865338 | 0.865338 | recovered_midpoint_only |
| 46130058 | SpotBrent | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.866214 | 0.866214 | recovered_midpoint_only |
| 47790949 | SpotBrent | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.000000 | 17 Nov 09:00:00.869241 | 0.869241 | recovered_midpoint_only |


All preparation receipts below precede 18:00. **20:00 crypto 8/8** and **22:30 NY 24/24** have base inputs and full marks ready. London 15:00 has **base 24/24 but full-table 0/24**, because every account/symbol lacks 420/450-minute marks: **11 same-slot endpoints;14 needed**. Earlier marks, including 120 minutes, are available. Those late marks would be reached around 22:00/22:30, beyond this 18:00 cutoff; the audit does not attribute the current losses to them. No incomplete late mark is silently passed or repaired.


| Account | Target open SGT | Slots | Base ready | Full marks ready | Latest causal prep receipt SGT |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 17 Nov 07:00:00.000000 | 5 | 5 | 5 | 16 Nov 22:30:30.110492 |
| 46130058 | 17 Nov 07:00:00.000000 | 5 | 5 | 5 | 16 Nov 22:30:31.586766 |
| 47790949 | 17 Nov 07:00:00.000000 | 5 | 5 | 5 | 16 Nov 22:30:30.551841 |
| 49236051 | 17 Nov 07:00:00.000000 | 5 | 5 | 5 | 16 Nov 22:30:32.982041 |
| 43002148 | 17 Nov 08:00:00.000000 | 2 | 2 | 2 | 17 Nov 04:00:21.426777 |
| 46130058 | 17 Nov 08:00:00.000000 | 2 | 2 | 2 | 17 Nov 04:00:22.002028 |
| 47790949 | 17 Nov 08:00:00.000000 | 2 | 2 | 2 | 17 Nov 04:00:21.900354 |
| 49236051 | 17 Nov 08:00:00.000000 | 2 | 2 | 2 | 17 Nov 04:00:24.825265 |
| 43002148 | 17 Nov 09:00:00.000000 | 1 | 1 | 1 | 16 Nov 22:30:28.060983 |
| 46130058 | 17 Nov 09:00:00.000000 | 1 | 1 | 1 | 16 Nov 22:30:29.410980 |
| 47790949 | 17 Nov 09:00:00.000000 | 1 | 1 | 1 | 16 Nov 22:30:28.493263 |
| 49236051 | 17 Nov 09:00:00.000000 | 1 | 1 | 1 | 16 Nov 22:30:30.940839 |
| 43002148 | 17 Nov 12:00:00.000000 | 2 | 2 | 2 | 17 Nov 08:00:21.712054 |
| 46130058 | 17 Nov 12:00:00.000000 | 2 | 2 | 2 | 17 Nov 08:00:22.479092 |
| 47790949 | 17 Nov 12:00:00.000000 | 2 | 2 | 2 | 17 Nov 08:00:22.244495 |
| 49236051 | 17 Nov 12:00:00.000000 | 2 | 2 | 2 | 17 Nov 08:00:24.619003 |
| 43002148 | 17 Nov 15:00:00.000000 | 6 | 6 | 0 | 17 Nov 09:00:20.368178 |
| 46130058 | 17 Nov 15:00:00.000000 | 6 | 6 | 0 | 17 Nov 09:00:19.979704 |
| 47790949 | 17 Nov 15:00:00.000000 | 6 | 6 | 0 | 17 Nov 09:00:20.111351 |
| 49236051 | 17 Nov 15:00:00.000000 | 6 | 6 | 0 | 17 Nov 09:00:22.827362 |
| 43002148 | 17 Nov 16:00:00.000000 | 2 | 2 | 2 | 17 Nov 12:00:21.566940 |
| 46130058 | 17 Nov 16:00:00.000000 | 2 | 2 | 2 | 17 Nov 12:00:22.402198 |
| 47790949 | 17 Nov 16:00:00.000000 | 2 | 2 | 2 | 17 Nov 12:00:22.353485 |
| 49236051 | 17 Nov 16:00:00.000000 | 2 | 2 | 2 | 17 Nov 12:00:24.874151 |
| 43002148 | 17 Nov 20:00:00.000000 | 2 | 2 | 2 | 17 Nov 16:00:21.541174 |
| 46130058 | 17 Nov 20:00:00.000000 | 2 | 2 | 2 | 17 Nov 16:00:23.963954 |
| 47790949 | 17 Nov 20:00:00.000000 | 2 | 2 | 2 | 17 Nov 16:00:24.172199 |
| 49236051 | 17 Nov 20:00:00.000000 | 2 | 2 | 2 | 17 Nov 16:00:24.936134 |
| 43002148 | 17 Nov 22:30:00.000000 | 6 | 6 | 6 | 17 Nov 15:00:34.980338 |
| 46130058 | 17 Nov 22:30:00.000000 | 6 | 6 | 6 | 17 Nov 15:00:30.714449 |
| 47790949 | 17 Nov 22:30:00.000000 | 6 | 6 | 6 | 17 Nov 15:00:30.712990 |
| 49236051 | 17 Nov 22:30:00.000000 | 6 | 6 | 6 | 17 Nov 15:00:33.921015 |


## §10. Assessment and next audit

The buffer has now demonstrated its intended practical effect: the carried Gold exits completed before maintenance, with no observed closed-market retry loop. Trading performance remains a separate failure to qualify: all six current closes lost money, and neither large demo is near 15/20. This audit supplies no justification for changing alpha, sizing or exits from these correlated observations.

The next activity interval should start exactly **17 Nov 18:00 SGT**. Finish the two ETH post-close rows at 19:02:31, verify 20:00 crypto and 22:30 NY opening capture against their preparation receipts, and explicitly observe the late London 420/450-mark vetoes if reached. Live's under-floor margin and accrued per-position financing remain visible; human ownership and the S1/S3 split must be retained. A read-only report does not authorise a position close or a trading-policy adjustment.

## §11. Evidence, checks and Gemini hand-off

Sources: **[1]** stored broker deals, S3 orders/fills, original guards/protection events and contract metadata; **[2]** exact deadline asks, close decision/send/broker/local timestamps and bounded 05:59–07:05 rollover records; **[3]** frozen activity/full-Tuesday/cumulative queries, successful history receipts and PR #238 baseline; **[4]** stored account marks, owner book/margin/health, decoded balance operations and bounded broker per-position P&L; **[5]** consumed-ID→intent→outcome joins and pre-cutoff goals; **[6]**10 existing-Postgres tick exports /327,318 rows, with hashes and coverage checks; **[7]**72 source openings, operational/midpoint evidence and causal preparation tables.

Seven focused checks passed: cash/period continuity, all frozen input and quote hashes/ranges, pip/R and stale/unelapsed endpoints with disjoint stop categories, causal trigger outcomes, live cash/P&L/ownership, honest opening/preparation qualification, and realised buffered Gold exits/no rollover retry storm. The read-only broker snapshot used **five account-data requests** plus **three authentication frames** (including one initial empty-response parsing attempt). There were **zero order, subscription, token-refresh or historical market-data requests**, and no production/database mutations. Raw auth values and private trader details are not published. Source text was treated as evidence, not instructions.

Only the three requested artifacts under `doc/` are committed. No protected file, production code, runtime, strategy, risk, configuration or Railway deployment is changed. Required CI: **check, docker, engineering**. Railway watch patterns exclude these docs. Trading baseline remains deployment `3d8b18d5-4d9f-4007-8fff-22eb25d5ea73`, commit `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`. Post-merge deployment/account-session continuity is verified separately and returned with the actual PR/merge SHA.

**Gemini instruction:** read this Markdown and matching JSON. Verify the two 05:57 Gold exits and their differently aged deadline asks; −$703.87 window, −$1,019.91 Tuesday, +$1,714.66 cumulative S3 cash; live $227.43 cash bridge and six per-position marks;0 recorded human interference with the owned S1 trail separated;4 complete and 2 partial post-close paths with Gold 1 h unavailable;9 individual trigger outcomes; and London's 24 base-ready versus 0 fully populated sigma tables. Keep 15/20 (75%) qualification, non-BE and BE counts, actual cash versus post-close observations, and post-cutoff snapshot timestamps distinct. Do not infer a profitable counterfactual or deploy a strategy change from this audit.


| Quote export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-ETHUSD-2026-11-17-002454.csv.gz | 50227 | 680d557dd79c4f1828a4114f225cecb1fd2aa7406a579163ef0445142f32f23c |
| 46130058-ETHUSD-2026-11-17-042454.csv.gz | 15262 | df4c7581550974b344d6d9fb2f29be85ee896ffcfdf32e12c316e4fea6bd60ee |
| 46130058-ETHUSD-2026-11-17-064226.csv.gz | 36864 | 39916d8fcb7cdc779d0107112f793f08687cf19a24e0a92dbabbddb3998c8840 |
| 46130058-XAUUSD-2026-11-16-184922.csv.gz | 30829 | 00fa5be988797244676ddf5996ee769a4a2d70f848028ed9e1031a0e5df18627 |
| 46130058-XAUUSD-2026-11-16-224922.csv.gz | 30561 | ba6ebfa12ddf3a180da37c75ea2f157b7977b22b398c024b43a569794aa8e58f |
| 47790949-ETHUSD-2026-11-17-002455.csv.gz | 50259 | 7c477bd4d469c8e694739a222d6afe5a3dec82b64c987485b2f0ace9ee17fd15 |
| 47790949-ETHUSD-2026-11-17-042455.csv.gz | 15059 | 440e8d688882d768e54f8d8e67efca2cb10fe7b79c917b9e2e0ef792429916de |
| 47790949-ETHUSD-2026-11-17-064226.csv.gz | 36864 | 46b24f9f77c44da7c2bd3fd201a06f9178d77b4a2ae6413d8bfdbd3c3b3ada4e |
| 47790949-XAUUSD-2026-11-16-184921.csv.gz | 30832 | 746678eac21b9217567dc88008d66f8c6604a8e258ee04d978c42388aa40de1d |
| 47790949-XAUUSD-2026-11-16-224921.csv.gz | 30561 | 6b87f666fe3cbb6a941c7e1a65774dc4d232601bd8dc89044c4e3002c2c62df5 |

| Frozen input | SHA256 |
| --- | --- |
| activity.json | e21052d3010c85414aefee3c5955bd26d6e515736be20dd5f89f93fe7c260748 |
| capture.json | 82fd1067132b5010e3c2cd707beec29547d0694829113925c5cb810920c652ec |
| cumulative.json | deeb058bbe183ec7e69696b0e4582de2fa3736e478c3e26bb2305f02d7ca3988 |
| day.json | f29a7c7de3881dadd9e9eb8e6b332073c22263796c38d7f9145a13a27bb20ab0 |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| evidence.json | 032f9609f848363104685ec005847fb79319c30e85031d4ff70e3b39d63ced9e |
| health.json | 2786c85cde64348ea9c8b59c93d8ff18a1050dea5b2811e776507252c923d51b |
| history-proof.json | 1aacdec4a75877acb63cd49c26a3528cd13399d4175610f4bbe7c99590858ca7 |
| human.json | 0b7f066ca233948dbb9874c002d0bbbfcf18edc30debb5a3e1bbd67669f68f05 |
| live-pnl-attempt1.json | 1f2f2d03016ca65aa74af4569f12e3b5217064530bdc47109ef1440e52a165f5 |
| live-pnl.json | e2bbb7b83b1a573c1051e24781830811eca43c0ea6c6d70b8580d7f4bbc6c74a |
| monitor-reconciled.json | d4d2fa65f44f08fcaa3bb3e23554ec9b0121fac7fa5ea92659b9d0f9a0f01d1e |
| monitor.json | 33dbe8b8cc14534e7e98039e8416e86895241847f75bb09722bac8e886bc0794 |
| opening-recovery.json | c964b21ee57a8d2e75d507cbd244e5e09c5afe8123317f2b771dee44bbf7981c |
| opening-ticks.json | 3885f2d0de1249445df74e5679bc0191e85b1e3624fb6b9f0256c00674a61f67 |
| preflight.json | d80908a89a1ba72c9676af99df4145632f3fec4a5510251d2022dde8e769c75f |
| quote-copy-receipts.json | 796ea534b641a88f61e587f466483981c7a6d065d087b33b5514ca3e2495b301 |
| rollover.json | e39f9d3042179f44bf1542670cdf7ee111f9edf89249b641afbe6bfd7b1375e4 |
| runtime.json | efddf546ac4d8d545456ae702b927f8db22801eabf81e7d1337fe0402c522dec |
| supplement.json | 9c897a27d39de2f8a8c9f0498c780567ca42184f745329f22750c8185333876a |
