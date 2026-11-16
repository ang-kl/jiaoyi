# Monday London Review & New York Preparation — №18,472

**Activity:** [16 Nov 2026 15:25:00, 19:45:00) Asia/Singapore. **Read-only audit.** Production `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`; policy `s3_macro_pullback_owner_18409_v1`.

[Measured JSON and evidence](MONDAY_LONDON_REVIEW_AND_NY_PREP-18472-2026-11-16.json) · [Standalone HTML](MONDAY_LONDON_REVIEW_AND_NY_PREP-18472-2026-11-16.html)

## §1. Assessment

- **London S3 closing cash: +$114.93 from ten closes**, four positive-net and six negative-net. The two carried BTC shorts closed at 16:00 for **−$95.91 each**. Ten new entries produced eight further closes and **two BTC shorts still open at the 19:45 cutoff**. [1,2]
- **Monday-to-cutoff S3 cash: +$102.67; post-PR #214 cumulative closing cash: +$2,860.92.** The activity window continues directly from №18,470 without re-counting its six excursion follow-ups. [2,3]
- **The account divergence is real:** demo 46130058 closed **−$56.74**, demo 47790949 **+$170.19**, and demo 49236051 **+$1.48**. One ETH short reached TP for +$210.29; the other had a lower entry and TP, reached BE, then closed with slippage for −$19.71. Their absolute target levels differed by 1.79. [1,4]
- **All six earlier follow-ups now have complete four-hour clocks.** Neither the old BTC/ETH nor Gold paths reached their original TP within those four hours. Across the six older rows plus ten current closes, **eight clocks are complete and eight partial** at the frozen **20:20 SGT** price horizon. The three new stopped Brent shorts later crossed original TP, after already breaching their stops. [4]
- **Live manual NatGas closed for +$326.85**, separate from S3: +$685.50 gross less $358.65 swap. No live bot intent is recorded in the activity window. Current assessed margin is **306.34%**, but pending-entry projection **138.17%** and stress projection **−19.98%** still fail the 250% entry floor. [5]
- **16:00 crypto:** 8/8 source opening quotes, **7/8 exact operational references**, one S1-owned ETH slot withheld; the small demo's BTC reference exists but its pair is blocked. **20:00 crypto preparation 8/8 and 22:30 NY traditional preparation 24/24** were complete in receipts recorded before the cutoff. Preparation does not override ownership or admission guards. [6,7]
- **29 arms →10 consumed triggers →10 intents/outcomes →10 fills.** Missing individual outcomes and unknown guard receipts are zero; maximum confirmation-to-final-guard latency is **34.868 ms**. Latest20 wins are **3/20, 9/20, 11/20, 12/20**; **financial pass=false**.

## §2. Clock, cutoff and continuity

The mandatory first clock check was **16 Nov 12:19:59 UTC /20:19:59 SGT**, after the requested cutoff. The audit therefore ran immediately. Activity is strictly **[07:25,11:45) UTC = [15:25,19:45) SGT**. Post-close prices stop at **12:20:00 UTC /20:20 SGT**, regardless of the later publication time. Current money, book and health observations are timestamped separately.

The preceding report is [№18,470](MONDAY_ASIAN_LONDON_OPEN_REVIEW-18470-2026-11-16.md), PR #236, merge `14df970458f2ed6df9c3ec0906fe5d769cc15759`. Its 15:25 exclusive end is this report's inclusive start. The six old follow-ups contribute **zero closing cash to the new window**. The two carried positions are **263028399 /263028398**, already open at 15:25 and closed inside this window. [1–3]


| Account | Window closes | Window S3 USD | Monday 00:00→19:45 S3 USD | Post-PR214 closing cash USD | Open S3 at cutoff | Open S3 in later book |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | -0.01 | 0 | 0 |
| 46130058 | 4 | -56.74 | -79.33 | 1,311.38 | 1 | 0 |
| 47790949 | 4 | 170.19 | 174.38 | 1,523.11 | 1 | 0 |
| 49236051 | 2 | 1.48 | 7.62 | 26.44 | 0 | 0 |


Cash continuity is exact: **−$12.26 +$114.93 =+$102.67** for Monday, and **$2,745.99 +$114.93 =$2,860.92** cumulatively. The post-PR214 period is **[10 Nov 09:42,16 Nov 19:45) SGT**; it includes **+$412.88 from three earlier-entry positions**. This is a mixed-entry closing-cash cohort, not a causal profit estimate for the release.

Stored closing deals are reconciled with decimal arithmetic and unique `(account_id,deal_id)` keys. Successful retained history fetches cover beyond 20:22 SGT on all accounts; repeating the same frozen-window monitor query returned identical parsed results. History `complete=false` marks a rolling, unsealed fetch window, and is retained in the JSON rather than overwritten. [1,2]

**Important post-cutoff distinction:** the two newer BTC shorts subsequently closed at **20:00:00.991 SGT** for **+$85.61 /+$83.09**, total **+$168.70**. These receipts explain why the later book has no S3 positions. They are excluded from this report's closing cash and latest20 cutoff assessment, and are retained as explicitly post-cutoff evidence for the next audit.

## §3. Current account money, margin and inventory

Money observations are **20:22:15.975–20:22:16.285 SGT**; books are approximately **20:22:14**, runtime assessment was captured **20:22:43.911**, and health **20:23:09.048**. These are not atomic historical marks at 19:45. Equity is recorded balance plus broker net unrealised P&L, excluding unverified credit and prospective closing charges. Assessed margin comes from the separate owner assessment; the money snapshot itself retains unverified aggregate margin. [5]


| Account | Balance USD | Floating USD | Equity USD | Assessed current margin % | Pending projected % | Pending stressed % | S3 / S1 / S2 / external | Pending entries |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 47.75 | 2,869.53 | 2,917.28 | 306.34 | 138.17 | -19.98 | 0 / 2 / 0 / 4 | 4 |
| 46130058 | 25,918.59 | 113.55 | 26,032.14 | 19,851.74 | Unverified / N/A | Unverified / N/A | 0 / 5 / 0 / 0 | 0 |
| 47790949 | 25,797.48 | 113.62 | 25,911.10 | 2,471.25 | Unverified / N/A | Unverified / N/A | 0 / 5 / 0 / 0 | 0 |
| 49236051 | 821.21 | -0.47 | 820.74 | 94,339.08 | Unverified / N/A | Unverified / N/A | 0 / 1 / 0 / 0 | 0 |


Live current margin now passes the 250% floor, while its **pending-entry** and **stress** projections fail. The negative stress percentage is a model projection, not the current broker margin level. The live S3 crypto gate independently reports latest candidate projected levels **137.42% for BTC /139.45% for ETH**, both below 250%. Slightly different values belong to separately timed candidate and whole-book assessments. The demos have no pending entries, so no current pending projection is emitted; those cells are not invented zeros.


| Account / ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Unverified / N/A |
| 43002148 / 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Unverified / N/A |
| 43002148 / 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,813.70000 | Unverified / N/A |
| 43002148 / 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,830.70000 | Unverified / N/A |
| 43002148 / 657180931 | NVDA.US | human_external / position | buy | 200 / 2.0000 | 165.22000 | 149.66000 | Unverified / N/A |
| 43002148 / 658574004 | XAUUSD | human_external / position | buy | 3000 / 0.3000 | 3,841.86000 | 3,830.00000 | Unverified / N/A |
| 43002148 / 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 / 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 / 402061476 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | Unverified / N/A |
| 43002148 / 402051459 | NatGas | human_external / pending_entry | buy | 300000 / 0.3000 | 2.92900 | 2.79900 | Unverified / N/A |
| 46130058 / 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,830.80000 | Unverified / N/A |
| 46130058 / 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69054 | Unverified / N/A |
| 46130058 / 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12058 | Unverified / N/A |
| 46130058 / 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 46130058 / 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56586 | Unverified / N/A |
| 47790949 / 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,830.80000 | Unverified / N/A |
| 47790949 / 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69049 | Unverified / N/A |
| 47790949 / 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12053 | Unverified / N/A |
| 47790949 / 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 47790949 / 262976216 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55202 | 0.56581 | Unverified / N/A |
| 49236051 / 262796763 | ETHUSD | s1 / position | sell | 1 / 0.0100 | 1,746.87000 | 2,027.27000 | Unverified / N/A |


## §4. Live human protection and separate S1 status

**Zero recorded bot interference:** the retained live journal has **no bot intent in [15:25,19:45)**. No S3-owned live entry or close appears. External ownership is matched against the previous and current books and closing-history IDs; this is an audit of recorded commands, not a claim about unrecorded outside actors. NatGas and JPN225 remain human-reserved, and the manual Gold position continues to lock live S3 Gold and its Silver pair. [1,5]


| External position / deal | Symbol | Close SGT / price | Gross USD | Swap USD | Commission USD | Net USD |
| --- | --- | --- | --- | --- | --- | --- |
| 637075128 / 348284463 | NatGas | 16 Nov 16:42:14.820 / 2.97700 | 685.50 | -358.65 | 0.00 | 326.85 |


NatGas **637075128** closed externally at **16:42:14.820 SGT**, price **2.977**. Its **+$326.85** net cash reconciles the live balance exactly: **−$279.10 +$326.85 =+$47.75**. There is **no new deposit in this activity window**. Full Monday external closing cash is **−$1,045.38**, combining the earlier −$1,372.23 manual Gold closes with this NatGas close. The two earlier $1,000 deposits remain outside trading P&L.

The current live external book contains **JPN225×2, NVDA.US and Gold 658574004**. Gold remains long **0.30 lot at 3,841.86**, SL **3,830**, with no recorded TP or resting Gold pending entry. NatGas is no longer an open position; two current external NatGas pending orders are listed above. The audit does not attribute the external closing or order replacement to the bot.

Live S1 **GER40 656513085 /AUS200 656795267** remain open at **26,809.0 /9,165.3**, each 0.1 lot. Their SLs remain **25,813.7 /8,830.7**. No live S1 entry, close or trail amendment is recorded in the activity window.

Demo **49236051** retains its separate S1 ETH short **262796763**, entered Sunday **06:37:46.205 SGT at 1,746.87**, native volume 1 /0.01 unit. SL remains **2,027.27**, with no new S1 closing or trailing fill in this window. Its later account float is **−$0.47**, with only that position observed. S3 yields to this external-to-S3 ownership; the generic yield message does not mean a human opened it.

The two large demos each have **46 successful S1 trail amendments** in this activity window and no S1 new entry or close. Those S1 events remain separate from S3's ten new entries and ten closing positions. Full current S1 inventories appear above.

## §5. Complete S3 ledger and exit anatomy

There are **12 active-window positions**: two carried shorts plus ten new entries. Ten closed by 19:45, and two remained open. All are shorts. Original R is the distance from the actual opening fill to the initial SL; BE does not reset R. Net R uses original stop risk, actual volume and recorded quote-to-deposit conversion. [1]


| Account / position | Symbol / side | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit classification | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 263028399 | BTCUSD / sell | 25 / 0.2500 | 16 Nov 14:32:50.084 / 64,549.03000 | 16 Nov 16:00:01.240 / 64,932.68000 | 65,152.09000 / 63,716.80000 | session end | -95.91 / -0.636156 |
| 47790949 / 263028398 | BTCUSD / sell | 25 / 0.2500 | 16 Nov 14:32:50.047 / 64,549.03000 | 16 Nov 16:00:01.167 / 64,932.68000 | 65,152.09000 / 63,716.80000 | session end | -95.91 / -0.636156 |
| 49236051 / 263049563 | SpotBrent / sell | 800 / 0.0800 | 16 Nov 16:00:57.379 / 98.07300 | 16 Nov 16:30:58.312 / 98.65000 | 98.64600 / 97.28200 | initial stop | -4.62 / -1.007853 |
| 47790949 / 263049562 | SpotBrent / sell | 26700 / 2.6700 | 16 Nov 16:00:57.343 / 98.06300 | 16 Nov 16:30:57.913 / 98.64500 | 98.63600 / 97.27200 | initial stop | -155.39 / -1.015681 |
| 46130058 / 263049566 | SpotBrent / sell | 27100 / 2.7100 | 16 Nov 16:00:57.431 / 98.07300 | 16 Nov 16:30:58.250 / 98.65000 | 98.64600 / 97.28200 | initial stop | -156.37 / -1.007000 |
| 47790949 / 263068868 | ETHUSD / sell | 1068 / 10.6800 | 16 Nov 16:46:46.340 / 1,809.71000 | 16 Nov 17:15:04.283 / 1,790.02000 | 1,823.97000 / 1,790.03000 | protective TP | 210.29 / 1.380791 |
| 46130058 / 263068905 | ETHUSD / sell | 1083 / 10.8300 | 16 Nov 16:46:50.335 / 1,807.92000 | 16 Nov 18:27:37.299 / 1,809.74000 | 1,822.18000 / 1,788.24000 | BE stop with slippage | -19.71 / -0.127626 |
| 49236051 / 263074670 | SpotCrude / sell | 800 / 0.0800 | 16 Nov 17:01:05.862 / 88.90700 | 16 Nov 17:26:34.184 / 88.14400 | 89.45600 / 88.14900 | protective TP | 6.10 / 1.388889 |
| 46130058 / 263074682 | SpotCrude / sell | 28100 / 2.8100 | 16 Nov 17:01:07.376 / 88.87700 | 16 Nov 17:31:27.416 / 88.11100 | 89.42600 / 88.11900 | protective TP | 215.25 / 1.395290 |
| 47790949 / 263074686 | SpotCrude / sell | 27900 / 2.7900 | 16 Nov 17:01:08.121 / 88.88700 | 16 Nov 17:31:07.588 / 88.13000 | 89.43300 / 88.13300 | protective TP | 211.20 / 1.386427 |
| 46130058 / 263124251 | BTCUSD / sell | 39 / 0.3900 | 16 Nov 19:08:40.945 / 64,611.13000 | Open / unavailable / Unverified / N/A | 65,009.53000 / 64,061.33000 | OPEN at cutoff | Unverified / N/A / Unverified / N/A |
| 47790949 / 263124254 | BTCUSD / sell | 38 / 0.3800 | 16 Nov 19:08:42.432 / 64,610.29000 | Open / unavailable / Unverified / N/A | 65,008.69000 / 64,060.49000 | OPEN at cutoff | Unverified / N/A / Unverified / N/A |

| Position | Observed MFE / MAE R | Confirmed BE amendment SGT | Registered exit deadline SGT |
| --- | --- | --- | --- |
| 263028399 | 0.348373 / 0.813302 | None observed | 16 Nov 16:00:00.000 |
| 263028398 | 0.348373 / 0.813302 | None observed | 16 Nov 16:00:00.000 |
| 263049563 | 0.087260 / 1.006981 | None observed | 16 Nov 22:30:00.000 |
| 263049562 | 0.069808 / 1.015707 | None observed | 16 Nov 22:30:00.000 |
| 263049566 | 0.087260 / 1.006981 | None observed | 16 Nov 22:30:00.000 |
| 263068868 | 1.386396 / 0.204067 | 16 Nov 17:04:13.480 | 16 Nov 20:00:00.000 |
| 263068905 | 1.272090 / 0.329593 | 16 Nov 17:08:05.897 | 16 Nov 20:00:00.000 |
| 263074670 | 1.389800 / 0.032787 | 16 Nov 17:18:21.383 | 16 Nov 22:30:00.000 |
| 263074682 | 1.389800 / 0.074681 | 16 Nov 17:19:50.005 | 16 Nov 22:30:00.000 |
| 263074686 | 1.402930 / 0.056777 | 16 Nov 17:19:21.390 | 16 Nov 22:30:00.000 |
| 263124251 | 0.351832 / 0.388680 | None observed | 16 Nov 20:00:00.000 |
| 263124254 | 0.349724 / 0.390788 | None observed | 16 Nov 20:00:00.000 |


Closing cash decomposes into **BTC −$191.82**, **Brent −$316.38**, **ETH +$190.58**, and **Crude +$432.55**, totalling **+$114.93**. Four positive-net closes and six negative-net closes give a 40% window net-win rate. The entry-price BE exit is a net loss after slippage, not a zero-P&L win. Account copies of one Brent or Crude setup are correlated observations.

**Carried BTC:** positions 263028399 /263028398 closed at the registered 16:00 session end. Their observed MFE was only **+0.3484R**, below the unchanged +1.00R BE threshold; MAE reached **0.8133R**. Neither BE nor original TP was reached during the held interval.

**ETH divergence:** demo 47790949 entered at **16:46:46.340 SGT,1,809.71**, while demo 46130058 entered **3.995 seconds later at 1,807.92**. Both had initial R **14.26**, but actual TP levels were **1,790.03 vs1,788.24**. The recorded low near **1,789.78** crossed the first target, not the second. The first trade made **+$210.29**; the second reached **1.2721R MFE**, received a BE amendment at **17:08:05.897**, then closed at **18:27:37.299,1,809.74**, **1.82 adverse price units beyond its entry-price stop**, for **−$19.71**. Both used the same deployed exit policy. Different confirmation times and fills explain the price-level distinction; this audit does not claim a proven software defect or isolate a causal latency effect.

The new BTC shorts **263124251 /263124254**, entered around **19:08:41**, were still open at the cutoff:


| Position / account | Cutoff ask / quote SGT | Price-only gross USD / R | Original SL / TP | Registered deadline |
| --- | --- | --- | --- | --- |
| 263124251 / 46130058 | 64,531.85 / 16 Nov 19:44:59.779 | 30.9192 / 0.198996 | 65,009.53 / 64,061.33 | 16 Nov 20:00:00.000 |
| 263124254 / 47790949 | 64,531.85 / 16 Nov 19:44:59.777 | 29.8072 / 0.196888 | 65,008.69 / 64,060.49 | 16 Nov 20:00:00.000 |


These cutoff marks exclude fees/swap and are not broker-certified S3 net unrealised P&L. Their later 20:00 realised profits are separately recorded in §2 and excluded from the cutoff totals.

## §6. Latest20 qualification


| Account | Net wins / count | Win rate | Realised payoff (R) | Profit factor (R) | Net R | Assessment SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | 3.4277 | 0.6049 | -5.9677 | 16 Nov 19:42:12.825 |
| 46130058 | 9/20 | 45.00% | 1.6058 | 1.3139 | 2.1123 | 16 Nov 19:40:18.450 |
| 47790949 | 11/20 | 55.00% | 1.2726 | 1.5555 | 3.6083 | 16 Nov 19:40:23.378 |
| 49236051 | 12/20 | 60.00% | 1.3690 | 2.0534 | 6.2047 | 16 Nov 19:40:26.636 |


All assessment timestamps precede 19:45. Relative to №18,470, wins change **11→9** on 46130058, **12→11** on 47790949 and **11→12** on 49236051; live stays 3/20. No account reaches **15/20**, the integer threshold corresponding to the 73% goal. The configured 1.38R gross target is not a guaranteed realised payoff. Positive London cash and correlated copied wins do not establish the required win-rate/payoff milestone. **Financial pass=false.**

## §7. Four-hour post-close excursions

The frozen quote horizon is **20:20:00 SGT**. All **six prior partial rows are now complete**, as are the two carried BTC exits; the other eight current exits have elapsed partial windows. The dataset therefore contains **16 post-close rows: eight complete clocks and eight partials**. [4]

Movement is signed in the original short direction from the **actual exit fill**, using the **ask** as executable exit-side price and the original R. Favourable/adverse values are **price / broker pips / R**. Broker pip size is **0.1 for BTC, ETH and Gold**, and **0.01 for Brent and Crude**. TP and SL tests use the original absolute levels, so an exit-anchored movement greater than 1.38R need not reach original TP.



### Current-window closes


| Position / symbol / exit | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 263028399 / BTCUSD / session end | 4.00000 complete clock | 877.14000 / 8,771.40 / 1.454482 | 23.32000 / 233.20 / 0.038669 | 5,397.90 / 0.895085 | 6,292.80 / 1.043478 | 5,344.50 / 0.886230 | False / False |
| 263028398 / BTCUSD / session end | 4.00000 complete clock | 877.14000 / 8,771.40 / 1.454482 | 23.32000 / 233.20 / 0.038669 | 5,397.90 / 0.895085 | 6,292.80 / 1.043478 | 5,410.50 / 0.897174 | False / False |
| 263049563 / SpotBrent / initial stop | 3.81714 PARTIAL | 1.87700 / 187.70 / 3.275742 | 0.00500 / 0.50 / 0.008726 | 170.50 / 2.975567 | 113.10 / 1.973822 | not_elapsed_at_capture | True / True |
| 263049562 / SpotBrent / initial stop | 3.81725 PARTIAL | 1.87200 / 187.20 / 3.267016 | 0.01000 / 1.00 / 0.017452 | 169.00 / 2.949389 | 112.60 / 1.965096 | not_elapsed_at_capture | True / True |
| 263049566 / SpotBrent / initial stop | 3.81715 PARTIAL | 1.87700 / 187.70 / 3.275742 | 0.00500 / 0.50 / 0.008726 | 170.50 / 2.975567 | 113.10 / 1.973822 | not_elapsed_at_capture | True / True |
| 263068868 / ETHUSD / protective TP | 3.08214 PARTIAL | 0.24000 / 2.40 / 0.016830 | 25.35000 / 253.50 / 1.777700 | -99.00 / -0.694250 | -183.20 / -1.284712 | not_elapsed_at_capture | True / False |
| 263068905 / ETHUSD / BE stop with slippage | 1.87297 PARTIAL | 19.79000 / 197.90 / 1.387798 | 5.63000 / 56.30 / 0.394811 | 18.00 / 0.126227 | not_elapsed_at_capture | not_elapsed_at_capture | False / False |
| 263074670 / SpotCrude / protective TP | 2.89050 PARTIAL | 0.28500 / 28.50 / 0.519126 | 0.82400 / 82.40 / 1.500911 | -61.70 / -1.123862 | -20.20 / -0.367942 | not_elapsed_at_capture | True / False |
| 263074682 / SpotCrude / protective TP | 2.80905 PARTIAL | 0.25200 / 25.20 / 0.459016 | 0.85700 / 85.70 / 1.561020 | -44.70 / -0.814208 | -33.00 / -0.601093 | not_elapsed_at_capture | True / False |
| 263074686 / SpotCrude / protective TP | 2.81456 PARTIAL | 0.27100 / 27.10 / 0.496337 | 0.83800 / 83.80 / 1.534799 | -49.30 / -0.902930 | -35.30 / -0.646520 | not_elapsed_at_capture | True / False |


### Completed №18,470 follow-ups — no new closing cash


| Position / symbol / exit | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 262978198 / BTCUSD / session end | 4.00000 complete clock | 836.27000 / 8,362.70 / 1.690596 | 97.50000 / 975.00 / 0.197105 | 2,796.90 / 0.565419 | 6,807.50 / 1.376198 | 1,283.60 / 0.259491 | False / False |
| 262978205 / BTCUSD / session end | 4.00000 complete clock | 834.76000 / 8,347.60 / 1.687543 | 99.01000 / 990.10 / 0.200158 | 2,781.80 / 0.562366 | 6,792.40 / 1.373145 | 1,268.50 / 0.256439 | False / False |
| 263005199 / ETHUSD / initial stop | 4.00000 complete clock | 27.34000 / 273.40 / 2.289782 | 0.00000 / 0.00 / 0.000000 | 80.60 / 0.675042 | 214.30 / 1.794807 | 203.90 / 1.707705 | False / True |
| 263005201 / ETHUSD / initial stop | 4.00000 complete clock | 27.34000 / 273.40 / 2.289782 | 0.00000 / 0.00 / 0.000000 | 80.60 / 0.675042 | 214.30 / 1.794807 | 203.90 / 1.707705 | False / True |
| 263023705 / XAUUSD / session end | 4.00000 complete clock | 3.21000 / 32.10 / 0.100313 | 22.75000 / 227.50 / 0.710938 | -49.90 / -0.155938 | -87.20 / -0.272500 | -222.30 / -0.694688 | False / False |
| 263024012 / XAUUSD / session end | 4.00000 complete clock | 3.21000 / 32.10 / 0.100281 | 22.75000 / 227.50 / 0.710715 | -49.90 / -0.155889 | -87.20 / -0.272415 | -222.30 / -0.694470 | False / False |


**Six completed earlier rows:** neither old BTC session-end path reached original TP; the low stayed at 64,224.77 versus targets 64,219.76 /64,219.08. The old ETH stops moved favourably 27.34 price units from the exit, yet the four-hour low 1,803.22 remained above targets 1,801.36 /1,801.57. The old Gold shorts moved only 3.21 favourably and 22.75 adversely from their actual closes; neither original TP nor SL was crossed. Their original fill/stop/cash and in-position excursions are unchanged from the frozen prior ledger.

**Carried BTC exits:** both have complete four-hour follow-ups. Price moved favourably **877.14 /8,771.4 pips /1.4545R from the exit**, but the lowest ask 64,055.54 remained above original TP 63,716.80. Neither original TP nor SL was touched. A large recovery from the realised loss is not proof that the original target would have completed.

**New Brent stop-outs:** all three later reached original TP, first at approximately **17:04:37–17:04:50 SGT**, after their 16:30 stop-outs. The observed post-close favourable movement is **1.872–1.877 price /187.2–187.7 pips /3.267–3.276R**, with little additional adverse movement from the stop fill. These are three account copies of one reversal episode. The initial stop had already been breached; the path does not establish a profitable executable alternative under unchanged risk rules. At the frozen horizon their windows are only **3.817h**, with full four-hour ends around 20:30:58.

**ETH and Crude:** the profitable ETH exit had only 0.24 further favourable price movement before a sizeable reversal; the BE-loss ETH path had not reached its original TP by 20:20. The three Crude TP exits had additional favourable movement but also sizeable adverse excursions. Full future windows remain pending around **21:15 (ETH TP),21:26–21:31 (Crude),22:27 (ETH BE)**.

All **39 elapsed 1h/2h/4h checkpoints** have quotes within six seconds. **Nine future checkpoints remain null**. Complete elapsed clocks do not imply continuous quotes: the maximum recorded inter-update gap is about 25.094 seconds for Brent,23.938 seconds for Crude and 6.612 seconds for ETH. No interpolation or simulated fills cover those gaps. The JSON retains first/last quotes, exact extrema/crossing times and endpoint ages.

## §8. Veto funnel and telemetry integrity


| Account | New arms | Consumed / intents / outcomes | Filled entries | Terminal refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 13 | 0 / 0 / 0 | 0 | 0 | 11 |
| 46130058 | 6 | 4 / 4 / 4 | 4 | 0 | 1 |
| 47790949 | 7 | 4 / 4 / 4 | 4 | 0 | 2 |
| 49236051 | 3 | 2 / 2 / 2 | 2 | 0 | 1 |


There are **29 new arms,10 consumed triggers, 10 persisted opening intents and 10 individual filled outcomes**. No consumed trigger lacks its own terminal receipt; unknown entry-guard receipts are **zero**. The window has **15 >61.8% retracement invalidations** and **58 pair-reservation diagnostic records**, with no terminal 24 bp peer refusal among the ten consumed triggers. These diagnostic counts are not independent missed trades. [6]

Maximum observed **confirmation-to-final-guard latency is 34.868 ms**. It is separate from touch-to-turn waiting or broker send-to-fill latency. The canonical monitor retains `refusals_all_stages_complete=false`; complete consumed-trigger evidence does not establish an exhaustive census of every deduplicated pre-arm veto.

Current, separately timed core-symbol states:


| Account | Symbol | Current gate / reason |
| --- | --- | --- |
| 43002148 | BTCUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 137.42% below floor 250.00% |
| 43002148 | ETHUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 139.45% below floor 250.00% |
| 43002148 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 5.001%, 20d: -4.379%) |
| 43002148 | SpotBrent | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 139.97% below floor 250.00% |
| 43002148 | SpotCrude | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 139.98% below floor 250.00% |
| 43002148 | US500 | Waiting: opening impulse below 48 bps |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Armed: waiting 50%–61.8% pullback + 8% turn (ref: 64124.6, extreme: 63869.17); touch not yet recorded |
| 46130058 | ETHUSD | Waiting: current decision mark already consumed |
| 46130058 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 5.001%, 20d: -4.379%) |
| 46130058 | SpotBrent | Blocked: one entry already recorded for this symbol/window |
| 46130058 | SpotCrude | Blocked: one entry already recorded for this symbol/window |
| 46130058 | US500 | Waiting: opening impulse below 48 bps |
| 46130058 | XAGUSD | Waiting: opening impulse below 48 bps |
| 46130058 | XAUUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.329%, 20d: -7.077%) |
| 47790949 | BTCUSD | Preparing: background entry admission is not ready yet |
| 47790949 | ETHUSD | Waiting: current decision mark already consumed |
| 47790949 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 5.001%, 20d: -4.379%) |
| 47790949 | SpotBrent | Blocked: one entry already recorded for this symbol/window |
| 47790949 | SpotCrude | Blocked: one entry already recorded for this symbol/window |
| 47790949 | US500 | Waiting: opening impulse below 48 bps |
| 47790949 | XAGUSD | Waiting: opening impulse below 48 bps |
| 47790949 | XAUUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.329%, 20d: -7.077%) |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 5.001%, 20d: -4.379%) |
| 49236051 | SpotBrent | Blocked: one entry already recorded for this symbol/window |
| 49236051 | SpotCrude | Blocked: one entry already recorded for this symbol/window |
| 49236051 | US500 | Waiting: opening impulse below 48 bps |
| 49236051 | XAGUSD | Waiting: opening impulse below 48 bps |
| 49236051 | XAUUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.329%, 20d: -7.077%) |


## §9. Crypto openings and New York readiness

All **eight 16:00 crypto slots** have first recorded two-sided quotes within 0.364 seconds. Exact full operational quote evidence exists for **seven** slots; the small demo's ETH slot is withheld by S1 ownership. Its BTC reference exists but S3 entry is pair-blocked. Live crypto opening references also exist even though current margin admission fails. Source quotes, operational references and permission to trade are distinct measurements. [7]


| Account | Symbol | Opening SGT | First two-sided quote SGT | Delay seconds | Verification | Ownership constraint |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.035 | 0.035571 | exact_quote_receipt | None recorded for this slot |
| 43002148 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.290 | 0.290892 | exact_quote_receipt | None recorded for this slot |
| 49236051 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.033 | 0.033269 | exact_quote_receipt | external_s1_crypto_pair |
| 49236051 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.316 | 0.316629 | ownership_locked | external_s1_crypto_pair |
| 47790949 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.031 | 0.031848 | exact_quote_receipt | None recorded for this slot |
| 46130058 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.351 | 0.351514 | exact_quote_receipt | None recorded for this slot |
| 47790949 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.363 | 0.363370 | exact_quote_receipt | None recorded for this slot |
| 46130058 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 16:00:00.019 | 0.019282 | exact_quote_receipt | None recorded for this slot |


The following preparation receipts were all recorded **before 19:45**. Both base inputs and full mark tables are complete: **16:00 crypto 8/8**, **20:00 crypto 8/8**, and **22:30 NY traditional 24/24**. Future preparation does not certify a future opening quote, a qualifying signal or successful admission. All four accounts are connected and unpaused on the expected commit. Current 20:23 snapshots are not substituted for pre-cutoff readiness evidence.


| Account | Target open SGT | Symbol slots | Base ready | Full marks ready | Latest preparation receipt SGT |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 16 Nov 16:00:00.000 | 2 | 2 | 2 | 16 Nov 12:00:23.181 |
| 46130058 | 16 Nov 16:00:00.000 | 2 | 2 | 2 | 16 Nov 12:00:23.569 |
| 47790949 | 16 Nov 16:00:00.000 | 2 | 2 | 2 | 16 Nov 12:00:23.567 |
| 49236051 | 16 Nov 16:00:00.000 | 2 | 2 | 2 | 16 Nov 12:00:27.182 |
| 43002148 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:23.026 |
| 46130058 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:23.884 |
| 47790949 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:23.779 |
| 49236051 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:26.006 |
| 43002148 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:35.664 |
| 46130058 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:30.605 |
| 47790949 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:30.383 |
| 49236051 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:32.941 |


## §10. Assessment and next audit

London cash is positive because Crude targets and one ETH target offset Brent stops, BTC timeouts and an ETH BE-slippage loss. The rolling win counts remain below qualification, and outcomes differ materially across the large demos. The data supports retaining separate account, symbol, entry-price and exit-reason analysis; it does not justify changing risk or exits from one session.

The most useful follow-ups are to complete the eight partial current-window paths, carry the two 20:00 BTC closes into the next activity window exactly once, and observe NY opening capture against the already-complete preparation receipts. The Brent reversal is a research observation for a separately authorised investigation. No strategy, stop, BE, TP, risk cap, margin floor, pair rule or ownership law was changed for this report.

## §11. Evidence, verification and Gemini hand-off

**Sources:** [1] read-only stored PostgreSQL deals, orders/fills, intent/protobuf events and contract metadata; [2] independently bounded window/Monday/cumulative monitor queries, decimal cash and successful history reconciliation; [3] merged №18,470 and exact 15:25 continuity; [4] **25 existing-tick exports /587,022 raw rows**, zero broker collection requests; [5] separately timed stored money marks, owner book/margin assessment, health and external closing evidence; [6] consumed-ID→individual-outcome→causal-intent joins and latest20 assessments before the cutoff; [7] first quote streams, immutable arm/order/session references and causal preparation receipts. Evidence text was treated as data, not instructions.

Six focused checks passed: cutoff/cash continuity, quote hashes/ranges/coverage, pip/R arithmetic and unchanged earlier history, carried-in and consumed-trigger accounting, external ownership/cash with post-cutoff exclusion, and causal opening/preparation denominators. The measured JSON includes complete ledgers, cash receipts, source/calculation/test code, hashes and timing evidence. Desktop/mobile rendering and secret checks precede publication.

Only these three artifacts under `doc/` are committed. Required CI is `check`, `docker`, `engineering`; Railway watch patterns exclude `doc/`. Initial Trading deployment is **3d8b18d5-4d9f-4007-8fff-22eb25d5ea73**, image **sha256:42609036a809db4674fdc007804d6a73ce15369fbf1e6f356981ba47ddd86d6c**, commit **65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2**. Post-merge deployment/account-session continuity is verified separately and returned with the PR and merge SHA. No trading code, settings, broker order, Telegram message or deployment is changed.

**Gemini:** read this Markdown and the matching JSON. Verify +$114.93 window /+$102.67 Monday /+$2,860.92 cumulative S3 cash; the excluded +$168.70 of 20:00 closes; separate +$326.85 manual NatGas cash; ten current closes and six older follow-ups; eight complete clocks and eight partials; three stopped Brent paths later crossing original TP; the two different ETH entry/TP levels; latest20 below 15 wins; and NY 24/24 /crypto 8/8 preparation with ownership/margin guards intact. Do not turn a post-close price touch into a simulated profitable trade or include post-cutoff closing cash in this window.


| Quote export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-BTCUSD-2026-11-16-035955.csv.gz | 56810 | c8371caee6249f4c59fc05df8775a3ad713b3207522fff6158d8dd2cf6033c67 |
| 46130058-BTCUSD-2026-11-16-075955.csv.gz | 59852 | 84b93d1f6467ab90ecd0b42de8b6187e944dcb1185f9f5784cb439ebf9ac3ca4 |
| 46130058-BTCUSD-2026-11-16-115955.csv.gz | 19 | d7c689c0065039d726f551b080368935b5370dfbbd40ef57fe8b00778e2f9698 |
| 46130058-ETHUSD-2026-11-16-042420.csv.gz | 44293 | 90bc467d2172da155952f7141de856dc1ddf7e3a2652838423fc0a3334251a9b |
| 46130058-ETHUSD-2026-11-16-082420.csv.gz | 6 | aaf5e0d7b605db1af125ef2e9ac93d1cc825110c96fe1ba214f942bad015d714 |
| 46130058-ETHUSD-2026-11-16-084644.csv.gz | 43767 | 9ef41cc5733277131f98ad67c96bae695b36d4d90dc9a0793d827391bd038521 |
| 46130058-SpotBrent-2026-11-16-080051.csv.gz | 15674 | abe9e8c64f399bea98e063cc5644d56f64b6d6dac01c2dd92dd90ba3322e456a |
| 46130058-SpotBrent-2026-11-16-120051.csv.gz | 1184 | 92ded86de9eec0b37f04122ec6bde0115e19c42d06f53d01bae54805be568cb0 |
| 46130058-SpotCrude-2026-11-16-090101.csv.gz | 14833 | 1f145626a02a7e6b7c83a0f739cf7872811798698b32d68d45646d3cd84b2518 |
| 46130058-XAUUSD-2026-11-16-065955.csv.gz | 41206 | 562ef8a6a434d4ffcb8d4025144d7bffbaa27ea96b5f4204bc1f1e8925b5ad0c |
| 46130058-XAUUSD-2026-11-16-105955.csv.gz | 16 | 5df7b21d3c475744c81ea0812efc13ce8d06e9b02a80a8000d730120cc1af42f |
| 47790949-BTCUSD-2026-11-16-035955.csv.gz | 56810 | 05156e2bbb43bbbfa28551883154fdb1b1eb158b49c14f2a0af3c2c2559b6a65 |
| 47790949-BTCUSD-2026-11-16-075955.csv.gz | 59852 | 3d747ad7932dd3beb1a4808b196273e963f75c4c8fc31584336a0f96541e048c |
| 47790949-BTCUSD-2026-11-16-115955.csv.gz | 18 | acd6051e68bb6766b1abd7b27f889df67f17a7c492f208df826b45bb61ad662e |
| 47790949-ETHUSD-2026-11-16-042420.csv.gz | 44292 | 08617b9ddd839d44882ca83e96975887e386859eb12561f754d794dadc2e6b55 |
| 47790949-ETHUSD-2026-11-16-082420.csv.gz | 6 | 4a143ae507f38515dd3976cffb2094796b41f12a91ae342d441fb85d136ca4b9 |
| 47790949-ETHUSD-2026-11-16-084640.csv.gz | 43780 | ed690b8f825735a6cfe09812409a868f71997702a3afd7797c3f8d5d9ee5044b |
| 47790949-SpotBrent-2026-11-16-080051.csv.gz | 15675 | 5af546156950a3a5d325f840e2644e372de35eff27b59b93aaf0cfc9ebba47e0 |
| 47790949-SpotBrent-2026-11-16-120051.csv.gz | 1184 | e19954fede49426a8fc7be5a04634b5954381bbc7126414b1c357a3728065c9d |
| 47790949-SpotCrude-2026-11-16-090102.csv.gz | 14830 | e72c54b6e0a3aa5e733070da462bd654252493e89b37f0539f4b4e15080bf445 |
| 47790949-XAUUSD-2026-11-16-065955.csv.gz | 41206 | 5c12d480ef9761934f91529d337e8d61676cc63d508cc8570a098759a894770f |
| 47790949-XAUUSD-2026-11-16-105955.csv.gz | 16 | 1c0b643db71f8fb9db50fc60298a32a488867145d232d3aa97e7db312c97aad9 |
| 49236051-SpotBrent-2026-11-16-080051.csv.gz | 15675 | efb2a3e71daf5dfd8ee1604a5d4a54adf69d73478c69db1d4d339f36d0f3b749 |
| 49236051-SpotBrent-2026-11-16-120051.csv.gz | 1184 | 5535bae71218c3096164ae57b64ffbb3fe3d5dfc18bc9e88ee102b993e6e60bb |
| 49236051-SpotCrude-2026-11-16-090059.csv.gz | 14834 | ff6c53811506878bb72a4588421e6ad4e4cad2fe3dccf865cfb444d29faf64c4 |

| Source | SHA256 |
| --- | --- |
| opening-ticks.json | 188c21afe7703b7628d2b59356fe5f3ea8f030032f1288ca42dbdcb9bcf99448 |
| preflight.json | c9750a8fa2ac9a13e49f4c4fd0cccd9ea9defff523fb370861276e0a4527d3af |
| monitor.json | 19af9ef515f902233db794da3035b4b0111275d39dcfb3c4872627a982356c39 |
| cumulative.json | 48770549405fd93ae2fb918795e35545167f37dc0d68902e343c832ecbe109ee |
| human.json | 83dd2a574b4de63b9b7f6ba85b3a53f690725f754ba5b6f4d081dc852a1cdbde |
| health.json | 46feb19bcdcd3b4a255292873587809633225ed9096c8a8c157f2d64c71d2a74 |
| supplement.json | bf4447d4869d08f4dfbfad8e599496af3357a66f55d190c47692b076458fc442 |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| activity.json | 1cf1782b935bbaf8811a3de971e252c3dc4c1a6f0eabfc032437caebbc2e1acb |
| evidence.json | 16a9a5399b790385b61fa010577771273de4ec1e9f072740ac4758f9eabfb2fe |
| day.json | c7b033823cc37ba774d441b14a131c03c3da970ebd808ecf22c2c7253be54b4a |
| opening-recovery.json | a00a1adf6099a45f8fdd496e293421d8424e77c8c692c90355b237f78592dfe2 |
| monitor-reconciled.json | 19af9ef515f902233db794da3035b4b0111275d39dcfb3c4872627a982356c39 |
| runtime.json | 02f6c3a814462065c5cef595ee47187b29923434985c2a0ccc84f9e34cc59a56 |
| history-proof.json | 70015eeb04686ee2cae6ce2df31d1c262eed6db1fe8231b9fa5914dd928a9ec7 |
