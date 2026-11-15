# Monday Gold Reopen & Weekend Crypto Review — №18,464

**Activity:** [14 Nov 2026 05:35:00, 16 Nov 2026 07:15:00) Asia/Singapore. **Read-only audit.** Production `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`; policy `s3_macro_pullback_owner_18409_v1`.

[Measured JSON and evidence](MONDAY_GOLD_REOPEN_AND_WEEKEND_CRYPTO_REVIEW-18464-2026-11-16.json) · [Standalone HTML](MONDAY_GOLD_REOPEN_AND_WEEKEND_CRYPTO_REVIEW-18464-2026-11-16.html)

## §1. Assessment

- **Both carried Gold shorts closed at Monday 07:01:01 SGT**, for **−$44.52 combined**. Recorded send-to-response latency was **494 ms / 477 ms**. The attempts were scheduled reopen probes; evidence does not establish that the first recorded fresh quote caused both dispatches. [1,4]
- **Weekend S3 closing cash: +$54.76 across 11 closes, five positive-net and six negative-net.** Crypto contributed **+$99.28**; Gold contributed **−$44.52**. Nine new entries plus two carried Gold positions account for the entire cohort; no S3 position remained open at the cutoff or subsequent book snapshot. [1,2]
- **Sunday admission repair has observed fills:** the repaired demos closed six crypto positions for **+$96.56**, including two ETH losses, two BTC TPs and two session-end closes. Earlier small-demo crypto contributes **+$2.72 from three closes**, including one loss. These account copies are correlated, and the repair does not establish trading edge. [1]
- **The continuous Gold retry storm stopped after PR #229, but rejection count is not zero:** 1,062 old-storm refusals precede the verified rollout; seven isolated reconnect-time refusals follow it. No post-rollout consecutive rejection gap on either position is within 20 seconds. No Monday 07:00–07:01 quote or close attempt was recorded, so the pre-open 10-second floor was not exercised on this reopening. [1,8]
- **17 post-close paths:** 15 complete four-hour clocks, two partial Gold paths at the frozen 07:18:10 capture horizon. Two initial-stop paths later touched original TP (one prior follow-up, one current-window close). Three prior oil 4h endpoints and two BTC 2h endpoints lack sufficiently fresh quotes and remain null. [4]
- **4/4 accounts connected and unpaused; zero recorded live human-position/order interference.** Asian first quotes exist for all 20 requested slots: **18 exact operational references plus two live human-locked metals slots**. Crypto 08:00 preparation is 8/8 complete. Monday Brent already opened at 07:00; 09:00 is a prepared mark, not a new session. Late Brent marks remain incomplete. [5–7]
- **Financial pass remains false.** Latest20 net wins are **3/20, 10/20, 12/20, 10/20**. None reaches the required 15/20. No trading code, risk/exit rule or deployment is changed.

## §2. Clock, cutoff continuity and account snapshot

The mandatory first clock check returned **15 Nov 23:17:12 UTC** and shell `date -u` returned **23:17:20 UTC**, both after the requested **23:15 UTC / Monday 07:15 SGT** cutoff. The audit therefore proceeded immediately without scheduling a later run.

The interval begins at the exclusive endpoint of [№18,452](SATURDAY_WEEKEND_WRAPUP_AND_EXCURSIONS-18452-2026-11-14.md), with no gap. All cash uses unique `(account_id, deal_id)` closing receipts inside the half-open interval. The six earlier partial rows are follow-up prices only; their prior closing cash is not counted again. Prior post-PR #214 cumulative cash **+$2,643.24** plus this window **+$54.76** reconciles to **+$2,698.00**. [1–3]


| Account | Window S3 closes | Window S3 net USD | Post-PR #214 closing cash USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | -0.01 | 0 |
| 46130058 | 4 | 13.47 | 1,346.29 | 0 |
| 47790949 | 4 | 38.57 | 1,332.90 | 0 |
| 49236051 | 3 | 2.72 | 18.82 | 0 |


The longer cumulative window is **[10 Nov 09:42, 16 Nov 07:15) SGT**. It includes **+$412.88 from three positions entered before 10 Nov 09:42**; it is a mixed-entry closing-cash cohort, not a causal profit estimate for PR #214. The latest20 metric is a third, distinct window.

Successful recorded deal-history refreshes through **Monday 07:21:14 SGT**, fetched at **07:21:15**, cover beyond the cutoff for every account. Re-reading the same frozen interval after those refreshes gives byte-identical parsed monitor results. Their `complete=false` denotes a rolling, unsealed history refresh; all statuses are `ok`, with no error or new/changed deals. The canonical monitor originally allowed a freshness tolerance; this separate later-history check avoids treating its 07:14 history watermark as literal 07:15 coverage. [1]

Money marks are **07:19:15.968–07:19:16.393 SGT**, books approximately **07:17:14**, and health **07:18:32.469**. These are separately timestamped post-cutoff observations, not reconstructed atomic 07:15 balances. Equity is balance plus recorded net unrealised P&L, excluding unverified credit and prospective closing fees. The monetary snapshot does not verify aggregate broker margin; the margin column is the owner endpoint's assessed current margin percentage. [5]


| Account | Balance USD | Float USD | Equity USD | Assessed margin % | S3 / S1 / S2 / external positions | Pending entries | Money mark SGT |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | -906.87 | 4,499.69 | 3,592.82 | 340.72 | 0 / 2 / 0 / 5 | 5 | 16 Nov 07:19:16.393 |
| 46130058 | 25,867.89 | 109.29 | 25,977.18 | 19,810.30 | 0 / 5 / 0 / 0 | 0 | 16 Nov 07:19:16.066 |
| 47790949 | 25,524.18 | 114.64 | 25,638.82 | 3,097.73 | 0 / 4 / 0 / 0 | 0 | 16 Nov 07:19:15.968 |
| 49236051 | 813.59 | -0.73 | 812.86 | 93,428.74 | 0 / 1 / 0 / 0 | 0 | 16 Nov 07:19:16.082 |


Live current assessed margin is **340.72%**, but the pending-entry projection is **166.25%**, below the unchanged 250% floor; stress projection is **−46.54%**. That stress projection is not current negative equity or proof of an actual margin call. Live balance is unchanged at **−$906.87**; its change in equity from the earlier report comes from combined floating P&L, while window S3 cash is zero.

Complete observed inventory:


| Account / ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Unverified / N/A |
| 43002148 / 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Unverified / N/A |
| 43002148 / 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.97900 | 3.87500 |
| 43002148 / 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,813.70000 | Unverified / N/A |
| 43002148 / 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,830.70000 | Unverified / N/A |
| 43002148 / 657180931 | NVDA.US | human_external / position | buy | 200 / 2.0000 | 165.22000 | 149.66000 | Unverified / N/A |
| 43002148 / 657187868 | XAUUSD | human_external / position | buy | 2500 / 0.2500 | 3,869.03800 | 3,859.77000 | Unverified / N/A |
| 43002148 / 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 / 401488805 | XAUUSD | human_external / pending_entry | buy | 1000 / 0.1000 | 3,813.63000 | 3,786.80000 | Unverified / N/A |
| 43002148 / 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 / 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 / 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Unverified / N/A |
| 46130058 / 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,830.80000 | Unverified / N/A |
| 46130058 / 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69277 | Unverified / N/A |
| 46130058 / 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12231 | Unverified / N/A |
| 46130058 / 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 46130058 / 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56702 | Unverified / N/A |
| 47790949 / 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,830.80000 | Unverified / N/A |
| 47790949 / 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69272 | Unverified / N/A |
| 47790949 / 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12226 | Unverified / N/A |
| 47790949 / 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 49236051 / 262796763 | ETHUSD | s1 / position | sell | 1 / 0.0100 | 1,746.87000 | 2,027.27000 | Unverified / N/A |


## §3. Human protection and separate Strategy 1 inventory

Live retains **five external positions and five external pending orders**. No live bot intent is recorded in this activity window; joining retained events/intents to prior and current human IDs finds **zero recorded interference**. External closing-deal cash is **$0.00**. Prior/current external volume, entry, SL and TP geometry is unchanged. This establishes absence of recorded bot intervention; it does not claim knowledge of unrecorded outside actors. [1,5]

Live XAUUSD remains external: **657187868**, long **0.25 lot at 3,869.038**, SL **3,859.77**, plus a separate pending Gold buy. XAUUSD yields to that ownership; XAGUSD is blocked by its sibling's pair reservation. NatGas and JPN225 remain reserved, while NVDA.US is also outside S3 authority. The current Gold inventory differs from older November screenshots and is reported from the observed book.

Live S1 **GER40 656513085** and **AUS200 656795267** remain open with SL **25,813.7 / 8,830.7**, unchanged from №18,452. There is no live S1 entry, close or trail command within this window. Reconnect authority-recovery journal entries are not new orders or discretionary closes.

Small demo S1 **ETHUSD sell 262796763** remains open, separately from S3. Broker opening execution is **15 Nov 06:37:46.205 SGT**, price **1,746.87**, native volume **1** (0.01 unit). Fifteen successful S1 trail amendments are recorded; the last is **15 Nov 09:40:46.833 SGT**, SL **2,027.27**. No closing deal is recorded through the cutoff. Its account's later net floating P&L is **−$0.73**, with this one observed position. S3 labels it external-to-S3 and yields on ETH; BTC is pair-blocked. It is bot-owned S1, not a human trade, despite sharing the generic yield diagnostic. Full retained S1 events are in JSON. [1,5]

## §4. Carried Gold: weekend deferral and Monday reopen

Both old positions retain their recorded **14 Nov 05:55 SGT** deadline; the later safety-buffer change did not rewrite their existing geometry. They closed on Monday's first executable reopening period. [1,8]


| Account / position | First recorded two-sided quote SGT | Bid / ask / midpoint | Broker quote timestamp UTC | Close sent SGT | Broker fill SGT / price | Send→broker execution ms | Send→response ms | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 262583136 | 16 Nov 07:01:01.230 | 3,880.73000/3,880.87000/3,880.80000 | 2026-11-15T23:01:01.041000+00:00 | 16 Nov 07:01:01.174 | 16 Nov 07:01:01.542 / 3,880.23 | 367.644 | 494 | -25.69 / -0.184634 |
| 46130058 / 262583980 | 16 Nov 07:01:01.185 | 3,880.69000/3,880.90000/3,880.79500 | 2026-11-15T23:01:01.041000+00:00 | 16 Nov 07:01:01.208 | 16 Nov 07:01:01.560 / 3,880.26 | 351.432 | 477 | -18.83 / -0.121484 |


The exact microsecond times and full fill/quote receipts are in JSON. For 47790949, the send was **56.035 ms before** its first recorded fresh quote; for 46130058, **23.201 ms after**. Both decision receipts were created before those recorded fresh quotes and carried the previous decision-side prices. These were successful **scheduled probes**, not proof of a fresh-quote-triggered close. Broker execution and local response latency are distinct. The first broker-timestamped Gold quote is **23:01:01.041 UTC**; recorder receipt is approximately **07:01:01.185–01.230 SGT** for these accounts.

Net reconciliation: 47790949 gross **−$27.36**, swap **+$2.31**, commission **−$0.64** gives **−$25.69**; 46130058 gross **−$21.40**, swap **+$2.57**, commission **$0.00** gives **−$18.83**. Combined **−$44.52** is included once in window cash.


| Rejection scope | Count | Interpretation |
| --- | --- | --- |
| Before verified PR #229 rollout (14 Nov 07:29:41.998 SGT) | 1062 | Legacy continuous retry storm; included in whole activity-window incident evidence |
| After verified PR #229 rollout | 7 | Isolated refusals around Sunday reconnects/releases; not zero receipts |
| Post-rollout <=20s consecutive rejection gaps | 0 | No continuing 10-second retry loop on either carried position |
| Monday 07:00–07:01 close attempts | 0 | No empirical exercise of the pre-open retry floor |

| Account / position | Post229 refusal count | Minimum consecutive refusal gap (seconds) | Last refusal SGT |
| --- | --- | --- | --- |
| 47790949 / 262583136 | 3 | 1,194.059 | 15 Nov 09:57:20.826 |
| 46130058 / 262583980 | 4 | 1,194.191 | 15 Nov 09:57:21.105 |


There are **1,069 total Gold MARKET_CLOSED refusals** across the whole activity interval: **1,062 before the verified rollout + seven isolated later refusals**. The last is **15 Nov 09:57:21.106 SGT**, followed by no rejection until the successful Monday closes. Both symbols have zero recorded indicative quotes in Monday 07:00–07:01. The observed result supports suppression of the continuous loop; it does not support claiming zero weekend rejections or a tested pre-open-quote floor.

## §5. Complete S3 ledger and pre/post-repair cohorts

All 11 positions are demo shorts. Every S3 position opened, closed or carried through the interval appears below: two carried Gold positions plus nine new crypto entries. Initial R is the actual fill-to-original-stop distance, scaled by actual volume for cash R. Later stop amendments do not reset the denominator. Broker TP classifications use protective closing receipts plus target price; session-end exits use matching close intents. [1]


| Account / position | Symbol / side | Native volume / lots | Entry SGT / fill | Exit SGT / fill | Initial SL / TP | Exit class | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 262583136 | XAUUSD / sell / carried | 900 / 0.0900 | 14 Nov 00:10:35.369 / 3,877.19000 | 16 Nov 07:01:01.542 / 3,880.23000 | 3,892.65000 / 3,855.85000 | session end | -25.69 / -0.184634 |
| 46130058 / 262583980 | XAUUSD / sell / carried | 1000 / 0.1000 | 14 Nov 00:13:41.377 / 3,878.12000 | 16 Nov 07:01:01.560 / 3,880.26000 | 3,893.62000 / 3,856.73000 | session end | -18.83 / -0.121484 |
| 49236051 / 262686344 | ETHUSD / sell | 14 / 0.1400 | 14 Nov 10:41:09.027 / 1,899.14000 | 14 Nov 12:00:01.358 / 1,885.21000 | 1,932.95000 / 1,852.48000 | session end | 1.95 / 0.411966 |
| 49236051 / 262794107 | BTCUSD / sell | 1 / 0.0100 | 15 Nov 03:25:54.509 / 65,451.73000 | 15 Nov 03:44:26.689 / 65,776.22000 | 65,776.25000 / 65,003.89000 | initial stop | -3.24 / -0.998398 |
| 49236051 / 262794771 | BTCUSD / sell | 1 / 0.0100 | 15 Nov 04:21:31.776 / 65,487.24000 | 15 Nov 04:53:58.138 / 65,086.68000 | 65,768.68000 / 65,098.85000 | protective TP | 4.01 / 1.424815 |
| 46130058 / 262803244 | ETHUSD / sell | 378 / 3.7800 | 15 Nov 10:05:43.550 / 1,739.62000 | 15 Nov 10:12:42.698 / 1,781.02000 | 1,780.75000 / 1,682.86000 | initial stop | -156.49 / -1.006552 |
| 47790949 / 262803245 | ETHUSD / sell | 372 / 3.7200 | 15 Nov 10:05:43.957 / 1,739.71000 | 15 Nov 10:12:42.677 / 1,781.02000 | 1,780.84000 / 1,682.95000 | initial stop | -153.67 / -1.004355 |
| 47790949 / 262831213 | BTCUSD / sell | 33 / 0.3300 | 15 Nov 14:17:49.681 / 64,430.86000 | 15 Nov 15:46:07.835 / 63,800.34000 | 64,885.65000 / 63,803.24000 | protective TP | 208.07 / 1.386387 |
| 46130058 / 262831214 | BTCUSD / sell | 34 / 0.3400 | 15 Nov 14:17:49.726 / 64,430.86000 | 15 Nov 15:46:07.820 / 63,800.34000 | 64,885.65000 / 63,803.24000 | protective TP | 214.38 / 1.386419 |
| 47790949 / 262942317 | BTCUSD / sell | 23 / 0.2300 | 15 Nov 23:02:15.265 / 64,745.71000 | 16 Nov 00:00:01.523 / 64,702.83000 | 65,412.04000 / 63,826.17000 | session end | 9.86 / 0.064337 |
| 46130058 / 262944867 | ETHUSD / sell | 856 / 8.5600 | 15 Nov 23:10:06.330 / 1,806.92000 | 16 Nov 00:00:00.874 / 1,809.91000 | 1,825.12000 / 1,781.80000 | session end | -25.59 / -0.164257 |

| Position | Observed MFE / MAE R | Confirmed BE amendment SGT | Original registered exit deadline SGT |
| --- | --- | --- | --- |
| 262583136 | 0.500000 / 0.897154 | None observed | 14 Nov 05:55:00.000 |
| 262583980 | 0.556774 / 0.836774 | None observed | 14 Nov 05:55:00.000 |
| 262686344 | 0.631174 / 0.516119 | None observed | 14 Nov 12:00:00.000 |
| 262794107 | 0.015315 / 1.001818 | None observed | 15 Nov 04:00:00.000 |
| 262794771 | 1.484970 / 0.312287 | 15 Nov 04:51:26.039 | 15 Nov 08:00:00.000 |
| 262803244 | 0.000000 / 1.002431 | None observed | 15 Nov 12:00:00.000 |
| 262803245 | 0.000000 / 1.000243 | None observed | 15 Nov 12:00:00.000 |
| 262831213 | 1.385365 / 0.502935 | 15 Nov 15:41:48.279 | 15 Nov 16:00:00.000 |
| 262831214 | 1.385365 / 0.502935 | 15 Nov 15:41:48.046 | 15 Nov 16:00:00.000 |
| 262942317 | 0.479867 / 0.184728 | None observed | 16 Nov 00:00:00.000 |
| 262944867 | 0.202198 / 0.495604 | None observed | 16 Nov 00:00:00.000 |


Position **262794107** is an initial-stop close: broker protective order type 4, unchanged original SL and an observed pre-fill stop crossing. Its fill improved on the SL by **0.03 price units**; favourable slippage does not make the exit unclassifiable. Gold in-position excursions exclude reconnect snapshots during the scheduled closed market; the large weekend interval is explicitly not continuous executable tick evidence.


| Cohort | Closes | Positive net | Net USD | Mean net R |
| --- | --- | --- | --- | --- |
| pre_pr233_control_crypto | 3 | 2 | 2.72 | 0.279461 |
| post_pr233_repaired_demo_crypto | 6 | 3 | 96.56 | 0.110330 |
| carried_gold | 2 | 0 | -44.52 | -0.153059 |


The PR #233 repair was active before the **Sunday 10:05 ETH fills** on both formerly blocked demos. Those paired losses total **−$310.16**. Their **14:17 BTC entries** later hit TP for **+$422.45**, and the later session-end exits total **−$15.73**; net **+$96.56**. The early small-demo cohort includes a stop loss as well as its two profitable closes. There is no live S3 entry or exit in the interval.

## §6. Latest20 and financial qualification


| Account | Net wins / count | Win rate | Realised payoff (R) | Profit factor (R) | Net R | Assessment recorded SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | 3.4277 | 0.6049 | -5.9677 | 16 Nov 07:09:12.815 |
| 46130058 | 10/20 | 50.00% | 1.9493 | 1.9493 | 5.0091 | 16 Nov 07:13:17.659 |
| 47790949 | 12/20 | 60.00% | 1.5337 | 2.3005 | 6.5906 | 16 Nov 07:13:22.350 |
| 49236051 | 10/20 | 50.00% | 1.2329 | 1.3699 | 2.5141 | 16 Nov 07:11:25.593 |


The 73% goal requires **15/20 positive-net closes (75%)**, together with the payoff condition. No account reaches 15. Live's latest20 includes older October registrations and is not a weekend sample. The two large demos often copy the same market setup; their trades are not independent trials. Positive closing cash, the 1.38R configured target, and a realised payoff statistic are different quantities. **Financial pass=false.**

## §7. Four-hour post-close excursions

This section has **17 paths: 11 current-window closes plus the six explicitly requested prior follow-ups**. Fifteen complete four-hour clocks have elapsed; both Gold paths are partial at **16 Nov 07:18:10 SGT**, approximately **0.28568h** after exit. Later publication time does not silently extend this frozen price capture. [4]

All paths use the account-specific executable side: **ask for these shorts**. Movement is signed in original trade direction from **actual exit fill**, with original trade R retained. Favourable/adverse magnitudes are **price / broker pips / R**. Pip sizes come from broker `pip_position`: BTC, ETH and Gold **0.1**; Crude **0.01**. TP/SL crossing tests use the original absolute levels, not an exit-anchored replacement bracket.



### Current-window closes — cash included in §2


| Position / symbol / exit | Elapsed h | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 262583136 / XAUUSD / session end | 0.28568 PARTIAL | 2.81000 / 28.10 / 0.181759 | 8.98000 / 89.80 / 0.580854 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | False / False |
| 262583980 / XAUUSD / session end | 0.28568 PARTIAL | 2.81000 / 28.10 / 0.181290 | 8.98000 / 89.80 / 0.579355 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | False / False |
| 262686344 / ETHUSD / session end | 4.00000 complete clock | 8.62000 / 86.20 / 0.254954 | 27.14000 / 271.40 / 0.802721 | -143.10 / -0.423248 | -194.60 / -0.575569 | -132.30 / -0.391304 | False / False |
| 262794107 / BTCUSD / initial stop | 4.00000 complete clock | 2,499.17000 / 24,991.70 / 7.701128 | 80.31000 / 803.10 / 0.247473 | 3,569.90 / 1.100055 | 10,035.50 / 3.092413 | 21,757.30 / 6.704456 | True / True |
| 262794771 / BTCUSD / protective TP | 4.00000 complete clock | 1,809.63000 / 18,096.30 / 6.429896 | 56.43000 / 564.30 / 0.200505 | 6,620.50 / 2.352366 | 15,764.20 / 5.601265 | 16,111.40 / 5.724630 | True / False |
| 262803244 / ETHUSD / initial stop | 4.00000 complete clock | 10.16000 / 101.60 / 0.247022 | 55.26000 / 552.60 / 1.343545 | -471.20 / -1.145636 | -338.50 / -0.823000 | -100.80 / -0.245077 | False / True |
| 262803245 / ETHUSD / initial stop | 4.00000 complete clock | 10.16000 / 101.60 / 0.247022 | 55.26000 / 552.60 / 1.343545 | -471.20 / -1.145636 | -338.50 / -0.823000 | -100.80 / -0.245077 | False / True |
| 262831213 / BTCUSD / protective TP | 4.00000 complete clock | 61.33000 / 613.30 / 0.134853 | 2,403.77000 / 24,037.70 / 5.285450 | -9,755.70 / -2.145100 | stale_asof_quote (62.017s old) | -16,917.20 / -3.719783 | True / True |
| 262831214 / BTCUSD / protective TP | 4.00000 complete clock | 61.33000 / 613.30 / 0.134853 | 2,403.77000 / 24,037.70 / 5.285450 | -9,755.70 / -2.145100 | stale_asof_quote (61.967s old) | -16,917.20 / -3.719783 | True / True |
| 262942317 / BTCUSD / session end | 4.00000 complete clock | 466.37000 / 4,663.70 / 0.699908 | 510.47000 / 5,104.70 / 0.766092 | 239.00 / 0.035868 | 1,701.30 / 0.255324 | -2,030.60 / -0.304744 | False / False |
| 262944867 / ETHUSD / session end | 4.00000 complete clock | 7.99000 / 79.90 / 0.439011 | 21.22000 / 212.20 / 1.165934 | -59.20 / -0.325275 | -32.60 / -0.179121 | -140.30 / -0.770879 | False / True |


### Six earlier partial rows — follow-up only, cash excluded


| Position / symbol / exit | Elapsed h | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 262579978 / SpotCrude / BE stop | 4.00000 complete clock | 0.46500 / 46.50 / 0.804498 | 0.40100 / 40.10 / 0.693772 | 19.10 / 0.330450 | 8.80 / 0.152249 | stale_asof_quote (2,563.097s old) | False / False |
| 262579991 / SpotCrude / BE stop | 4.00000 complete clock | 0.46500 / 46.50 / 0.804498 | 0.40100 / 40.10 / 0.693772 | 19.10 / 0.330450 | 8.80 / 0.152249 | stale_asof_quote (2,563.140s old) | False / False |
| 262579998 / SpotCrude / BE stop | 4.00000 complete clock | 0.48100 / 48.10 / 0.832180 | 0.38500 / 38.50 / 0.666090 | 32.40 / 0.560554 | 8.70 / 0.150519 | stale_asof_quote (2,297.627s old) | False / False |
| 262581332 / ETHUSD / initial stop | 4.00000 complete clock | 15.72000 / 157.20 / 1.495718 | 20.00000 / 200.00 / 1.902950 | -52.60 / -0.500476 | -101.90 / -0.969553 | 117.60 / 1.118934 | False / True |
| 262582229 / BTCUSD / initial stop | 4.00000 complete clock | 894.97000 / 8,949.70 / 2.483820 | 171.15000 / 1,711.50 / 0.474994 | 1,644.00 / 0.456261 | -517.90 / -0.143733 | 7,589.40 / 2.106294 | True / True |
| 262586283 / BTCUSD / initial stop | 4.00000 complete clock | 918.08000 / 9,180.80 / 2.202951 | 148.04000 / 1,480.40 / 0.355225 | 1,854.40 / 0.444967 | -270.40 / -0.064883 | 7,518.20 / 1.804007 | False / True |


Two stopped shorts later touched original TP: **262582229** (prior BTC follow-up) and **262794107** (current-window BTC). Both also traded beyond original SL during their follow-up. No BE-stop follow-up touched original TP in this report. The two Sunday 10:12 ETH initial stops did **not** touch original TP during their four-hour follow-ups and continued adversely. Successful BTC TP exits on the large demos also reversed sharply afterward. These are observed post-exit price paths, not executable alternative trades or evidence for changing the exit rule.


| Stopped position reaching TP | Exit SGT | First post-close original SL crossing SGT | First original TP touch SGT |
| --- | --- | --- | --- |
| 262582229 | 14 Nov 01:41:03.902 | 14 Nov 01:41:04.031 | 14 Nov 05:35:49.448 |
| 262794107 | 15 Nov 03:44:26.689 | 15 Nov 03:44:26.969 | 15 Nov 04:54:05.268 |


**Coverage limits:** complete clock does not mean uninterrupted quotes. The three prior Crude 4h endpoints fall after Friday's market close; last quotes are **2,297.627–2,563.140 seconds** old, so those signed endpoints are null. The two large-demo BTC TP 2h endpoints have last quotes **61.967 / 62.017 seconds** old; those are also null under the unchanged six-second freshness test. There is no interpolation. Other elapsed endpoint values meet that test. Gold's 1h/2h/4h checkpoints have not elapsed at capture and remain null. Raw observed extrema are lower bounds on unobserved excursions when gaps exist; a missing TP observation cannot rule out a touch within a recording gap.

The JSON retains first/last quote times, maximum gaps, tail gaps, exact extrema timestamps, all checkpoint ages and the original-level crossing order for every row. Four reconnect snapshots per Gold account inside the weekend closure are excluded from executable excursion evidence.

## §8. Trigger funnel and telemetry integrity


| Account | New arms | Consumed / persisted intents / outcomes | Filled entries | Terminal refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 37 | 0 / 0 / 0 | 0 | 0 | 30 |
| 46130058 | 35 | 8 / 8 / 8 | 3 | 5 | 17 |
| 47790949 | 35 | 8 / 8 / 8 | 3 | 5 | 17 |
| 49236051 | 19 | 8 / 7 / 8 | 3 | 5 | 6 |


**126 arms → 24 consumed triggers → 24 individual terminal outcomes**, comprising **nine fills and 15 refusals**. There are **23 persisted opening intents**, because one refusal occurred before intent persistence. Missing individual outcomes and unknown entry-guard receipts are both **zero**. There are **70 >61.8% retracement invalidations** and **50 pair-reservation diagnostic records**. Idle diagnostic counts are deduplicated and are not an exhaustive count of every failed evaluation or hypothetical missed trade. [6]


| Terminal refusal reason | Count |
| --- | --- |
| ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse | 7 |
| ENTRY_EXPIRED: entry dispatch deadline elapsed | 3 |
| prepared admission expired; background refresh required | 1 |
| ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction | 4 |


The pre-intent refusal occurred on **14 Nov 18:59:16.432 SGT**, consumed decision **217720**, outcome **217722**, because prepared admission had expired. The same planned entry key was persisted later at **19:08:14.893**, for a different trigger. Joining only on that key would incorrectly attach a future intent to the earlier refusal. This audit joins each consumed decision to its own outcome and requires intent creation no later than outcome time.

Maximum observed **confirmation-to-final-guard latency is 410.539 ms**. Touch-to-confirmation waiting time and broker send-to-response time are different measures. The monitor's `refusals_all_stages_complete=false` is retained: complete consumed-trigger attribution does not claim an exhaustive census of every idle/pre-arm veto.

## §9. Monday opening references and readiness

Asian requested slots are **SpotCrude, US500, NAS100, XAUUSD, XAGUSD × four accounts =20**. All have first recorded two-sided quotes within the versioned **15-second** observation allowance. **18** match operational bid/ask prices and side/observation timestamps exactly. Live Gold and Silver are intentionally human/pair locked, so they do not supply operational references; their source quotes still exist. Metal observation starts **07:01**, after the nominal07:00 session anchor. [7]


| Account | Symbol | Observation start SGT | First quote SGT | Delay s | Midpoint | Operational status |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | NAS100 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.662 | 6.662242 | 28,030.950000 | exact_quote_receipt |
| 43002148 | XAUUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.234 | 1.234896 | 3,880.800000 | human_locked |
| 43002148 | XAGUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.234 | 1.234894 | 48.717000 | human_locked |
| 43002148 | US500 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.746 | 6.746057 | 7,572.600000 | exact_quote_receipt |
| 49236051 | NAS100 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693218 | 28,030.950000 | exact_quote_receipt |
| 49236051 | US500 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693221 | 7,572.600000 | exact_quote_receipt |
| 49236051 | XAGUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.177 | 1.177835 | 48.717000 | exact_quote_receipt |
| 49236051 | SpotCrude | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693219 | 88.839000 | exact_quote_receipt |
| 43002148 | SpotCrude | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.662 | 6.662243 | 88.839000 | exact_quote_receipt |
| 46130058 | NAS100 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693908 | 28,030.950000 | exact_quote_receipt |
| 49236051 | XAUUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.177 | 1.177836 | 3,880.800000 | exact_quote_receipt |
| 46130058 | US500 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693910 | 7,572.600000 | exact_quote_receipt |
| 46130058 | XAUUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.185 | 1.185367 | 3,880.795000 | exact_quote_receipt |
| 47790949 | XAUUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.230 | 1.230391 | 3,880.800000 | exact_quote_receipt |
| 47790949 | XAGUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.230 | 1.230390 | 48.717000 | exact_quote_receipt |
| 47790949 | US500 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.703 | 6.703152 | 7,572.600000 | exact_quote_receipt |
| 47790949 | NAS100 | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.703 | 6.703150 | 28,030.950000 | exact_quote_receipt |
| 46130058 | SpotCrude | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693908 | 88.839000 | exact_quote_receipt |
| 47790949 | SpotCrude | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.703 | 6.703150 | 88.839000 | exact_quote_receipt |
| 46130058 | XAGUSD | 16 Nov 07:01:00.000 | 16 Nov 07:01:01.185 | 1.185366 | 48.717000 | exact_quote_receipt |


SpotCrude's four first quotes arrived **6.662243–6.703150 seconds** after 07:00, with broker timestamp approximately **07:00:06.529–06.530**. All four exact references are retained; the prior six-second-only opening failure did not recur on this evidence.

**Crypto 08:00:** all **8/8** base preparations and complete mark tables are ready. Those are pre-open inputs, not future opening prices or a promise that all accounts can enter. Human/S1 ownership, live quote/FX freshness, risk, margin and pair checks still apply.


| Account | Symbol | Target open SGT | Preparation recorded SGT | Base ready | All marks ready |
| --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:19.272 | True | True |
| 43002148 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:21.235 | True | True |
| 46130058 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:19.914 | True | True |
| 46130058 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:21.854 | True | True |
| 47790949 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:20.111 | True | True |
| 47790949 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:22.214 | True | True |
| 49236051 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:22.689 | True | True |
| 49236051 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 04:00:24.955 | True | True |


**Monday Brent calendar correction:** native Monday starts at **Sunday 18:00 America/New_York =Monday 07:00 SGT**. The Tuesday–Friday 20:00NY convention would yield09:00, but does not apply to this Monday. Four Brent opening references match the recorded source quotes. **09:00 is the 120-minute mark**, and its coefficient is ready on **4/4** accounts. Base preparation is4/4; full-session mark tables are **0/4 complete** because the450/480-minute endpoints have **6 observations versus 14 required**. Those later14:30/15:00 marks must not be advertised as fully ready. No calendar or preparation data is changed in this audit. [7,8]


| Account | Brent session open SGT | Base ready | 09:00 mark | Missing marks |
| --- | --- | --- | --- | --- |
| 43002148 | 16 Nov 07:00:00.000 | True | {'Ok': 0.006222915783050509} | 450: 6 same-slot endpoints;14 needed; 480: 6 same-slot endpoints;14 needed |
| 46130058 | 16 Nov 07:00:00.000 | True | {'Ok': 0.006222915783050509} | 450: 6 same-slot endpoints;14 needed; 480: 6 same-slot endpoints;14 needed |
| 47790949 | 16 Nov 07:00:00.000 | True | {'Ok': 0.006222915783050509} | 450: 6 same-slot endpoints;14 needed; 480: 6 same-slot endpoints;14 needed |
| 49236051 | 16 Nov 07:00:00.000 | True | {'Ok': 0.006222915783050509} | 450: 6 same-slot endpoints;14 needed; 480: 6 same-slot endpoints;14 needed |


Current post-cutoff admission states are listed below. A stored refusal message is a snapshot of that actor's latest guard; it is not retrospectively assigned to all weekend non-entries. In particular, a stale FX conversion refusal during an open FX market is still subject to the original freshness bounds; future base preparation is not permission to bypass it.


| Account | Symbol | Current gate / message |
| --- | --- | --- |
| 43002148 | BTCUSD | Waiting: opening impulse below 48 bps |
| 43002148 | ETHUSD | Waiting: opening impulse below 48 bps |
| 43002148 | NAS100 | Waiting: opening impulse below 48 bps |
| 43002148 | SpotBrent | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 166.10% below floor 250.00% |
| 43002148 | SpotCrude | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 167.56% below floor 250.00% |
| 43002148 | US500 | Waiting: opening impulse below 48 bps |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Waiting: opening impulse below 48 bps |
| 46130058 | ETHUSD | Waiting: opening impulse below 48 bps |
| 46130058 | NAS100 | Waiting: opening impulse below 48 bps |
| 46130058 | SpotBrent | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 5 |
| 46130058 | SpotCrude | Blocked: prepared admission unavailable; last refusal: prepared admission expired; background refresh required |
| 46130058 | US500 | Waiting: opening impulse below 48 bps |
| 46130058 | XAGUSD | Waiting: opening impulse below 48 bps |
| 46130058 | XAUUSD | Waiting: opening impulse below 48 bps |
| 47790949 | BTCUSD | Waiting: opening impulse below 48 bps |
| 47790949 | ETHUSD | Waiting: opening impulse below 48 bps |
| 47790949 | NAS100 | Waiting: opening impulse below 48 bps |
| 47790949 | SpotBrent | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 5 |
| 47790949 | SpotCrude | Blocked: prepared admission unavailable; last refusal: prepared admission expired; background refresh required |
| 47790949 | US500 | Waiting: opening impulse below 48 bps |
| 47790949 | XAGUSD | Waiting: opening impulse below 48 bps |
| 47790949 | XAUUSD | Waiting: opening impulse below 48 bps |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Waiting: opening impulse below 48 bps |
| 49236051 | SpotBrent | Preparing: background entry admission is not ready yet |
| 49236051 | SpotCrude | Armed: waiting 50%–61.8% pullback + 8% turn (ref: 88.5815, extreme: 88.324); touch not yet recorded |
| 49236051 | US500 | Waiting: opening impulse below 48 bps |
| 49236051 | XAGUSD | Waiting: opening impulse below 48 bps |
| 49236051 | XAUUSD | Waiting: opening impulse below 48 bps |


## §10. Assessment and next bounded audit work

1. **Retain the measured repair outcome:** both carried Gold positions closed promptly at reopening, and the previously blocked demos produced Sunday crypto fills. Preserve the distinction between a stopped retry loop and seven isolated reconnect refusals. A later infrastructure review can inspect whether reconnect refusal state should persist; no such change is made here.
2. **Finish two Gold follow-ups in the next report after 11:01 SGT.** Their present 0.286h paths cannot establish a four-hour TP outcome. Keep this cash cutoff and capture horizon frozen.
3. **Track Brent's missing late marks and transient FX/admission freshness separately.** The calendar has a Monday exception; a new09:00 opening should not be invented. Missing historical endpoints need a targeted preparation investigation, not an alpha adjustment.
4. **Keep financial qualification separate from availability.** Six repaired-demo closes have only three positive-net outcomes. The correlated BTC winners and two stopped trades' later TP touches do not justify changing 1R BE,1.38R TP, risk budgets or holding periods. All four latest20 windows remain below 15wins.
5. **Continue ownership separation.** Live human inventory and all S1 positions stay outside S3 management. Live margin projections still constrain new exposure even though current account margin is above 250%.

## §11. Sources, verification, publication and Gemini instructions

**[1]** Read-only PostgreSQL deals, S3 orders/fills, intent/event receipts, contract metadata and history records. **[2]** Separately bounded canonical window/cumulative monitor queries, decimal closing-cash reconciliation and unchanged refetch. **[3]** Prior merged №18,452 and its exact05:35 cutoff. **[4]** Existing PostgreSQL ticks only: **56 bounded exports /1,022,543 raw rows**, zero broker collection requests; exact query intervals and SHA256 are in JSON. **[5]** Separately timed public health, authenticated owner assessment, stored account-money marks and live ownership journal. **[6]** Consumed decision→terminal outcome→causally existing intent, official latest20 records before cutoff. **[7]** Durable preparation/opening receipts and account-specific recorded first quotes. **[8]** Deployed `production_sessions.rs`, prior PR #229 release verification and PR #233/234 release evidence. Evidence text was treated as data, not executable instructions.

The matching JSON embeds the measured ledger, account inventory, closing cash receipts, full quote metadata/hashes, trade/trigger joins, read-only query sources and calculation/test source. Raw quote exports remain private; no workspace-only download is needed to read the report or its measured evidence. Six focused checks passed: cutoff/cash continuity, export reconciliation, signed price/pip/R arithmetic, causal outcome linkage, reopen/ownership and readiness denominators. Desktop/mobile rendering and secret/path checks precede publication. Required CI is `check`, `docker`, `engineering`.

Only these three `doc/` artifacts are committed. Trading deployment observed before publication is **3d8b18d5-4d9f-4007-8fff-22eb25d5ea73**, commit **65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2**. The repository's Railway watch patterns exclude `doc/`. Final merge and unchanged deployment/actor verification are returned with the PR URL and merge SHA; this report does not fabricate those future publication facts.

**Gemini review:** read this Markdown with its matching JSON. Verify **11 closes versus six earlier follow-ups**, **+$54.76** window versus **+$2,698.00** cumulative cash, **1,062 old-storm versus seven later isolated Gold refusals**, scheduled-probe versus fresh-quote causality, **24 outcomes versus 23 intents**, **18 operational Asian references plus two human locks**, and Monday Brent's07:00 calendar exception. Preserve null stale/future checkpoints and the distinction between15 elapsed four-hour clocks and uninterrupted quote coverage. Do not interpret a post-exit TP touch as a simulated fill or financial qualification.


| Quote export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-BTCUSD-2026-11-13-160718.csv.gz | 60376 | 32b91e7d74c22ea6eb5c490664819c37842d02d304dd6174cc4da3900ca4d07f |
| 46130058-BTCUSD-2026-11-13-200718.csv.gz | 23600 | dc3c2462d2924a247c74f806f30af940bd496d7eeb5d982779755fb00165aac1 |
| 46130058-BTCUSD-2026-11-15-061743.csv.gz | 67816 | ed69f8012fcd7d1c97a45b73910e8f189d6f94bb111b0ccec0484dccabb5054d |
| 46130058-BTCUSD-2026-11-15-101743.csv.gz | 23831 | 4f9a0a797253180f81df93399f3911ac4fab08139e74be57d66522c8e139bf70 |
| 46130058-ETHUSD-2026-11-15-020537.csv.gz | 55367 | fe4da35d6278e09be7d44037916978caa72d2498314dbf573fc8a561bfe69de7 |
| 46130058-ETHUSD-2026-11-15-060537.csv.gz | 1796 | 1aedfa43dd6ea101c5dc33d24ed31de92405b61247ba25e20beeaa64d3f5876a |
| 46130058-ETHUSD-2026-11-15-151000.csv.gz | 49019 | ad5b4468452ebdc0bfbba42530e749edf19fcecfcff55db605ee7bcb8439c5e7 |
| 46130058-ETHUSD-2026-11-15-191000.csv.gz | 8900 | abfb64b30dc774d3d89bb6b5e306229875b93d42e7658cd9636e828c269ca905 |
| 46130058-SpotCrude-2026-11-13-155625.csv.gz | 17731 | 41b62d30b4087a16d4ebb0c29fbbdfbd3291f169b8bb1c04e3685bb766173969 |
| 46130058-SpotCrude-2026-11-13-195625.csv.gz | 4248 | 6d09a86dabd2c7f7b4ccebf48b531e57048a7e5e758a146dfbccfedb1c72d6b3 |
| 46130058-XAUUSD-2026-11-13-161335.csv.gz | 42288 | 573fb15068f1dd69f3c895d465e5c1e80a53144d13b7e362d55df7bfd7a96b4f |
| 46130058-XAUUSD-2026-11-13-201335.csv.gz | 16452 | 92e07a629a78058b65d257ba1a278044231bd2658092b552ef90f095a327a4b6 |
| 46130058-XAUUSD-2026-11-14-001335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-14-041335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-14-081335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-14-121335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-14-161335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-14-201335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-15-001335.csv.gz | 4 | ee5292618d9f82110e4679fb68c5839a3cd10bc1202582194409f4771859bfa2 |
| 46130058-XAUUSD-2026-11-15-041335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-15-081335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-15-121335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-15-161335.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 46130058-XAUUSD-2026-11-15-201335.csv.gz | 3268 | 1acc24a44b417fe4ce848594ae8ed6bad40f46d099814c535c199c559d00fee3 |
| 47790949-BTCUSD-2026-11-15-061743.csv.gz | 67813 | e93cf7913185807f8469ded4f60bf7f4c4268a60b5000819d1c5bdf9cd484bab |
| 47790949-BTCUSD-2026-11-15-101743.csv.gz | 23831 | 7ae392515c3ac200509311c55417ffe5e34125dc7353e267302b99875ddef601 |
| 47790949-BTCUSD-2026-11-15-150209.csv.gz | 64826 | d987c4cfed42af990f6454db1bb1bab23f4ae641a66d06a4a8318d55442d70ee |
| 47790949-BTCUSD-2026-11-15-190209.csv.gz | 14612 | a5573e294776d16e7e0723c4e411ce9701e6bd85552dad901e97fad6ef3b2c96 |
| 47790949-ETHUSD-2026-11-13-160253.csv.gz | 50060 | e7216ec5e296f7826bd2c3b377cba12fdb6ade933e7209582da8402eced331e8 |
| 47790949-ETHUSD-2026-11-13-200253.csv.gz | 19607 | ad3641d8be91c9fbd1b4ce77f4ace38ad9b078309c66fd658521f07b9fd401ec |
| 47790949-ETHUSD-2026-11-15-020537.csv.gz | 55412 | 1be4dcae2a5346aae4cf046b38877a9d86e30ba951f6e2a915a1a597fcd2787d |
| 47790949-ETHUSD-2026-11-15-060537.csv.gz | 1796 | 609cb8518d17de8612f372392ec2f4d10207cbf1cb6be3746ab9772e487d856c |
| 47790949-SpotCrude-2026-11-13-155628.csv.gz | 17725 | 51a3dd2d11864d5e9739f87c818d915484738079f5e86b8beadb72ef8a71f352 |
| 47790949-SpotCrude-2026-11-13-195628.csv.gz | 4248 | c28dfc6ade2870ea926037aef71ee5a5ea8f6298627b0f6c4c101278f52b2853 |
| 47790949-XAUUSD-2026-11-13-161029.csv.gz | 42459 | 245e142a6fa1619799916370a644a17b9fa37035dd86ad522456204c0f178ad9 |
| 47790949-XAUUSD-2026-11-13-201029.csv.gz | 16955 | 982374333ddb94a2e9fd2e200201ad460e0948277c8530c310e664cbb1a7bdb4 |
| 47790949-XAUUSD-2026-11-14-001029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-14-041029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-14-081029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-14-121029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-14-161029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-14-201029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-15-001029.csv.gz | 4 | 99e0f2eeb2c135e7cb10f3841b90ab401e6571fe3645a735befad6143218a2e0 |
| 47790949-XAUUSD-2026-11-15-041029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-15-081029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-15-121029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-15-161029.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-XAUUSD-2026-11-15-201029.csv.gz | 3268 | 37ed1534b96fe3ded749a1683ab7b004310f37ba688ccf36a2a6c6932dc029ec |
| 49236051-BTCUSD-2026-11-13-162335.csv.gz | 59783 | 78539115f2008d73263162b3254a238b5b2f31f237e4fd52f6591a2f6df7a039 |
| 49236051-BTCUSD-2026-11-13-202335.csv.gz | 19755 | c379027a7def809dea4ad7c80352c91cde624cf696c8903dc1a5579a9c7ea717 |
| 49236051-BTCUSD-2026-11-14-192548.csv.gz | 69372 | 02749ef66398f4d99100e22e018f41239f18c7a6008d1022e66368ff21aab768 |
| 49236051-BTCUSD-2026-11-14-232548.csv.gz | 25859 | 84a282acb07d0646f87ebed75dc9a00b14aa88d4167a1c5434bedaef1c2f00c0 |
| 49236051-ETHUSD-2026-11-14-024103.csv.gz | 52948 | 845924ff409d2af50731831b443a0e10133db8b411f172f9e23e39c01e4602c1 |
| 49236051-ETHUSD-2026-11-14-064103.csv.gz | 15527 | 1c0c1bc0b70cd15f770662b5819aebc2b84e795ff39ded5819ace05134ebb55f |
| 49236051-SpotCrude-2026-11-13-155621.csv.gz | 17735 | 3b4081aa5f4baa2773d15e5142c7599c6738101e1c06a257e5eff924ed367225 |
| 49236051-SpotCrude-2026-11-13-195621.csv.gz | 4252 | 9f36a7153e2f8371a36b854588deb640d3cc4581b4eb70c839e0e4adb4af89a1 |

| Source capture | SHA256 |
| --- | --- |
| preflight.json | 2b55af422541836de587e602303909a0e834812190c101c93468f8c85d3a4545 |
| monitor.json | 6ad8cf272e24263e1261552f45e564afc675c6f2d39f9719d18dc703cdeaed3b |
| brent-opening-ticks.json | 4ccdaafb3c80683a48120c0d3ad27d8e7e99aa5b783465076b6052918f24a51c |
| cumulative.json | e49ef97afed0322203ec9b94bc97fa5a59e2b60f897c3c117b9d3714e31471b3 |
| opening-ticks-with-broker-time.json | 6f15faa61c291159428d7f706842606861f1e1bc1b6238b01cd41ef89dd991ab |
| human.json | f4517e3ae7187d3744e75f45e6a441e10b085a1436ada5602564a5c0bc57ee05 |
| health.json | 6d4745a0e4f584b146402f29afb0016d102c3c8d9e55c923facc10b8e7d778cb |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| activity.json | 516fdd53f2708776d0dcf8f5dc80a7afc1879b0783f9675765fa91e0dde68bf4 |
| evidence.json | e6678ee9d6249d2291537229b73de1f08175ca6dbe2655157d0989567c1bc902 |
| monitor-reconciled.json | 054c8fae5fc3760823274936d26a8b5a38438c549c9204d669066bfc0d002de6 |
| runtime.json | 896088796fd050be2066eace01346e92bf0cf45c29d2b7c1c87e076213c37cb6 |
| history-proof.json | 2c7ccd77e087236c530f98239f524f26ea83f9efe1cb12ec4c5ae5bec2ced165 |
