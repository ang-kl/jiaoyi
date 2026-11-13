# Friday midday Asian and crypto audit, excursions and London readiness — №18,448

**Cutoff: 13 November 2026, 13:55 SGT. Activity [04:55, 13:55) SGT**, continuing from [№18,446](FRIDAY_PREDAWN_NY_REVIEW_AND_EXCURSIONS-18446-2026-11-13.md), PR #226, merge `6fe47fc7b215db9758386941d402452617589bc8`. Post-close quote horizon is **13:55:22 SGT**; later account and execution observations are explicitly separated. Read-only production audit.

## §1. TL;DR

- **Morning-window S3 cash: +$452.94** from **11 closes: three TP wins, six BE-stop exits and two deadline losses**. Seven closes have positive net cash; four lose. Full Friday 00:00–13:55 SGT S3 cash is **+$452.20**, including the earlier −$0.74 ETH close.
- **Both carried silver shorts closed late**, around **07:01 SGT**, versus their registered **05:59 deadline**. Each had **350 broker `MARKET_CLOSED` rejections**, then filled on attempt 351. Total silver loss: **−$33.15**. Successful entry telemetry does not imply successful on-time exits.
- **Three BTC shorts remained open at 13:55**, then hit TP around **13:56:06**, realising another **+$471.64 outside this audit window**. The later broker books contain zero S3 positions. Their later wins are not added to the cutoff's cash or latest20.
- All **11 closed-position follow-ups** are included: **five complete 4.000h, six partial**. Both silver paths subsequently crossed original SL without touching TP. All six ETH BE exits subsequently touched original TP during the observed partial paths; these are two correlated episodes copied across three accounts, not six independent edge observations.
- **No live S3 fill/close; no external closing cash in-window; zero recorded interference** with the audited human inventory. Live retains two S1 positions and five external positions. Current live crypto admission explicitly reports projected margin below the 250% floor.
- Friday openings: **crude 4/4**, with first quotes about **7.05 seconds** after 07:00; traditional Asian plus Brent **22/24 operational references plus two human-locked live metals**; crypto 08:00/12:00 **16/16**.
- London 15:00 is **24/24 base-prepared but 0/24 fully covered**: every traditional table lacks 420/450-minute marks, each with 9 endpoints versus 14 required. Crypto 16:00 is **8/8 fully prepared**. Do not label London fully ready for its entire session.
- **27 consumed triggers → 27 intents → 27 individual outcomes:** 12 fills, 15 peer refusals; no missing individual outcome or unknown entry-guard receipt. Max confirmation-to-guard latency **34.354 ms**.
- Friday traditional deadline is **Saturday 14 Nov 05:55 SGT**. Crypto has a daily **05:59–06:01 SGT maintenance entry gate**; this module does not define a crypto weekend shutdown or force all crypto positions flat at 05:59.
- **Financial pass: false.** Latest20 wins at cutoff: live 3/20, demos 9/20, 10/20, 8/20. Trading code, policies and deployment are unchanged.

## §2. Evidence boundaries and account snapshot

The first execution clock was **13:55:22 SGT**, after the requested cutoff. Activity queries use the exact half-open interval; quote follow-ups are frozen at that capture time. SQL queries run read-only and use already-recorded account/deal/tick data. No historical broker collection, order action or deployment was initiated by the audit.

Money marks are **13:55:52.950–13:55:53.315 SGT**. The live inventory book is 13:55:51; demo books are 13:56:06.58–06.65, just after the three later BTC closes. Health was observed at 13:57:09. These are separate observations. The monetary snapshot still contains pre-close BTC floating P&L, whereas the later books have none. Actor tracked-position counts temporarily still show one per demo; deal receipts and broker books establish actual closure. Do not use that actor cache as an inventory ledger.

The deployed policy is `s3_macro_pullback_owner_18409_v1`, commit `aebf404dedf89dbcdb8cee78c95a582eea042d51`. Equity equals recorded balance plus broker net unrealised P&L, excluding unverified credit and prospective closing charges. Margin below is the separately timed owner-assessment projection; an aggregate broker margin level is not verified in the monetary snapshot.


| Account | Balance USD | Account float USD | Equity USD | Assessed margin % | Cutoff S3 / later-book S3 | Later S1 / S2 / external | Pending |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | -906.87 | 5,059.84 | 4,152.97 | 396.94 | 0 / 0 | 2 / 0 / 5 | 5 |
| 46130058 | 25,521.95 | 111.31 | 25,633.26 | 24,944.04 | 1 / 0 | 4 / 0 / 0 | 0 |
| 47790949 | 25,186.59 | 99.23 | 25,285.82 | 3,076.23 | 1 / 0 | 4 / 0 / 0 | 0 |
| 49236051 | 805.43 | 1.55 | 806.98 | Not verifiable / N/A | 1 / 0 | 0 / 0 / 0 | 0 |


Live balance remains **−$906.87**, with positive equity **$4,152.97** supported by **+$5,059.84** floating P&L. No external closing deal is recorded in the activity window. Current assessed margin is **396.94%**, but pending-entry projection is **191.53%** and the stress projection is **−59.48%**. The negative stress result is a scenario projection, not a claim of current negative equity or an actual margin call.

The account-level pending projection differs from the symbol-specific prepared-admission forecasts. Current live BTC/ETH gates cite **195.16%/198.00%**, below 250%. These observed reasons support the present block; they are not assigned retrospectively to every earlier non-entry.

Complete later inventory follows, including every external pending entry:


| Account | Position/order ID | Symbol | Owner / kind | Side | Native volume / lots | Entry / trigger | Current SL | Current TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Not verifiable / N/A |
| 43002148 | 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Not verifiable / N/A |
| 43002148 | 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.91300 | 3.87500 |
| 43002148 | 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,813.70000 | Not verifiable / N/A |
| 43002148 | 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,830.70000 | Not verifiable / N/A |
| 43002148 | 657180931 | NVDA.US | human_external / position | buy | 200 / 2.0000 | 165.22000 | 149.66000 | Not verifiable / N/A |
| 43002148 | 657187868 | XAUUSD | human_external / position | buy | 2500 / 0.2500 | 3,869.03800 | 3,847.97000 | 4,521.49000 |
| 43002148 | 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Not verifiable / N/A |
| 43002148 | 401488805 | XAUUSD | human_external / pending_entry | buy | 1000 / 0.1000 | 3,813.63000 | 3,786.80000 | Not verifiable / N/A |
| 43002148 | 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 | 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 | 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Not verifiable / N/A |
| 46130058 | 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,830.80000 | Not verifiable / N/A |
| 46130058 | 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69506 | Not verifiable / N/A |
| 46130058 | 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12513 | Not verifiable / N/A |
| 46130058 | 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Not verifiable / N/A |
| 47790949 | 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,830.80000 | Not verifiable / N/A |
| 47790949 | 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69501 | Not verifiable / N/A |
| 47790949 | 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12508 | Not verifiable / N/A |
| 47790949 | 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Not verifiable / N/A |


## §3. Human protection and live S1

Live human/external inventory remains JPN225 **649992251/649772867**, NatGas **637075128**, NVDA.US **657180931**, and gold **657187868**. Original gold **652821184** was closed in the prior interval and is not reopened or misclassified here. Gold additions at 04:59:18.604 and 05:01:42.905 belong to this morning's activity interval; they were already visible in the prior report's separately timed post-cutoff book. They are opening/adding deals, not new closing profits or losses.

The retained bot intent/owned-event audit includes prior and current external position/order IDs. **Recorded bot interference=0; external closing cash=0.00.** External protection/order edits are not linked to bot commands, but the bot journal does not prove which external person or application made them. Current gold TP is **4,521.49** versus 4,146.31 in the prior book. The Cocoa pending entry changed from 5,052.0 to 4,973.2 and SL from 4,925.9 to 4,852.4. Gold SL 3,847.97, JPN stops 59,000/58,500, NatGas SL 2.913 and NVDA SL 149.66 remain as previously recorded.

Law 1 is supported by distinct ownership records; Law 2 is visible in live XAUUSD yield and XAGUSD pair-reservation gates; Law 3 keeps NatGas/JPN225 reserved. Live gold's external label never grants S3 management authority. No bot instruction targets these external changes.

Live has no new S1 opening or close in-window. Its two existing S1 positions each received one recorded trailing amendment, separately from S3:


| S1 position / symbol | Long entry / lots | Previous SL | New SL | Amendment SGT | Owner |
| --- | --- | --- | --- | --- | --- |
| 656513085 / GER40 | 26,809.0 /0.1 | 25,802.9 | 25,813.7 | 13 Nov 07:00:02.389 | jiaoyi-s1 |
| 656795267 / AUS200 | 9,165.3 /0.1 | 8,826.3 | 8,830.7 | 13 Nov 06:50:03.075 | jiaoyi-s1 |


GER40 656513085 now has SL 25,813.7; AUS200 656795267 has SL 8,830.7. Both remain below their long entries and are not breakeven locks. Full retained SL histories are in JSON. S1 cash/amendments are excluded from S3 statistics.

## §4. Complete morning S3 ledger

The cohort is **two carried silver shorts plus 12 new entries = 14 positions**: 11 closed by 13:55, three open at cutoff. All are demo shorts with S3 ownership evidence. Initial R is fixed to actual fill-to-initial-SL distance and its cash-risk basis; later BE changes do not reset the denominator.


| Account / position | Carryover? / symbol | Side / native volume / lots | Entry SGT / price | Exit by cutoff SGT / price | Initial SL / TP | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 262253752 | Carried / XAGUSD | sell / 25000 / 0.0500 | 13 Nov 00:01:03.112 / 50.01600 | 13 Nov 07:01:03.170 / 50.07800 | 50.58800 / 49.22600 | -14.52 / -0.101538 |
| 46130058 / 262253774 | Carried / XAGUSD | sell / 30000 / 0.0600 | 13 Nov 00:01:06.420 / 50.01200 | 13 Nov 07:01:07.189 / 50.07800 | 50.51400 / 49.31900 | -18.63 / -0.123705 |
| 49236051 / 262341576 | New / BTCUSD | sell / 1 / 0.0100 | 13 Nov 08:32:30.920 / 71,157.59000 | 13 Nov 08:36:32.131 / 70,644.05000 | 71,471.38000 / 70,724.55000 | 5.14 / 1.638038 |
| 47790949 / 262341577 | New / BTCUSD | sell / 47 / 0.4700 | 13 Nov 08:32:30.992 / 71,157.63000 | 13 Nov 08:36:32.185 / 70,644.05000 | 71,471.42000 / 70,724.59000 | 241.38 / 1.636682 |
| 46130058 / 262341584 | New / BTCUSD | sell / 48 / 0.4800 | 13 Nov 08:32:41.724 / 71,143.58000 | 13 Nov 08:36:32.758 / 70,651.19000 | 71,457.37000 / 70,710.54000 | 236.35 / 1.569189 |
| 47790949 / 262343996 | New / ETHUSD | sell / 977 / 9.7700 | 13 Nov 09:15:22.177 / 2,006.23000 | 13 Nov 10:22:30.166 / 2,006.22000 | 2,021.71000 / 1,984.86000 | 0.10 / 0.000661 |
| 49236051 / 262343998 | New / ETHUSD | sell / 31 / 0.3100 | 13 Nov 09:15:22.309 / 2,006.17000 | 13 Nov 10:22:30.132 / 2,006.22000 | 2,021.65000 / 1,984.80000 | -0.02 / -0.004168 |
| 46130058 / 262344000 | New / ETHUSD | sell / 990 / 9.9000 | 13 Nov 09:15:23.188 / 2,006.18000 | 13 Nov 10:22:30.190 / 2,006.22000 | 2,021.66000 / 1,984.81000 | -0.40 / -0.002610 |
| 49236051 / 262411892 | New / ETHUSD | sell / 47 / 0.4700 | 13 Nov 12:39:09.146 / 1,925.75000 | 13 Nov 13:01:01.148 / 1,925.56000 | 1,935.89000 / 1,911.75000 | 0.09 / 0.018885 |
| 47790949 / 262411893 | New / ETHUSD | sell / 1493 / 14.9300 | 13 Nov 12:39:09.111 / 1,925.71000 | 13 Nov 13:01:01.145 / 1,925.56000 | 1,935.85000 / 1,911.71000 | 2.24 / 0.014796 |
| 46130058 / 262411894 | New / ETHUSD | sell / 1512 / 15.1200 | 13 Nov 12:39:09.665 / 1,925.64000 | 13 Nov 13:01:01.081 / 1,925.56000 | 1,935.78000 / 1,911.64000 | 1.21 / 0.007892 |
| 49236051 / 262420070 | New / BTCUSD | sell / 1 / 0.0100 | 13 Nov 13:11:35.743 / 68,599.56000 | Open / unavailable / Not verifiable / N/A | 69,073.20000 / 67,945.93000 | Not verifiable / N/A / Not verifiable / N/A |
| 46130058 / 262420072 | New / BTCUSD | sell / 32 / 0.3200 | 13 Nov 13:11:36.087 / 68,637.64000 | Open / unavailable / Not verifiable / N/A | 69,111.28000 / 67,984.01000 | Not verifiable / N/A / Not verifiable / N/A |
| 47790949 / 262420079 | New / BTCUSD | sell / 31 / 0.3100 | 13 Nov 13:11:38.146 / 68,582.25000 | Open / unavailable / Not verifiable / N/A | 69,055.89000 / 67,928.62000 | Not verifiable / N/A / Not verifiable / N/A |



| Position | Original R price / USD risk | Observed MFE R | Observed MAE R | Entry-price BE amendment SGT | Exit classification | Registered deadline SGT | Max in-position quote gap s |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 262253752 | 0.57200 / 143.0000 | 0.152098 | 0.578671 | None by cutoff | native_session_end | 13 Nov 05:59:00.000 | 3,723.343 |
| 262253774 | 0.50200 / 150.6000 | 0.165339 | 0.667331 | None by cutoff | native_session_end | 13 Nov 05:59:00.000 | 3,723.339 |
| 262341576 | 313.79000 / 3.1379 | 1.446509 | 0.203066 | 13 Nov 08:36:12.859 | broker_protective_fill_at_or_better_than_tp | 13 Nov 12:00:00.000 | 0.835 |
| 262341577 | 313.79000 / 147.4813 | 1.446636 | 0.202938 | 13 Nov 08:36:12.874 | broker_protective_fill_at_or_better_than_tp | 13 Nov 12:00:00.000 | 0.847 |
| 262341584 | 313.79000 / 150.6192 | 1.591925 | 0.247713 | 13 Nov 08:36:19.690 | broker_protective_fill_at_or_better_than_tp | 13 Nov 12:00:00.000 | 0.812 |
| 262343996 | 15.48000 / 151.2396 | 1.006460 | 0.845607 | 13 Nov 10:18:51.844 | breakeven_stop_with_fill_slippage | 13 Nov 12:00:00.000 | 2.852 |
| 262343998 | 15.48000 / 4.7988 | 1.002584 | 0.849483 | 13 Nov 10:18:52.747 | breakeven_stop_with_fill_slippage | 13 Nov 12:00:00.000 | 2.884 |
| 262344000 | 15.48000 / 153.2520 | 1.003230 | 0.848837 | 13 Nov 10:18:51.858 | breakeven_stop_with_fill_slippage | 13 Nov 12:00:00.000 | 2.753 |
| 262411892 | 10.14000 / 4.7658 | 1.014793 | 0.478304 | 13 Nov 13:00:26.707 | breakeven_stop_with_fill_slippage | 13 Nov 16:00:00.000 | 2.235 |
| 262411893 | 10.14000 / 151.3902 | 1.010848 | 0.482249 | 13 Nov 13:00:26.694 | breakeven_stop_with_fill_slippage | 13 Nov 16:00:00.000 | 2.238 |
| 262411894 | 10.14000 / 153.3168 | 1.003945 | 0.489152 | 13 Nov 13:00:26.667 | breakeven_stop_with_fill_slippage | 13 Nov 16:00:00.000 | 2.231 |
| 262420070 | 473.64000 / 4.7364 | 0.523098 | 0.492822 | None by cutoff | open | 13 Nov 16:00:00.000 | 2.979 |
| 262420072 | 473.64000 / 151.5648 | 0.603496 | 0.412423 | None by cutoff | open | 13 Nov 16:00:00.000 | 2.960 |
| 262420079 | 473.64000 / 146.8284 | 0.486551 | 0.529368 | None by cutoff | open | 13 Nov 16:00:00.000 | 2.990 |


Three BTC target closes generated **+$482.87**. Six ETH BE exits generated **+$3.22** combined: four small positive fills and two small negative fills. Two silver deadline losses total **−$33.15**. Thus seven net cash wins do not mean seven targets. Targets can realise netR different from 1.38 because of fill differences and costs. Observed quote extrema are not guaranteed executable fills.

### Silver deadline execution failure and recovery

Both carried silver positions had a recorded 05:59 deadline. The first closing requests were sent about 05:59:00.83 and rejected as `MARKET_CLOSED`. The actor kept the original deadline, retried roughly every 10–11 seconds and eventually filled after the metal market reopened at 07:01. **702 attempts: 700 rejections and two fills.** This is an exit failure/retry sequence, not 700 entry refusals and not activity caused by this read-only audit.


| Position | Deadline SGT | First request SGT | Rejected / filled / total | First fill SGT | Deadline delay seconds | Fill ask / net USD |
| --- | --- | --- | --- | --- | --- | --- |
| 262253752 | 13 Nov 05:59:00.000 | 13 Nov 05:59:00.833 | 350 / 1 / 351 | 13 Nov 07:01:03.170 | 3,723.170 | 50.078 / -14.52 |
| 262253774 | 13 Nov 05:59:00.000 | 13 Nov 05:59:00.827 | 350 / 1 / 351 | 13 Nov 07:01:07.189 | 3,727.189 | 50.078 / -18.63 |


Every rejected attempt carries the broker's explicit market-closed code. This supports a deadline/availability collision at the Thursday NY 16:59 boundary, not a claim that the local actor stopped running. The recorded quote gap through maintenance limits continuous excursion observation. Repeated retries did not make the closed market executable. The Friday 16:55 traditional deadline gives four minutes before 16:59, but actual timely execution still requires verification; it is not guaranteed by the calendar definition.

No exit timing or retry behaviour is changed in this audit. A separate engineering proposal should examine closing before the executable market boundary and market-aware retry pacing while preserving ownership and recorded-position protections.

## §5. Cutoff cash, later BTC closes and rolling goal


| Account | Window closes / net wins | Window net USD | Friday 00:00–13:55 net USD | Open at cutoff / later book | Latest20 wins | Realised payoff | Net-R PF |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 / 0 | 0.00 | 0.00 | 0 / 0 | 3/20 | 3.4277 | 0.6049 |
| 46130058 | 4 / 2 | 218.53 | 218.53 | 1 / 0 | 9/20 | 2.0962 | 1.7150 |
| 47790949 | 4 / 3 | 229.20 | 229.20 | 1 / 0 | 10/20 | 1.7143 | 1.7143 |
| 49236051 | 3 / 2 | 5.21 | 4.47 | 1 / 0 | 8/20 | 1.8228 | 1.3256 |


The full-day total is prior 00:00–04:55 **−$0.74** plus morning **+$452.94 = +$452.20**. The previous day's +$919.13 is not carried into Friday. Open-position marks are not realised cash.

At 13:55, the three later BTC entries were still open. Their cutoff marks and subsequent closing deals are separated below:


| Position | 13:55 executable ask | Quote SGT / age s | Price-only cutoff R / USD | Later TP fill SGT | Later fill / net USD |
| --- | --- | --- | --- | --- | --- |
| 262420070 | 68,506.55000 | 13 Nov 13:54:59.880 / 0.119302 | 0.196373 / 0.9301 | 13 Nov 13:56:06.273 | 67,873.27000 / 7.26 |
| 262420072 | 68,506.55000 | 13 Nov 13:54:59.907 / 0.092308 | 0.276771 / 41.9488 | 13 Nov 13:56:06.179 | 67,873.27000 / 244.60 |
| 262420079 | 68,506.55000 | 13 Nov 13:54:59.914 / 0.085532 | 0.159826 / 23.4670 | 13 Nov 13:56:06.240 | 67,873.27000 / 219.78 |


These later TP fills total **+$471.64**. They explain the later empty S3 broker books but do not alter the fixed 11-close cohort, morning cash, post-close denominator or cutoff latest20. Their own four-hour follow-ups belong to the next audit. Source equity/float and current inventory are not atomic; the monetary rows precede these closes.

Latest20 is the last official assessment before 13:55. Fifteen of 20 equals 75%, the first integer result meeting 73%. None meets that win requirement. The sample spans older registrations and contains correlated account copies. Positive realised payoff or this profitable window alone does not qualify the program. **Financial pass: false.**

## §6. Four-hour post-close price, pip and R movement

Every one of the 11 in-window closes is retained: **five complete 4.000h and six partial** through 13:55:22. Pip sizes from broker metadata: silver 0.01, BTC 0.1, ETH 0.1. All are shorts, so ask is the liquidation side. Favourable movement is exit fill minus later ask; adverse movement is the reverse. Post-close R uses the original position risk. SL/TP tests use original absolute levels.


| Position / symbol | Pip size | Hours / status | Max favourable price / pips / R | Max adverse price / pips / R | Original TP touch / first SGT | Original SL crossing / first SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 262253752 / XAGUSD | 0.010 | 4.000000 / complete | 0.11400 / 11.40000 / 0.19930 | 0.74500 / 74.50000 / 1.30245 | Not observed | 13 Nov 09:23:00.126 |
| 262253774 / XAGUSD | 0.010 | 4.000000 / complete | 0.11400 / 11.40000 / 0.22709 | 0.74500 / 74.50000 / 1.48406 | Not observed | 13 Nov 09:22:03.144 |
| 262341576 / BTCUSD | 0.100 | 4.000000 / complete | 2,216.47000 / 22,164.70000 / 7.06355 | 548.09000 / 5,480.90000 / 1.74668 | 13 Nov 08:36:32.284 | Not observed within observed horizon |
| 262341577 / BTCUSD | 0.100 | 4.000000 / complete | 2,216.47000 / 22,164.70000 / 7.06355 | 548.09000 / 5,480.90000 / 1.74668 | 13 Nov 08:36:32.261 | Not observed within observed horizon |
| 262341584 / BTCUSD | 0.100 | 4.000000 / complete | 2,223.61000 / 22,236.10000 / 7.08630 | 540.95000 / 5,409.50000 / 1.72392 | 13 Nov 08:36:32.830 | Not observed within observed horizon |
| 262343996 / ETHUSD | 0.100 | 3.547732 / partial | 96.44000 / 964.40000 / 6.22997 | 0.81000 / 8.10000 / 0.05233 | 13 Nov 10:44:09.293 | Not observed within observed horizon |
| 262343998 / ETHUSD | 0.100 | 3.547741 / partial | 96.44000 / 964.40000 / 6.22997 | 0.81000 / 8.10000 / 0.05233 | 13 Nov 10:44:09.294 | Not observed within observed horizon |
| 262344000 / ETHUSD | 0.100 | 3.547725 / partial | 96.44000 / 964.40000 / 6.22997 | 0.81000 / 8.10000 / 0.05233 | 13 Nov 10:44:09.315 | Not observed within observed horizon |
| 262411892 / ETHUSD | 0.100 | 0.905792 / partial | 15.78000 / 157.80000 / 1.55621 | 7.17000 / 71.70000 / 0.70710 | 13 Nov 13:05:20.444 | Not observed within observed horizon |
| 262411893 / ETHUSD | 0.100 | 0.905793 / partial | 15.78000 / 157.80000 / 1.55621 | 7.17000 / 71.70000 / 0.70710 | 13 Nov 13:05:20.516 | Not observed within observed horizon |
| 262411894 / ETHUSD | 0.100 | 0.905811 / partial | 15.78000 / 157.80000 / 1.55621 | 7.17000 / 71.70000 / 0.70710 | 13 Nov 13:05:20.514 | Not observed within observed horizon |



| Position | 1h signed price / pips / R | 2h signed price / pips / R | 4h signed price / pips / R | Largest quote-update gap s | Last quote age at horizon s |
| --- | --- | --- | --- | --- | --- |
| 262253752 | -0.07500 / -7.50000 / -0.13112 | -0.09600 / -9.60000 / -0.16783 | -0.55600 / -55.60000 / -0.97203 | 30.206588 | 0.104868 |
| 262253774 | -0.09900 / -9.90000 / -0.19721 | -0.12000 / -12.00000 / -0.23904 | -0.56300 / -56.30000 / -1.12151 | 29.926916 | 0.167945 |
| 262341576 | -350.06000 / -3,500.60000 / -1.11559 | 229.42000 / 2,294.20000 / 0.73113 | 1,753.88000 / 17,538.80000 / 5.58934 | 3.606742 | 0.016008 |
| 262341577 | -350.06000 / -3,500.60000 / -1.11559 | 229.42000 / 2,294.20000 / 0.73113 | 1,753.88000 / 17,538.80000 / 5.58934 | 3.562534 | 0.061646 |
| 262341584 | -341.80000 / -3,418.00000 / -1.08926 | 237.11000 / 2,371.10000 / 0.75563 | 1,757.93000 / 17,579.30000 / 5.60225 | 3.591885 | 0.002857 |
| 262343996 | 52.89000 / 528.90000 / 3.41667 | 68.83000 / 688.30000 / 4.44638 | not_elapsed_at_capture | 3.405862 | 0.193529 |
| 262343998 | 52.89000 / 528.90000 / 3.41667 | 68.83000 / 688.30000 / 4.44638 | not_elapsed_at_capture | 3.338747 | 0.188499 |
| 262344000 | 52.89000 / 528.90000 / 3.41667 | 68.83000 / 688.30000 / 4.44638 | not_elapsed_at_capture | 3.449167 | 0.176799 |
| 262411892 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | 3.338747 | 0.188499 |
| 262411893 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | 3.405862 | 0.193529 |
| 262411894 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | 3.449167 | 0.176799 |


All elapsed 1h/2h/4h endpoints have a quote at/before target no more than 6 seconds old. Future endpoints remain null. The silver post-close paths contain quote-update gaps up to 30.207 seconds; BTC/ETH gaps are shorter. Full elapsed duration does not prove an uninterrupted tape. Maxima are exact over retained quotes, not over unobserved prices.

Exit-specific interpretation:

- **Initial stop-loss closes:0.** No BE exit is relabelled as an initial stop-loss.
- **Two silver deadline closes:** neither reached original TP in its full four-hour follow-up. Original SL was subsequently crossed around09:22/09:23 SGT. Observed adverse movement from exit reached 1.302R/1.484R. Closing after reopen avoided further observed adverse exposure, but that does not excuse the earlier 62-minute deadline miss.
- **Three BTC target closes:** all were already winners. Subsequent favourable movement reached about 7.064–7.086 original R from exit. Their TP re-touches are not rescued stopped trades. Subsequent adverse movement from exit does not imply original SL was crossed; none was observed.
- **First ETH BE episode:** three account copies closed around 10:22:30 and touched original TP around 10:44:09. By13:55:22, their paths cover only 3.548 hours. No original-stop crossing was observed in that partial period; the complete endpoint is around 14:22:30.
- **Second ETH BE episode:** three copies closed around 13:01:01 and touched original TP around 13:05:20. Only 0.906 hours are observed; even the 1h checkpoint had not elapsed. Four-hour completion is around 17:01:01.

The six ETH paths show an opportunity cost of entry-price stops in these two episodes. They are not six independent demonstrations that removing BE improves the strategy. Previous oil paths worsened after BE exits. A valid alternative needs a frozen, matched replay including those adverse paths, costs, occupancy and independent dates. This report neither performs that counterfactual nor changes live exits.

## §7. Openings, human locks and the veto funnel

Friday traditional opening identity is 07:00 SGT, with metals' actual observation start at **07:01**; Brent starts 09:00. Crude's four first quotes arrived **7.051306–7.060376 seconds** after 07:00 and match operational references under the compiled 15-second bound. There was no late mid-session replacement.

All 40 requested slots have recorded two-sided quotes. Operational matches are **38/40**: Asian 18/20 plus Brent 4/4, and crypto 08:00/12:00 16/16. The two withheld operational Asian references are live gold and silver under human/pair guards; recorded quotes exist. Missing permission is distinct from missing market data.


| Account | Symbol | Session SGT | Observation start SGT | First quote delay s | Mid | Operational source / match |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | NAS100 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.858263 | 28,304.25000 | runtime_actor_opening_reference / yes |
| 43002148 | BTCUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.178541 | 71,375.99000 | stored_arm_opening_quote:134319 / yes |
| 43002148 | BTCUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.127391 | 68,930.45000 | runtime_actor_opening_reference / yes |
| 49236051 | SpotBrent | 13 Nov 09:00:00.000 | 13 Nov 09:00:00.000 | 0.606459 | 98.12200 | runtime_actor_opening_reference / yes |
| 43002148 | XAUUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 1.045813 | 3,877.34500 | Live human/pair guard; no operational reference |
| 43002148 | ETHUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.280030 | 2,018.00000 | stored_arm_opening_quote:134311 / yes |
| 43002148 | ETHUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.060116 | 1,932.10000 | runtime_actor_opening_reference / yes |
| 43002148 | XAGUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 0.933685 | 50.06700 | Live human/pair guard; no operational reference |
| 43002148 | US500 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.858262 | 7,599.05000 | runtime_actor_opening_reference / yes |
| 49236051 | NAS100 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.858512 | 28,304.25000 | runtime_actor_opening_reference / yes |
| 49236051 | US500 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.858511 | 7,599.05000 | runtime_actor_opening_reference / yes |
| 49236051 | XAGUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 0.942568 | 50.06700 | runtime_actor_opening_reference / yes |
| 49236051 | BTCUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.162861 | 71,375.99000 | stored_arm_opening_quote:134315 / yes |
| 49236051 | BTCUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.167533 | 68,931.10000 | runtime_actor_opening_reference / yes |
| 49236051 | SpotCrude | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 7.052053 | 89.02000 | runtime_actor_opening_reference / yes |
| 49236051 | ETHUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.218931 | 2,018.00000 | stored_arm_opening_quote:134304 / yes |
| 49236051 | ETHUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.167538 | 1,932.10000 | runtime_actor_opening_reference / yes |
| 43002148 | SpotCrude | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 7.051306 | 89.02000 | runtime_actor_opening_reference / yes |
| 43002148 | SpotBrent | 13 Nov 09:00:00.000 | 13 Nov 09:00:00.000 | 0.589911 | 98.12200 | runtime_actor_opening_reference / yes |
| 46130058 | NAS100 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.859070 | 28,304.25000 | runtime_actor_opening_reference / yes |
| 49236051 | XAUUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 0.942573 | 3,877.34500 | runtime_actor_opening_reference / yes |
| 46130058 | US500 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.859070 | 7,599.05000 | runtime_actor_opening_reference / yes |
| 46130058 | XAUUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 1.021803 | 3,877.34000 | runtime_actor_opening_reference / yes |
| 46130058 | SpotBrent | 13 Nov 09:00:00.000 | 13 Nov 09:00:00.000 | 0.616834 | 98.12200 | runtime_actor_opening_reference / yes |
| 47790949 | XAUUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 0.958886 | 3,877.34500 | runtime_actor_opening_reference / yes |
| 47790949 | BTCUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.187824 | 71,375.99000 | stored_arm_opening_quote:134316 / yes |
| 47790949 | BTCUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.134470 | 68,931.10000 | runtime_actor_opening_reference / yes |
| 46130058 | ETHUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.219909 | 2,018.00000 | stored_arm_opening_quote:134306 / yes |
| 46130058 | ETHUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.164010 | 1,932.10000 | runtime_actor_opening_reference / yes |
| 47790949 | XAGUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 0.958884 | 50.06700 | runtime_actor_opening_reference / yes |
| 47790949 | US500 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.861303 | 7,599.05000 | runtime_actor_opening_reference / yes |
| 47790949 | SpotCrude | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 7.060376 | 89.02000 | runtime_actor_opening_reference / yes |
| 46130058 | SpotCrude | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 7.054084 | 89.02000 | runtime_actor_opening_reference / yes |
| 47790949 | NAS100 | 13 Nov 07:00:00.000 | 13 Nov 07:00:00.000 | 1.861304 | 28,304.25000 | runtime_actor_opening_reference / yes |
| 47790949 | SpotBrent | 13 Nov 09:00:00.000 | 13 Nov 09:00:00.000 | 0.689350 | 98.12200 | runtime_actor_opening_reference / yes |
| 46130058 | XAGUSD | 13 Nov 07:00:00.000 | 13 Nov 07:01:00.000 | 0.930400 | 50.06700 | runtime_actor_opening_reference / yes |
| 46130058 | BTCUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.162209 | 71,375.99000 | stored_arm_opening_quote:134317 / yes |
| 46130058 | BTCUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.164004 | 68,931.10000 | runtime_actor_opening_reference / yes |
| 47790949 | ETHUSD | 13 Nov 08:00:00.000 | 13 Nov 08:00:00.000 | 0.187839 | 2,018.00000 | stored_arm_opening_quote:134305 / yes |
| 47790949 | ETHUSD | 13 Nov 12:00:00.000 | 13 Nov 12:00:00.000 | 0.134473 | 1,932.10000 | runtime_actor_opening_reference / yes |



| Account | New arms | Consumed = intents | Fills | Terminal peer refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 13 | 0 | 0 | 0 | 9 |
| 46130058 | 11 | 9 | 4 | 5 | 1 |
| 47790949 | 11 | 9 | 4 | 5 | 1 |
| 49236051 | 12 | 9 | 4 | 5 | 1 |


Totals: **47 arms, 27 consumed triggers, 27 persisted opening intents, 12 fills, 12 deep invalidations and 45 pair-reservation diagnostic rows**. The 15 final peer refusals comprise 12 below 24 bps and three opposing-peer conflicts. Diagnostic counts can be deduplicated and are not unique trade opportunities or every tick's gate assessment.

Each consumed decision has exactly one attributable terminal outcome joined to its intent. **Missing individual outcomes=0; unknown entry-guard receipts=0.** Max recorded confirmation-to-final-guard latency is **34.354 ms**. Waiting from touch until turn confirmation is not dispatch processing latency. These checks concern entry telemetry; the 700 market-closed exit rejections remain a separate material finding.


| Consumed → outcome ID | Account / symbol | Consumed SGT | Terminal status / exact reason |
| --- | --- | --- | --- |
| 132371 → 132375 | 49236051 / BTCUSD | 13 Nov 07:28:35.770 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 132372 → 132376 | 46130058 / BTCUSD | 13 Nov 07:28:35.772 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 132373 → 132377 | 47790949 / BTCUSD | 13 Nov 07:28:35.785 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 133395 → 133401 | 47790949 / BTCUSD | 13 Nov 07:53:54.195 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 133396 → 133402 | 49236051 / BTCUSD | 13 Nov 07:53:54.195 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 133397 → 133403 | 46130058 / BTCUSD | 13 Nov 07:53:54.203 | refused / ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse |
| 134405 → 134413 | 49236051 / BTCUSD | 13 Nov 08:32:30.493 | filled / Filled { position_id: Some(262341576), price: Some(71157.59), volume: Some(1) } |
| 134406 → 134420 | 47790949 / BTCUSD | 13 Nov 08:32:30.497 | filled / Filled { position_id: Some(262341577), price: Some(71157.63), volume: Some(47) } |
| 134425 → 134428 | 46130058 / BTCUSD | 13 Nov 08:32:41.250 | filled / Filled { position_id: Some(262341584), price: Some(71143.58), volume: Some(48) } |
| 135355 → 135357 | 49236051 / SpotCrude | 13 Nov 09:01:51.337 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 135359 → 135361 | 47790949 / SpotCrude | 13 Nov 09:01:56.966 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 135362 → 135364 | 46130058 / SpotCrude | 13 Nov 09:02:09.803 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 135642 → 135649 | 49236051 / ETHUSD | 13 Nov 09:15:21.769 | filled / Filled { position_id: Some(262343998), price: Some(2006.17), volume: Some(31) } |
| 135643 → 135652 | 47790949 / ETHUSD | 13 Nov 09:15:21.769 | filled / Filled { position_id: Some(262343996), price: Some(2006.23), volume: Some(977) } |
| 135644 → 135655 | 46130058 / ETHUSD | 13 Nov 09:15:22.766 | filled / Filled { position_id: Some(262344000), price: Some(2006.18), volume: Some(990) } |
| 136889 → 136892 | 47790949 / SpotCrude | 13 Nov 10:16:57.794 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 136890 → 136894 | 46130058 / SpotCrude | 13 Nov 10:16:57.794 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 136891 → 136893 | 49236051 / SpotCrude | 13 Nov 10:16:57.809 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 137071 → 137074 | 47790949 / SpotCrude | 13 Nov 10:23:10.547 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 137072 → 137075 | 46130058 / SpotCrude | 13 Nov 10:23:10.547 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 137073 → 137076 | 49236051 / SpotCrude | 13 Nov 10:23:10.548 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 139685 → 139695 | 49236051 / ETHUSD | 13 Nov 12:39:08.633 | filled / Filled { position_id: Some(262411892), price: Some(1925.75), volume: Some(47) } |
| 139686 → 139698 | 47790949 / ETHUSD | 13 Nov 12:39:08.646 | filled / Filled { position_id: Some(262411893), price: Some(1925.71), volume: Some(1493) } |
| 139687 → 139701 | 46130058 / ETHUSD | 13 Nov 12:39:09.245 | filled / Filled { position_id: Some(262411894), price: Some(1925.64), volume: Some(1512) } |
| 140476 → 140483 | 49236051 / BTCUSD | 13 Nov 13:11:35.205 | filled / Filled { position_id: Some(262420070), price: Some(68599.56), volume: Some(1) } |
| 140477 → 140489 | 46130058 / BTCUSD | 13 Nov 13:11:35.605 | filled / Filled { position_id: Some(262420072), price: Some(68637.64), volume: Some(32) } |
| 140480 → 140492 | 47790949 / BTCUSD | 13 Nov 13:11:37.617 | filled / Filled { position_id: Some(262420079), price: Some(68582.25), volume: Some(31) } |


Current live gates, observed separately after cutoff:


| Symbol | Exact gate | Observed reason |
| --- | --- | --- |
| BTCUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 195.16% below floor 250.00% |
| ETHUSD | prepared_admission_unavailable | Blocked: prepared admission unavailable; last refusal: margin buffer: projected margin level 198.00% below floor 250.00% |
| NAS100 | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| SpotBrent | invalidated_deep_retracement_gt_61_8pct | Invalidated: retraced > 61.8% without turn confirmation |
| SpotCrude | invalidated_deep_retracement_gt_61_8pct | Invalidated: retraced > 61.8% without turn confirmation |
| US500 | waiting_for_qualified_setup | Waiting: opening impulse below 48 bps |
| XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| XAUUSD | yield_to_human_position_or_order | Blocked: yield_to_human_position_or_order |


Live produced no consumed S3 trigger or persisted opening intent. Current crypto prepared-admission margin refusals, human/pair locks, insufficient impulses and deep invalidations are explicit. Their snapshots must not be assigned as the unique historical cause for every non-entry without matching per-arm evidence.

## §8. London preparation and Friday/weekend boundaries

All four accounts are connected and unpaused on the same production commit. London 15:00 has **24/24 base preparations** with metadata, spread/commission readiness and causal macro context. However, **all 24 traditional tables lack both 420- and450-minute marks**: each has **9 same-slot endpoints;14 needed**. Earlier marks are present. Those late marks correspond to 22:00/22:30 SGT relative to 15:00; they matter near the NY handover. This is not a global lack of preparation, and it is not full-session readiness. The existing refusal behaviour is retained.

Crypto 16:00 has **8/8 complete tables** with no missing marks. Future London/crypto opening quotes cannot yet be observed in this audit snapshot. Human/pair protection, current market availability and fresh dispatch evidence must still pass at entry.


| Account | Symbol | Prepared session SGT | Base prepared | Full table prepared | Missing marks / errors |
| --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 43002148 | ETHUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 43002148 | NAS100 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 43002148 | SpotBrent | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 43002148 | SpotCrude | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 43002148 | US500 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 43002148 | XAGUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 43002148 | XAUUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 46130058 | BTCUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 46130058 | ETHUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 46130058 | NAS100 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 46130058 | SpotBrent | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 46130058 | SpotCrude | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 46130058 | US500 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 46130058 | XAGUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 46130058 | XAUUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 47790949 | BTCUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 47790949 | ETHUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 47790949 | NAS100 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 47790949 | SpotBrent | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 47790949 | SpotCrude | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 47790949 | US500 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 47790949 | XAGUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 47790949 | XAUUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 49236051 | BTCUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 49236051 | ETHUSD | 13 Nov 16:00:00.000 | True | True | 0 / {} |
| 49236051 | NAS100 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 49236051 | SpotBrent | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 49236051 | SpotCrude | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 49236051 | US500 | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 49236051 | XAGUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |
| 49236051 | XAUUSD | 13 Nov 15:00:00.000 | True | False | 2 / {'420': {'Err': '9 same-slot endpoints;14 needed'}, '450': {'Err': '9 same-slot endpoints;14 needed'}} |


The inspected `production_sessions.rs` is byte-identical to the active production commit. IANA `America/New_York` is UTC−05:00 on 13 November 2026; Singapore is UTC+08:00.


| Code boundary | New York time | Singapore time | Meaning |
| --- | --- | --- | --- |
| Traditional Friday native close | 13 Nov 16:55 EST | 14 Nov 05:55 SGT | Registered Friday deadline; four minutes before 16:59 |
| Crypto maintenance begins | 13 Nov 16:59 EST | 14 Nov 05:59 SGT | Entry availability guard begins; inclusive |
| Crypto maintenance ends | 13 Nov 17:01 EST | 14 Nov 06:01 SGT | Entry availability guard ends; exclusive |


**Crypto is not configured for a full weekend shutdown here.** Its module creates six UTC four-hour windows on every date. The NY [16:59,17:01) guard excludes daily maintenance. It does not itself force every held crypto position to close at 16:59; existing recorded position deadlines remain governed by the opening ledger. Runtime broker holidays/trading mode remain final constraints.

Traditional Saturday/Sunday NY trading dates have no windows. Monday's Asian session starts on Sunday NY 18:00 (metals executable observation 18:01); actual broker availability still applies. No future successful close is asserted merely because a deadline is configured. After this morning's silver rejections, the Friday deadline-to-fill check is an operational priority.

## §9. Assessment and bounded next work

1. **Prioritise a separate deadline/availability repair proposal.** The silver evidence is specific: broker rejection at the programmed weekday deadline,350 retries each, and fills only after reopening. Examine an executable pre-close boundary and market-aware retries with focused regression tests. This directive is read-only, so no production fix is made or implied.
2. **Retain the mixed exit evidence.** Six ETH BE paths later touched TP, whereas both silver paths worsened and earlier oil BE paths also worsened. Freeze a complete cohort before any exit experiment; do not optimise only on missed favourable movement.
3. **Finish partial follow-ups in the next audit:** first ETH group after 14:22:31, second after 17:01:02, and the three post-cutoff BTC targets after 17:56:07 SGT. Do not silently replace this report's frozen partial observations.
4. **Keep London base readiness separate from late-mark coverage.** The 420/450-minute gap remains an engineering/data-preparation follow-up, with exact 9/14 evidence preserved. Verify actual 15:00/16:00 opening quotes and Friday closing execution later.
5. **Keep live external capital and S3 performance separate.** Negative live balance, positive floating equity and insufficient projected margin are distinct facts. This audit does not alter human orders, add capital, increase leverage or bypass risk guards.

Financial qualification remains false. A profitable morning and later BTC targets do not establish an independent trading edge or satisfaction of the 73%/1:1.38 goal.

## §10. Verification, publication and Gemini instructions

Four focused analytical tests passed: signed units and stale/future endpoints; cash/cutoff isolation; partial horizons and exit-retry accounting; outcome attribution, ownership, calendar conversion and preparation coverage. **24 quote-export segments / 636,578 rows** reconcile to uncompressed SHA256 hashes. All recorded opening matches are checked against their original quotes. Full source queries and calculation scripts, plus the 700 rejection receipts, are in JSON. Hashes identify evidence copies; they do not replace authorised access for independent source reproduction.

Only the named Markdown, HTML and JSON files under `doc/` change. Required CI is `check`, `docker`, `engineering`. The final response supplies the merged PR/commit and checks the deployment ID/image and actor continuity after merge. Baseline deployment is `c0fedaca-0370-4cec-9234-f59f8c7cafce`, commit `aebf404dedf89dbcdb8cee78c95a582eea042d51`, image `sha256:631e668c1ca138d5810f22fdab1ba27f3b7f4f34c6a28fa544c7ad09c2a9fad3`. These report paths do not match Trading deployment watch patterns.

**Gemini:** read the Markdown and JSON together. Inspect `silver_close_attempts`, `ledger`, `post_cutoff_execution_observations`, `post_close_summary`, `telemetry`, `opening_audit`, `preparations`, `calendar_audit` and `methodology`. Verify 11 cutoff closes versus three later targets, seven cash wins versus three targets, five complete versus six partial follow-ups, and base readiness versus full late-mark coverage. Do not call the crypto maintenance gate a weekend shutdown or treat proposed repair/research work as deployment approval.


| Quote segment | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 47790949-XAGUSD-2026-11-12-160057.csv.gz | 29083 | 6f6b8a1e6bc3b90edc48c5de3b92815aa5e950c3843bf72b20a6f172586b2816 |
| 47790949-XAGUSD-2026-11-12-200057.csv.gz | 13041 | 99936f2b4942cc5cc4d0a5b69028abc053c11bf62d07d636df7491ba2b17f1c2 |
| 47790949-XAGUSD-2026-11-13-000057.csv.gz | 25319 | c95a1cb842d90c9219ee8b4bb66a4d489f1464e46cdd98dd11b107e9d8f8eb9f |
| 46130058-XAGUSD-2026-11-12-160100.csv.gz | 29120 | ad2863be1ad527c4ba2e629f9e7d5ae7eff5eff4328012f94e362a51920b41a1 |
| 46130058-XAGUSD-2026-11-12-200100.csv.gz | 13059 | 484abeca86babf855ac3c85cfe0c490ad3c505e036adcd8aee62140ed2d24a27 |
| 46130058-XAGUSD-2026-11-13-000100.csv.gz | 25352 | d580695a1496713c4c67ba648f8eb241b2bb6104f84363aa6223523df05e1bb2 |
| 49236051-BTCUSD-2026-11-13-003224.csv.gz | 68427 | e19e0630780959e506630a47406c1318b35d438408effe708abd2010ebc98ba0 |
| 49236051-BTCUSD-2026-11-13-043224.csv.gz | 1195 | 9e444a001ef4fe1228c301a65737cb9d79a3f0ce4cc8b3a26a6c75c4c38b4028 |
| 47790949-BTCUSD-2026-11-13-003224.csv.gz | 68427 | 0229c7081591f85d3684274ef6897d97895a2cf7fc8dba2129e6917c6a62a0b3 |
| 47790949-BTCUSD-2026-11-13-043224.csv.gz | 1194 | 0b229d560261a3dbc893c37a37b7e3637b2ae16ed275d4d92413950745697b2c |
| 46130058-BTCUSD-2026-11-13-003235.csv.gz | 68429 | 5e2fc5d93f798d7b39aa40bd607099d8b3747d4848b62426155dab46324dfd09 |
| 46130058-BTCUSD-2026-11-13-043235.csv.gz | 1144 | 2ec137d63d624b9092a32d83133bad8ea171cc17fe3ef730571aad53568a6fc9 |
| 47790949-ETHUSD-2026-11-13-011516.csv.gz | 58466 | 4bdfc28d6f9b5f03697c3ddb1258be3057eaf87f88cf431b810facf5a5445b84 |
| 47790949-ETHUSD-2026-11-13-051516.csv.gz | 9306 | 62d81cc07466e1003d60f762cdaa6b7d045fcb2e16afa382acaf492b1376d926 |
| 49236051-ETHUSD-2026-11-13-011516.csv.gz | 58466 | f03021046c5db8e92a31b2dab02c03c5ef12c746ba043a1b9d5024043b26534d |
| 49236051-ETHUSD-2026-11-13-051516.csv.gz | 9306 | 9183ad7fdec12a4c479e0c32ba831c205a3a18bdc91f30338b884e14fa9f7ace |
| 46130058-ETHUSD-2026-11-13-011517.csv.gz | 58464 | dec4e7c0d2942c8fadaf0a0cf533557f23f53d54cf80d3876041d1afb1c923b4 |
| 46130058-ETHUSD-2026-11-13-051517.csv.gz | 9305 | ad2d7e2b60df0d511be3fe20309b80813bc83b76ddc35fdc7bdb4cc7682e969c |
| 49236051-ETHUSD-2026-11-13-043903.csv.gz | 17962 | d97c506e103c7cde2356cfaf4599b33cb5b925ad3ed35e0c174547a3b1e4e0a1 |
| 47790949-ETHUSD-2026-11-13-043903.csv.gz | 17963 | fe156ee7ef0487dd72d9af14b9f7d1fe304fb0a00d47627fb999e6ed81729ac8 |
| 46130058-ETHUSD-2026-11-13-043903.csv.gz | 17960 | b5afcc6a6373833ef047dd65b79df75be4e7d0b571a470103fe0b99dd4442d65 |
| 49236051-BTCUSD-2026-11-13-051129.csv.gz | 11868 | c973a70e31543e0c2a097b8c413c37f551f207f47841c04611b7fc0b158e721b |
| 46130058-BTCUSD-2026-11-13-051130.csv.gz | 11866 | 482ca821e4c28bf57aee4771af0c76df4c49cc8be867d98edb240423f10cf9a3 |
| 47790949-BTCUSD-2026-11-13-051132.csv.gz | 11856 | bfc3fa9e9159ef32874ffaf3bd03e2dbf6df77be06b030925910c4ec390d4637 |



| Source | SHA256 |
| --- | --- |
| opening-ticks.json | 63f5b4ad536762761d78eb22c0ad5a4f0be3e91d1750a3f611c28f4d015832c6 |
| preflight.json | 252979ca23f654eb655fb6a81018eae1357d9753f32cd64410b2082159ca04bc |
| monitor.json | a4b3082d452f0794be5c3a1dc8edb51ce7ec9541164123364b79bbce01c28f33 |
| postcutoff.json | 7da33fc30bb3ed720d154d1ae115d9430e33121e25e3347883f160f70fa77704 |
| human.json | 728f89991cc9ed5536cab4fa91a4429a71bdeb505d7be01fd05eb20da8a663ae |
| health.json | 12e70431c5b29db4a53d5a2274c8d0cdf1eff53f5801b645fce9789691cf2ea7 |
| silver-close-attempts.json | 0d215c8985cd3340b20039eea1151a184c2d3c2404c8b99d67fb99ff9eed5690 |
| activity.json | 0231dc2d7ef53c1f32a51b6f9502eb5505eb64553bd15efa74fe4b2d8a0edf99 |
| evidence.json | 97455b6f29942f15df8e88ce603d1624b935cde0220c9a4de56f99e992db7ad0 |
| deployment-before.json | 6cab90a4e75ee1ed9ad74d89b5cb42a8c91d062e0a39cbe9e231c04b4049604a |
| runtime.json | bdad8ae510b72e5f6dcf487696b6db811cc161c5649d656200bacc80035498c4 |
| calendar-audit.json | 616356b209f638e1d8cef9010a7c0d6cf7703630bcdce44042bea8887c0e1e68 |
