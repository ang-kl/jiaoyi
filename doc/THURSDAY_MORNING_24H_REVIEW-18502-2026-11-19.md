# Thursday Morning 24-Hour Ledger Review — №18,502

**Activity:** [18 November 2026 07:48:00, 19 November 2026 08:13:00) Asia/Singapore. **Read-only production, database and broker audit.**

[Measured JSON and evidence](THURSDAY_MORNING_24H_REVIEW-18502-2026-11-19.json) · [Standalone HTML](THURSDAY_MORNING_24H_REVIEW-18502-2026-11-19.html)

## §1. Assessment

- **Window closed S3 cash: $1,056.90 from 22 closes**: 12 positive-net wins, 8 losses and 2 exact-zero outcomes. 0 S3 positions remained open at the activity cutoff.
- **Full Wednesday S3 cash: $1,087.27. Thursday 00:00→08:13 S3 cash: −$162.99. Post-PR #214 cumulative closing cash: $2,325.10.** Latest-20 counts are 43002148 3/20, 46130058 6/20, 47790949 7/20, 49236051 12/20. No account meets 15/20 (75%). Financial qualification remains false.
- **25 post-close tracks: 23 complete, 2 partial.** The three carried rows from PR #240 are updated without recounting their closing cash. Initial-stop and breakeven-stop target crossings are reported separately.
- The live account cash bridge, broker-driven partial JPN225 liquidation, S1 NatGas provenance and manually closed S1 position are audited separately from S3 in §4. A profitable partial liquidation is not a bot strategy win.
- Thursday London is **20/24 base-prepared at cutoff**, with Brent's four London receipts pending its nearer 09:00 session. All twenty available London tables have **13/14** late winter-length endpoints. This differs from the directive's expected 24/24 and is recorded without assuming future preparation succeeds.
- Production remains on `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`, policy `s3_macro_pullback_owner_18409_v1`. Only these documentation artifacts change; no trading code, account settings, risk parameters or deployment was altered.

## §2. Frozen clock, PR #240 anchor and cash reconciliation

The first clock read was **19 November 00:13:14 UTC / 08:13:14 SGT**. The activity cutoff is therefore **08:13:00 SGT**, with a separately frozen **08:13:15 SGT post-close quote horizon**. The actual interval is **24 hours 25 minutes**, not a rounded 24-hour selection. Account money and runtime snapshots captured afterward are timestamped separately; they are not retroactive cutoff marks.

The anchor is [№18,484 JSON](WEDNESDAY_MORNING_NY_ASIA_REVIEW-18484-2026-11-18.json), **PR #240**, merge `9ff2f55472f42785875f3f3c65f84baf639a00c8`, SHA256 `385409b7df6f051f6a3bb38e6a2f91893321dd3f35f565d8e0a6bb21d0bea85b`. It ended at **18 November 07:48 SGT**, with Wednesday-to-cutoff S3 cash **−$132.62** and post-PR #214 cumulative S3 cash **+$1,268.20**. Its only unfinished tracks were **263762252, 263762285 and 264006264**. PR #241 was a readiness/research audit and is not the ledger anchor.


| Account | Window closes | Window S3 USD | Full Wed S3 USD | Thu to cutoff S3 USD | Post-PR214 S3 USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | 0.00 | -0.01 | 0 |
| 46130058 | 10 | 505.02 | 554.32 | -96.85 | 1,038.34 | 0 |
| 47790949 | 10 | 546.01 | 532.95 | -72.01 | 1,254.46 | 0 |
| 49236051 | 2 | 5.87 | 0.00 | 5.87 | 32.31 | 0 |


The exact bridges are **$1,268.20 + $1,056.90 = $2,325.10 cumulative**, and **−$132.62 + $1,056.90 = $1,087.27 full Wednesday + −$162.99 Thursday-to-cutoff**. These are broker closing-cash intervals, not entry-cohort returns attributable solely to a software release. The earlier three pre-PR214 entries remain disclosed in the cumulative cohort.

Cash uses unique account/deal IDs, recorded money digits and signed gross/swap/commission/conversion components. Prior follow-up cash is excluded. Open floating P&L, external closes, S1 closes and deposits are not added to S3 realised performance. History coverage and the persisted monitor's completeness flags remain in JSON; a recent successful history pass does not convert incomplete attribution into a verified zero.

## §3. Thursday 05:57 buffered exits and daily rollover


### Verified Thursday buffered exits

Exactly **2** NY S3 positions reached the stored Thursday 05:57 deadline, realizing **+$21.32** together. Every enumerated NY deadline retains `native_preclose_v18455_120s`: exit deadline 05:57, source `Session.close` 05:59. Both actual fills arrived before the hard close. Other NY positions in the ledger exited earlier; they are not additional 05:57 executions.

| Account / position | Symbol | Trigger SGT | Send SGT | Broker fill SGT | Fill | Net USD | Net R | Send→broker fill |
|---|---|---|---|---|---:|---:|---:|---:|
| 49236051 / 264496289 | US500 short | 05:57:00.622887 | 05:57:00.923080 | 05:57:01.326000 | 7318.7 | -0.48 | -0.105033 | 402.920ms |
| 47790949 / 264505974 | XAGUSD short | 05:57:00.874190 | 05:57:01.300751 | 05:57:01.731000 | 45.067 | +21.80 | +0.154830 | 430.249ms |

The deadline is exact; an executable ask exactly at 05:57:00 is not present in the retained tick stream. The following comparisons use the last recorded ask **at or before** the deadline. Both are older than 6 seconds, so these are carry-forward reference marks rather than fresh executable quotes at that instant. No later price is relabelled as the deadline quote.

| Symbol / account | Last pre-deadline ask | Receipt SGT | Age at 05:57 | Fill minus ask | First post-deadline ask | Decision ask |
|---|---:|---|---:|---:|---|---:|
| US500 / 49236051 | 7319.0 | 05:56:53.754885 | 6.245115s | -0.300 | 7318.9 at 05:57:00.287913 | 7318.7 |
| XAGUSD / 47790949 | 45.067 | 05:56:49.183962 | 10.816038s | +0.000 | 45.065 at 05:57:00.285673 | 45.064 |

The short US500 fill 7318.7 equals its decision ask; Silver 45.067 is 0.003 above its decision ask 45.064. Local fill acknowledgements arrived at 05:57:01.460655 and05:57:01.854590 respectively; send-to-local-receipt intervals 537.575 ms and 553.839 ms differ from broker execution latency because they include return delivery.

### Rollover hygiene

**0 recorded `MARKET_CLOSED` rejections, rejection/error events, and S3 `session_end` retry attempts** occurred in `[19 Nov 05:59, 07:05) SGT`. Two S3 intents exist during that interval: BTC opening intents at 07:02:33.792771 on 46130058 and07:02:35.807803 on 47790949. Both were refused by peer conflict; these were not fills or close retries. This conclusion is confined to retained PostgreSQL events/orders; no trading container was restarted for this audit.


## §4. Account snapshots, live cash bridge and strategy ownership


### Account money, inventory and assessed margin

All four accounts were connected and unpaused at the separately captured health snapshot. No S3 position was open in the current books. The account money marks below are **19 November 08:14:16 SGT**, after the activity cutoff of **08:13:00 SGT**; the JSON preserves each exact timestamp. Runtime margin assessments are later, around **08:14:47 SGT**. They are not an atomic historical account mark. Equity is cash balance plus broker net unrealized P&L, without unverified credit or prospective closing commission.

| Account | Money mark SGT | Cash USD | Net float USD | Equity USD | Assessed margin % | Pending projected % | Pending stressed % | Open S3/S1/S2/External | Pending entries |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 2026-11-19T08:14:16.510000+08:00 | -177.19 | 1,219.01 | 1,041.82 | 209.23 | 41.46 | not assessed / unknown | 0/0/0/4 | 3 |
| 46130058 | 2026-11-19T08:14:16.049000+08:00 | 25,326.85 | 277.56 | 25,604.41 | 10,354.90 | not assessed / unknown | not assessed / unknown | 0/6/0/0 | 0 |
| 47790949 | 2026-11-19T08:14:16.058000+08:00 | 25,235.30 | 263.25 | 25,498.55 | 1,607.56 | not assessed / unknown | not assessed / unknown | 0/5/0/0 | 0 |
| 49236051 | 2026-11-19T08:14:16.105000+08:00 | 827.08 | -0.24 | 826.84 | 95,039.08 | not assessed / unknown | not assessed / unknown | 0/1/0/0 | 0 |

Live current margin **209.23%** is below the 250% new-entry floor; the pending-order forecast is **41.46%**. Stressed pending margin remains unknown, with a 1-lot NatGas pending entry (#402627798) lacking an SL. These are reasons to withhold new entries, not authority to close human positions. The three pending live entries are NatGas 0.1 lot @ 2.856 with SL 2.806, NatGas 1 lot @ 2.532 without SL, and XAUUSD 0.1 lot @ 3700.10 with SL 3654.02. Live metals are human-locked. Demo accounts have no pending entries; absent pending forecasts are not zero percent.

Open S1 inventory: demo 46130058 holds AUDUSD, EURUSD, NZDUSD, USDCAD, GBPUSD and NatGas; demo 47790949 holds the same five FX positions without NatGas; demo 49236051 retains its S1 ETHUSD short #262796763, 0.01 lot @ 1746.87, SL 2026.76. Live has no S1 or S2 inventory at this snapshot.

The last stored trader read for demo 47790949 was $25,213.50 at 03:58:38 SGT, before its 05:57:01.731 XAGUSD close of +$21.80. The stable account mark of **$25,235.30** includes that close. The difference is stale sampling, not additional post-cutoff trading.

### Live cash bridge and broker partial stop-outs

**−$76.17 + $1,950.00 − $2,577.85 + $526.83 = −$177.19**, with an exact $0.00 residual. Seven XAUUSD closing deals cover six positions, including a partial close; the five JPN225 closing deals belong to one position and are all counted separately. No S1/S3 cash is included in these external cash rows.

| Deposit history ID | Executed SGT | Deposit USD | Balance after USD |
| --- | --- | --- | --- |
| 134215911 | 2026-11-18T09:27:35.868000+08:00 | 800.00 | 110.12 |
| 134222236 | 2026-11-18T10:07:18.927000+08:00 | 350.00 | -106.18 |
| 134285895 | 2026-11-18T21:21:05.167000+08:00 | 400.00 | -370.99 |
| 134287944 | 2026-11-18T21:33:20.884000+08:00 | 250.00 | -120.99 |
| 134345567 | 2026-11-19T03:57:10.133000+08:00 | 150.00 | -177.19 |

| External XAUUSD deal | Position | Executed SGT | Exit | Native volume closed | Net USD |
| --- | --- | --- | --- | --- | --- |
| 348717547 | 658574004 | 2026-11-18T08:32:55.742000+08:00 | 3823 | 3000 | -613.71 |
| 348739839 | 659811622 | 2026-11-18T09:43:33.573000+08:00 | 3809.86 | 2000 | -179.80 |
| 348744937 | 659811622 | 2026-11-18T10:02:19.795000+08:00 | 3801.96 | 2000 | -386.50 |
| 348795567 | 659839866 | 2026-11-18T15:03:22.424000+08:00 | 3773.38 | 3100 | -783.64 |
| 348836639 | 660136642 | 2026-11-18T19:06:57.486000+08:00 | 3768.36 | 3000 | -408.00 |
| 348871627 | 660301587 | 2026-11-18T21:36:46.659000+08:00 | 3774.26 | 1000 | 12.20 |
| 348888043 | 660326730 | 2026-11-18T22:31:55.315000+08:00 | 3770.87 | 2000 | -218.40 |

| JPN225 deal | Executed SGT | Lots closed | Exit | Net USD | Server / stop-out |
| --- | --- | --- | --- | --- | --- |
| 348854861 | 2026-11-18T20:43:34.895000+08:00 | 0.1 | 58904 | 89.39 | true / true |
| 348861764 | 2026-11-18T21:07:16.810000+08:00 | 0.1 | 58895.7 | 88.87 | true / true |
| 348861851 | 2026-11-18T21:07:20.406000+08:00 | 0.2 | 58873.9 | 175.00 | true / true |
| 348861883 | 2026-11-18T21:07:23.007000+08:00 | 0.1 | 58867.5 | 87.10 | true / true |
| 348861894 | 2026-11-18T21:07:24.423000+08:00 | 0.1 | 58857.5 | 86.47 | true / true |

Position **#649772867 fell from 1.20 to 0.60 lots** through 0.10 + 0.10 + 0.20 + 0.10 + 0.10-lot reductions. Every receipt has `isServerEvent=true`, `isStopOut=true`, a closing order, no bot label, and a broker `Stop-Out:` comment reporting margin below its 50% stop-out threshold. Thus these were broker cServer margin liquidations, not S3 closes. **Zero live program intents and zero live S3 execution events** were recorded in the audit window. This establishes zero recorded S3 interference; it does not claim that every external UI interaction is streamed to this database.

### Live per-position floating P&L

A separate bounded read-only cTrader snapshot returned P&L at **08:14:31.293 SGT**, with the same book and balance before/after. It used five account-data reads plus two authentication frames, zero historical market requests, subscriptions, orders or mutations. Its later net float **$1,237.46** and equity **$1,060.27** differ from the earlier account table because prices moved; do not combine the two timestamps.

| External position | Symbol | Lots | Entry | SL | Gross float USD | Net float USD |
| --- | --- | --- | --- | --- | --- | --- |
| 649992251 | JPN225 | 0.4 | 57126.25 | 56284.2 | 511.57 | 491.57 |
| 649772867 | JPN225 | 0.6 | 57340.1 | 56323.4 | 686.87 | 656.89 |
| 657180931 | NVDA.US | 4.0 | 160.825 | 90.82 | -40.90 | -41.54 |
| 660692549 | XAUUSD | 0.1 | 3723.34 | 3709.98 | 153.00 | 130.54 |

The broker supplied three additional zero-P&L position IDs corresponding exactly to the three pending-entry orders; they are not open positions and were excluded from floating P&L. All four open positions have external ownership.

### NatGas S1 provenance and the manual close

| Account | Position | Opening deal | Actual broker fill SGT | Lots @ price | Ownership | Latest status |
| --- | --- | --- | --- | --- | --- | --- |
| 47790949 | 264518045 | 335065754 | 2026-11-19T01:55:14.687000+08:00 | 0.03 @ 3.264 | jiaoyi-s1 | closed_manually |
| 46130058 | 264518049 | 335065757 | 2026-11-19T01:55:14.930000+08:00 | 0.03 @ 3.264 | jiaoyi-s1 | open_s1 |

Both entries are positively linked to `jiaoyi-s1` opening instructions (`s1:<account>:10055:2026-11-16:retry:3`) and the Strategy 1 daily breakout / tick-triggered trend record. Entry evidence contains level 3.263, EMA fast ≈ 2.89562 above slow ≈ 2.89455, and a 0.41600 price stop distance. These are not external opening trades and **not S3 trades**. S3’s NatGas reservation and label laws remain in effect; those S3 laws do not prohibit separately owned S1 positions.

On demo 47790949, closing deal **335081763** (order 378086162) filled all 30,000 native units at **3.320** at **19 November 03:51:44.480 SGT**, yielding **+$16.80**. The closing client order ID begins `ctm-`, `isServerEvent=false`, and no matching program close intent exists. The owner identifies cTrader 2 iPad; the broker receipt independently establishes an external client close but does not encode the device/app name, so that device detail remains owner-supplied.

The closing execution was received at 03:51:44.584650 SGT; the protection order 378068762 was cancelled by the broker at 03:51:44.585562 SGT. At **03:52:20.730130 SGT**, Jiaoyi persisted `AUTHORITY_DISAPPEARED`: “Strategy 1 observed broker position absence,” **36.25013 seconds after the broker fill**. The position is absent from the current book. There were **zero subsequent NatGas 477 program intents or S1 fill/amendment receipts through the cutoff**; it was neither reopened nor trailed after the manual close. The live code marks disappeared ownership during reconciliation, so the old opening cannot regain management authority.

The sibling demo 46130058 position **#264518049** remains S1-owned at 0.03 lot. Its last accepted trail moved SL to **2.906** at **03:52:30.397227 SGT**; manual action on 477 did not close the independently owned 461 position.

### S1 GER40 closes, kept separate from S3

| Account | Position | Deal | Fill SGT | Entry → exit | Gross USD | Swap USD | Net USD |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 | 261813391 | 335036864 | 2026-11-19T00:07:02.515000+08:00 | 26714.9 → 25806 | -99.65 | -2.78 | -102.43 |
| 46130058 | 261813392 | 335036865 | 2026-11-19T00:07:02.551000+08:00 | 26714.9 → 25806 | -99.65 | -2.78 | -102.43 |

Both carry `jiaoyi-s1`, were broker-executed protective stops, and closed 0.1 lot. Combined S1 GER40 net cash is **−$204.86**; NatGas manual realized cash is **+$16.80** on 477 alone. Neither is counted in S3 results.

Five focused verification groups passed: exact-cent cash reconciliation; five distinct server stop-outs; S1/manual-close provenance and post-close suppression; stable live P&L/book/pending-ID reconciliation; and both GER40 cost calculations. Source SHA256 hashes, exact evidence IDs and raw-receipt hashes are in the companion JSON.


## §5. Complete S3 ledger, protection and latest-20 performance

The table covers every S3 position closed, newly opened or still open within the exact activity interval. Retained account history outside the interval is used only for attribution and cumulative/latest-20 calculations. The 22 closes comprise **9 protective TP fills, 6 native-session closes, 3 BE stops and 4 initial stops**; their net-cash outcomes are 12 wins, 8 losses and 2 zeroes. The original price R is the actual fill-to-initial-stop distance; the original USD risk uses filled units and the recorded conversion. Net-positive cash defines a net win; a BE-classified exit can be zero or negative after execution.


| Account / position | Symbol / side | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit class | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 264117913 | XAGUSD / sell | 15000 / 0.0300 | 18 Nov 09:07:45.150000 / 46.91900 | 18 Nov 11:20:56.576000 / 45.85800 | 47.68400 / 45.86300 | protective TP | 159.15 / 1.386928 |
| 47790949 / 264117915 | XAGUSD / sell | 15000 / 0.0300 | 18 Nov 09:07:45.406000 / 46.92100 | 18 Nov 11:21:58.124000 / 45.76300 | 47.75600 / 45.76800 | protective TP | 173.70 / 1.386826 |
| 46130058 / 264140510 | BTCUSD / sell | 26 / 0.2600 | 18 Nov 10:12:12.233000 / 64,364.07000 | 18 Nov 12:00:01.032000 / 64,406.76000 | 64,944.02000 / 63,563.73000 | session end | -11.10 / -0.073614 |
| 47790949 / 264140769 | BTCUSD / sell | 25 / 0.2500 | 18 Nov 10:13:07.787000 / 64,384.91000 | 18 Nov 12:00:00.951000 / 64,406.76000 | 64,964.79000 / 63,584.67000 | session end | -5.46 / -0.037663 |
| 47790949 / 264176404 | NAS100 / sell | 60 / 0.6000 | 18 Nov 12:14:34.836000 / 27,171.80000 | 18 Nov 15:00:01.957000 / 27,140.70000 | 27,401.60000 / 26,854.60000 | session end | 18.66 / 0.135335 |
| 46130058 / 264176446 | NAS100 / sell | 60 / 0.6000 | 18 Nov 12:14:40.753000 / 27,175.70000 | 18 Nov 15:00:01.947000 / 27,140.70000 | 27,405.50000 / 26,858.50000 | session end | 21.00 / 0.152306 |
| 47790949 / 264192158 | SpotBrent / sell | 25200 / 2.5200 | 18 Nov 13:19:13.803000 / 94.47100 | 18 Nov 14:47:31.141000 / 93.64700 | 95.06800 / 93.64700 | protective TP | 207.65 / 1.380248 |
| 46130058 / 264192194 | SpotBrent / sell | 25300 / 2.5300 | 18 Nov 13:19:21.647000 / 94.46100 | 18 Nov 14:47:32.421000 / 93.63700 | 95.05800 / 93.63700 | protective TP | 208.47 / 1.380221 |
| 47790949 / 264263041 | XAGUSD / sell | 50000 / 0.1000 | 18 Nov 15:50:55.729000 / 45.90200 | 18 Nov 16:28:01.493000 / 45.90200 | 46.20100 / 45.48900 | BE stop | 0.00 / 0.000000 |
| 46130058 / 264263042 | XAGUSD / sell | 65000 / 0.1300 | 18 Nov 15:50:55.808000 / 45.90200 | 18 Nov 16:28:01.541000 / 45.90200 | 46.13100 / 45.58500 | BE stop | 0.00 / 0.000000 |
| 47790949 / 264432765 | BTCUSD / sell | 53 / 0.5300 | 18 Nov 22:30:20.462000 / 63,939.80000 | 18 Nov 22:34:56.040000 / 63,518.16000 | 64,225.73000 / 63,545.21000 | protective TP | 223.47 / 1.474632 |
| 46130058 / 264432991 | BTCUSD / sell | 53 / 0.5300 | 18 Nov 22:30:39.195000 / 63,941.46000 | 18 Nov 22:34:56.058000 / 63,518.16000 | 64,226.91000 / 63,547.53000 | protective TP | 224.35 / 1.482928 |
| 46130058 / 264438594 | XAGUSD / sell | 30000 / 0.0600 | 18 Nov 22:39:43.469000 / 45.33600 | 19 Nov 02:59:50.351000 / 45.34800 | 45.77700 / 44.72700 | BE stop | -3.60 / -0.027211 |
| 49236051 / 264450816 | SpotBrent / sell | 500 / 0.0500 | 18 Nov 23:03:03.552000 / 93.84100 | 19 Nov 00:57:49.654000 / 92.57200 | 94.75900 / 92.57400 | protective TP | 6.35 / 1.383442 |
| 46130058 / 264450935 | SpotBrent / sell | 16800 / 1.6800 | 18 Nov 23:03:16.753000 / 93.82700 | 19 Nov 00:57:50.724000 / 92.54700 | 94.74500 / 92.56000 | protective TP | 215.04 / 1.394336 |
| 47790949 / 264451099 | SpotBrent / sell | 16600 / 1.6600 | 18 Nov 23:03:39.390000 / 93.82700 | 19 Nov 00:57:50.663000 / 92.54700 | 94.74500 / 92.56000 | protective TP | 212.48 / 1.394336 |
| 47790949 / 264481512 | BTCUSD / sell | 28 / 0.2800 | 19 Nov 00:25:19.848000 / 62,558.47000 | 19 Nov 01:23:54.717000 / 63,104.10000 | 63,106.03000 / 61,802.83000 | initial stop | -152.78 / -0.996499 |
| 46130058 / 264481564 | BTCUSD / sell | 28 / 0.2800 | 19 Nov 00:25:29.074000 / 62,555.28000 | 19 Nov 01:23:52.682000 / 63,101.56000 | 63,102.84000 / 61,799.64000 | initial stop | -152.96 / -0.997673 |
| 47790949 / 264484937 | NAS100 / sell | 70 / 0.7000 | 19 Nov 00:34:02.922000 / 26,978.00000 | 19 Nov 01:26:09.811000 / 27,197.30000 | 27,196.90000 / 26,675.90000 | initial stop | -153.51 / -1.001827 |
| 46130058 / 264484944 | NAS100 / sell | 70 / 0.7000 | 19 Nov 00:34:04.616000 / 26,979.30000 | 19 Nov 01:26:14.628000 / 27,201.20000 | 27,198.20000 / 26,677.20000 | initial stop | -155.33 / -1.013705 |
| 49236051 / 264496289 | US500 / sell | 10 / 0.1000 | 19 Nov 01:03:49.031000 / 7,313.90000 | 19 Nov 05:57:01.326000 / 7,318.70000 | 7,359.60000 / 7,250.80000 | session end | -0.48 / -0.105033 |
| 47790949 / 264505974 | XAGUSD / sell | 20000 / 0.0400 | 19 Nov 01:28:04.639000 / 45.17600 | 19 Nov 05:57:01.731000 / 45.06700 | 45.88000 / 44.20400 | session end | 21.80 / 0.154830 |

| Position | Held MFE / MAE R | BE amendment receipt SGT | Exit deadline SGT | Deadline policy |
| --- | --- | --- | --- | --- |
| 264117913 | 1.383007 / 0.149020 | 18 Nov 10:07:02.252798 | 18 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264117915 | 1.383234 / 0.134132 | 18 Nov 10:17:33.836446 | 18 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264140510 | 0.458884 / 0.366652 | None | 18 Nov 12:00:00.000000 | native_preclose_v18455_120s |
| 264140769 | 0.494878 / 0.330758 | None | 18 Nov 12:00:00.000000 | native_preclose_v18455_120s |
| 264176404 | 0.452567 / 0.184073 | None | 18 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264176446 | 0.469539 / 0.167102 | None | 18 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264192158 | 1.380235 / 0.242881 | 18 Nov 14:35:50.682779 | 18 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264192194 | 1.380235 / 0.259631 | 18 Nov 14:36:09.041205 | 18 Nov 15:00:00.000000 | native_preclose_v18455_120s |
| 264263041 | 1.006689 / 0.605351 | 18 Nov 16:13:55.601093 | 18 Nov 22:30:00.000000 | native_preclose_v18455_120s |
| 264263042 | 1.314410 / 0.790393 | 18 Nov 16:13:39.256012 | 18 Nov 22:30:00.000000 | native_preclose_v18455_120s |
| 264432765 | 1.428112 / 0.308957 | 18 Nov 22:34:12.296504 | 19 Nov 00:00:00.000000 | native_preclose_v18455_120s |
| 264432991 | 1.436329 / 0.303661 | 18 Nov 22:34:12.296901 | 19 Nov 00:00:00.000000 | native_preclose_v18455_120s |
| 264438594 | 1.267574 / 0.154195 | 19 Nov 00:20:42.555607 | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264450816 | 1.387800 / 0.470588 | 19 Nov 00:53:25.017810 | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264450935 | 1.394336 / 0.485839 | 19 Nov 00:53:26.535646 | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264451099 | 1.394336 / 0.485839 | 19 Nov 00:53:26.676004 | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264481512 | 0.643966 / 1.000785 | None | 19 Nov 04:00:00.000000 | native_preclose_v18455_120s |
| 264481564 | 0.638140 / 1.000091 | None | 19 Nov 04:00:00.000000 | native_preclose_v18455_120s |
| 264484937 | 0.592965 / 1.005482 | None | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264484944 | 0.598904 / 1.017360 | None | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264496289 | 0.056893 / 0.995624 | None | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 264505974 | 0.498580 / 0.247159 | None | 19 Nov 05:57:00.000000 | native_preclose_v18455_120s |


Held excursions are observed executable-side movements from entry while the position remained open. They are not extrapolated across missing quotes. A protective stop can fill at a better price than its stop level; classification follows recorded broker order/protection provenance rather than requiring a fill worse than the stop. The retained BTC cases are checked under this rule, not mislabeled as unknown exits.


| Account | Latest20 wins / closes | Win rate | Rolling Net R | Payoff ratio (R) | Profit factor (R) | Assessment SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | -5.967682 | 3.4277 | 0.6049 | 19 Nov 08:10:12.818199 |
| 46130058 | 6/20 | 30.00% | -1.581272 | 1.7346 | 0.8006 | 19 Nov 08:11:21.814449 |
| 47790949 | 7/20 | 35.00% | -1.525967 | 1.3873 | 0.8093 | 19 Nov 08:09:29.241961 |
| 49236051 | 12/20 | 60.00% | 6.521042 | 1.4542 | 2.1814 | 19 Nov 08:11:34.497423 |


Live's 3/20 is a recorded legacy rolling mix that includes 19 October positions not independently re-exported for this audit; it is not a current-policy performance proof. The three demo latest-20 Net R and profit-factor values are recomputed from the current and retained prior ledgers.

The fixed target is **15/20 (75%)**, alongside the original 1:1.38 planning goal. Rolling net R, payoff and profit factor include realised outcomes and can move differently from the win count as older trades leave the sample. Account copies of one market setup are correlated. The cash recovery in this window alone does not establish a predictive edge or qualify a changed entry/exit policy.

## §6. Mandatory four-hour post-close excursion audit

All tracks stop at the earlier of exit+4h and **19 November 08:13:15 SGT**. Price/pip/R movement is signed in the original trade direction from the exit fill. Long liquidation uses the bid; short liquidation uses the ask. Original absolute TP and SL crossings are recorded separately. The explicit broker conventions used are BTC/ETH/Gold 0.1, Brent/Crude/Silver 0.01, and index pip sizes from the retained symbol specification. The per-row pip size is included below.


| Position / symbol / scope | Pip size | Exit / original SL / TP | Hours / status | Max favourable price / pips / R | Max adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 264117913 / XAGUSD / current | 0.01 | 45.85800 / 47.68400 / 45.86300 | 4.000000 COMPLETE | 0.10400 / 10.40 / 0.135948 | 0.57900 / 57.90 / 0.756863 | -16.90 / -0.220915 | -27.10 / -0.354248 | -11.40 / -0.149020 | True / False |
| 264117915 / XAGUSD / current | 0.01 | 45.76300 / 47.75600 / 45.76800 | 4.000000 COMPLETE | 0.00900 / 0.90 / 0.010778 | 0.67400 / 67.40 / 0.807186 | -25.60 / -0.306587 | -38.80 / -0.464671 | -16.90 / -0.202395 | True / False |
| 264140510 / BTCUSD / current | 0.1 | 64,406.76000 / 64,944.02000 / 63,563.73000 | 4.000000 COMPLETE | 187.52000 / 1,875.20 / 0.323338 | 514.59000 / 5,145.90 / 0.887301 | 296.10 / 0.051056 | -1,963.00 / -0.338477 | -3,706.10 / -0.639038 | False / False |
| 264140769 / BTCUSD / current | 0.1 | 64,406.76000 / 64,964.79000 / 63,584.67000 | 4.000000 COMPLETE | 187.52000 / 1,875.20 / 0.323377 | 514.59000 / 5,145.90 / 0.887408 | 296.10 / 0.051062 | -2,137.00 / -0.368525 | -3,769.60 / -0.650066 | False / False |
| 264176404 / NAS100 / current | 1 | 27,140.70000 / 27,401.60000 / 26,854.60000 | 4.000000 COMPLETE | 5.00000 / 5.00 / 0.021758 | 203.80000 / 203.80 / 0.886858 | -165.80 / -0.721497 | -113.90 / -0.495648 | -176.60 / -0.768494 | False / False |
| 264176446 / NAS100 / current | 1 | 27,140.70000 / 27,405.50000 / 26,858.50000 | 4.000000 COMPLETE | 5.00000 / 5.00 / 0.021758 | 203.80000 / 203.80 / 0.886858 | -165.80 / -0.721497 | -113.90 / -0.495648 | -178.80 / -0.778068 | False / False |
| 264192158 / SpotBrent / current | 0.01 | 93.64700 / 95.06800 / 93.64700 | 4.000000 COMPLETE | 0.90500 / 90.50 / 1.515913 | 0.52200 / 52.20 / 0.874372 | 3.40 / 0.056951 | 57.20 / 0.958124 | -30.60 / -0.512563 | True / False |
| 264192194 / SpotBrent / current | 0.01 | 93.63700 / 95.05800 / 93.63700 | 4.000000 COMPLETE | 0.89500 / 89.50 / 1.499162 | 0.53200 / 53.20 / 0.891122 | 2.40 / 0.040201 | 55.40 / 0.927973 | -32.10 / -0.537688 | True / False |
| 264263041 / XAGUSD / current | 0.01 | 45.90200 / 46.20100 / 45.48900 | 4.000000 COMPLETE | 0.29100 / 29.10 / 0.973244 | 0.12200 / 12.20 / 0.408027 | -1.20 / -0.040134 | 18.40 / 0.615385 | 13.10 / 0.438127 | False / False |
| 264263042 / XAGUSD / current | 0.01 | 45.90200 / 46.13100 / 45.58500 | 4.000000 COMPLETE | 0.29100 / 29.10 / 1.270742 | 0.12200 / 12.20 / 0.532751 | -1.20 / -0.052402 | 18.40 / 0.803493 | 13.10 / 0.572052 | False / False |
| 264432765 / BTCUSD / current | 0.1 | 63,518.16000 / 64,225.73000 / 63,545.21000 | 4.000000 COMPLETE | 1,312.30000 / 13,123.00 / 4.589585 | 368.26000 / 3,682.60 / 1.287938 | -517.50 / -0.180988 | 10,340.60 / 3.616480 | 4,028.80 / 1.409016 | True / False |
| 264432991 / BTCUSD / current | 0.1 | 63,518.16000 / 64,226.91000 / 63,547.53000 | 4.000000 COMPLETE | 1,312.30000 / 13,123.00 / 4.597303 | 368.26000 / 3,682.60 / 1.290103 | -519.50 / -0.181993 | 10,340.60 / 3.622561 | 4,028.80 / 1.411386 | True / False |
| 264438594 / XAGUSD / current | 0.01 | 45.34800 / 45.77700 / 44.72700 | 4.000000 COMPLETE | 0.52300 / 52.30 / 1.185941 | 0.00200 / 0.20 / 0.004535 | 43.60 / 0.988662 | 26.50 / 0.600907 | stale_asof_quote | False / False |
| 264450816 / SpotBrent / current | 0.01 | 92.57200 / 94.75900 / 92.57400 | 4.000000 COMPLETE | 0.47800 / 47.80 / 0.520697 | 1.29700 / 129.70 / 1.412854 | -16.60 / -0.180828 | -58.70 / -0.639434 | -71.10 / -0.774510 | True / False |
| 264450935 / SpotBrent / current | 0.01 | 92.54700 / 94.74500 / 92.56000 | 4.000000 COMPLETE | 0.45300 / 45.30 / 0.493464 | 1.32200 / 132.20 / 1.440087 | -18.60 / -0.202614 | -61.20 / -0.666667 | -73.60 / -0.801743 | True / False |
| 264451099 / SpotBrent / current | 0.01 | 92.54700 / 94.74500 / 92.56000 | 4.000000 COMPLETE | 0.45300 / 45.30 / 0.493464 | 1.32200 / 132.20 / 1.440087 | -18.60 / -0.202614 | -61.20 / -0.666667 | -73.60 / -0.801743 | True / False |
| 264481512 / BTCUSD / current | 0.1 | 63,104.10000 / 63,106.03000 / 61,802.83000 | 4.000000 COMPLETE | 776.22000 / 7,762.20 / 1.417598 | 389.81000 / 3,898.10 / 0.711904 | -497.10 / -0.090785 | 3,354.60 / 0.612645 | 6,373.50 / 1.163982 | False / True |
| 264481564 / BTCUSD / current | 0.1 | 63,101.56000 / 63,102.84000 / 61,799.64000 | 4.000000 COMPLETE | 773.68000 / 7,736.80 / 1.412959 | 392.35000 / 3,923.50 / 0.716542 | -398.90 / -0.072850 | 3,345.30 / 0.610947 | 6,348.00 / 1.159325 | False / True |
| 264484937 / NAS100 / current | 1 | 27,197.30000 / 27,196.90000 / 26,675.90000 | 4.000000 COMPLETE | 188.20000 / 188.20 / 0.859753 | 44.20000 / 44.20 / 0.201919 | 89.50 / 0.408862 | 93.90 / 0.428963 | 45.40 / 0.207401 | False / True |
| 264484944 / NAS100 / current | 1 | 27,201.20000 / 27,198.20000 / 26,677.20000 | 4.000000 COMPLETE | 192.10000 / 192.10 / 0.877570 | 40.30000 / 40.30 / 0.184102 | 88.30 / 0.403381 | 101.80 / 0.465053 | 51.30 / 0.234354 | False / True |
| 264496289 / US500 / current | 1 | 7,318.70000 / 7,359.60000 / 7,250.80000 | 2.270465 PARTIAL | 0.50000 / 0.50 / 0.010941 | 46.90000 / 46.90 / 1.026258 | stale_asof_quote | -35.90 / -0.785558 | not_elapsed_at_capture | False / True |
| 264505974 / XAGUSD / current | 0.01 | 45.06700 / 45.88000 / 44.20400 | 2.270352 PARTIAL | 0.02200 / 2.20 / 0.031250 | 0.51500 / 51.50 / 0.731534 | stale_asof_quote | -41.00 / -0.582386 | not_elapsed_at_capture | False / False |
| 263762252 / NAS100 / prior | 1 | 27,019.20000 / 27,300.90000 / 26,280.30000 | 4.000000 COMPLETE | 0.00000 / 0.00 / 0.000000 | 509.70000 / 509.70 / 1.188666 | stale_asof_quote | -321.40 / -0.749534 | -79.30 / -0.184935 | False / True |
| 263762285 / NAS100 / prior | 1 | 27,019.20000 / 27,303.30000 / 26,282.70000 | 4.000000 COMPLETE | 0.00000 / 0.00 / 0.000000 | 509.70000 / 509.70 / 1.188666 | stale_asof_quote | -321.40 / -0.749534 | -79.30 / -0.184935 | False / True |
| 264006264 / XAGUSD / prior | 0.01 | 47.40600 / 47.71700 / 46.70700 | 4.000000 COMPLETE | 0.94200 / 94.20 / 2.221698 | 0.23900 / 23.90 / 0.563679 | stale_asof_quote | -10.80 / -0.254717 | 76.40 / 1.801887 | True / False |

| Scope | Rows | Complete | Partial | Initial stops later reaching original TP | BE stops later reaching original TP |
| --- | --- | --- | --- | --- | --- |
| Current activity | 22 | 20 | 2 | 0 | 0 |
| Prior follow-ups | 3 | 3 | 0 | 0 | 0 |
| Combined path audit | 25 | 23 | 2 | 0 | 0 |


The three old NAS100/Silver tracks are complete and their old cash is not recounted. The Thursday 05:57 closes remain partial at this horizon; their full four-hour clocks finish around 09:57 SGT. A clock-complete track can still contain the daily maintenance gap. Missing/stale 1h/2h/4h quotes are marked unavailable; no interpolation or hypothetical fill is substituted. The three prior follow-up rows have stale 1h checkpoints across maintenance. The two 05:57 partial rows also have stale 1h checkpoints and have not reached 4h. Silver BE position 264438594 has a clock-complete 4h window but its 4h quote checkpoint lies in maintenance, with the last quote roughly 61 minutes old, so that checkpoint remains unavailable. **Clock-complete does not mean uninterrupted quote coverage.**

The requested three Silver BE exits are isolated here:


| Position | Exit SGT | Net USD | Original TP later crossed? | First original TP crossing SGT | 4h status |
| --- | --- | --- | --- | --- | --- |
| 264263041 | 18 Nov 16:28:01.493000 | 0.00 | False | N/A | Complete |
| 264263042 | 18 Nov 16:28:01.541000 | 0.00 | False | N/A | Complete |
| 264438594 | 19 Nov 02:59:50.351000 | -3.60 | False | N/A | Complete |


A later TP crossing after a stop is path evidence, not realised profit or proof that retaining the original trade would have been safe. Its original SL may already have been crossed. Initial-stop and BE-stop counts are disjoint; the two zero-cash BE exits are not positive-net wins. The current exit rules remain unchanged.

## §7. Execution/veto funnel and telemetry integrity


| Metric | Measured count / value |
| --- | --- |
| new_arms | 163 |
| consumed_triggers | 33 |
| terminal_outcomes | 33 |
| pre_intent_refusals | 1 |
| persisted_opening_intents | 32 |
| entries_dispatched | 22 |
| terminal_refusal_receipts | 11 |
| deep_retracement_invalidations | 76 |
| pair_reservation_diagnostics | 319 |
| consumed_triggers_without_individual_outcome | 0 |
| unknown_entry_guard_receipts | 0 |
| confirmation_to_guard_max_ms | 35.528 |

| Terminal refusal reason | Count |
| --- | --- |
| ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction | 6 |
| prepared admission expired; background refresh required | 1 |
| ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse | 4 |


The full consumed-trigger→opening-intent→terminal-outcome joins are retained in JSON. **33 consumed triggers have 33 attributable outcomes: 22 dispatched fills and 11 refusals.** There are only **32 persisted opening intents** because one expired-admission refusal occurred before intent creation; it is still counted as its trigger's terminal outcome. This is not a missing receipt. Missing individual outcomes are **0**, unknown guard receipts **0**. Confirmation-to-guard timing measures local confirmation through the final guard, not broker send-to-fill or network latency. The 319 pair-reservation rows are recorded diagnostics, not 319 distinct consumed-trigger refusals. Deduplicated idle/pre-arm diagnostics do not represent an exhaustive hypothetical-trade census; repeated diagnostic rows are not independent attempted entries.


| Account | Symbol | Gate in separately timed current snapshot |
| --- | --- | --- |
| 43002148 | BTCUSD | Invalidated: retraced > 61.8% without turn confirmation |
| 43002148 | ETHUSD | Waiting: opening impulse below 48 bps |
| 43002148 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.540%, 20d: -8.913%) |
| 43002148 | SpotBrent | Blocked: outside active native broker session |
| 43002148 | SpotCrude | Waiting: opening impulse below 48 bps |
| 43002148 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.055%, 20d: -5.145%) |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Waiting: current decision mark already consumed |
| 46130058 | ETHUSD | Waiting: opening impulse below 48 bps |
| 46130058 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.540%, 20d: -8.913%) |
| 46130058 | SpotBrent | Blocked: outside active native broker session |
| 46130058 | SpotCrude | Waiting: opening impulse below 48 bps |
| 46130058 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.055%, 20d: -5.145%) |
| 46130058 | XAGUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -10.947%, 20d: -20.486%) |
| 46130058 | XAUUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.409%, 20d: -8.783%) |
| 47790949 | BTCUSD | Waiting: current decision mark already consumed |
| 47790949 | ETHUSD | Waiting: opening impulse below 48 bps |
| 47790949 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.540%, 20d: -8.913%) |
| 47790949 | SpotBrent | Blocked: outside active native broker session |
| 47790949 | SpotCrude | Waiting: opening impulse below 48 bps |
| 47790949 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.055%, 20d: -5.145%) |
| 47790949 | XAGUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -10.947%, 20d: -20.486%) |
| 47790949 | XAUUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.409%, 20d: -8.783%) |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.540%, 20d: -8.913%) |
| 49236051 | SpotBrent | Blocked: outside active native broker session |
| 49236051 | SpotCrude | Waiting: opening impulse below 48 bps |
| 49236051 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: -3.055%, 20d: -5.145%) |
| 49236051 | XAGUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -10.947%, 20d: -20.486%) |
| 49236051 | XAUUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -5.409%, 20d: -8.783%) |


## §8. Session openings, London DST impact and Friday countdown


### Opening references: distinguish recorded quotes from usable actor references

All 128 requested account-symbol opening slots have a recorded two-sided boundary quote. A matching stored full quote receipt proves the actor reference for 101 slots; 11 more are corroborated only by a later band midpoint, not the full bid/ask receipt. 15 have no matching operational reference evidence and 1 has a recorded opening refusal despite an in-window quote. Recorder coverage is distinct from admission readiness.

| Session SGT | Slots / recorded quotes | Exact actor quote receipt | Midpoint-only corroboration | Recorded opening refusal | No matching reference evidence |
|---|---:|---:|---:|---:|---:|
| 2026-11-18 08:00 | 8/8 | 6 | 0 | 0 | 2 |
| 2026-11-18 09:00 | 4/4 | 4 | 0 | 0 | 0 |
| 2026-11-18 12:00 | 8/8 | 6 | 0 | 0 | 2 |
| 2026-11-18 15:00 | 24/24 | 14 | 10 | 0 | 0 |
| 2026-11-18 16:00 | 8/8 | 6 | 0 | 0 | 2 |
| 2026-11-18 20:00 | 8/8 | 6 | 0 | 0 | 2 |
| 2026-11-18 22:30 | 24/24 | 24 | 0 | 0 | 0 |
| 2026-11-19 00:00 | 8/8 | 6 | 0 | 0 | 2 |
| 2026-11-19 04:00 | 8/8 | 5 | 1 | 0 | 2 |
| 2026-11-19 07:00 | 20/20 | 18 | 0 | 1 | 1 |
| 2026-11-19 08:00 | 8/8 | 6 | 0 | 0 | 2 |

At the separately timestamped health snapshot, 49236051 ETH is held under `yield_to_human_position_or_order` and BTC under the ETH pair reservation. Those 14 historical Crypto slots without an opening reference must not be presented as 14 successful actor captures. Live Gold is likewise human-locked; its current stored reference remains the preceding NY session.

**Thursday 07:00 Asian:** 20/20 base/full preparation receipts and 20/20 recorded boundary quotes, but 18 exact current operational references. Live Silver is an explicit exception: its preparation completed 07:01:00.572138; the first stored valid quote arrived 07:01:00.620447; receipt 377976 at 07:01:00.883493 reported `opening_midpoint_unavailable`. Health still shows no Silver midpoint, alongside the separate human-Gold pair lock. The early preparation versus later quote timing is consistent with an early assessment/visibility problem, but retained receipts do not uniquely identify the race. This is reported for investigation, not silently counted as ready or repaired.

SpotCrude first quotes were recorded within the 15-second policy window on all four accounts. Exact per-account delays and reference evidence are retained in the JSON. Metals correctly use 07:01 observation start; their delay is not measured from 07:00.

### Thursday preparation and London DST countdown

| Thursday session | Scheduled slots | Base receipts ready | Full tables ready | Missing receipts |
|---|---:|---:|---:|---:|
| 07:00 | 20 | 20 | 20 | 0 |
| 08:00 | 8 | 8 | 8 | 0 |
| 09:00 | 4 | 4 | 4 | 0 |
| 12:00 | 8 | 8 | 8 | 0 |
| 15:00 | 24 | 20 | 0 | 4 |

At 08:13 cutoff, **London is 20/24 base and 0/24 full**, not the directive’s assumed 24/24 base. Four future Brent London receipts remain absent while the current/next preparation covers its 09:00 window. Every one of the 20 present London receipts reports exactly `13 same-slot endpoints;14 needed` at 420m and 450m. No earlier short-session bar is fabricated to fill the gap.

The calendar independently reproduces the count: Wednesday had 12 eligible winter-length weekdays (2–17 Nov); Thursday has 13 (2–18 Nov). The previous US DST regime ended London at 21:30 SGT; those older sessions cannot supply the new 420/450-minute endpoints because `bar_time >= prior_session.close` is excluded. The receipts indicate **zero missing eligible dates**, rather than missing arbitrary database history.

Friday 20 Nov 15:00 is the earliest calendar opportunity for 14/14, conditional on usable Thursday closed bars and fresh Friday preparation. Friday full readiness is a forecast, not an observed fact.

### Recorded Wednesday late-London effect

**20 deduplicated `no_band` assessment receipts across 20 account-symbol pairs** explicitly report `12 same-slot endpoints;14 needed` during `[18 Nov 21:30, 22:30) SGT`. Counts: Live 43002148: 4; demo 46130058: 5; demo 47790949: 5; demo 49236051: 6. The latest coefficient failure therefore did reach the runtime assessment path. No new traditional arm was recorded in that interval, but this does not establish how many profitable setups would otherwise have existed.

One already-armed 49236051 Silver path continued: 3 running-extreme extensions, 14 pullback-zone touches, then 1 deep-retracement invalidation. Thus the missing late coefficients blocked new band assessments; they did not universally cancel all earlier armed state. The number of counterfactual skipped trades is **unverified**, and no lost-P&L estimate is assigned.

All preparation queries used the fixed 08:13 cutoff; later health snapshots are labelled separately. Read-only transactions, statement bounds, receipt hashes, exact slot rows, source commit hashes, and focused arithmetic/calendar checks are included in the JSON.


## §9. Assessment and next handoff

The evidence separates execution hygiene from strategy performance. Buffered exits and per-trigger accounting can function correctly while an account remains below the rolling win/expectancy target. Likewise, the live broker's human-position partial stop-outs and S1 manual exits are not S3 losses or bot interference merely because they appear in the same account history.

The cash and path tables support review of the unchanged baseline. They do not authorise crypto removal, a new BE offset, altered stops, changes to ownership rules, or any production deployment. The late-London DST observations must be assessed as measured missing coefficients and recorded skips; absence of exhaustive skipped-arm telemetry is not proof of zero missed opportunities.

The next activity window begins exactly **19 November 08:13:00 SGT**. Carry only the incomplete excursion IDs listed below, using their original risk and fills. Verify the nearer 09:00 Brent opening before assuming all later London preparations exist; confirm Friday's 14th eligible winter-length endpoint only after the required Thursday bars and new Friday receipts are available.


| Incomplete position | Symbol | Current elapsed hours | Four-hour endpoint SGT |
| --- | --- | --- | --- |
| 264496289 | US500 | 2.270465 | 19 Nov 09:57:01.326000 |
| 264505974 | XAGUSD | 2.270352 | 19 Nov 09:57:01.731000 |


## §10. Evidence, validation and Gemini handoff

Sources are retained broker closing deals, labelled S3/S1 orders and fills, account cashflow receipts, current owner book/margin assessments, bounded read-only broker P&L, trigger/intent/outcome joins, saved native opening/preparation receipts, and bounded copies of existing PostgreSQL quotes. Exact source hashes, read-only queries, calculation scripts and tests are included in JSON. No new historical market data was requested and no broker order, live setting, protected source or deployment was changed.

Only the three requested Markdown, JSON and HTML files under `doc/` are publication changes. Required CI is `check`, `docker`, `engineering`. The PR records the actual merge SHA and checks; post-merge deployment and actor continuity are verified separately by the publishing agent. A green documentation PR is not financial qualification.

**Gemini:** begin with §§1–2 and preserve PR #240's 07:48 handoff. Reconcile window/full-Wednesday/Thursday/cumulative S3 cash and keep the deposits, human Gold closes, JPN225 broker stop-outs and S1 outcomes separate. Inspect the full trade and 4h tables, the three Silver BE tracks, exact 05:57 send/fills and rollover evidence. Do not replace 20/24 observed London preparation with the prompt's expected 24/24 or treat conditional Friday self-healing as already verified. Review the JSON completeness flags and later money-snapshot times before proposing any change.


| Tick export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-BTCUSD-2026-11-18-021206.csv.gz | 59223 | 7dcb2f2988df343ed8b755c2d0273a5715d693f1159c40bf2368e626dca282e6 |
| 46130058-BTCUSD-2026-11-18-061206.csv.gz | 25038 | a3a6145ec6bce61f54ae09db6601115148f08f3458f03056c5dc28dc79806073 |
| 46130058-BTCUSD-2026-11-18-143033.csv.gz | 64943 | 7c37e1bfc671098ba381c838ee7af94601ee619e188463a9524e3444b733f5dd |
| 46130058-BTCUSD-2026-11-18-183033.csv.gz | 42350 | 7676c710e185107b1aa49b717fdeb9ccb4986fac44a330eb2f4d42e0ec7c2f49 |
| 46130058-NAS100-2026-11-17-215655.csv.gz | 42075 | a84b9c2c139b21c4ba130e8a37c0c7b985f0b5ab0e45addc4c6f7d84fc7f4ad3 |
| 46130058-NAS100-2026-11-18-015655.csv.gz | 27 | f2ce479be625fbfdf6f119959d0ca7626d47c8916ae83d41073254b2b2a99e99 |
| 46130058-NAS100-2026-11-18-041434.csv.gz | 54151 | 43359ad8927b54747398b3c04f42c7a1bbd22cc943db93202e9c26848bcd3a16 |
| 46130058-NAS100-2026-11-18-081434.csv.gz | 38770 | 8fc837e74ed47f402c32ab51e03696ac10236781fdf52ae1e722c6022621a96c |
| 46130058-NAS100-2026-11-18-163358.csv.gz | 63884 | 0d509e8922f7a73ae13e5d65067a564435b23de152951a8c1d3673cef1a91155 |
| 46130058-NAS100-2026-11-18-203358.csv.gz | 12734 | 65ab5f486388c547c37c7bae334126fe8979dcb728dce01319d4fe32f5e67fa8 |
| 46130058-SpotBrent-2026-11-18-051915.csv.gz | 14380 | 6f74f9dd411b69f63ae98b5de9651b689d39a4d124b02c31ddb8bec10fc4fa8a |
| 46130058-SpotBrent-2026-11-18-091915.csv.gz | 4893 | 4d817a763f4bfdb79b2d1156c668e573991f22c51e642c373c1ae32e20045228 |
| 46130058-SpotBrent-2026-11-18-150310.csv.gz | 19096 | b2cc50d2f5feac6363ed4dfef060d104e05700bc5f88334324c73d58228b6a87 |
| 46130058-SpotBrent-2026-11-18-190310.csv.gz | 4758 | 1022690b853feaccd9dc87184b2b450443dec0a196924538d09810a698ac342c |
| 46130058-XAGUSD-2026-11-18-010739.csv.gz | 41966 | 17efd6c3b4fe0159f5b9d09af3c2f464cd8dafc1b2cd41d406cb338f77158058 |
| 46130058-XAGUSD-2026-11-18-050739.csv.gz | 24889 | 888f7109332853c69efc7aa86539f6ddb2a49fe06971665172214beafb012c4f |
| 46130058-XAGUSD-2026-11-18-075049.csv.gz | 32183 | f1f0ad1095bbcfa0952c93ec566ff32f068ba6715b22d120d34dff1886cecb9a |
| 46130058-XAGUSD-2026-11-18-115049.csv.gz | 4304 | b2a11f77ecd0dffea8f679b7bf29409492e865981c0c725b59aa1005375d5e46 |
| 46130058-XAGUSD-2026-11-18-143937.csv.gz | 43895 | 62da3a82d5706f419de6d01ec10754b443013bf51f6a15e5cef4a15a23937c8d |
| 46130058-XAGUSD-2026-11-18-183937.csv.gz | 24216 | d9b49555a1cbcc0ba64344855aebdb4d752a7f6486c016b0bf68d7f1f72616a7 |
| 46130058-XAGUSD-2026-11-18-223937.csv.gz | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| 47790949-BTCUSD-2026-11-18-021301.csv.gz | 59168 | 075516d01195962dbd3c78d2564ef8df215af2a761edfbfcaac82fab9411c636 |
| 47790949-BTCUSD-2026-11-18-061301.csv.gz | 24845 | 2fd882328b8d871f4d1b50b12dc62fc81d6c074d0dda3df38facf57643d0a60c |
| 47790949-BTCUSD-2026-11-18-143014.csv.gz | 64951 | d47216c3a4fdfec416b2d1719f9ee2ce318fb2fc38c4f429ee73b4c29b56dffb |
| 47790949-BTCUSD-2026-11-18-183014.csv.gz | 42434 | ccabb2eff3e7f4018b453eb271b119ca4a402c6635ed5a55074df2526899e063 |
| 47790949-NAS100-2026-11-17-215655.csv.gz | 42075 | 41ab69a3b51c96923a79190f522a65147178f4e93c0ca436edd074dc642ba2a8 |
| 47790949-NAS100-2026-11-18-015655.csv.gz | 27 | 5da5316df7d2336d78cb680454843c5c87e75693a16e724125162a9a37d1a155 |
| 47790949-NAS100-2026-11-18-041428.csv.gz | 54151 | 840ed85f2ef4193e0326ebc78a645ad317e105ee331f6dd22d0df85204d0820d |
| 47790949-NAS100-2026-11-18-081428.csv.gz | 38796 | a4eb2a8def319d9019ba6c3fa267704ad5b2e9e7b14d587813101dd1aee51a60 |
| 47790949-NAS100-2026-11-18-163356.csv.gz | 63885 | c91e295f44848bc757b16e451f60ee70c0f0dbf686997974397473d0631aa2ca |
| 47790949-NAS100-2026-11-18-203356.csv.gz | 12732 | e5c59caafe89f62cad1b373451869296ef7245432457d2b387ea70e4c957d246 |
| 47790949-SpotBrent-2026-11-18-051907.csv.gz | 14371 | 183d7772a6d30a969cdafd0f676e8d769b1b4055a5848da87de8d0d0a8bb5d3d |
| 47790949-SpotBrent-2026-11-18-091907.csv.gz | 4909 | 2010adbfee8f9f8e160bb7be742ad7ec4e9a1fc1deb2009eecaab56de594456d |
| 47790949-SpotBrent-2026-11-18-150333.csv.gz | 19074 | a3f2b7de52c5eb12838fe9571da70370afd7c0ad79b1c70ee54ea362bee675b1 |
| 47790949-SpotBrent-2026-11-18-190333.csv.gz | 4732 | 2e16c0a358b78dfca72f44e4c96ee1f1481c3af82191be5c702277897fbe47ac |
| 47790949-XAGUSD-2026-11-17-215656.csv.gz | 24760 | abc4ce6963be4314359054ee9ea8e1e801c43d11bc0aef742fee9aac46c85f06 |
| 47790949-XAGUSD-2026-11-18-015656.csv.gz | 40363 | 161fadba515330cc8599ca12f0b574ffc4def5b1f4ca03358e3743dc4631a72d |
| 47790949-XAGUSD-2026-11-18-055656.csv.gz | 15548 | 0af281794653e7ece614c13196cacc17af9bf9a5b862eab04fd836eba91f6654 |
| 47790949-XAGUSD-2026-11-18-075049.csv.gz | 32185 | 0315617336b56febebd6d39ab6fb26f8c7683e4061ead9ca23f3ddb8151c0fa9 |
| 47790949-XAGUSD-2026-11-18-115049.csv.gz | 4304 | 9f9d93b7e7e76f9ae14bb672fb3b167e27ed20991fa86ad55b759b0d498383d4 |
| 47790949-XAGUSD-2026-11-18-172758.csv.gz | 34227 | 4ae14529e2f0398fa7563586e3aede29e202be20f110099b12e7fddba9facdb9 |
| 47790949-XAGUSD-2026-11-18-212758.csv.gz | 8037 | ea1922bc5dfe96ee2d74c20b1e89da387b16787af3c5a64aaf509484623be0b1 |
| 49236051-SpotBrent-2026-11-18-150257.csv.gz | 19105 | 29dc755f86ce1db37f4980899ab2c64d2b3397f27fcd451f7648d0f72c4b007a |
| 49236051-SpotBrent-2026-11-18-190257.csv.gz | 4772 | 661d4a0bb748233b152506c1fd504e16b70b5d86e574965ea66e4419cc46a031 |
| 49236051-US500-2026-11-18-170343.csv.gz | 43624 | 96ef3ae70d490773a3c0a1c34af3f73351b3b5f98b7e64f9ff72b72e55b2852f |
| 49236051-US500-2026-11-18-210343.csv.gz | 10936 | 87d6ea18d0db314d3485777b32a36cdf615d1c70a757a3a7c8aac7049551d449 |

| Frozen input | SHA256 |
| --- | --- |
| activity.json | 3e1a64b637589309c1ea1b004fb0b89ee4c2357b2629f43e2b8e885211c85685 |
| anchor.json | 385409b7df6f051f6a3bb38e6a2f91893321dd3f35f565d8e0a6bb21d0bea85b |
| capture.json | 0adea3fa0b1d3bc58e665b0e36000d6e7432d631a688beffcfd62a8b97a30403 |
| cumulative.json | 7a258e2256d8c5898d2be3d27aa6f689c249481311f63ee58aeb5516f7a6db30 |
| day.json | d29035152264dacd9caebac1b8898eafb3a13c18f03429c9a88d9bc8416ddcec |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| evidence.json | feedd1cd0929476fab954da10f1dd06b4b5c4a09ce183fb88e8582385058a74f |
| health.json | f01af58ee36e0ba684773cff03d1ff0d6538c78fb553beb819d18ab622227b6d |
| history-proof.json | 0a15fc8ec59e3db3300c34e5eab4900d150601071979829bff203f18ef91a3f4 |
| history-reconciliation.json | 558c1b3f16733c0db91f2ba39af98710736a13e111c4f2a406fa8c239a7bb7eb |
| monitor-reconciled.json | 125f34a340d244abe9e24a4ebd058c4cea77a6cd2baaead1fe659467da2b775d |
| monitor.json | 5acc53b1c719252f0d795b800031ffd16787b8b0409269759c8a116a65da9f1f |
| preflight.json | 7ec21f102d499b806f3accdd464ad67b2ea9e1d00d93dab4c358d4a423a55ce7 |
| quote-copy-receipts.json | 6a85d8188c555f1cd5b83d4edf40f4235c6181b473651b612000c640d903e0fe |
| runtime.json | 31f50c6d25e738c8178bb7690d5cb4a456079b9984ab9e501b7da93dcb566252 |
| thursday.json | 2b323c0a644ed66aa7c972d400fbecc1150691f17a9d135438f4f44712013435 |
