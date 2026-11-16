# Tuesday Pre-Dawn NY Review & Post-Close Excursions — №18,474

**Activity:** [16 Nov 2026 19:45:00, 17 Nov 2026 04:45:00) Asia/Singapore. **Read-only audit.** Production `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`; policy `s3_macro_pullback_owner_18409_v1`.

[Measured JSON and evidence](TUESDAY_PREDAWN_NY_REVIEW_AND_EXCURSIONS-18474-2026-11-17.json) · [Standalone HTML](TUESDAY_PREDAWN_NY_REVIEW_AND_EXCURSIONS-18474-2026-11-17.html)

## §1. Assessment

- **Overnight S3 closing cash: −$442.39 from six closes.** The two carried BTC shorts closed at 20:00 for **+$168.70** combined; four new ETH shorts stopped for **−$611.09**. Two new Gold shorts remain open at the cutoff. [1,2]
- **Full Monday S3 cash: −$23.68; Tuesday 00:00–04:45: −$316.04; post-PR #214 cumulative closing cash: +$2,418.53.** The cumulative figure includes earlier-entry positions and is not a causal return estimate for the release. [2,3]
- **Both open Gold positions store the correct 05:57 SGT exit deadline**, policy `native_preclose_v18455_120s`, exactly 120 seconds before the 05:59 maintenance boundary. The later execution of that exit is outside this cutoff. [1,8]
- **All 14 required post-close clocks are complete:** six current closes plus eight prior partial rows. None of the four overnight ETH stop-outs subsequently reached original TP within four hours. Three earlier Brent initial stops and the earlier ETH BE stop did. Both carried BTC time exits crossed original TP first, then original SL. These are observed paths, not assumed executions. [4]
- **Live margin is below the entry floor:** assessed current **123.55%**, pending projection **88.53%**, stress **−17.17%**. Live balance is **$27.39**, equity **$2,162.23** in the later snapshot. Separate cash comprises **−$190.00 manual NatGas**, **−$30.36 from an S1-owned GER40 close**, and **+$200 deposits**. No live bot intent or recorded interference appears in this interval; the S1 closing initiator is not established by the program journal. [5]
- **24 arms →14 consumed triggers/intents →6 fills and 8 terminal refusals.** Every consumed trigger has an individual outcome; unknown guard receipts are zero. Six refusals were peer-hold failures and two exceeded the funded pullback bound. Maximum confirmation-to-final-guard latency: **23.045 ms**. [6]
- **48 opening slots:** all have recorded source quotes; **36 exact operational quote receipts, 4 recovered-midpoint proofs, 8 ownership-withheld slots**. Live BTC had an early 20:00 opening refusal, but a later assessment confirms the correct original midpoint. Future base/full preparation is **Asian 20/20, crypto 8/8, Brent 4/4**, with four accounts connected/unpaused. [7]
- Latest20 net wins are **3/20 live, 8/20 demo 46130058,9/20 demo 47790949,12/20 demo 49236051**. No account meets 15/20; **financial pass=false**.

## §2. Clock, cutoff and cash continuity

The mandatory clock check returned **16 Nov 20:42:53 UTC /17 Nov 04:42:53 SGT**. Bounded waits ended at **20:45:01.780 UTC** before activity queries began. The exact activity interval is **[16 Nov 11:45:00,20:45:00) UTC**, equivalent to the requested **[Monday 19:45, Tuesday 04:45) SGT**. The frozen post-close quote horizon is **17 Nov 04:45:30 SGT**. Later capture/publication time does not move the activity cutoff or this price horizon.

The previous [№18,472 report](MONDAY_LONDON_REVIEW_AND_NY_PREP-18472-2026-11-16.md), **PR #237 /`d5a42d3f04821fa7c23b29f412c27b6bb0261352`**, ends exactly at this interval's start. Its eight partial excursion rows contribute no closing cash here. Its carried BTC positions **263124251 /263124254** closed at 20:00 and are counted exactly once in this interval. [1–3]


| Account | Window closes | Window S3 USD | Full Monday S3 USD | Tuesday 00:00→04:45 S3 USD | Post-PR214 cash USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | 0.00 | -0.01 | 0 |
| 46130058 | 3 | -215.79 | -132.91 | -162.21 | 1,095.59 | 1 |
| 47790949 | 3 | -226.60 | 101.61 | -153.83 | 1,296.51 | 1 |
| 49236051 | 0 | 0.00 | 7.62 | 0.00 | 26.44 | 0 |


Cash continuity: **$2,860.92 −$442.39 =$2,418.53** cumulatively. Monday's previous cutoff cash **+$102.67** plus its later **−$126.35** gives **−$23.68** for the complete calendar day; Tuesday contributes **−$316.04**. Monday and Tuesday deal sets are disjoint. The cumulative period is **[10 Nov 09:42,17 Nov 04:45) SGT** and retains **+$412.88 from three positions entered before its start**. This mixed-entry closing-cash cohort must not be described as the release's standalone performance.

Net cash = recorded gross + swap −absolute commission −absolute P&L conversion fee, using each receipt's money digits and decimal arithmetic. Unique `(account_id,deal_id)` keys prevent duplicate receipts. All four successful retained history fetches extend beyond 04:48; re-reading the frozen window returned identical parsed monitor results. History `complete=false` denotes rolling, unsealed fetch windows and is retained as a limitation, not overwritten. [1,2]

## §3. Current account money, margin and inventory

Money marks are **04:48:16.004–04:48:16.464 SGT**, books approximately **04:48:14**, runtime assessment **17 Nov 04:48:27.022**, and health **17 Nov 04:48:54.768**. These are separately timed post-cutoff snapshots, not an atomic historical book at 04:45. Equity uses recorded balance plus broker net unrealised P&L. Margin comes from the owner assessment; the money mark retains unverified aggregate margin fields. [5]


| Account | Balance USD | Floating USD | Equity USD | Assessed margin % | Pending projected % | Pending stressed % | S3 / S1 / S2 / external | Pending entries |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 27.39 | 2,134.84 | 2,162.23 | 123.55 | 88.53 | -17.17 | 0 / 1 / 0 / 5 | 3 |
| 46130058 | 25,617.19 | 109.99 | 25,727.18 | 7,562.11 | Unverified / N/A | Unverified / N/A | 1 / 6 / 0 / 0 | 0 |
| 47790949 | 25,487.79 | 110.30 | 25,598.09 | 940.71 | Unverified / N/A | Unverified / N/A | 1 / 6 / 0 / 0 | 0 |
| 49236051 | 821.21 | -0.72 | 820.49 | 94,309.20 | Unverified / N/A | Unverified / N/A | 0 / 1 / 0 / 0 | 0 |


Live current, pending and stressed assessments all fail the **250% new-entry floor**. The negative stress number is a hypothetical adverse projection, not current equity or a command to liquidate. Demos have no pending entries, so pending projections are not emitted; unavailable cells are not invented zeros. Passing a margin check does not bypass signal, peer, pair or human ownership gates. No live consumed trigger occurred in this interval, so current margin is not presented as the proven cause of a particular historical missed trade.


| Account / ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 58,403.20000 | Unverified / N/A |
| 43002148 / 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Unverified / N/A |
| 43002148 / 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,830.70000 | Unverified / N/A |
| 43002148 / 657180931 | NVDA.US | human_external / position | buy | 400 / 4.0000 | 160.82500 | 149.66000 | Unverified / N/A |
| 43002148 / 658574004 | XAUUSD | human_external / position | buy | 3000 / 0.3000 | 3,841.86000 | 3,830.00000 | Unverified / N/A |
| 43002148 / 658793840 | NatGas | human_external / position | buy | 500000 / 0.5000 | 2.99500 | 2.93200 | Unverified / N/A |
| 43002148 / 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 / 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 / 402061476 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | Unverified / N/A |
| 46130058 / 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,830.80000 | Unverified / N/A |
| 46130058 / 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.68880 | Unverified / N/A |
| 46130058 / 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.11939 | Unverified / N/A |
| 46130058 / 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 46130058 / 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56433 | Unverified / N/A |
| 46130058 / 263178401 | USDCAD | s1 / position | buy | 1100000 / 0.1100 | 1.43006 | 1.41137 | Unverified / N/A |
| 46130058 / 263304849 | XAUUSD | s3 / position | sell | 800 / 0.0800 | 3,852.27000 | 3,870.05000 | 3,827.73000 |
| 47790949 / 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,830.80000 | Unverified / N/A |
| 47790949 / 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.68875 | Unverified / N/A |
| 47790949 / 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.11934 | Unverified / N/A |
| 47790949 / 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 47790949 / 262976216 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55202 | 0.56428 | Unverified / N/A |
| 47790949 / 263178410 | USDCAD | s1 / position | buy | 1100000 / 0.1100 | 1.43006 | 1.41142 | Unverified / N/A |
| 47790949 / 263304831 | XAUUSD | s3 / position | sell | 800 / 0.0800 | 3,852.33000 | 3,870.09000 | 3,827.82000 |
| 49236051 / 262796763 | ETHUSD | s1 / position | sell | 1 / 0.0100 | 1,746.87000 | 2,027.27000 | Unverified / N/A |


## §4. Human protection, external cash and separate S1 status

**Zero recorded bot interference on Live 43002148:** no live program intent is recorded in the activity interval. Current/prior ownership and closing-deal IDs are joined before attribution; labels alone do not establish who initiated a close. NatGas and JPN225 remain reserved. Manual Gold remains external and locks live Gold plus its Silver pair. This is evidence about recorded bot commands, not a claim that external positions or stops stayed unchanged. [5]


| Ownership / position / deal | Symbol | Close SGT / price | Gross USD | Swap USD | Net USD |
| --- | --- | --- | --- | --- | --- |
| External / 658742237 / 348362823 | NatGas | 16 Nov 22:50:44.774 / 3.01900 | -190.00 | 0.00 | -190.00 |
| S1-owned; initiator unproven / 656513085 / 348427604 | GER40 | 17 Nov 03:33:43.427 / 26,547.70000 | -28.76 | -1.60 | -30.36 |

| Balance operation | Broker execution SGT | Amount USD | Recorded balance after USD |
| --- | --- | --- | --- |
| BALANCE_DEPOSIT | 17 Nov 03:31:57.597 | 150.00 | 7.75 |
| BALANCE_DEPOSIT | 17 Nov 03:36:07.082 | 50.00 | 27.39 |


The live balance bridge is exact: **$47.75 −$190.00 +$150.00 −$30.36 +$50.00 =$27.39**. The deposits are decoded from retained broker balance-operation messages, excluded from trading P&L. Full Monday external closing cash is **−$1,235.38**; Tuesday external closing cash is zero through 04:45. The GER40 close is separately S1-owned cash.

Manual **NatGas 658742237** closed at **22:50:44.774 SGT**, price 3.019, for −$190.00. A different current manual NatGas position **658793840** is in the book. The current live external inventory is **JPN225×2, NVDA.US, Gold 658574004, NatGas 658793840**. There are three external pending entries (JPN225, Cocoa, NatGas), **no resting Gold pending order**. The JPN225 649992251 stop changed **59,000.0→58,403.2** between snapshots without a recorded bot command; the report does not attribute that change to the bot.

Live S1 **GER40 656513085**, originally long 0.1 lot at 26,809.0, closed at **Tuesday 03:33:43.427 SGT /26,547.7**, net **−$30.36** including −$1.60 swap. Its inherited stop was 25,813.7, and no live S1 close intent appears. Report this as an **S1-owned close with unproven initiator**, not a confirmed algorithmic stop or manual action. Live S1 **AUS200 656795267** remains long 0.1 lot at 9,165.3 with SL 8,830.7.

The two large demos each recorded **one new S1 USDCAD long** and **91 successful S1 trail amendments**. They now have six S1 positions each, separate from their open S3 Gold short. Demo 49236051's S1 ETH short **262796763** remains open from Sunday 06:37:46.205 at 1,746.87; SL 2,027.27, native volume 1 /0.01 unit. Its account float is−$0.72 with only this position observed. S3 yields to that S1 ownership; the generic human-yield diagnostic does not mean a human opened the S1 trade.

## §5. Complete S3 ledger, excursions while held and exit deadline

The active-window cohort has **eight positions**: two carried BTC shorts, four new ETH shorts and two new Gold shorts. Six closed and two remain open. All eight are shorts. Original R is the distance from actual entry fill to initial SL; it is never reset after breakeven. Net R divides realised broker cash by original stop risk converted using actual volume and the recorded conversion rate. [1]


| Account / position | Symbol / side | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit classification | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 263124251 | BTCUSD / sell | 39 / 0.3900 | 16 Nov 19:08:40.945 / 64,611.13000 | 16 Nov 20:00:00.991 / 64,391.63000 | 65,009.53000 / 64,061.33000 | session end | 85.61 / 0.550986 |
| 47790949 / 263124254 | BTCUSD / sell | 38 / 0.3800 | 16 Nov 19:08:42.432 / 64,610.29000 | 16 Nov 20:00:00.991 / 64,391.63000 | 65,008.69000 / 64,060.49000 | session end | 83.09 / 0.548840 |
| 47790949 / 263149439 | ETHUSD / sell | 919 / 9.1900 | 16 Nov 20:34:38.336 / 1,792.66000 | 16 Nov 22:12:45.076 / 1,809.62000 | 1,809.57000 / 1,769.32000 | initial stop | -155.86 / -1.002941 |
| 46130058 / 263149441 | ETHUSD / sell | 923 / 9.2300 | 16 Nov 20:34:38.753 / 1,793.73000 | 16 Nov 22:13:29.426 / 1,808.81000 | 1,810.64000 / 1,770.39000 | initial stop | -139.19 / -0.891790 |
| 47790949 / 263224759 | ETHUSD / sell | 1439 / 14.3900 | 17 Nov 00:13:06.590 / 1,811.54000 | 17 Nov 00:17:34.838 / 1,822.23000 | 1,822.28000 / 1,796.71000 | initial stop | -153.83 / -0.995350 |
| 46130058 / 263224768 | ETHUSD / sell | 1447 / 14.4700 | 17 Nov 00:13:08.185 / 1,811.98000 | 17 Nov 00:18:06.767 / 1,823.19000 | 1,822.72000 / 1,797.15000 | initial stop | -162.21 / -1.043770 |
| 47790949 / 263304831 | XAUUSD / sell | 800 / 0.0800 | 17 Nov 02:49:27.938 / 3,852.33000 | Open / unavailable / Unverified / N/A | 3,870.09000 / 3,827.82000 | OPEN at cutoff | Unverified / N/A / Unverified / N/A |
| 46130058 / 263304849 | XAUUSD / sell | 800 / 0.0800 | 17 Nov 02:49:28.612 / 3,852.27000 | Open / unavailable / Unverified / N/A | 3,870.05000 / 3,827.73000 | OPEN at cutoff | Unverified / N/A / Unverified / N/A |

| Position | Observed held MFE / MAE R | Confirmed +1R BE amendment SGT | Registered exit deadline SGT |
| --- | --- | --- | --- |
| 263124251 | 0.622841 / 0.388680 | None observed | 16 Nov 20:00:00.000 |
| 263124254 | 0.620733 / 0.390788 | None observed | 16 Nov 20:00:00.000 |
| 263149439 | 0.117090 / 1.002957 | None observed | 17 Nov 00:00:00.000 |
| 263149441 | 0.180367 / 1.082791 | None observed | 17 Nov 00:00:00.000 |
| 263224759 | 0.100559 / 1.001862 | None observed | 17 Nov 04:00:00.000 |
| 263224768 | 0.141527 / 1.039106 | None observed | 17 Nov 04:00:00.000 |
| 263304831 | 0.006194 / 0.808559 | None observed | 17 Nov 05:57:00.000 |
| 263304849 | 0.001125 / 0.812711 | None observed | 17 Nov 05:57:00.000 |


**BTC:** carried 263124251 /263124254 closed at 20:00:00.991 for +$85.61 /+$83.09. Held MFE was about 0.623R /0.621R, below the unchanged 1.00R breakeven trigger. They were explicitly excluded as post-cutoff closes from №18,472 and are included here exactly once.

**ETH:** the first pair entered about 20:34:38 and stopped around 22:12–22:13; the second entered about 00:13:07 and stopped around 00:17–00:18. Held MFE was only 0.101–0.180R. None reached 1.00R, and no BE amendment is recorded. The combined realised loss is **−$611.09**. Two protective fills improved on their SL prices:263149441 by 1.83 and 263224759 by 0.05 price units. Their unchanged original stops, protective order type4 and observed pre-fill stop crossings support initial-stop classification; a favourable stop fill is not an unknown exit.

**Gold:** both new shorts entered around 02:49:28 and remain open at 04:45. Held MFE is near zero while observed MAE is about 0.81R. The account-level positive floating P&L includes S1 and must not be mistaken for Gold profit. The separately reconstructed cutoff Gold marks are:


| Position / account | Cutoff ask / quote SGT | Price-only gross USD / R | Original SL / TP | Registered deadline |
| --- | --- | --- | --- | --- |
| 263304831 / 47790949 | 3,853.24 / 17 Nov 04:44:59.834 | -7.2800 / -0.051239 | 3,870.09 / 3,827.82 | 17 Nov 05:57:00.000 |
| 263304849 / 46130058 | 3,853.27 / 17 Nov 04:44:59.862 | -8.0000 / -0.056243 | 3,870.05 / 3,827.73 | 17 Nov 05:57:00.000 |

| Position | Stored session close | Stored exit deadline | Difference seconds | Stored policy |
| --- | --- | --- | --- | --- |
| 263304831 | 17 Nov 05:59:00.000 | 17 Nov 05:57:00.000 | 120 | native_preclose_v18455_120s |
| 263304849 | 17 Nov 05:59:00.000 | 17 Nov 05:57:00.000 | 120 | native_preclose_v18455_120s |


The Gold marks exclude fees/swap and are not broker-certified S3 net unrealised P&L. Both original order receipts retain **Session.close 05:59 SGT** and the distinct **exit-only 05:57 deadline**, preserving sigma/entry-band timing. The deployed helper at `crates/jiaoyi-core/src/strategy3/production_sessions.rs` matches commit65a3ba7a byte-for-byte; its hash and code are in the JSON. **Policy assignment passes; future 05:57 execution is not yet observed.** [8]

## §6. Rolling latest20 and qualification


| Account | Net wins / count | Win rate | Realised payoff (R) | Profit factor (R) | Net R | Assessment SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | 3.4277 | 0.6049 | -5.9677 | 17 Nov 04:44:12.817 |
| 46130058 | 8/20 | 40.00% | 1.3456 | 0.8971 | -0.8914 | 17 Nov 04:40:20.037 |
| 47790949 | 9/20 | 45.00% | 1.3153 | 1.0761 | 0.6466 | 17 Nov 04:40:26.197 |
| 49236051 | 12/20 | 60.00% | 1.3690 | 2.0534 | 6.2047 | 17 Nov 04:42:30.099 |


All assessments precede 04:45. Relative to №18,472, wins change **9→8** on 46130058 and **11→9** on 47790949; 49236051 stays 12 and live stays 3. No account reaches **15/20**, the integer threshold corresponding to the 73% goal. Six current closes contain two net wins (33.33%), but paired account copies are correlated observations. The configured 1.38R gross target does not guarantee a1.38 realised payoff. **Financial pass=false.**

## §7. Complete four-hour post-close paths

All **six current closes and eight prior partials** now have complete **4.000h elapsed windows** at the frozen 04:45:30 SGT capture horizon. No partial row remains. Their42 elapsed1h/2h/4h checkpoints all have a quote within six seconds of the exact endpoint. [4]

Movement is signed in the original short direction from the **actual exit fill**, using the **ask** side and original R. Favourable/adverse columns are **price / broker pips / R**. Pip size is **0.1 for BTC/ETH/Gold**, **0.01 for Brent/Crude**, from recorded broker metadata. TP/SL tests use original absolute levels. A movement of 1.38R from a stop-out fill does not necessarily reach the original entry-based TP.



### Six current-window closes


| Position / symbol / exit | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 263124251 / BTCUSD / session end | 4.000 complete clock | 570.80000 / 5,708.00 / 1.432731 | 1,277.25000 / 12,772.50 / 3.205949 | 2,940.40 / 0.738052 | 2,656.50 / 0.666792 | 203.60 / 0.051104 | True / True |
| 263124254 / BTCUSD / session end | 4.000 complete clock | 570.80000 / 5,708.00 / 1.432731 | 1,277.25000 / 12,772.50 / 3.205949 | 2,940.40 / 0.738052 | 2,656.50 / 0.666792 | 203.60 / 0.051104 | True / True |
| 263149439 / ETHUSD / initial stop | 4.000 complete clock | 9.00000 / 90.00 / 0.532229 | 58.47000 / 584.70 / 3.457717 | -51.60 / -0.305145 | -26.10 / -0.154347 | -86.50 / -0.511532 | False / True |
| 263149441 / ETHUSD / initial stop | 4.000 complete clock | 8.19000 / 81.90 / 0.484329 | 59.28000 / 592.80 / 3.505618 | -88.30 / -0.522176 | -38.20 / -0.225902 | -76.10 / -0.450030 | False / True |
| 263224759 / ETHUSD / initial stop | 4.000 complete clock | 13.95000 / 139.50 / 1.298883 | 16.18000 / 161.80 / 1.506518 | -77.70 / -0.723464 | 47.10 / 0.438547 | 4.30 / 0.040037 | False / True |
| 263224768 / ETHUSD / initial stop | 4.000 complete clock | 14.91000 / 149.10 / 1.388268 | 15.22000 / 152.20 / 1.417132 | -87.90 / -0.818436 | 46.30 / 0.431099 | 40.80 / 0.379888 | False / True |


### Eight completed №18,472 follow-ups — no new closing cash


| Position / symbol / exit | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 263049563 / SpotBrent / initial stop | 4.000 complete clock | 1.97200 / 197.20 / 3.441536 | 0.00500 / 0.50 / 0.008726 | 170.50 / 2.975567 | 113.10 / 1.973822 | 190.10 / 3.317627 | True / True |
| 263049562 / SpotBrent / initial stop | 4.000 complete clock | 1.96700 / 196.70 / 3.432810 | 0.01000 / 1.00 / 0.017452 | 169.00 / 2.949389 | 112.60 / 1.965096 | 189.60 / 3.308901 | True / True |
| 263049566 / SpotBrent / initial stop | 4.000 complete clock | 1.97200 / 197.20 / 3.441536 | 0.00500 / 0.50 / 0.008726 | 170.50 / 2.975567 | 113.10 / 1.973822 | 190.10 / 3.317627 | True / True |
| 263068868 / ETHUSD / protective TP | 4.000 complete clock | 5.68000 / 56.80 / 0.398317 | 25.35000 / 253.50 / 1.777700 | -99.00 / -0.694250 | -183.20 / -1.284712 | -84.50 / -0.592567 | True / False |
| 263068905 / ETHUSD / BE stop with slippage | 4.000 complete clock | 25.40000 / 254.00 / 1.781206 | 17.38000 / 173.80 / 1.218794 | 18.00 / 0.126227 | 174.10 / 1.220898 | -173.80 / -1.218794 | True / True |
| 263074670 / SpotCrude / protective TP | 4.000 complete clock | 0.63000 / 63.00 / 1.147541 | 0.82400 / 82.40 / 1.500911 | -61.70 / -1.123862 | -20.20 / -0.367942 | 31.80 / 0.579235 | True / False |
| 263074682 / SpotCrude / protective TP | 4.000 complete clock | 0.59700 / 59.70 / 1.087432 | 0.85700 / 85.70 / 1.561020 | -44.70 / -0.814208 | -33.00 / -0.601093 | 21.00 / 0.382514 | True / False |
| 263074686 / SpotCrude / protective TP | 4.000 complete clock | 0.61600 / 61.60 / 1.128205 | 0.83800 / 83.80 / 1.534799 | -49.30 / -0.902930 | -35.30 / -0.646520 | 18.70 / 0.342491 | True / False |


**Current BTC time exits:** both original targets were observed around **20:10:54 SGT**, roughly11 minutes after the 20:00 close; both original stops were crossed around **22:33:29**. Their four-hour extrema were **570.80 favourable /1,277.25 adverse price units** from the exit (1.4327R /3.2059R). Target-before-stop order is observed in recorded quotes, but an extended hold would be a changed policy requiring separate evaluation, not a realised profit in this audit.

**Current ETH stops:** none of the four reached original TP during the subsequent four hours. All crossed original SL again. First-pair adverse excursions were **58.47 /59.28 price units** (3.458R /3.506R), versus only 9.00 /8.19 favourable. The second pair later recovered13.95 /14.91 from its exits, but still failed original TP and first moved adversely16.18 /15.22. The actual stops limited the observed loss on these paths.

**Earlier Brent initial stops:** all three retained their later target crossing around 17:04:38–17:04:50 and finished with favourable extrema1.967–1.972 (about 3.433–3.442R from the exit). These are three copies of one setup. **Earlier ETH BE stop 263068905** now shows its original target crossed at **20:28:33.611 SGT**, after the previous audit's 20:20 quote horizon; its original SL was crossed later at 22:14:52.226. Its four-hour favourable/adverse excursions are25.40 /17.38 price units. The previously unresolved endpoint is now observed, not retroactively included in earlier cash.

The earlier ETH TP exit and three Crude TP exits also have complete paths and further favourable movement. Counts are **three initial-stop paths plus one BE-stop path reaching original TP**. The JSON's `stopped_trades_reaching_original_tp=4` includes the one BE stop; the separate BE count1 is a subset, not another trade. Ten of 14 paths crossed original SL after their actual exit.

Clock completeness does not imply continuously observed prices. The maximum recorded inter-update gap is **25.094 seconds** (Brent); JSON retains gap counts, endpoint ages, exact extrema and crossing timestamps. No interpolation, invented fills or profit adjustments fill those gaps. Prior fills, stops, cash and in-position excursion records are preserved byte-for-value from the frozen earlier ledger; only post-close paths are extended.

## §8. Veto funnel and telemetry integrity


| Account | New arms | Consumed / intents / outcomes | Filled entries | Terminal refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 3 | 0 / 0 / 0 | 0 | 0 | 4 |
| 46130058 | 9 | 7 / 7 / 7 | 3 | 4 | 1 |
| 47790949 | 9 | 7 / 7 / 7 | 3 | 4 | 1 |
| 49236051 | 3 | 0 / 0 / 0 | 0 | 0 | 3 |

| Terminal refusal | Count |
| --- | --- |
| ENTRY_PULLBACK_BOUND_EXCEEDED: executable price exceeds the funded structural bound | 2 |
| ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction | 6 |


The14 consumed triggers each join to exactly one terminal outcome and a persisted opening intent whose creation precedes that outcome. Six were filled; eight were refused. **Missing individual outcomes=0; unknown entry-guard receipts=0.** Nine >61.8% retracement invalidations and 68 pair-reservation diagnostic records are retained. The six peer-hold refusals and two funded-bound refusals refer to unique consumed outcomes; repeated stage diagnostics must not be added as separate rejected trades. [6]

Maximum confirmation-to-final-guard latency is **23.045ms**; it is separate from broker send-to-fill latency or touch/turn waiting. Canonical monitoring retains `refusals_all_stages_complete=false`; verified consumed-trigger coverage does not imply an exhaustive count of every deduplicated idle/pre-arm veto. Current separately timed symbol states are:


| Account | Symbol | Current gate / reason |
| --- | --- | --- |
| 43002148 | BTCUSD | Waiting: opening impulse below 48 bps |
| 43002148 | ETHUSD | Invalidated: retraced > 61.8% without turn confirmation |
| 43002148 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.247%, 20d: -5.928%) |
| 43002148 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.138%, 20d: -9.902%) |
| 43002148 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.180%, 20d: -9.509%) |
| 43002148 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.352%, 20d: -2.055%) |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Waiting: opening impulse below 48 bps |
| 46130058 | ETHUSD | Waiting: current decision mark already consumed |
| 46130058 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.247%, 20d: -5.922%) |
| 46130058 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.138%, 20d: -9.902%) |
| 46130058 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.179%, 20d: -9.503%) |
| 46130058 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.349%, 20d: -2.055%) |
| 46130058 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 46130058 | XAUUSD | Managing existing position or pending entry; no additional entry |
| 47790949 | BTCUSD | Waiting: opening impulse below 48 bps |
| 47790949 | ETHUSD | Waiting: current decision mark already consumed |
| 47790949 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.247%, 20d: -5.922%) |
| 47790949 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.138%, 20d: -9.902%) |
| 47790949 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.179%, 20d: -9.503%) |
| 47790949 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.349%, 20d: -2.055%) |
| 47790949 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 47790949 | XAUUSD | Managing existing position or pending entry; no additional entry |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.247%, 20d: -5.922%) |
| 49236051 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.138%, 20d: -9.902%) |
| 49236051 | SpotCrude | Vetoed: 5d/20d macro drift opposes impulse (5d: 2.179%, 20d: -9.503%) |
| 49236051 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.349%, 20d: -2.055%) |
| 49236051 | XAGUSD | Invalidated: retraced > 61.8% without turn confirmation |
| 49236051 | XAUUSD | Invalidated: retraced > 61.8% without turn confirmation |


## §9. Opening references and Tuesday readiness

All48 scheduled slots have timely stored source quotes. Operational evidence is **36 exact full quote receipts, 4 recovered original midpoints, 8 ownership-withheld slots**. The latter comprise six small-demo crypto slots (BTC/ETH at three windows, due to its S1 ETH pair occupancy) plus the two live NY metals slots. Stored quotes alone do not prove operational capture or permission to trade. [7]


| Opening SGT | Slots / recorded source quotes | Exact full reference | Recovered midpoint only | Ownership withheld |
| --- | --- | --- | --- | --- |
| 16 Nov 20:00:00.000 | 8 | 5 | 1 | 2 |
| 16 Nov 22:30:00.000 | 24 | 22 | 0 | 2 |
| 17 Nov 00:00:00.000 | 8 | 3 | 3 | 2 |
| 17 Nov 04:00:00.000 | 8 | 6 | 0 | 2 |


**Live BTC 20:00 nuance:** the session receipt at **20:00:05.487 SGT** records `opening_midpoint_unavailable`, although the first two-sided stored quote arrived **20:00:00.361**, bid 64,372.53  / ask 64,387.53  / mid 64,380.03. A later **22:32:28.894** assessment uses exactly 64,380.03 as the 20:00 opening reference and vetoes the up-impulse on5d/20d macro trend. This proves midpoint recovery by that assessment, not full immutable quote persistence or the exact recovery time. An early recorder/reader visibility issue is a hypothesis; its mechanism is not established by this audit. At 00:00, three other BTC slots also have midpoint-only recovery evidence. No later mid-session quote was substituted in these proofs.


| Account | Symbol | Opening SGT | First quote SGT | Delay seconds | Verification |
| --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.360 | 0.360954 | recovered_midpoint_only |
| 43002148 | BTCUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.096 | 0.096619 | recovered_midpoint_only |
| 43002148 | BTCUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.432 | 0.432612 | exact_quote_receipt |
| 43002148 | ETHUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.143 | 0.143099 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.425 | 0.425795 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.162 | 0.162494 | exact_quote_receipt |
| 49236051 | BTCUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.351 | 0.351905 | ownership_locked |
| 49236051 | BTCUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.105 | 0.105763 | ownership_locked |
| 49236051 | BTCUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.469 | 0.469326 | ownership_locked |
| 49236051 | ETHUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.137 | 0.137330 | ownership_locked |
| 49236051 | ETHUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.413 | 0.413915 | ownership_locked |
| 49236051 | ETHUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.157 | 0.157012 | ownership_locked |
| 47790949 | BTCUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.417 | 0.417181 | exact_quote_receipt |
| 47790949 | BTCUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.119 | 0.119781 | recovered_midpoint_only |
| 47790949 | BTCUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.442 | 0.442847 | exact_quote_receipt |
| 46130058 | ETHUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.141 | 0.141900 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.431 | 0.431476 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.137 | 0.137741 | exact_quote_receipt |
| 47790949 | ETHUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.119 | 0.119632 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.392 | 0.392463 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.194 | 0.194722 | exact_quote_receipt |
| 46130058 | BTCUSD | 16 Nov 20:00:00.000 | 16 Nov 20:00:00.387 | 0.387394 | exact_quote_receipt |
| 46130058 | BTCUSD | 17 Nov 00:00:00.000 | 17 Nov 00:00:00.100 | 0.100740 | recovered_midpoint_only |
| 46130058 | BTCUSD | 17 Nov 04:00:00.000 | 17 Nov 04:00:00.452 | 0.452195 | exact_quote_receipt |
| 43002148 | NAS100 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.109 | 0.109518 | exact_quote_receipt |
| 49236051 | SpotBrent | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.001 | 0.001149 | exact_quote_receipt |
| 43002148 | XAUUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.109 | 0.109518 | ownership_locked |
| 43002148 | XAGUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.222 | 0.222874 | ownership_locked |
| 43002148 | US500 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.222 | 0.222873 | exact_quote_receipt |
| 49236051 | NAS100 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.134 | 0.134819 | exact_quote_receipt |
| 49236051 | US500 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.242 | 0.242911 | exact_quote_receipt |
| 49236051 | XAGUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.001 | 0.001147 | exact_quote_receipt |
| 49236051 | SpotCrude | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.001 | 0.001150 | exact_quote_receipt |
| 43002148 | SpotCrude | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.058 | 0.058437 | exact_quote_receipt |
| 43002148 | SpotBrent | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.438 | 0.438346 | exact_quote_receipt |
| 46130058 | NAS100 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.124 | 0.124003 | exact_quote_receipt |
| 49236051 | XAUUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.134 | 0.134811 | exact_quote_receipt |
| 46130058 | US500 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.234 | 0.234143 | exact_quote_receipt |
| 47790949 | XAUUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.108 | 0.108235 | exact_quote_receipt |
| 46130058 | XAUUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.123 | 0.123997 | exact_quote_receipt |
| 46130058 | SpotBrent | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.068 | 0.068814 | exact_quote_receipt |
| 47790949 | XAGUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.006 | 0.006776 | exact_quote_receipt |
| 47790949 | US500 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.006 | 0.006774 | exact_quote_receipt |
| 47790949 | SpotCrude | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.051 | 0.051601 | exact_quote_receipt |
| 46130058 | SpotCrude | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.068 | 0.068819 | exact_quote_receipt |
| 47790949 | NAS100 | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.185 | 0.185042 | exact_quote_receipt |
| 47790949 | SpotBrent | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.051 | 0.051598 | exact_quote_receipt |
| 46130058 | XAGUSD | 16 Nov 22:30:00.000 | 16 Nov 22:30:00.234 | 0.234147 | exact_quote_receipt |


All preflight rows below were recorded before 04:45. **Future Asian 07:00:20/20** slots (five traditional symbols×four accounts), **Crypto 08:00:8/8**, **Brent 09:00:4/4** have both base inputs and complete marks. Together Asia+Brent cover24 traditional account-symbol slots. This verifies preparation, not future executable quotes, signals or entry approval. Current and final post-merge health must independently confirm four connected/unpaused accounts.


| Account | Target open SGT | Slots | Base ready | Full marks ready | Latest preparation receipt SGT |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:23.026 |
| 46130058 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:23.884 |
| 47790949 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:23.779 |
| 49236051 | 16 Nov 20:00:00.000 | 2 | 2 | 2 | 16 Nov 16:00:26.006 |
| 43002148 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:35.664 |
| 46130058 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:30.605 |
| 47790949 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:30.383 |
| 49236051 | 16 Nov 22:30:00.000 | 6 | 6 | 6 | 16 Nov 15:00:32.941 |
| 43002148 | 17 Nov 00:00:00.000 | 2 | 2 | 2 | 16 Nov 20:00:22.725 |
| 46130058 | 17 Nov 00:00:00.000 | 2 | 2 | 2 | 16 Nov 20:00:22.045 |
| 47790949 | 17 Nov 00:00:00.000 | 2 | 2 | 2 | 16 Nov 20:00:21.975 |
| 49236051 | 17 Nov 00:00:00.000 | 2 | 2 | 2 | 16 Nov 20:00:24.982 |
| 43002148 | 17 Nov 04:00:00.000 | 2 | 2 | 2 | 17 Nov 00:00:21.671 |
| 46130058 | 17 Nov 04:00:00.000 | 2 | 2 | 2 | 17 Nov 00:00:23.392 |
| 47790949 | 17 Nov 04:00:00.000 | 2 | 2 | 2 | 17 Nov 00:00:23.407 |
| 49236051 | 17 Nov 04:00:00.000 | 2 | 2 | 2 | 17 Nov 00:00:25.916 |
| 43002148 | 17 Nov 07:00:00.000 | 5 | 5 | 5 | 16 Nov 22:30:30.110 |
| 46130058 | 17 Nov 07:00:00.000 | 5 | 5 | 5 | 16 Nov 22:30:31.586 |
| 47790949 | 17 Nov 07:00:00.000 | 5 | 5 | 5 | 16 Nov 22:30:30.551 |
| 49236051 | 17 Nov 07:00:00.000 | 5 | 5 | 5 | 16 Nov 22:30:32.982 |
| 43002148 | 17 Nov 08:00:00.000 | 2 | 2 | 2 | 17 Nov 04:00:21.426 |
| 46130058 | 17 Nov 08:00:00.000 | 2 | 2 | 2 | 17 Nov 04:00:22.002 |
| 47790949 | 17 Nov 08:00:00.000 | 2 | 2 | 2 | 17 Nov 04:00:21.900 |
| 49236051 | 17 Nov 08:00:00.000 | 2 | 2 | 2 | 17 Nov 04:00:24.825 |
| 43002148 | 17 Nov 09:00:00.000 | 1 | 1 | 1 | 16 Nov 22:30:28.060 |
| 46130058 | 17 Nov 09:00:00.000 | 1 | 1 | 1 | 16 Nov 22:30:29.410 |
| 47790949 | 17 Nov 09:00:00.000 | 1 | 1 | 1 | 16 Nov 22:30:28.493 |
| 49236051 | 17 Nov 09:00:00.000 | 1 | 1 | 1 | 16 Nov 22:30:30.940 |


## §10. Assessment and next audit

The overnight result is a loss because four ETH stop-outs outweighed two profitable BTC time exits. Those ETH paths support the protective value of the existing stop on this observed interval; they do not establish a general edge or justify changing entry/exit rules. Earlier Brent/ETH recoveries and BTC target-before-stop observations belong in separately frozen counterfactual research, with the actual risk path and account-copy correlation preserved.

The next activity audit should begin exactly **17 Nov 04:45 SGT**, carry the two Gold shorts, and verify their eventual broker exit against the stored 05:57 deadline. At this cutoff no new unfinished four-hour row remains. Check fresh 07:00 Asian, 08:00 crypto and 09:00 Brent capture against the preparation evidence, and keep live margin, external cash and S1 events separate. A focused engineering investigation could examine the early opening-refusal/recovery discrepancy; this read-only audit does not implement a fix.

The 0.6% entry risk, 3.0% open-risk cap, 250% margin floor, 1.00R BE, 1.38R TP, pair limits and Three Human Protection Laws were not changed. No trading action, deployment or broker-data collection request was made for this report.

## §11. Evidence, verification and Gemini hand-off

**Sources:** [1] stored PostgreSQL deals, orders/fills, intents, protobuf protection events and broker contract metadata; [2] independently bounded overnight/full-Monday/Tuesday/cumulative monitor queries and successful history fetch evidence; [3] merged №18,472 and exact 19:45 continuity; [4] **24 existing-tick exports /565,561 raw rows**, zero new broker requests; [5] separately timed owner book/margin/health, money marks and live external/S1/balance-operation receipts; [6] consumed-ID→outcome→causal-intent joins and pre-cutoff latest20 assessments; [7] first quote streams, arm/order/session references, recovered-midpoint assessments and causal preparation receipts; [8] original Gold order deadlines plus deployed-commit exit-helper code/hash. Source text was treated as evidence, not instructions.

Seven focused checks passed: half-open cash/continuity, quote hashes/ranges/coverage, pip/R arithmetic and unchanged follow-up history, consumed-trigger/carried-in accounting, external/S1 cash and ownership, opening/preparation denominators, and calendar-day partition with stored120-second deadlines. Measured JSON includes ledgers, exact cash receipts, query/calculation/test sources, hashes and timing. Desktop/mobile rendering and secret checks precede publication. No raw authentication material or private external deposit note is published.

Only the three requested `doc/` files are committed. Required CI: **check, docker, engineering**. Railway watch patterns exclude `doc/`; initial Trading deployment **3d8b18d5-4d9f-4007-8fff-22eb25d5ea73**, image **sha256:42609036a809db4674fdc007804d6a73ce15369fbf1e6f356981ba47ddd86d6c**, commit **65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2**. Post-merge deployment/session continuity is checked separately and returned with the actual PR/merge SHA. These artifacts describe pre-merge evidence; they do not claim CI success before it occurs.

**Gemini review instruction:** read this Markdown and matching JSON together. Check **−$442.39 overnight,−$23.68 full Monday,−$316.04 Tuesday,+$2,418.53 cumulative** S3 cash; separate−$190 external NatGas,−$30.36 S1-owned GER40,+$200 deposits; two open Gold shorts with 05:57 deadlines; all 14 complete post-close clocks; four overnight ETH stops failing to reach TP; three earlier Brent stops plus one BE stop later reaching TP; BTC target-before-stop ordering; opening proof tiers and recovered live BTC reference; latest20 below 15/20. Do not treat exit-anchored excursions as entry-anchored profits or four correlated account paths as four independent hypotheses. Keep the future 05:57 exit unverified until it actually occurs.


| Quote export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-BTCUSD-2026-11-16-110834.csv.gz | 60433 | 3e7f7e80aef68a518c4f5bb0b126537aadea2e7aa8e163291ee18130a4af4a98 |
| 46130058-BTCUSD-2026-11-16-150834.csv.gz | 13854 | 831092dbad6ea4f9e872009718cc7e4eaedd94f5800157fc3fab9309931897cd |
| 46130058-ETHUSD-2026-11-16-102731.csv.gz | 50165 | eabce9ca9ac24640bb05f43fcd969f73640360701b7b5a161213b77ecacf4f46 |
| 46130058-ETHUSD-2026-11-16-142731.csv.gz | 56462 | 360874108c8fcafdcd62534254ce594d9449eed02e5236ca611cbb916412a889 |
| 46130058-ETHUSD-2026-11-16-182731.csv.gz | 23386 | b4ff09811e14eb873637c839c5eb1343039a7cee388ada99e414f69b93212244 |
| 46130058-SpotBrent-2026-11-16-083052.csv.gz | 14684 | 502b8150e48a62b98a0117d295020439b16c91922ef40b8a213c2d02874de018 |
| 46130058-SpotBrent-2026-11-16-123052.csv.gz | 6 | 8cfb99925bcf62a1bbee528ecbfb5bf84fd8534cb8f96b681ab462d8254851a8 |
| 46130058-SpotCrude-2026-11-16-093121.csv.gz | 18523 | 0cb7196f66fd01093166767e989e86efe7f84c1fbebe1a41743cc5b7504c6e44 |
| 46130058-SpotCrude-2026-11-16-133121.csv.gz | 7 | 48f9690a309cf9e1373ef7a6db5b05eee183ae820774e63bf4752102a6c78e33 |
| 46130058-XAUUSD-2026-11-16-184922.csv.gz | 21378 | 0493632ec7de27f04afbff706d61735e241004324065447701c7823db211256a |
| 47790949-BTCUSD-2026-11-16-110836.csv.gz | 60432 | 97d587462796260d843ccc63a1818820e8cc0175380265d81499085ffd1b8d59 |
| 47790949-BTCUSD-2026-11-16-150836.csv.gz | 13848 | dc0d0cb763c36522b484208c07af82277ae8ee02228d41237aa8912ab8b6aa5c |
| 47790949-ETHUSD-2026-11-16-091458.csv.gz | 49798 | 9dbe9754cbaba0db0907e03fb21774798e3cb56921305b15c37f69620fb935af |
| 47790949-ETHUSD-2026-11-16-131458.csv.gz | 55682 | 80591866a5508848585a038804cda5b47744debbdae52ff816d4f233c0b2a043 |
| 47790949-ETHUSD-2026-11-16-171458.csv.gz | 39010 | 0333d7bd1b473e80aaf054eada954d72d47bb778ecf333d9e8526ca19405c570 |
| 47790949-SpotBrent-2026-11-16-083051.csv.gz | 14684 | 498f5f32964f481700e7db3525535ac7b68ac99bb8725858d826d519c7714366 |
| 47790949-SpotBrent-2026-11-16-123051.csv.gz | 7 | 2769563810d8baaacdfa4bd58ac1daf9b1c3814c598ee8e152847d354781e956 |
| 47790949-SpotCrude-2026-11-16-093101.csv.gz | 18529 | ed11594e1c5c51d2e8222242dd7ea232b0f2315a47c4d5f2617c1f99cc721cd6 |
| 47790949-SpotCrude-2026-11-16-133101.csv.gz | 5 | 76f2e891fbabd730d7a876b65430d213fadf8f9dcaa37181f51470ee5184d015 |
| 47790949-XAUUSD-2026-11-16-184921.csv.gz | 21381 | 9abc76eb4585673f92109dae99c89b30b6161b55b315e5287ccc1a63ff054602 |
| 49236051-SpotBrent-2026-11-16-083052.csv.gz | 14684 | ef7e89fd599780345bf91a55aef541c3acc726c6f5967f86ee32b3277e7c637b |
| 49236051-SpotBrent-2026-11-16-123052.csv.gz | 6 | e825cf1b555bd3e2c758dfe80bd2d6256b2ebfe639e68f602cf05f0541131b66 |
| 49236051-SpotCrude-2026-11-16-092628.csv.gz | 18590 | 99b1e88704faa75a8f1eae7ea7d3af864d58737a0c98d2438f3c44d96028ea6b |
| 49236051-SpotCrude-2026-11-16-132628.csv.gz | 7 | e34a894d6fcd7a4dd52a31ee33dcd3f9fd387516ae2eafb3f072bee9320c9103 |

| Source | SHA256 |
| --- | --- |
| tuesday.json | e6da7164021f2b6abc8861b7ef0a7235dcb9e70fdff012915753e17c1e151871 |
| opening-gap.json | 3cc2ae7561f56c036bf331d82a729f74aa6b0c53ade9dadb39f6eed297f884b5 |
| opening-ticks.json | 310194b3474c85338a15117ed94c35a434704e5e86b5a6446245b7ef46dfcba9 |
| preflight.json | 86c6af5255ab2fcc1ec1373060ebfba9c48a2f9ac0fdae6461350be41f8a5227 |
| monitor.json | 2be15fc3a89500d0fb6ce9a5316c78acc64d0e0c1e5259708e6354fcabedf603 |
| cumulative.json | ae8d36ab19f59223b552f3c6328b83f370d09daddeb5331ab769131b853ad100 |
| human.json | 46543f92bfe4201e35dd616af094e6470c61b88066b8025f9c117803482a1514 |
| health.json | 6aa16989dce1cf3bd253cfd2455b069e1f620c78718e89d71e6b1e764005b36b |
| supplement.json | 2746c52372e93678a16fb25e9e6b413cbfcc0d7dc98ee6a50d6366442e2c53be |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| activity.json | 9d72c88c3b1ead3ec9c7ee6e965ead5475c34dad93f33ea6292d4d510c8515df |
| evidence.json | d54580a22e2e2c3398f992621fcaa71e2521b5137208bfec21fcfa7d06d76062 |
| day.json | f7bf875f94115c1e63d0b00558ceb2309321c3640ec780906381ba4168f9eeb6 |
| opening-recovery.json | 6651d9776cf3e545fc97e877e38642ec54562d34e6b07180a8edb51bb8c02e6f |
| monitor-reconciled.json | c79dc9f5333f9b7d62565c49283d4cc11f152b92b1196bf4266f0269efd96bb1 |
| runtime.json | ea09628637f5440cd984d70a2b1a8c262a13e305a32002ce3a7533284bbcb1ca |
| history-proof.json | 74488f6420f463bb479b2246a024913d54a4ee97070cc821befc4e0a399c77d6 |
