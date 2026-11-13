# Saturday end-of-week S3 audit, post-close excursions and weekend posture — №18,452

**Cutoff: 14 November 2026, 05:35 SGT. Activity: [13 Nov 13:55, 14 Nov 05:35) SGT.** Quote paths are frozen through **14 Nov 05:35:31.000 SGT**; later weekend-close observations are separate. Read-only production audit.

## §1. TL;DR

- **Post-cutoff weekend exit failure:** both demo gold positions remained open after their 05:55 deadline. Four `MARKET_CLOSED` rejections were recorded by 05:55:21 SGT. These later failed attempts add no closing cash to the frozen 05:35 totals; see §9.

- **Window S3 closed cash: +$636.93**, from **19 closes: nine TP exits, seven initial stops and three BE-stop exits**. Eleven have positive net cash and eight lose. Two demo XAUUSD shorts remain open at the cutoff, with registered 05:55 SGT exit deadlines. [1]
- **Full Friday: +$1,408.69; Saturday 00:00–05:35: −$319.56.** Cumulative post-10 Nov09:42 closing cash is **+$2,643.24**, including **+$412.88 from three earlier-entry positions**. These figures are different periods and must not be added together. [1,2]
- Continuity is verified from **№18,448 / PR #227**, since №18,450 is absent from merged main and №18,451 was not executed. The three **13:56:06 BTC TP closes, +$471.64**, are counted exactly once here. Six prior ETH follow-ups are completed without recounting their cash. [3]
- **25 post-close paths: 19 complete clocks, six partial.** This includes 19 new-window closes plus six prior ETH closes. All six earlier ETH BE exits touched TP; the second group subsequently also crossed its original SL. None of this window's seven initial-stop or three BE exits had touched original TP within its observed follow-up. [4]
- **All four accounts connected and unpaused; zero recorded human-position/order interference; no live S3 fill or close.** Live equity is $2,980.78 versus $4,152.97 at the earlier midday money mark. The $1,172.19 change is in combined floating P&L; live balance remains −$906.87 and window S3 cash is zero. [1,5]
- **75 arms → 25 consumed triggers → 25 intents/outcomes → 18 fills and seven peer refusals.** Missing individual outcomes and unknown entry-guard receipts are both zero. No `MARKET_CLOSED` rejection is recorded inside the activity window. [6]
- **All 80 scheduled openings have recorded first-window two-sided quotes.** Exact full-quote operational evidence matches 72; four additional recovered midpoints are verified in later assessments, with incomplete durable recovery receipts; four live-metal slots are human-locked. Saturday 08:00 crypto preparation is **8/8 complete**. [7]
- **Financial pass remains false.** Latest20 net wins are live **3/20**, demos **10/20, 11/20, 8/20**. The required 15/20 is not met. Trading code, risk/exit rules and deployment are untouched.

## §2. Continuity, cash periods and account snapshot

Main was fetched before collection and resolved to `b98c37e2111390762db2d1c15efed5b8e27ea285`, the merged №18,448 report. The interval starts exactly at its exclusive 13:55 boundary. Broker closing-deal IDs are unique within account and the 13:56 BTC receipts reconcile to the prior report's separately labelled post-cutoff observations. The six earlier ETH positions appear only in the follow-up section, not in this interval's cash, closes or new-entry counts. [1–4]

The cash periods are:


| Account | Window closes | Window net USD | Full Friday net USD | Saturday to cutoff USD | Post-PR214 closing cash USD |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | 0.00 | -0.01 |
| 46130058 | 6 | 332.47 | 702.20 | -151.20 | 1,332.82 |
| 47790949 | 6 | 299.02 | 687.67 | -159.45 | 1,294.33 |
| 49236051 | 7 | 5.44 | 18.82 | -8.91 | 16.10 |


Full Friday **+$1,408.69** plus Saturday **−$319.56** equals **+$1,089.13** since Friday midnight. Prior №18,448 Friday-to-13:55 cash **+$452.20** plus this window **+$636.93** gives the same **+$1,089.13**. Post-PR214 cash is a separate, longer cumulative period. [1,2,3]

A later read-only reconciliation at **14 Nov05:49 SGT** found successful broker-history fetches through **05:48:51 SGT** for all four accounts, with no new or changed deals in those fetches. Re-reading the same frozen activity interval left every account’s cash, close count and entry metrics unchanged. The `complete=false` field denotes the rolling, unsealed history window in `refresh_recent_deals_with_lock`; all four fetch statuses are successful, with no error. These later fetches provide follow-up coverage past the cutoff; they do not change the monetary snapshot time. [1]

Money marks are **14 Nov05:34:52.980–05:34:53.533 SGT**. Live/small-demo books are **05:34:51**, while the other demo books are **05:35:24**. Health is observed at **14 Nov 05:35:57.310 SGT**. Quote marks, money and inventory are separately timed, not an atomic cutoff observation. Equity is recorded balance plus broker net unrealised P&L; no unverified credit or prospective closing fee is invented. Margin percentages below are owner-assessment projections, not a verified aggregate margin value in the monetary snapshot. [1,5]


| Account | Balance USD | Float USD | Equity USD | Assessed margin % | S3 / S1 / S2 / external positions | Pending | Book observed SGT |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | -906.87 | 3,887.65 | 2,980.78 | 282.61 | 0 / 2 / 0 / 5 | 5 | 14 Nov 05:34:51.250 |
| 46130058 | 25,854.42 | 126.37 | 25,980.79 | 7,996.18 | 1 / 5 / 0 / 0 | 0 | 14 Nov 05:35:24.346 |
| 47790949 | 25,485.61 | 117.49 | 25,603.10 | 1,152.04 | 1 / 4 / 0 / 0 | 0 | 14 Nov 05:35:24.382 |
| 49236051 | 810.87 | 0.00 | 810.87 | Not verifiable / N/A | 0 / 0 / 0 / 0 | 0 | 14 Nov 05:34:51.239 |


Live current assessed margin is **282.61%**; pending-entry projection is **137.83%**, and stress projection is **−41.68%**. The stress number is a hypothetical projection, not current negative equity or an observed margin call. Current live BTC/ETH admission cites **135.03%/137.23%**, below the unchanged 250% floor. These current guard reasons are not retrospectively assigned to every earlier non-entry.

Complete observed inventory, including all pending entries:


| Account | ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Not verifiable / N/A |
| 43002148 | 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Not verifiable / N/A |
| 43002148 | 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.97900 | 3.87500 |
| 43002148 | 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,813.70000 | Not verifiable / N/A |
| 43002148 | 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,830.70000 | Not verifiable / N/A |
| 43002148 | 657180931 | NVDA.US | human_external / position | buy | 200 / 2.0000 | 165.22000 | 149.66000 | Not verifiable / N/A |
| 43002148 | 657187868 | XAUUSD | human_external / position | buy | 2500 / 0.2500 | 3,869.03800 | 3,859.77000 | Not verifiable / N/A |
| 43002148 | 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Not verifiable / N/A |
| 43002148 | 401488805 | XAUUSD | human_external / pending_entry | buy | 1000 / 0.1000 | 3,813.63000 | 3,786.80000 | Not verifiable / N/A |
| 43002148 | 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 | 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 | 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Not verifiable / N/A |
| 46130058 | 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,830.80000 | Not verifiable / N/A |
| 46130058 | 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69287 | Not verifiable / N/A |
| 46130058 | 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12259 | Not verifiable / N/A |
| 46130058 | 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Not verifiable / N/A |
| 46130058 | 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56724 | Not verifiable / N/A |
| 46130058 | 262583980 | XAUUSD | s3 / position | sell | 1000 / 0.1000 | 3,878.12000 | 3,893.62000 | 3,856.73000 |
| 47790949 | 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,830.80000 | Not verifiable / N/A |
| 47790949 | 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69282 | Not verifiable / N/A |
| 47790949 | 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12254 | Not verifiable / N/A |
| 47790949 | 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Not verifiable / N/A |
| 47790949 | 262583136 | XAUUSD | s3 / position | sell | 900 / 0.0900 | 3,877.19000 | 3,892.65000 | 3,855.85000 |


## §3. Human protection and Strategy 1

The live inventory retains five external positions: JPN225 ×2, NatGas, NVDA.US and XAUUSD, plus five external pending entries. **Recorded bot interference=0; external closing cash=0.00** in this activity window. The retained intent/event journal was joined against both prior and current external IDs. This establishes absence of recorded bot commands against those IDs; it is not a claim about unidentified external actors. [1,5]

External NatGas SL changed **2.913 → 2.979**. External gold SL changed **3,847.97 → 3,859.77** and its TP is now absent in the observed book. These changes are not linked to bot intents. Human gold continues to lock live XAUUSD; its sibling XAGUSD is held by pair reservation. NatGas/JPN225 remain permanently reserved. The bot does not acquire authority because an external position has profit or a changed stop.

Live S1 GER40 **656513085** and AUS200 **656795267** remain open. No live S1 entry, close or trailing amendment is recorded within this interval. Their retained SLs are **25,813.7** and **8,830.7**, unchanged from №18,448 and below their respective long entries. The last recorded changes were 13 Nov07:00:02.389 and 06:50:03.075 SGT, outside this window. Full retained SL histories are in JSON. Demo 46130058 also has an S1 NZDUSD position; it is listed separately and excluded from S3 cash.

## §4. Complete S3 activity ledger and risk geometry

The cohort contains **three carried BTC positions plus 18 new entries = 21 positions**. Nineteen are closed by 05:35; two gold positions remain open. All are demo shorts with S3 ownership evidence. Initial R is actual entry-fill to original SL, with cash risk scaled by actual filled units and quote-to-deposit conversion. Changing the stop later does not reset R. [1]


| Account / position | Symbol / carryover | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit class | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 262420070 | BTCUSD / carried | 1 / 0.0100 | 13 Nov 13:11:35.743 / 68,599.56000 | 13 Nov 13:56:06.273 / 67,873.27000 | 69,073.20000 / 67,945.93000 | broker_tp | 7.26 / 1.532810 |
| 46130058 / 262420072 | BTCUSD / carried | 32 / 0.3200 | 13 Nov 13:11:36.087 / 68,637.64000 | 13 Nov 13:56:06.179 / 67,873.27000 | 69,111.28000 / 67,984.01000 | broker_tp | 244.60 / 1.613831 |
| 47790949 / 262420079 | BTCUSD / carried | 31 / 0.3100 | 13 Nov 13:11:38.146 / 68,582.25000 | 13 Nov 13:56:06.240 / 67,873.27000 | 69,055.89000 / 67,928.62000 | broker_tp | 219.78 / 1.496849 |
| 49236051 / 262471965 | SpotBrent | 700 / 0.0700 | 13 Nov 16:01:04.388 / 98.46200 | 13 Nov 18:32:05.216 / 97.61200 | 99.07600 / 97.61400 | broker_tp | 5.95 / 1.384365 |
| 46130058 / 262471984 | SpotBrent | 25200 / 2.5200 | 13 Nov 16:01:08.070 / 98.45200 | 13 Nov 18:42:04.393 / 97.60200 | 99.06600 / 97.60400 | broker_tp | 214.20 / 1.384365 |
| 47790949 / 262471983 | SpotBrent | 24800 / 2.4800 | 13 Nov 16:01:07.853 / 98.45200 | 13 Nov 18:42:04.416 / 97.60200 | 99.06600 / 97.60400 | broker_tp | 210.80 / 1.384365 |
| 49236051 / 262493029 | SpotCrude | 600 / 0.0600 | 13 Nov 18:49:52.880 / 88.55100 | 13 Nov 21:46:52.699 / 89.47600 | 89.31600 / 87.49500 | initial_stop_loss | -5.55 / -1.209150 |
| 47790949 / 262505400 | SpotCrude | 26000 / 2.6000 | 13 Nov 19:28:37.244 / 88.73400 | 13 Nov 21:46:52.696 / 89.47600 | 89.32600 / 87.91700 | initial_stop_loss | -192.92 / -1.253378 |
| 46130058 / 262505405 | SpotCrude | 26300 / 2.6300 | 13 Nov 19:28:38.735 / 88.73100 | 13 Nov 21:46:52.663 / 89.47600 | 89.32300 / 87.91400 | initial_stop_loss | -195.94 / -1.258478 |
| 49236051 / 262564786 | ETHUSD | 38 / 0.3800 | 13 Nov 22:51:52.669 / 1,949.35000 | 13 Nov 23:21:51.659 / 1,931.74000 | 1,962.09000 / 1,931.76000 | broker_tp | 6.69 / 1.381889 |
| 47790949 / 262568953 | BTCUSD | 34 / 0.3400 | 13 Nov 23:06:08.419 / 68,900.59000 | 13 Nov 23:31:01.738 / 68,251.14000 | 69,345.51000 / 68,286.60000 | broker_tp | 220.81 / 1.459681 |
| 46130058 / 262568952 | BTCUSD | 34 / 0.3400 | 13 Nov 23:06:08.432 / 68,900.59000 | 13 Nov 23:31:01.697 / 68,251.14000 | 69,345.51000 / 68,286.60000 | broker_tp | 220.81 / 1.459681 |
| 49236051 / 262579978 | SpotCrude | 800 / 0.0800 | 13 Nov 23:56:27.261 / 89.17300 | 14 Nov 02:37:43.101 / 89.14500 | 89.75100 / 88.37500 | breakeven_stop | 0.22 / 0.047578 |
| 46130058 / 262579991 | SpotCrude | 27000 / 2.7000 | 13 Nov 23:56:31.935 / 89.17100 | 14 Nov 02:37:43.128 / 89.14500 | 89.74900 / 88.37300 | breakeven_stop | 7.02 / 0.044983 |
| 47790949 / 262579998 | SpotCrude | 26700 / 2.6700 | 13 Nov 23:56:34.286 / 89.15100 | 14 Nov 02:33:17.625 / 89.16100 | 89.72900 / 88.35300 | breakeven_stop | -2.67 / -0.017301 |
| 49236051 / 262581322 | ETHUSD | 46 / 0.4600 | 14 Nov 00:02:58.183 / 1,926.61000 | 14 Nov 00:09:30.019 / 1,937.18000 | 1,937.08000 / 1,912.16000 | initial_stop_loss | -4.86 / -1.009094 |
| 47790949 / 262581332 | ETHUSD | 1468 / 14.6800 | 14 Nov 00:02:59.712 / 1,927.22000 | 14 Nov 01:37:41.403 / 1,937.90000 | 1,937.73000 / 1,912.71000 | initial_stop_loss | -156.78 / -1.016160 |
| 46130058 / 262582229 | BTCUSD | 43 / 0.4300 | 14 Nov 00:07:24.648 / 68,574.73000 | 14 Nov 01:41:03.902 / 68,942.69000 | 68,935.05000 / 68,077.48000 | initial_stop_loss | -158.22 / -1.021185 |
| 49236051 / 262586283 | BTCUSD | 1 / 0.0100 | 14 Nov 00:23:41.476 / 68,539.20000 | 14 Nov 01:41:05.128 / 68,965.80000 | 68,955.95000 / 67,964.08000 | initial_stop_loss | -4.27 / -1.024595 |
| 47790949 / 262583136 | XAUUSD | 900 / 0.0900 | 14 Nov 00:10:35.369 / 3,877.19000 | Open / unavailable / Not verifiable / N/A | 3,892.65000 / 3,855.85000 | open | Not verifiable / N/A / Not verifiable / N/A |
| 46130058 / 262583980 | XAUUSD | 1000 / 0.1000 | 14 Nov 00:13:41.377 / 3,878.12000 | Open / unavailable / Not verifiable / N/A | 3,893.62000 / 3,856.73000 | open | Not verifiable / N/A / Not verifiable / N/A |


Exit labels are evidence-derived: TP means a protective closing fill at or beyond the original target, BE means the last broker-confirmed stop matched entry, and initial-stop classification reconciles the fill with the retained original stop. A matching close intent identifies a session-end exit. This does not assume every broker receipt contains an explicit TP/SL reason field.

Observed in-position excursions use executable ask prices for these shorts. BE times below are broker-confirmed stop amendments, rather than merely the first price touch. Net cash wins are not synonymous with TP exits: two BE exits have small positive net cash and one loses.


| Position | MFE R | MAE R | +1R BE confirmation SGT | Registered exit deadline SGT | Last recorded SL |
| --- | --- | --- | --- | --- | --- |
| 262420070 | 1.654463 | 0.492822 | 13 Nov 13:55:56.681 | 13 Nov 16:00:00.000 | 68,599.56000 |
| 262420072 | 1.734862 | 0.412423 | 13 Nov 13:55:58.343 | 13 Nov 16:00:00.000 | 68,637.64000 |
| 262420079 | 1.617917 | 0.529368 | 13 Nov 13:55:57.872 | 13 Nov 16:00:00.000 | 68,582.25000 |
| 262471965 | 1.384365 | 0.842020 | 13 Nov 18:03:32.977 | 13 Nov 22:30:00.000 | 98.46200 |
| 262471984 | 1.400651 | 0.858306 | 13 Nov 18:04:25.367 | 13 Nov 22:30:00.000 | 98.45200 |
| 262471983 | 1.400651 | 0.858306 | 13 Nov 18:04:25.378 | 13 Nov 22:30:00.000 | 98.45200 |
| 262493029 | 0.190850 | 1.168627 | No amendment observed | 13 Nov 22:30:00.000 | 89.31600 |
| 262505400 | 0.408784 | 1.201014 | No amendment observed | 13 Nov 22:30:00.000 | 89.32600 |
| 262505405 | 0.403716 | 1.206081 | No amendment observed | 13 Nov 22:30:00.000 | 89.32300 |
| 262564786 | 1.383046 | 0.399529 | 13 Nov 23:09:39.484 | 14 Nov 00:00:00.000 | 1,949.35000 |
| 262568953 | 1.459701 | 0.412793 | 13 Nov 23:30:39.051 | 14 Nov 00:00:00.000 | 68,900.59000 |
| 262568952 | 1.459701 | 0.412793 | 13 Nov 23:30:38.823 | 14 Nov 00:00:00.000 | 68,900.59000 |
| 262579978 | 1.190311 | 0.615917 | 14 Nov 00:56:03.358 | 14 Nov 05:55:00.000 | 89.17300 |
| 262579991 | 1.186851 | 0.619377 | 14 Nov 00:56:05.988 | 14 Nov 05:55:00.000 | 89.17100 |
| 262579998 | 1.152249 | 0.653979 | 14 Nov 00:56:17.219 | 14 Nov 05:55:00.000 | 89.15100 |
| 262581322 | 0.306590 | 1.008596 | No amendment observed | 14 Nov 04:00:00.000 | 1,937.08000 |
| 262581332 | 0.960990 | 1.003806 | No amendment observed | 14 Nov 04:00:00.000 | 1,937.73000 |
| 262582229 | 0.767929 | 1.046736 | No amendment observed | 14 Nov 04:00:00.000 | 68,935.05000 |
| 262586283 | 0.187738 | 1.019652 | No amendment observed | 14 Nov 04:00:00.000 | 68,955.95000 |
| 262583136 | 0.500000 | 0.897154 | No amendment observed | 14 Nov 05:55:00.000 | 3,892.65000 |
| 262583980 | 0.556774 | 0.836774 | No amendment observed | 14 Nov 05:55:00.000 | 3,893.62000 |


The open gold positions **262583136 / 262583980** have deadlines **14 Nov05:55 SGT**, 20 minutes after the activity cutoff. Their fresh cutoff price-only floating projections are **+$28.71 / +$40.90**, respectively; these exclude swap/commission/future slippage and are not verified strategy-specific net unrealised broker P&L. Their actual closing results, if observed later, belong only to §9's post-cutoff note.

By instrument, window closed cash is:


| Symbol | Closes | Positive-net closes | Window net USD |
| --- | --- | --- | --- |
| BTCUSD | 7 | 5 | 750.77 |
| SpotBrent | 3 | 3 | 430.95 |
| SpotCrude | 6 | 2 | -389.84 |
| ETHUSD | 3 | 1 | -154.95 |


## §5. Latest20 assessment and limits of the result


| Account | Net wins / count | Win rate | Realised payoff (R) | Profit factor (R) | Net R | Goal assessment recorded SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | 3.4277 | 0.6049 | -5.9677 | 14 Nov 05:29:50.054 |
| 46130058 | 10/20 | 50.00% | 1.4782 | 1.4782 | 3.3267 | 14 Nov 05:32:55.638 |
| 47790949 | 11/20 | 55.00% | 1.5592 | 1.9057 | 5.5319 | 14 Nov 05:28:59.959 |
| 49236051 | 8/20 | 40.00% | 1.1579 | 0.8421 | -1.4013 | 14 Nov 05:31:06.153 |


The 73% target requires **15 positive-net closes out of 20 (75%)**, together with the payoff requirement. No account meets the win-count target. The small demo's latest20 net R is negative despite positive cumulative cash over the different post-PR214 period. Varying exposure, different intervals and different entry paths prevent those two metrics from being interchangeable. Live latest20 includes much older registrations; the demo copies are correlated and not independent experiments. [2,6]

## §6. Four-hour post-close paths, grouped by exit type

There are **25 follow-ups**: **19 current-window closes** (13 complete clocks, six partial) plus **six completed prior ETH follow-ups**. The combined denominator is **19 complete / six partial**, not 25 current-window closes. Money for the prior six is excluded from §2. [4]

Movement is signed in the original trade direction from the **actual exit price**. Favourable/adverse values below are positive magnitudes, each expressed as **price / broker pips / original R**. Broker pip sizes are derived from the account's `pip_position`: BTC/ETH 0.1, oil 0.01, gold 0.1. Original TP/SL touches use those absolute pre-trade levels, not a reset bracket at exit.



### initial_stop_loss — 7 current-window closes


| Account / position / symbol | Elapsed hours / coverage | Max favourable price / pips / R | Max adverse price / pips / R | 1h pips / R | 2h pips / R | 4h pips / R | Original TP / SL touched |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 262493029 / SpotCrude | 4.0000 complete clock | 0.99100 / 99.10 / 1.295425 | 0.44300 / 44.30 / 0.579085 | -13.90 / -0.181699 | 45.70 / 0.597386 | 55.90 / 0.730719 | False / True |
| 47790949 / 262505400 / SpotCrude | 4.0000 complete clock | 0.99100 / 99.10 / 1.673986 | 0.44300 / 44.30 / 0.748311 | -13.90 / -0.234797 | 45.70 / 0.771959 | 55.90 / 0.944257 | False / True |
| 46130058 / 262505405 / SpotCrude | 4.0000 complete clock | 0.99100 / 99.10 / 1.673986 | 0.44300 / 44.30 / 0.748311 | -13.90 / -0.234797 | 45.70 / 0.771959 | 55.90 / 0.944257 | False / True |
| 49236051 / 262581322 / ETHUSD | 4.0000 complete clock | 20.06000 / 200.60 / 1.915950 | 20.72000 / 207.20 / 1.978988 | 36.90 / 0.352436 | -8.40 / -0.080229 | -163.90 / -1.565425 | False / True |
| 47790949 / 262581332 / ETHUSD | 3.9638 PARTIAL | 12.10000 / 121.00 / 1.151284 | 20.00000 / 200.00 / 1.902950 | -52.60 / -0.500476 | -101.90 / -0.969553 | not_elapsed_at_capture | False / True |
| 46130058 / 262582229 / BTCUSD | 3.9075 PARTIAL | 776.16000 / 7,761.60 / 2.154085 | 171.15000 / 1,711.50 / 0.474994 | 1,644.00 / 0.456261 | -517.90 / -0.143733 | not_elapsed_at_capture | False / True |
| 49236051 / 262586283 / BTCUSD | 3.9072 PARTIAL | 799.27000 / 7,992.70 / 1.917864 | 148.04000 / 1,480.40 / 0.355225 | 1,854.40 / 0.444967 | -270.40 / -0.064883 | not_elapsed_at_capture | False / True |


### breakeven_stop — 3 current-window closes


| Account / position / symbol | Elapsed hours / coverage | Max favourable price / pips / R | Max adverse price / pips / R | 1h pips / R | 2h pips / R | 4h pips / R | Original TP / SL touched |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 262579978 / SpotCrude | 2.9633 PARTIAL | 0.46500 / 46.50 / 0.804498 | 0.32100 / 32.10 / 0.555363 | 19.10 / 0.330450 | 8.80 / 0.152249 | not_elapsed_at_capture | False / False |
| 46130058 / 262579991 / SpotCrude | 2.9633 PARTIAL | 0.46500 / 46.50 / 0.804498 | 0.32100 / 32.10 / 0.555363 | 19.10 / 0.330450 | 8.80 / 0.152249 | not_elapsed_at_capture | False / False |
| 47790949 / 262579998 / SpotCrude | 3.0370 PARTIAL | 0.48100 / 48.10 / 0.832180 | 0.30500 / 30.50 / 0.527682 | 32.40 / 0.560554 | 8.70 / 0.150519 | not_elapsed_at_capture | False / False |


### native_session_end — 0 current-window closes


No position in this exit class closed inside the activity window.


### broker_tp — 9 current-window closes


| Account / position / symbol | Elapsed hours / coverage | Max favourable price / pips / R | Max adverse price / pips / R | 1h pips / R | 2h pips / R | 4h pips / R | Original TP / SL touched |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 262420070 / BTCUSD | 4.0000 complete clock | 353.41000 / 3,534.10 / 0.746157 | 1,475.70000 / 14,757.00 / 3.115657 | -10,684.30 / -2.255785 | -7,315.10 / -1.544443 | -11,557.50 / -2.440144 | True / True |
| 46130058 / 262420072 / BTCUSD | 4.0000 complete clock | 353.41000 / 3,534.10 / 0.746157 | 1,475.70000 / 14,757.00 / 3.115657 | -10,714.80 / -2.262224 | -7,315.10 / -1.544443 | -11,559.40 / -2.440546 | True / True |
| 47790949 / 262420079 / BTCUSD | 4.0000 complete clock | 353.41000 / 3,534.10 / 0.746157 | 1,475.70000 / 14,757.00 / 3.115657 | -10,714.80 / -2.262224 | -7,315.10 / -1.544443 | -11,559.40 / -2.440546 | True / True |
| 49236051 / 262471965 / SpotBrent | 4.0000 complete clock | 0.03000 / 3.00 / 0.048860 | 1.89700 / 189.70 / 3.089577 | -84.40 / -1.374593 | -108.00 / -1.758958 | -176.10 / -2.868078 | True / True |
| 46130058 / 262471984 / SpotBrent | 4.0000 complete clock | 0.02000 / 2.00 / 0.032573 | 1.90700 / 190.70 / 3.105863 | stale_asof_quote (6.021s old) | stale_asof_quote (8.905s old) | -162.60 / -2.648208 | True / True |
| 47790949 / 262471983 / SpotBrent | 4.0000 complete clock | 0.02000 / 2.00 / 0.032573 | 1.90700 / 190.70 / 3.105863 | stale_asof_quote (6.003s old) | stale_asof_quote (8.898s old) | -163.40 / -2.661238 | True / True |
| 49236051 / 262564786 / ETHUSD | 4.0000 complete clock | 20.07000 / 200.70 / 1.575353 | 18.85000 / 188.50 / 1.479592 | 131.70 / 1.033752 | 43.20 / 0.339089 | -167.10 / -1.311617 | True / False |
| 47790949 / 262568953 / BTCUSD | 4.0000 complete clock | 233.49000 / 2,334.90 / 0.524791 | 848.86000 / 8,488.60 / 1.907894 | -4,130.70 / -0.928414 | -4,250.50 / -0.955340 | -7,148.10 / -1.606603 | True / False |
| 46130058 / 262568952 / BTCUSD | 4.0000 complete clock | 233.49000 / 2,334.90 / 0.524791 | 848.86000 / 8,488.60 / 1.907894 | -4,143.10 / -0.931201 | -4,249.70 / -0.955160 | -7,148.10 / -1.606603 | True / False |


### Completed prior ETH follow-ups — cash not recounted


| Position | Exit SGT | Coverage | Max favourable price / pips / R | Max adverse price / pips / R | 1h pips / R | 2h pips / R | 4h pips / R | First TP touch SGT | First original SL cross SGT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 262343996 | 13 Nov 10:22:30.166 | 4.000h complete clock | 113.32000 / 1,133.20 / 7.320413 | 0.81000 / 8.10 / 0.052326 | 528.90 / 3.416667 | 688.30 / 4.446382 | 765.30 / 4.943798 | 13 Nov 10:44:09.293 | Open / unavailable |
| 262343998 | 13 Nov 10:22:30.132 | 4.000h complete clock | 113.32000 / 1,133.20 / 7.320413 | 0.81000 / 8.10 / 0.052326 | 528.90 / 3.416667 | 688.30 / 4.446382 | 765.30 / 4.943798 | 13 Nov 10:44:09.294 | Open / unavailable |
| 262344000 | 13 Nov 10:22:30.190 | 4.000h complete clock | 113.32000 / 1,133.20 / 7.320413 | 0.81000 / 8.10 / 0.052326 | 528.90 / 3.416667 | 688.30 / 4.446382 | 765.30 / 4.943798 | 13 Nov 10:44:09.315 | Open / unavailable |
| 262411892 | 13 Nov 13:01:01.148 | 4.000h complete clock | 32.66000 / 326.60 / 3.220907 | 23.05000 / 230.50 / 2.273176 | 184.40 / 1.818540 | -165.90 / -1.636095 | -68.20 / -0.672584 | 13 Nov 13:05:20.444 | 13 Nov 14:24:41.081 |
| 262411893 | 13 Nov 13:01:01.145 | 4.000h complete clock | 32.66000 / 326.60 / 3.220907 | 23.05000 / 230.50 / 2.273176 | 184.40 / 1.818540 | -165.90 / -1.636095 | -68.20 / -0.672584 | 13 Nov 13:05:20.516 | 13 Nov 14:24:41.082 |
| 262411894 | 13 Nov 13:01:01.081 | 4.000h complete clock | 32.66000 / 326.60 / 3.220907 | 23.05000 / 230.50 / 2.273176 | 184.40 / 1.818540 | -165.80 / -1.635108 | -68.20 / -0.672584 | 13 Nov 13:05:20.514 | 13 Nov 14:24:41.082 |


The first ETH group, closed around **10:22 SGT**, touched TP at **10:44:09** without an original-SL crossing in its four-hour path. The second, closed around **13:01**, touched TP at **13:05:20**, then crossed original SL at **14:24:41**. The ordering matters: both levels being touched does not mean an actual TP-exited trade subsequently lost.

The three London Crude initial stops had only **+0.191R / +0.409R / +0.404R** observed MFE before stopping, followed by no TP touch over their complete four-hour follow-ups. A BE trigger at +0.80R would not have activated on those observed paths. For the later ETH stop on 47790949, MFE was **+0.961R**; whether an earlier BE amendment would fill or preserve portfolio outcomes remains an offline, sequence-aware counterfactual, not a result of this audit.

All six carried ETH BE paths touched TP, but they are only two correlated market episodes copied across three accounts. Conversely, the three current Crude BE exits have no later TP touch within their **2.963–3.037h partial** paths. Six already-profitable BTC/Brent TP exits subsequently crossed their original SL within four hours. These mixed paths do not justify a global exit change.

Four exact checkpoint values are deliberately **unverified**: the 1h and 2h endpoints for Brent **262471984/262471983** have last quotes **6.003–8.905 seconds** old, beyond the frozen six-second threshold. The remaining 65 elapsed checkpoints are fresh; six future four-hour checkpoints are null. The largest post-close update gap is **36.537 seconds**, and worst tail gap is **1.120 seconds**. Full elapsed clocks are not described as uninterrupted tick coverage. No interpolation fills these gaps.

## §7. Trigger funnel, guards and outcome attribution


| Account | New arms | Consumed / intents / individual outcomes | Fills | Peer refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 26 | 0 / 0 / 0 | 0 | 0 | 26 |
| 46130058 | 16 | 9 / 9 / 9 | 6 | 3 | 3 |
| 47790949 | 15 | 9 / 9 / 9 | 6 | 3 | 2 |
| 49236051 | 18 | 7 / 7 / 7 | 6 | 1 | 7 |


There are **75 arms, 25 consumed triggers, 25 persisted opening intents, 18 fills and seven final peer refusals**: four below the 24 bp hold requirement and three opposing-direction refusals. Every consumed decision ID joins to exactly one terminal outcome and one intent. **Missing individual outcomes=0; unknown entry-guard receipts=0**. There are **38 deep-retracement invalidations** and **232 pair-reservation diagnostic rows**. Diagnostic rows are not independent missed trades, and arms need not all terminate within this same interval. [6]

Maximum observed confirmation-to-final-guard time is **136.353 ms**. This is distinct from broker order-to-fill time. The maximum spans only measured final-guard receipts; background preparation waits are not silently included or assigned zero. The window's fill receipts comprise **18 filled entries, 19 broker exits and 12 confirmed amendments**, with **zero recorded market-closed or other broker rejections**.

Current live guards are explicit: BTC/ETH projected margin below 250%; XAUUSD yields to human inventory; XAGUSD pair reservation; both indices have mixed 5d/20d macro drift; oil invalidation persists. Several demo crypto symbols have background-admission-not-ready or expired-admission diagnostics, while the small demo also records minimum-volume risk blocks. These current snapshots do not establish that every previous setup had the same refusal.


| Account | Symbol | Current gate | Message |
| --- | --- | --- | --- |
| 43002148 | BTCUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 135.03% below floor 250.00% |
| 43002148 | ETHUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 137.23% below floor 250.00% |
| 43002148 | NAS100 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 4.614%, 20d: -5.112%) |
| 43002148 | SpotBrent | invalidated_deep_retracement_gt_61_8pct | Invalidated: retraced > 61.8% without turn confirmation |
| 43002148 | SpotCrude | invalidated_deep_retracement_gt_61_8pct | Invalidated: retraced > 61.8% without turn confirmation |
| 43002148 | US500 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.128%, 20d: -2.030%) |
| 43002148 | XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | yield_to_human_position_or_order | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | prepared_admission_unavailable | Preparing: background entry admission is not ready yet |
| 46130058 | ETHUSD | prepared_admission_unavailable | Preparing: background entry admission is not ready yet |
| 46130058 | NAS100 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 4.614%, 20d: -5.112%) |
| 46130058 | SpotBrent | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| 46130058 | SpotCrude | session_already_entered | Blocked: one entry already recorded for this symbol/window |
| 46130058 | US500 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.128%, 20d: -2.030%) |
| 46130058 | XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| 46130058 | XAUUSD | own_position_or_pending_entry | Managing existing position or pending entry; no additional entry |
| 47790949 | BTCUSD | prepared_admission_unavailable | Preparing: background entry admission is not ready yet |
| 47790949 | ETHUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: prepared admission expired; background refresh required |
| 47790949 | NAS100 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 4.614%, 20d: -5.112%) |
| 47790949 | SpotBrent | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| 47790949 | SpotCrude | session_already_entered | Blocked: one entry already recorded for this symbol/window |
| 47790949 | US500 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.128%, 20d: -2.030%) |
| 47790949 | XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| 47790949 | XAUUSD | own_position_or_pending_entry | Managing existing position or pending entry; no additional entry |
| 49236051 | BTCUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: min_volume_exceeds_trade_risk_budget |
| 49236051 | ETHUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: prepared admission expired; background refresh required |
| 49236051 | NAS100 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 4.614%, 20d: -5.112%) |
| 49236051 | SpotBrent | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| 49236051 | SpotCrude | session_already_entered | Blocked: one entry already recorded for this symbol/window |
| 49236051 | US500 | vetoed_by_5d_20d_macro_trend | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.128%, 20d: -2.030%) |
| 49236051 | XAGUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: min_volume_exceeds_trade_risk_budget |
| 49236051 | XAUUSD | invalidated_deep_retracement_gt_61_8pct | Invalidated: retraced > 61.8% without turn confirmation |


## §8. Session openings, recovery evidence and Saturday crypto preparation

The window covers London 15:00 and NY 22:30 traditional openings, plus crypto 16:00, 20:00, 00:00 and 04:00 SGT: **80 account-symbol-window slots**. All have recorded two-sided opening quotes inside the versioned 15-second observation limit. Full bid/ask prices and bid/ask/observation timestamps match operational receipts for **72**. Four live-metal slots are human-locked. [7]

The other four slots initially recorded opening-reference unavailability, then subsequent assessments used the exact first recorded midpoint:


| Account / symbol | Session open SGT | First recorded quote SGT / midpoint | Recovery evidenced by SGT | Evidence qualification |
| --- | --- | --- | --- | --- |
| 49236051 / XAUUSD | 13 Nov 15:00:00.000 | 13 Nov 15:00:00.137 / 3,900.435000 | 13 Nov 15:01:42.276 | Midpoint recovered; complete recovery quote receipt not retained |
| 46130058 / XAUUSD | 13 Nov 15:00:00.000 | 13 Nov 15:00:00.139 / 3,900.430000 | 13 Nov 15:01:42.277 | Midpoint recovered; complete recovery quote receipt not retained |
| 47790949 / BTCUSD | 14 Nov 00:00:00.000 | 14 Nov 00:00:00.066 / 68,861.750000 | 14 Nov 00:01:28.670 | Midpoint recovered; complete recovery quote receipt not retained |
| 47790949 / ETHUSD | 13 Nov 16:00:00.000 | 13 Nov 16:00:00.206 / 1,932.080000 | 13 Nov 16:06:35.119 | Midpoint recovered; complete recovery quote receipt not retained |


This is **76 operational openings supported, plus four human-locked slots**, but not 76 complete durable quote receipts. The deployed opening-only retry updates `day.open` and `day.opening_quote` in memory and returns before the ordinary session-note writer (`strategy3.rs`, opening retry in `prepare`). Thus an early failure note can outlive a successful recovery in the durable journal. Later band values prove these four recovered midpoints; the original bid/ask recovery timestamps cannot be reconstructed as complete operational receipts. This is a bounded telemetry follow-up, not evidence that the sessions stayed blocked. No trading code is changed here.

The prior London late-mark limitation is still visible: 420/450-minute preparation marks have **9 endpoints versus 14 required**. Several London assessments recorded `no_band` at the late marks. Base preparation and full-session eligibility remain separate; the audit does not fill missing historical endpoints.

Saturday 08:00 BTC/ETH preparations are complete for **8/8 account-symbol slots**, including macro inputs and all required marks. These are future-window preparations, not advance observations of the 08:00 opening price. Prepared data does not bypass ownership, risk, margin, peer or broker-calendar checks.


| Account | Symbol | Future session open SGT | Preparation recorded SGT | Base ready | All marks ready |
| --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:25.149 | True | True |
| 43002148 | ETHUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:27.902 | True | True |
| 46130058 | BTCUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:24.528 | True | True |
| 46130058 | ETHUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:26.690 | True | True |
| 47790949 | BTCUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:24.526 | True | True |
| 47790949 | ETHUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:26.555 | True | True |
| 49236051 | BTCUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:25.056 | True | True |
| 49236051 | ETHUSD | 14 Nov 08:00:00.000 | 14 Nov 04:00:27.610 | True | True |


## §9. Friday close and post-cutoff weekend observation

The deployed calendar sets Friday traditional closing deadlines to **16:55 America/New_York = Saturday 14 Nov05:55 SGT**. The two open gold S3 orders have that exact stored deadline. At 05:35, having those positions open is consistent with their registered policy. There are no market-closed rejection receipts inside the frozen activity interval. [1,8]

Crypto has six UTC four-hour entry windows each day and a daily New York **[16:59,17:01)** maintenance entry exclusion, equivalent to Saturday **05:59–06:01 SGT**. The calendar does not define a full crypto weekend shutdown or a universal forced crypto liquidation at 05:59. Broker trading mode/holiday checks and the position's own registered exit deadline still apply. Saturday 08:00 preparation is complete; a future executable opening quote is not yet known.


**Separate post-cutoff observation — 2026-11-14T05:55:21.296402+08:00** (owner books/health have their own capture timestamps). This does not extend the 05:35 cash cutoff.

| Account / position | Deadline SGT | Actual closing time SGT | Filled / original native volume | Net USD / original R | Fully closed |
| --- | --- | --- | --- | --- | --- |
| 47790949 / 262583136 | 2026-11-14T05:55:00+08:00 | Not observed | 0 / 900 | No close / N/A | False |
| 46130058 / 262583980 | 2026-11-14T05:55:00+08:00 | Not observed | 0 / 1000 | No close / N/A | False |

Later health was captured at **2026-11-14T05:55:41.939894+08:00**; owner endpoint capture at **2026-11-14T05:55:20.428333+08:00**. The underlying account books span **2026-11-14T05:54:51.252422+08:00 .. 2026-11-14T05:54:51.295354+08:00**.

The observed closing cash is **$0.00**, excluded from all cutoff totals and cutoff latest20 figures. **0/2** gold positions are fully closed by recorded deals; **2** target positions remain in the separately captured owner books. **4 MARKET_CLOSED** refusal receipts occur after 05:35 through this observation. All **4/4** account actors remain connected and unpaused. Exact close intents, dispatch/fill receipts, cash components and later inventories are retained in JSON. No instruction to close a position was sent by this audit.

**Weekend exit failure observed:** first attempts were sent at **05:55:00.847 SGT (47790949)** and **05:55:00.885 SGT (46130058)**; both were rejected as `MARKET_CLOSED`. Repeated attempts around **05:55:10.902/10.925** were also rejected. No closing deal was recorded by this query. The two positions were still present in the later owner books, so a scheduled deadline must not be reported as successful liquidation. The zero post-cutoff cash here means no realised close, not flat positions or zero exposure.

The receipts prove that closing at the stored deadline failed. They do not establish the exact earlier broker tradability boundary. A follow-up should reconcile that boundary with the versioned calendar, choose and test an appropriate pre-close execution margin, and bound repeated closed-market retries while retaining human isolation and existing risk/exit geometry. No such repair is part of this read-only audit.



## §10. Assessment and bounded next work

1. **Retain the measured distinction between entry failure and exit sensitivity.** The London Crude losses never approached the current BE threshold; their observed MFE also fails a 0.80R trigger. Investigate the original impulse, macro/peer evidence and executable stop slippage on those same paths before proposing an exit change. The snapshot does not establish a profitable replacement rule.
2. **Persist successful opening recovery as its own evidence.** A future infrastructure-only proposal should record account, symbol, session, original bounded quote and recovery time once when opening preparation changes from unavailable to available. Verification should include delayed recorder flush and restart reconstruction. The observed four recovered midpoints make the audit gap concrete; no change is deployed in this task.
3. **Keep late London coverage and admission freshness visible.** Resolve the missing 420/450-minute historical inputs through a separately approved data/preparation task; distinguish a prepared future table from an executable current admission. Do not infer all accounts can enter merely because future crypto data is 8/8 ready.
4. **Finish remaining elapsed follow-ups only in the next report.** The six partial rows have explicit capture horizons. Subsequent prices must not rewrite this report's frozen activity cash or imply a fill. Trading targets, BE rules, pair caps, 0.6% risk and 250% margin floor remain unchanged.
5. **Preserve human authority and separate live capital from demo performance.** Live floating equity declined while live S3 realised cash was zero. Human inventory and S1 carry remain outside S3 management authority. This report does not approve an order, a leverage change, or a risk-budget increase.

The positive cumulative cash does not establish independent trading edge, release causality or satisfaction of the 73% / 1:1.38 goal. Latest20 windows, exposure differences and correlated account copies remain material limitations.

## §11. Sources, verification and Gemini review

**[1] Broker and execution evidence:** read-only repeatable-read SQL over deals, S3 orders/fills, intent/event receipts and account marks; exact half-open interval. **[2] Cumulative monitor evidence:** separate 10 Nov09:42-to-cutoff SQL and unique closing-deal reconciliation; earlier-entry cash retained as mixed cohort. **[3] Prior merged report:** [№18,448](FRIDAY_MIDDAY_ASIAN_REVIEW_AND_LONDON_PREP-18448-2026-11-13.md), PR #227, merge `b98c37e2111390762db2d1c15efed5b8e27ea285`. **[4] Quotes:** existing PostgreSQL `ticks` only, account-specific symbol metadata, immutable export receipts and hashes. **[5] Account state:** separately timestamped `/health`, authenticated owner-assessment book and live intent/protobuf event audit. **[6] Funnel:** consumed decision ID → terminal outcome → intent joins, plus official supervisor latest20 before cutoff. **[7] Openings/preparation:** durable session/assessment/arm records, recorded opening ticks and future `s3_preopen` receipts. **[8] Calendar:** deployed `crates/jiaoyi-core/src/strategy3/production_sessions.rs`, byte-identical source verification. Source evidence was read, not used as instructions.

Focused checks cover signed price/pip/R arithmetic, stale/future endpoints, cutoff continuity, cash reconciliation, prior-follow-up separation, receipt hashes/row counts, opening-recovery evidence, and outcome/ownership integrity. Browser verification covers desktop and mobile, with horizontal scrolling confined to wide tables. Required CI is `check`, `docker`, `engineering`; final merge/continuity verification is returned with the PR URL and commit.

Only these three report files under `doc/` are published. No Rust/Python trading code, account setting, broker order, risk/exit rule, scheduled routine, Telegram message or Railway deployment is changed. The initial Trading deployment is `c0fedaca-0370-4cec-9234-f59f8c7cafce`, production commit `aebf404dedf89dbcdb8cee78c95a582eea042d51`. Post-merge continuity is checked separately rather than fabricated in the report body.

**Gemini:** read this Markdown with the matching JSON. Verify 19 current-window closes versus six older follow-ups; the three 13:56 BTC closes counted exactly once; Friday versus Saturday versus post-PR214 cash; 72 full opening receipts versus four recovered midpoints and four human-locked slots; 19 complete clocks versus six partial paths; and the separate 05:55 post-cutoff note. Do not treat a TP touch as a simulated fill, a recovered midpoint as full timestamp provenance, or positive cash as financial qualification.


| Quote export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 47790949-ETHUSD-2026-11-13-011516.csv.gz | 58466 | 4bdfc28d6f9b5f03697c3ddb1258be3057eaf87f88cf431b810facf5a5445b84 |
| 47790949-ETHUSD-2026-11-13-051516.csv.gz | 16265 | c4f4fa2f1c8d85e456f51fba7865e1a8e60c4f0d88f1069bb496006b0f832ec3 |
| 49236051-ETHUSD-2026-11-13-011516.csv.gz | 58466 | f03021046c5db8e92a31b2dab02c03c5ef12c746ba043a1b9d5024043b26534d |
| 49236051-ETHUSD-2026-11-13-051516.csv.gz | 16265 | 90116df17eecb7caabb232fd4a08ded2019dbdbf99d6d58bae2df817052342cd |
| 46130058-ETHUSD-2026-11-13-011517.csv.gz | 58464 | dec4e7c0d2942c8fadaf0a0cf533557f23f53d54cf80d3876041d1afb1c923b4 |
| 46130058-ETHUSD-2026-11-13-051517.csv.gz | 16264 | ed0d82a5ddb4e7a8652e5c1d5febbf064b6c3050aa51bc5493731ee7d3f27e01 |
| 49236051-ETHUSD-2026-11-13-043903.csv.gz | 53839 | 9f6676d2cb3fc7b03345dbde099f8e5afd8c988ec4d222dd25aca70bb16a1312 |
| 49236051-ETHUSD-2026-11-13-083903.csv.gz | 3340 | 27ad447f65aca273e8520d771fe452bca5a82e41e66be0867d97b42027881d97 |
| 47790949-ETHUSD-2026-11-13-043903.csv.gz | 53841 | 65976035b4f4b6fc4f177269444936ad4b429e8e70bd6c1bc092afaca61c9975 |
| 47790949-ETHUSD-2026-11-13-083903.csv.gz | 3340 | c9f46d0491e0d68c981b8660159b058a9772bfafdfcbab4c8734015dd2cb4073 |
| 46130058-ETHUSD-2026-11-13-043903.csv.gz | 53839 | 35d82ede0f73de829caa462667bd714f88f76013acceda88254f496d544b6f50 |
| 46130058-ETHUSD-2026-11-13-083903.csv.gz | 3338 | 38e0f7efdf7aff69d4748ea210859f7fe371efcc8744ebec777c1d7809052eb9 |
| 49236051-BTCUSD-2026-11-13-051129.csv.gz | 65151 | 66c0dad3318595bcc528bc6f68062c134c591d91e2fc51c135b3d6437c04eaf9 |
| 49236051-BTCUSD-2026-11-13-091129.csv.gz | 10867 | f9c8303df7d18a70f54f230d5aab923ca1bcba9b20b34a4f27a6ff597d719021 |
| 46130058-BTCUSD-2026-11-13-051130.csv.gz | 65151 | 24622032877fb62638ddb73fa649d70fa9dd4929e8d057d8c81b677ad30ad2bf |
| 46130058-BTCUSD-2026-11-13-091130.csv.gz | 10864 | c473a1a49e011d0a3d585ef5941f03b2e7b57f1eea3ff22cf22633f60f85835e |
| 47790949-BTCUSD-2026-11-13-051132.csv.gz | 65151 | 4e4ab876abc96859e84e31d36affab01ec6a2d0812cb733bc5fa0f5748faeb8c |
| 47790949-BTCUSD-2026-11-13-091132.csv.gz | 10854 | 27d2acd8df698a3801eda8526276390d8f7f666f8497d83e95158013af15426b |
| 49236051-SpotBrent-2026-11-13-080058.csv.gz | 13437 | 8895da3427a59fc66153b29db330f5e7d6a98410b90810ba56769a7ad0b8d8e5 |
| 49236051-SpotBrent-2026-11-13-120058.csv.gz | 9551 | 4a9adc6eac3e5b0cdff052fce9cfc8971f71deea252944da94ae1031150a327f |
| 46130058-SpotBrent-2026-11-13-080102.csv.gz | 13454 | 1782ff6c6d371e6dd8f2eedd38869e198390d7ca0d9d0b075a08daa6180b5614 |
| 46130058-SpotBrent-2026-11-13-120102.csv.gz | 10404 | b7c055accec0b7c6ea907f6130829a9c2d140f9464ac17b91e9987b21a8bc0ae |
| 47790949-SpotBrent-2026-11-13-080101.csv.gz | 13439 | 953a61e3db5a29a2432c0755dc4960267598375e93797a643427abb259aa7a13 |
| 47790949-SpotBrent-2026-11-13-120101.csv.gz | 10401 | bf8efacc8e156fb57ea748e73a1ef7618f892baef091ff125ac8bb1439d4ca1b |
| 49236051-SpotCrude-2026-11-13-104946.csv.gz | 18946 | 7312e007802083ef29213f62b372b7b261b1b88fd442d40fba0c2fbde97441f7 |
| 49236051-SpotCrude-2026-11-13-144946.csv.gz | 17731 | 5110820ae94cc4b7b9b8ec0d677f441a98daeb8ed296f81227712a67dca66f84 |
| 47790949-SpotCrude-2026-11-13-112831.csv.gz | 21142 | bee8a08085241fc63b22632ffdd67cdc949404230bda915ff8f69d01162b436b |
| 47790949-SpotCrude-2026-11-13-152831.csv.gz | 13345 | 1836d2c494712904a52eca1dcb75824dbd1b1cd56c548a24bc48d51edc626fae |
| 46130058-SpotCrude-2026-11-13-112832.csv.gz | 21153 | 1d41bb4a781ffe39939c3751f1737ec2d273c40a38fc6d85ac8fc0eb3a750c95 |
| 46130058-SpotCrude-2026-11-13-152832.csv.gz | 13343 | f15b03d38d52b60d3975e99f33fc4877206691d0454bab54435eaba3232605ce |
| 49236051-ETHUSD-2026-11-13-145146.csv.gz | 54688 | f31c9fd505a8bb475b2fb4b758d4e45170b1e74e871588310ec50d0201a3c070 |
| 49236051-ETHUSD-2026-11-13-185146.csv.gz | 5322 | 946db092769ee40811d26220e62c7d73f1bb45167a28d3677e80802518548e85 |
| 47790949-BTCUSD-2026-11-13-150602.csv.gz | 63011 | bee08b21f20494ee3ce7aef46fa3f1fa2563d3e15f73f418d059a6ce70cd6ac7 |
| 47790949-BTCUSD-2026-11-13-190602.csv.gz | 5808 | e5d0757bab638a006b5becfa95328063c381dc7d52c15ddec5f51c282157f201 |
| 46130058-BTCUSD-2026-11-13-150602.csv.gz | 63011 | b11b2e70ec731fc7518371a5888bf274df5bc45c6b44d53ba53173b46174f8ae |
| 46130058-BTCUSD-2026-11-13-190602.csv.gz | 5808 | c2a7ad2c1278a4bbad4fd37f30ac7abbd2bdbde0fd9a2752c27333540b9f5c99 |
| 49236051-SpotCrude-2026-11-13-155621.csv.gz | 17735 | 3b4081aa5f4baa2773d15e5142c7599c6738101e1c06a257e5eff924ed367225 |
| 49236051-SpotCrude-2026-11-13-195621.csv.gz | 3456 | 48e04119aa472635b5a3964bc7b35f58c010b22452b4cba7123ce2e29406316b |
| 46130058-SpotCrude-2026-11-13-155625.csv.gz | 17731 | 41b62d30b4087a16d4ebb0c29fbbdfbd3291f169b8bb1c04e3685bb766173969 |
| 46130058-SpotCrude-2026-11-13-195625.csv.gz | 3452 | 31bb1f99ad5608f9210dd448a67f8c8a45d18adf3e1ad9395e141f02acbb80c6 |
| 47790949-SpotCrude-2026-11-13-155628.csv.gz | 17725 | 51a3dd2d11864d5e9739f87c818d915484738079f5e86b8beadb72ef8a71f352 |
| 47790949-SpotCrude-2026-11-13-195628.csv.gz | 3452 | 60f951977d642e4b53596a7ece45ca17558f5dc7966143602f4d31d25b2299c8 |
| 49236051-ETHUSD-2026-11-13-160252.csv.gz | 50063 | 6afe0c4d0f95dd217f99ed8e5d0a2e30fbb7f914966e6c24ee8fb3f4c11c5474 |
| 49236051-ETHUSD-2026-11-13-200252.csv.gz | 1255 | 4aad3bddc640bafb252647141c1b9ee23a763d96aadfb91f68d53d349b54882e |
| 47790949-ETHUSD-2026-11-13-160253.csv.gz | 50060 | e7216ec5e296f7826bd2c3b377cba12fdb6ade933e7209582da8402eced331e8 |
| 47790949-ETHUSD-2026-11-13-200253.csv.gz | 19074 | 909e7b4b75bbd951b83ea6e03cd166018d1e3238eb026c1eed0a8b1a0eeadfbf |
| 46130058-BTCUSD-2026-11-13-160718.csv.gz | 60376 | 32b91e7d74c22ea6eb5c490664819c37842d02d304dd6174cc4da3900ca4d07f |
| 46130058-BTCUSD-2026-11-13-200718.csv.gz | 22132 | 5d469da66681c0e8b97c742796fe88894f7e3c390a01de7cc32788269bab2b7c |
| 47790949-XAUUSD-2026-11-13-161029.csv.gz | 42459 | 245e142a6fa1619799916370a644a17b9fa37035dd86ad522456204c0f178ad9 |
| 47790949-XAUUSD-2026-11-13-201029.csv.gz | 13834 | 8490d74001f71a3d9e0f8aaddc3b966baec45bc53c2c533df2765eb36f48328c |
| 46130058-XAUUSD-2026-11-13-161335.csv.gz | 42288 | 573fb15068f1dd69f3c895d465e5c1e80a53144d13b7e362d55df7bfd7a96b4f |
| 46130058-XAUUSD-2026-11-13-201335.csv.gz | 13332 | a2ed25b43bfc09de52fb4a33f7062ea84092a5300cdf979ddeadcda678e5a3b8 |
| 49236051-BTCUSD-2026-11-13-162335.csv.gz | 59783 | 78539115f2008d73263162b3254a238b5b2f31f237e4fd52f6591a2f6df7a039 |
| 49236051-BTCUSD-2026-11-13-202335.csv.gz | 18281 | e66f8920d8175a6dde1ae12bde72ed2eb164c98c153cbcc9dbd4bbe5d389911d |



| Source | SHA256 |
| --- | --- |
| opening-ticks.json | 256ad7a3f4b53699601519ba5f29031f50a2a9aa1c67a5d5f9816d53b262cc2a |
| preflight.json | 95bf976a88898fa4ca6234feccd4f0e53dd08b5e10a1cb3efab8723cbffd06d9 |
| monitor.json | 37d16aec10c0257eca9e6c64de792b5d8fe0e104ae8422c5321461cef0996cc2 |
| cumulative.json | b13ea0cfd4abc13306176f353e1678ffedbf87fcb8794a108c6d90afa8a76681 |
| human.json | a27c01c8a868bbb8bf35a012688e2031737bb0639f1e663855058ea19e4579e9 |
| health.json | 20585204e750e62d98825f18e78760422fb8a32676730b95007e60fa3e400ade |
| activity.json | c7a12d4d54d93fc658a9050fcfabb2f4019a9c4f8b6d17c6246728db9a62dc6b |
| evidence.json | e18b4c6c1570eb0767fd59e194bd9bdff8aae53bce5ddb395e42f3584f86c9c0 |
| deployment-before.json | 6cab90a4e75ee1ed9ad74d89b5cb42a8c91d062e0a39cbe9e231c04b4049604a |
| runtime.json | 227e84af7ddf498043640524115093d78d2e91921cd6c58a2e9789d84c5587b4 |
| opening-recovery.json | 65a73de89cf183fe561de8cafb8b85b359aa8849fbaa6b4fc6364b44a61f1b5d |
