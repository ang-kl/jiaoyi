# Thursday London review, post-close excursions and New York readiness — №18,444

**Activity cutoff: 12 November 2026, 19:45 SGT.** Interval **[12:55, 19:45) SGT**, continuing from [№18,435](THURSDAY_MIDDAY_ASIAN_REVIEW_AND_LONDON_PREP-18435-2026-11-12.md). Post-close quote horizon **20:28 SGT**; later snapshots are separately timestamped. Read-only production audit.

## §1. TL;DR

- **London-window S3 realised cash: −$238.83**, eight closes: six losses and two wins. Twelve new demo entries; four remained open at 19:45 and in the later book. No live S3 fill occurred in this window.
- **Thursday 00:00–19:45 SGT closed S3 cash: +$98.57**, down from +$337.40 at 12:55. Open P&L is not included in realised cash.
- **Oil and ETH have different post-close paths.** The two oil stop-outs did not reach their original TP within four hours and prices crossed their original stops again. The three ETH deadline closes later crossed their original TP around **19:46 SGT**, without an observed original-SL crossing. These are three correlated account copies, not three independent experiments or a proven alternative exit strategy.
- Six follow-ups have a full **4.000 hours**. The two BTC target exits have only **0.788/0.789 hours** through 20:28; their four-hour observations remain incomplete until about **23:40 SGT**.
- **Manual gold remains external:** position **652821184**, order **400465199**, 0.1 lot at 3,959.95, filled 15:13:19.255 SGT. Zero bot commands/amendments linked to the current human inventory were recorded. Gold/silver remain human/pair-locked. Human protection levels differ from earlier records; these changes are not linked to bot commands.
- Live has **two S1 positions**, GER40 and newly opened AUS200; neither is S3. GER40 recorded 29 in-window stop raises, ending at **25,802.9**.
- **24 consumed triggers → 24 intents → 24 attributable outcomes:** 12 fills, 11 peer-hold refusals, one expired-entry refusal. Unknown guard receipts and missing individual outcomes are both zero.
- Opening references: **London 22/24 operational plus two guarded live metals; crypto 16:00 8/8; crypto 20:00 8/8**. New York 22:30 preparations are **24/24 complete**, including all required marks. These are separate NY tables; this does not repair the late-London 420/450 coverage issue noted earlier.
- **Financial pass: false.** Latest20 wins: live 3/20, demo 46130058 7/20, demo 47790949 8/20, demo 49236051 6/20. No production change was made.

## §2. Time boundaries, sources and current account snapshot

The execution clock was **20:28:00 SGT**, after the requested 19:45 cutoff. Trade/intent/outcome counts use the exact half-open activity interval. The post-close quote horizon is frozen at 20:28:00. Account monetary marks were observed at **20:29:52.918–20:29:53.308**, inventory around **20:28:51**, and health at **20:29:50.853**. They are current post-cutoff observations, not invented 19:45 snapshots.

Existing successful stored deal-fetch receipts after 19:45 cover the full activity interval. The official monitor's last pre-cutoff history reached 19:44:51; the separate later stored-history reconciliation closes that small reporting tail. No new broker request was made. Data were copied using bounded read-only PostgreSQL queries. The deployed strategy policy is `s3_macro_pullback_owner_18409_v1` on `aebf404dedf89dbcdb8cee78c95a582eea042d51`.

Equity is recorded balance plus broker net unrealised P&L, excluding unverified credit and prospective closing costs. Assessed margin percentages below come from the separately timed owner-assessment projection; the monetary snapshot does not supply a verified aggregate broker margin level.


| Account | Balance USD | Account float USD | Equity USD | Assessed margin % | S3 / S1 / S2 / external | Pending |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 501.83 | 3,981.04 | 4,482.87 | 649.30 | 0 / 2 / 0 / 4 | 4 |
| 46130058 | 24,892.73 | 79.66 | 24,972.39 | 7,327.35 | 2 / 4 / 0 / 0 | 0 |
| 47790949 | 24,547.67 | 64.28 | 24,611.95 | 1,522.43 | 1 / 4 / 0 / 0 | 0 |
| 49236051 | 800.81 | 0.71 | 801.52 | 18,995.73 | 1 / 0 / 0 / 0 | 0 |


Live's current margin projection is 649.30%, but its pending-entry projection is **249.35%**, just below the existing 250% floor; the separate stressed projection is 36.90%. These are current forecasts, not evidence that margin caused every earlier S3 non-entry. No human pending order was cancelled or amended by this audit.


| Account | Position/order ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry / trigger | Current SL | Current TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Not verifiable / N/A |
| 43002148 | 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Not verifiable / N/A |
| 43002148 | 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.91300 | 3.87500 |
| 43002148 | 652821184 | XAUUSD | human_external / position | buy | 1000 / 0.1000 | 3,959.95000 | 3,923.72000 | 4,339.75000 |
| 43002148 | 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,802.90000 | Not verifiable / N/A |
| 43002148 | 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,826.30000 | Not verifiable / N/A |
| 43002148 | 401341593 | US500 | human_external / pending_entry | buy | 800 / 8.0000 | 7,562.60000 | 7,552.40000 | Not verifiable / N/A |
| 43002148 | 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 | 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Not verifiable / N/A |
| 43002148 | 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Not verifiable / N/A |
| 46130058 | 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,826.30000 | Not verifiable / N/A |
| 46130058 | 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69727 | Not verifiable / N/A |
| 46130058 | 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12736 | Not verifiable / N/A |
| 46130058 | 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,795.40000 | Not verifiable / N/A |
| 46130058 | 262101929 | XAUUSD | s3 / position | sell | 500 / 0.0500 | 3,942.25000 | 3,970.38000 | 3,903.43000 |
| 46130058 | 262132606 | SpotCrude | s3 / position | sell | 23000 / 2.3000 | 86.22900 | 86.87600 | 85.33600 |
| 47790949 | 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,826.30000 | Not verifiable / N/A |
| 47790949 | 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69722 | Not verifiable / N/A |
| 47790949 | 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12731 | Not verifiable / N/A |
| 47790949 | 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,795.40000 | Not verifiable / N/A |
| 47790949 | 262101991 | XAUUSD | s3 / position | sell | 500 / 0.0500 | 3,942.25000 | 3,970.36000 | 3,903.45000 |
| 49236051 | 262132524 | SpotCrude | s3 / position | sell | 700 / 0.0700 | 86.23900 | 86.88600 | 85.34600 |


## §3. Human protection, manual gold and live S1

The broker deal **347682660** links gold order **400465199** to position **652821184** at 15:13:19.255, BUY 1,000 native units (=0.1 lot), price 3,959.95. The current book classifies it as `human_external`; it has no S3 ownership label or S3 opening intent. The recorded deal carries $0.35 entry commission, separately from S3 closing cash. No human/external closing deal was recorded during the audit interval.

No persisted bot intent targets any current human position/order in the inspected interval through 20:28, and no recorded owned amendment/close event is linked to those objects. **Recorded bot interference:0.** This is a statement about retained evidence; external broker actions are not comprehensively attributed by the bot event journal. No raw external execution-event history was available for all human modifications.

The gold current stop is 3,923.72, versus 3,877.30 on the earlier pending-order snapshot; the NatGas stop changed from 2.800 to 2.913. Those changes are not linked to bot commands. JPN225 positions retain their recorded stops 59,000 and 58,500. A new human US500 pending order **401341593** also locks US500 and reserves NAS100. The gold pending order has become the verified filled position rather than an unexplained disappearance.

Law 1 label isolation is supported by the distinct S3/S1 ownership records. Law 2 is visible in gold and US500 yield gates and their silver/NAS100 pair guards. Law 3 keeps NatGas/JPN225 human-reserved. Present opening-reference availability does not confer authority to trade a human-locked symbol.

Live GER40 **656513085** remains S1-owned: BUY 0.1 lot at 26,809.0, initial SL 25,589.7, current SL 25,802.9. Its prior midday stop was 25,656.4. All 50 recorded stop replacements since entry are monotonic raises with label `jiaoyi-s1`; **29 occurred in this audit interval**, listed below. These stops are still below the entry price; a stop raise is not automatically a breakeven lock.

Live AUS200 **656795267** is a separate new S1 BUY 0.1 lot, filled 17:39:19.380 at 9,165.3, initial SL 8,822.1, now 8,826.3 after seven recorded raises. Neither S1 position is included in S3 counts, cash or latest20.


| GER40 amendment SGT | New stop | Recorded owner |
| --- | --- | --- |
| 12 Nov 14:00:33.725 | 25,658.0 | jiaoyi-s1 |
| 12 Nov 14:01:34.384 | 25,687.0 | jiaoyi-s1 |
| 12 Nov 14:02:35.057 | 25,690.6 | jiaoyi-s1 |
| 12 Nov 14:03:36.422 | 25,702.7 | jiaoyi-s1 |
| 12 Nov 14:04:36.618 | 25,706.3 | jiaoyi-s1 |
| 12 Nov 14:05:36.417 | 25,739.4 | jiaoyi-s1 |
| 12 Nov 14:06:38.454 | 25,745.6 | jiaoyi-s1 |
| 12 Nov 14:07:39.523 | 25,748.0 | jiaoyi-s1 |
| 12 Nov 14:08:39.736 | 25,749.9 | jiaoyi-s1 |
| 12 Nov 14:09:40.060 | 25,750.4 | jiaoyi-s1 |
| 12 Nov 16:11:24.477 | 25,751.4 | jiaoyi-s1 |
| 12 Nov 16:12:24.659 | 25,755.4 | jiaoyi-s1 |
| 12 Nov 16:14:22.239 | 25,756.7 | jiaoyi-s1 |
| 12 Nov 16:15:22.638 | 25,757.4 | jiaoyi-s1 |
| 12 Nov 16:16:22.691 | 25,757.9 | jiaoyi-s1 |
| 12 Nov 16:17:22.714 | 25,760.4 | jiaoyi-s1 |
| 12 Nov 16:18:23.189 | 25,762.5 | jiaoyi-s1 |
| 12 Nov 16:19:23.198 | 25,769.2 | jiaoyi-s1 |
| 12 Nov 16:20:23.391 | 25,774.2 | jiaoyi-s1 |
| 12 Nov 16:21:49.031 | 25,776.0 | jiaoyi-s1 |
| 12 Nov 16:22:49.197 | 25,776.7 | jiaoyi-s1 |
| 12 Nov 16:31:39.732 | 25,777.5 | jiaoyi-s1 |
| 12 Nov 16:32:39.770 | 25,777.7 | jiaoyi-s1 |
| 12 Nov 16:33:39.793 | 25,784.0 | jiaoyi-s1 |
| 12 Nov 16:36:19.899 | 25,784.3 | jiaoyi-s1 |
| 12 Nov 16:37:20.022 | 25,787.2 | jiaoyi-s1 |
| 12 Nov 16:38:20.190 | 25,800.3 | jiaoyi-s1 |
| 12 Nov 17:39:05.161 | 25,801.0 | jiaoyi-s1 |
| 12 Nov 17:40:06.388 | 25,802.9 | jiaoyi-s1 |


## §4. Complete London-window S3 ledger

All 12 newly opened positions were shorts on the demo accounts. The cohort contains **eight closes and four still open positions**; none is dropped because its outcome is unresolved. Initial R remains fixed at the actual fill-to-initial-stop distance. SL/TP values below are the broker protections recorded after the actual fill, not an earlier planning price.


| Account / position | Symbol / side | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- |
| 49236051 / 261979352 | SpotCrude / sell | 500 / 0.0500 | 12 Nov 13:30:05.590 / 86.03300 | 12 Nov 15:00:01.081 / 86.75300 | 86.88000 / 84.86400 | -3.60 / -0.850059 |
| 46130058 / 261979867 | SpotCrude / sell | 17600 / 1.7600 | 12 Nov 13:32:20.269 / 86.00300 | 12 Nov 14:11:56.028 / 86.86400 | 86.85600 / 84.82500 | -151.54 / -1.009405 |
| 47790949 / 261979868 | SpotCrude / sell | 17300 / 1.7300 | 12 Nov 13:32:20.228 / 86.00300 | 12 Nov 14:11:56.075 / 86.86400 | 86.85600 / 84.82500 | -148.95 / -1.009358 |
| 49236051 / 262007079 | ETHUSD / sell | 35 / 0.3500 | 12 Nov 15:00:36.759 / 2,042.43000 | 12 Nov 16:00:01.114 / 2,048.76000 | 2,055.78000 / 2,024.00000 | -2.22 / -0.475120 |
| 47790949 / 262007088 | ETHUSD / sell | 1100 / 11.0000 | 12 Nov 15:00:36.953 / 2,042.39000 | 12 Nov 16:00:01.213 / 2,048.81000 | 2,055.74000 / 2,023.96000 | -70.62 / -0.480899 |
| 46130058 / 262007168 | ETHUSD / sell | 1124 / 11.2400 | 12 Nov 15:00:39.685 / 2,042.71000 | 12 Nov 16:00:01.434 / 2,048.83000 | 2,056.06000 / 2,024.28000 | -68.79 / -0.458435 |
| 49236051 / 262076304 | BTCUSD / sell | 1 / 0.0100 | 12 Nov 17:26:01.293 / 73,317.19000 | 12 Nov 19:40:42.962 / 72,850.60000 | 73,642.13000 / 72,868.77000 | 4.67 / 1.437188 |
| 47790949 / 262076346 | BTCUSD / sell | 45 / 0.4500 | 12 Nov 17:26:02.401 / 73,317.93000 | 12 Nov 19:40:38.697 / 72,868.56000 | 73,642.86000 / 72,869.52000 | 202.22 / 1.382999 |
| 46130058 / 262101929 | XAUUSD / sell | 500 / 0.0500 | 12 Nov 18:07:13.954 / 3,942.25000 | Open / unavailable / Not verifiable / N/A | 3,970.38000 / 3,903.43000 | Not verifiable / N/A / Not verifiable / N/A |
| 47790949 / 262101991 | XAUUSD / sell | 500 / 0.0500 | 12 Nov 18:07:16.701 / 3,942.25000 | Open / unavailable / Not verifiable / N/A | 3,970.36000 / 3,903.45000 | Not verifiable / N/A / Not verifiable / N/A |
| 49236051 / 262132524 | SpotCrude / sell | 700 / 0.0700 | 12 Nov 19:16:35.196 / 86.23900 | Open / unavailable / Not verifiable / N/A | 86.88600 / 85.34600 | Not verifiable / N/A / Not verifiable / N/A |
| 46130058 / 262132606 | SpotCrude / sell | 23000 / 2.3000 | 12 Nov 19:16:43.244 / 86.22900 | Open / unavailable / Not verifiable / N/A | 86.87600 / 85.33600 | Not verifiable / N/A / Not verifiable / N/A |



| Position | Original R price / USD risk | MFE R | MAE R | BE amendment SGT | Exit / status | Stored deadline SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 261979352 | 0.8470 / 4.2350 | 0.345927 | 0.984652 | None recorded | native_session_end | 12 Nov 15:00:00.000 |
| 261979867 | 0.8530 / 150.1280 | 0.308324 | 1.009379 | None recorded | initial_stop_loss | 12 Nov 15:00:00.000 |
| 261979868 | 0.8530 / 147.5690 | 0.308324 | 1.009379 | None recorded | initial_stop_loss | 12 Nov 15:00:00.000 |
| 262007079 | 13.3500 / 4.6725 | 0.000000 | 0.674906 | None recorded | native_session_end | 12 Nov 16:00:00.000 |
| 262007088 | 13.3500 / 146.8500 | 0.000000 | 0.677903 | None recorded | native_session_end | 12 Nov 16:00:00.000 |
| 262007168 | 13.3500 / 150.0540 | 0.000000 | 0.653933 | None recorded | native_session_end | 12 Nov 16:00:00.000 |
| 262076304 | 324.9400 / 3.2494 | 1.419216 | 0.818428 | 12 Nov 19:35:47.847 | broker_protective_fill_at_or_better_than_tp | 12 Nov 20:00:00.000 |
| 262076346 | 324.9300 / 146.2185 | 1.380420 | 0.816176 | 12 Nov 19:35:47.630 | broker_protective_fill_at_or_better_than_tp | 12 Nov 20:00:00.000 |
| 262101929 | 28.1300 / 140.6500 | 0.012798 | 0.482403 | None recorded | open | 12 Nov 22:30:00.000 |
| 262101991 | 28.1100 / 140.5500 | 0.013874 | 0.481679 | None recorded | open | 12 Nov 22:30:00.000 |
| 262132524 | 0.6470 / 4.5290 | 0.241113 | 0.352396 | None recorded | open | 12 Nov 22:30:00.000 |
| 262132606 | 0.6470 / 148.8100 | 0.225657 | 0.367852 | None recorded | open | 12 Nov 22:30:00.000 |


The two BTC positions received entry-price stop amendments at **19:35:47.630** (262076346) and **19:35:47.848** (262076304), then filled their original targets at 19:40:38.697 and 19:40:42.962. They are TP wins, not breakeven exits. No other cohort position has a recorded +1R breakeven amendment before 19:45.

The first oil shorts were admitted during the Asian source window. The large demos hit their initial stops at 14:11:56; the small demo had a different entry/risk and closed at the 15:00 native session deadline. The ETH shorts opened around 15:00 from the carried 12:00 crypto source window and closed at its **16:00 deadline**, leaving about 59 minutes after entry. A fresh fill does not reset that frozen source-window deadline to four hours after entry. The gold and later oil positions have stored 22:30 deadlines and remain open in the observed book.

The measured in-position MFE/MAE uses every copied liquidation-side quote through the earlier of exit or 19:45. It is observed price excursion, not a guarantee of continuous tick capture or a broker fill at every extrema.

## §5. Realised cash, open marks and rolling goal


| Account | Window closes | Window net USD | Day net USD | Open S3 | Current S3 price-only float USD | Latest20 net wins | Realised payoff | Net-R profit factor |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | -0.01 | 0 | 0.0000 | 3/20 | 3.4277 | 0.6049 |
| 46130058 | 2 | -220.33 | -45.69 | 2 | 29.0600 | 7/20 | 1.6541 | 0.8907 |
| 47790949 | 3 | -17.35 | 145.42 | 1 | 6.7500 | 8/20 | 1.4893 | 0.9929 |
| 49236051 | 3 | -1.15 | -1.15 | 1 | 0.7490 | 6/20 | 1.5287 | 0.7056 |


Window attribution: oil **−$304.09**; ETH **−$141.63**; BTC **+$206.89**; total **−$238.83**. Full-day total: earlier +$337.40 plus this window −$238.83 = **+$98.57**, across 14 day closes. Human/S1 positions and open-mark changes are excluded.

The current open-S3 price-only projection is **+$36.559** across the four positions, using owner-book quotes around 20:29. This excludes fees/swaps/future slippage and is not verified net S3 unrealised P&L. The account-wide broker float includes S1 and human inventory and cannot be substituted for S3 float. Open realised netUSD/netR remain null.


| Open position | 19:45 executable mark | Quote time SGT | Price-only R from entry | Quote age seconds |
| --- | --- | --- | --- | --- |
| 262101929 | 3,945.19000 | 12 Nov 19:44:59.998 | -0.104515 | 0.001 |
| 262101991 | 3,945.16000 | 12 Nov 19:44:59.982 | -0.103522 | 0.018 |
| 262132524 | 86.26000 | 12 Nov 19:44:59.094 | -0.032457 | 0.905 |
| 262132606 | 86.26000 | 12 Nov 19:44:59.105 | -0.047913 | 0.894 |


Latest20 is the last official supervisor assessment before 19:45. It may span prior registrations and strategies within S3. At 20 trades, **15/20 = 75%** is the first integer count meeting 73%. None meets the win threshold. Realised payoff above 1.38 alone does not satisfy the combined goal. Cross-account copies and overlapping rolling windows are not independent validation. **Financial pass remains false.**

## §6. Four-hour post-close pip and R excursions

Broker pip sizes from each account's symbol metadata: **SpotCrude 0.01; ETHUSD 0.1; BTCUSD 0.1; XAUUSD 0.1**. The post-close cohort has no closed gold position. All trades are shorts, so executable-side movement uses **ask**, not mid or bid. Favourable movement is exit fill minus later ask; adverse movement is the opposite, each reported as a non-negative maximum. R uses the original position risk, never a new exit-anchored risk size.

The full observation window is (exit, exit+4h], bounded by the fixed 20:28 capture. Six windows reach 4.000 hours; two BTC windows are partial. Prices beyond 19:45 are used only in this expressly requested follow-up analysis. TP/SL tests use the **original absolute levels**, not 1.38R measured from the exit.


| Position / symbol | Pip size | Observed hours / status | Max favourable price / pips / R | Max adverse price / pips / R | Original TP touched? / first SGT | Original SL crossed? |
| --- | --- | --- | --- | --- | --- | --- |
| 261979352 / SpotCrude | 0.010 | 4.000000 / complete4.000h | 0.88300 / 88.30000 / 1.04250 | 1.07700 / 107.70000 / 1.27155 | Not observed | Yes |
| 261979867 / SpotCrude | 0.010 | 4.000000 / complete4.000h | 0.88100 / 88.10000 / 1.03283 | 0.96600 / 96.60000 / 1.13247 | Not observed | Yes |
| 261979868 / SpotCrude | 0.010 | 4.000000 / complete4.000h | 0.88100 / 88.10000 / 1.03283 | 0.96600 / 96.60000 / 1.13247 | Not observed | Yes |
| 262007079 / ETHUSD | 0.100 | 4.000000 / complete4.000h | 37.43000 / 374.30000 / 2.80375 | 4.47000 / 44.70000 / 0.33483 | Yes / 12 Nov 19:46:34.510 | Not observed |
| 262007088 / ETHUSD | 0.100 | 4.000000 / complete4.000h | 37.48000 / 374.80000 / 2.80749 | 4.42000 / 44.20000 / 0.33109 | Yes / 12 Nov 19:46:34.528 | Not observed |
| 262007168 / ETHUSD | 0.100 | 4.000000 / complete4.000h | 37.50000 / 375.00000 / 2.80899 | 4.40000 / 44.00000 / 0.32959 | Yes / 12 Nov 19:46:31.268 | Not observed |
| 262076304 / BTCUSD | 0.100 | 0.788066 / partial | 206.29000 / 2,062.90000 / 0.63486 | 230.64000 / 2,306.40000 / 0.70979 | Yes / 12 Nov 19:40:43.029 | Not observed |
| 262076346 / BTCUSD | 0.100 | 0.789251 / partial | 224.25000 / 2,242.50000 / 0.69015 | 212.68000 / 2,126.80000 / 0.65454 | Yes / 12 Nov 19:40:38.783 | Not observed |


Interpretation:

1. **Oil stop-outs were not rescued within four hours.** Both stopped large-demo trades crossed their original stops again and did not touch their original targets. The small-demo oil deadline close also saw the original stop crossed, without target completion. A later decline from the exit price is not equivalent to recovering the original entry-to-target objective.
2. **ETH shows a timing question worth freezing for offline comparison.** All three deadline-closed positions touched the original TP at **19:46:31–19:46:35**, after the activity cutoff and before their four-hour follow-ups ended. No original-SL crossing was observed in those paths. Their post-close favourable excursions were about 2.804–2.809original R from the losing exit. This is an observed price-level test, not a counterfactual replay of intervening +1R breakeven amendments, execution, costs and alternative position occupancy. No claim is made that an unchanged full trading system would necessarily have banked those hypothetical wins.
3. **BTC's partial paths are not stopped-trade recoveries.** Both trades already realised target wins. A later original-TP touch is expected when prices remain near/below their realised target. Their full windows end at **23:40:38.697** and **23:40:42.962 SGT**; no missing future movement is filled with zero.


| Position | 1h favourable signed pips / R | 2h pips / R | 4h pips / R | Largest observed quote-update gap s |
| --- | --- | --- | --- | --- |
| 261979352 | -49.200 / -0.580874 | -34.100 / -0.402597 | 62.600 / 0.739079 | 15.115 |
| 261979867 | -14.800 / -0.173505 | -51.100 / -0.599062 | 76.700 / 0.899179 | 17.240 |
| 261979868 | -14.800 / -0.173505 | -51.100 / -0.599062 | 76.700 / 0.899179 | 17.251 |
| 262007079 | 7.700 / 0.057678 | 43.000 / 0.322097 | 300.200 / 2.248689 | 19.929 |
| 262007088 | 8.200 / 0.061423 | 43.500 / 0.325843 | 301.700 / 2.259925 | 19.914 |
| 262007168 | 8.400 / 0.062921 | 44.500 / 0.333333 | 301.900 / 2.261423 | 19.910 |
| 262076304 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | 4.006 |
| 262076346 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | 3.960 |


Endpoint marks require a recorded quote at/before the horizon no more than 6 seconds old. Later, absent or stale quotes never substitute for an endpoint. Copy receipt hashes and row counts are published with the JSON; maxima reflect recorded updates and do not certify unobserved price paths.

## §7. London and crypto opening-reference verification

All 40 requested account-symbol-window slots have a recorded first two-sided quote within the compiled 15-second native opening bound. **38 match operational reference evidence**: London 22/24, crypto 16:00 8/8 and crypto 20:00 8/8. The two deliberately withheld London slots are live gold/silver under human ownership/pair protection. Opening reference existence and permission to enter are separate: the live US500/NAS100 opening evidence exists even though a later human pending order blocks new S3 entries.

The 20:00 crypto evidence is post-cutoff readiness information; it is not counted as London activity. Initial durable session rows can be stale after an in-memory recovery, as identified in№18,435. This audit matches original quote timestamps/prices against runtime or durable operational evidence instead of equating an initial refusal row with permanent failure. No late mid-session quote substitution was used.


| Account | Symbol | Open SGT | First quote delay s | Mid | Operational source / match |
| --- | --- | --- | --- | --- | --- |
| 43002148 | NAS100 | 12 Nov 15:00:00.000 | 0.117376 | 28,097.95000 | runtime_actor_opening_reference / yes |
| 43002148 | BTCUSD | 12 Nov 16:00:00.000 | 0.161553 | 73,568.61000 | stored_session_receipt / yes |
| 43002148 | BTCUSD | 12 Nov 20:00:00.000 | 0.099951 | 72,850.76000 | runtime_actor_opening_reference / yes |
| 49236051 | SpotBrent | 12 Nov 15:00:00.000 | 0.834083 | 95.32000 | runtime_actor_opening_reference / yes |
| 43002148 | XAUUSD | 12 Nov 15:00:00.000 | 0.047641 | 3,966.03500 | Live human/pair guard |
| 43002148 | ETHUSD | 12 Nov 16:00:00.000 | 0.161545 | 2,048.21000 | stored_session_receipt / yes |
| 43002148 | ETHUSD | 12 Nov 20:00:00.000 | 0.099950 | 2,020.65000 | runtime_actor_opening_reference / yes |
| 43002148 | XAGUSD | 12 Nov 15:00:00.000 | 0.283559 | 51.40400 | Live human/pair guard |
| 43002148 | US500 | 12 Nov 15:00:00.000 | 0.316825 | 7,584.00000 | runtime_actor_opening_reference / yes |
| 49236051 | NAS100 | 12 Nov 15:00:00.000 | 0.117400 | 28,097.95000 | runtime_actor_opening_reference / yes |
| 49236051 | US500 | 12 Nov 15:00:00.000 | 0.335217 | 7,584.00000 | runtime_actor_opening_reference / yes |
| 49236051 | XAGUSD | 12 Nov 15:00:00.000 | 0.295902 | 51.40400 | runtime_actor_opening_reference / yes |
| 49236051 | BTCUSD | 12 Nov 16:00:00.000 | 0.190446 | 73,568.61000 | stored_session_receipt / yes |
| 49236051 | BTCUSD | 12 Nov 20:00:00.000 | 0.121660 | 72,850.76000 | runtime_actor_opening_reference / yes |
| 49236051 | SpotCrude | 12 Nov 15:00:00.000 | 0.473633 | 86.75600 | runtime_actor_opening_reference / yes |
| 49236051 | ETHUSD | 12 Nov 16:00:00.000 | 0.190445 | 2,048.21000 | stored_session_receipt / yes |
| 49236051 | ETHUSD | 12 Nov 20:00:00.000 | 0.121657 | 2,020.65000 | runtime_actor_opening_reference / yes |
| 43002148 | SpotCrude | 12 Nov 15:00:00.000 | 0.443941 | 86.75600 | runtime_actor_opening_reference / yes |
| 49236051 | XAUUSD | 12 Nov 15:00:00.000 | 0.048299 | 3,966.03500 | runtime_actor_opening_reference / yes |
| 43002148 | SpotBrent | 12 Nov 15:00:00.000 | 0.846400 | 95.32000 | runtime_actor_opening_reference / yes |
| 46130058 | NAS100 | 12 Nov 15:00:00.000 | 0.117332 | 28,097.95000 | runtime_actor_opening_reference / yes |
| 46130058 | US500 | 12 Nov 15:00:00.000 | 0.330899 | 7,584.00000 | runtime_actor_opening_reference / yes |
| 47790949 | XAUUSD | 12 Nov 15:00:00.000 | 0.077585 | 3,966.03500 | runtime_actor_opening_reference / yes |
| 47790949 | BTCUSD | 12 Nov 16:00:00.000 | 0.022285 | 73,568.60000 | stored_session_receipt / yes |
| 47790949 | BTCUSD | 12 Nov 20:00:00.000 | 0.132751 | 72,850.76000 | runtime_actor_opening_reference / yes |
| 46130058 | XAUUSD | 12 Nov 15:00:00.000 | 0.069408 | 3,966.03000 | runtime_actor_opening_reference / yes |
| 46130058 | SpotBrent | 12 Nov 15:00:00.000 | 0.842619 | 95.32000 | runtime_actor_opening_reference / yes |
| 46130058 | ETHUSD | 12 Nov 16:00:00.000 | 0.168313 | 2,048.21000 | stored_session_receipt / yes |
| 46130058 | ETHUSD | 12 Nov 20:00:00.000 | 0.141899 | 2,020.65000 | runtime_actor_opening_reference / yes |
| 47790949 | XAGUSD | 12 Nov 15:00:00.000 | 0.282217 | 51.40400 | runtime_actor_opening_reference / yes |
| 47790949 | US500 | 12 Nov 15:00:00.000 | 0.330930 | 7,584.00000 | runtime_actor_opening_reference / yes |
| 47790949 | NAS100 | 12 Nov 15:00:00.000 | 0.118211 | 28,097.95000 | runtime_actor_opening_reference / yes |
| 47790949 | SpotCrude | 12 Nov 15:00:00.000 | 0.446156 | 86.75600 | runtime_actor_opening_reference / yes |
| 46130058 | SpotCrude | 12 Nov 15:00:00.000 | 0.444312 | 86.75600 | runtime_actor_opening_reference / yes |
| 47790949 | SpotBrent | 12 Nov 15:00:00.000 | 0.860064 | 95.32000 | runtime_actor_opening_reference / yes |
| 46130058 | XAGUSD | 12 Nov 15:00:00.000 | 0.280838 | 51.40400 | runtime_actor_opening_reference / yes |
| 46130058 | BTCUSD | 12 Nov 16:00:00.000 | 0.168315 | 73,568.61000 | stored_session_receipt / yes |
| 46130058 | BTCUSD | 12 Nov 20:00:00.000 | 0.141907 | 72,850.76000 | runtime_actor_opening_reference / yes |
| 47790949 | ETHUSD | 12 Nov 16:00:00.000 | 0.022284 | 2,048.26000 | stored_session_receipt / yes |
| 47790949 | ETHUSD | 12 Nov 20:00:00.000 | 0.132746 | 2,020.65000 | runtime_actor_opening_reference / yes |


## §8. Veto funnel, live inactivity and telemetry integrity


| Account | New arms | Consumed / intents | Fills | Final refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 13 | 0 | 0 | 0 | 13 |
| 46130058 | 16 | 10 | 4 | 6 | 3 |
| 47790949 | 14 | 8 | 4 | 4 | 5 |
| 49236051 | 15 | 6 | 4 | 2 | 6 |


Totals: **58 new arms, 24 consumed triggers, 24 persisted opening intents, 12 fills, 27 deep-retracement invalidations and 115 pair-reservation diagnostic rows**. Final refusals comprise 11 peer-hold-below 24 bp and one `ENTRY_EXPIRED` on demo 46130058 ETH at 18:16:32.521 SGT. That expired intent was explicitly refused before send; no broker order resulted. The longest recorded confirmation-to-final-guard interval is **18.324ms** across 12 successful samples. Touch-to-guard time includes waiting for the required turn and is not dispatch latency.

Each consumed decision ID joins to exactly one terminal outcome, including repeated reasons: **consumed without individual outcome=0; unknown entry-guard receipts=0**. Pair diagnostics are not a count of unique reservations; deduplicated idle/pre-admission rows are not a per-tick refusal census.

Live had 13 new arms and 13 invalidation receipts, but zero consumed triggers and zero persisted S3 opening intents. Those equal counts are not a one-to-one cohort match: some invalidations belong to carried arms and some late arms remained active. Recorded human/pair gates, macro vetoes and deep invalidations demonstrate deliberate gating. The retained admission diagnostics do not establish one exact pre-intent cause for every live non-entry. Current pending-margin forecasts must not be retrospectively labelled the cause of all earlier attempts.

Current live gates, separately observed at 20:29:


| Symbol | Exact gate | Observed message |
| --- | --- | --- |
| BTCUSD | peer_hold_below_24bps_or_opposing | Vetoed: peer ETHUSD < 24 bps or opposing (1.19 signed bps); mandatory at dispatch |
| ETHUSD | peer_hold_below_24bps_or_opposing | Vetoed: peer BTCUSD < 24 bps or opposing (10.20 signed bps); mandatory at dispatch |
| NAS100 | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol US500 |
| SpotBrent | invalidated_deep_retracement_gt_61_8pct | Invalidated: retraced > 61.8% without turn confirmation |
| SpotCrude | prepared_admission_unavailable | Preparing: background entry admission is not ready yet |
| US500 | yield_to_human_position_or_order | Blocked: yield_to_human_position_or_order |
| XAGUSD | pair_risk_reservation_active | Blocked: pair reservation held by sibling symbol XAUUSD |
| XAUUSD | yield_to_human_position_or_order | Blocked: yield_to_human_position_or_order |


Individual consumed-trigger outcomes:


| Consumed → outcome ID | Account / symbol | Consumed SGT | Terminal status / reason |
| --- | --- | --- | --- |
| 110880 → 110883 | 49236051 / SpotCrude | 12 Nov 13:30:05.112 | filled / Filled { position_id: Some(261979352), price: Some(86.033), volume: Some(500) } |
| 110908 → 110910 | 49236051 / ETHUSD | 12 Nov 13:31:29.171 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 110926 → 110928 | 47790949 / ETHUSD | 12 Nov 13:32:13.321 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 110929 → 110931 | 46130058 / ETHUSD | 12 Nov 13:32:16.120 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 110932 → 110941 | 46130058 / SpotCrude | 12 Nov 13:32:19.795 | filled / Filled { position_id: Some(261979867), price: Some(86.003), volume: Some(17600) } |
| 110933 → 110938 | 47790949 / SpotCrude | 12 Nov 13:32:19.797 | filled / Filled { position_id: Some(261979868), price: Some(86.003), volume: Some(17300) } |
| 111132 → 111133 | 49236051 / ETHUSD | 12 Nov 13:38:16.844 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 111153 → 111154 | 46130058 / ETHUSD | 12 Nov 13:38:37.206 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 111155 → 111156 | 47790949 / ETHUSD | 12 Nov 13:38:38.014 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 112634 → 112643 | 49236051 / ETHUSD | 12 Nov 15:00:36.418 | filled / Filled { position_id: Some(262007079), price: Some(2042.43), volume: Some(35) } |
| 112635 → 112655 | 47790949 / ETHUSD | 12 Nov 15:00:36.419 | filled / Filled { position_id: Some(262007088), price: Some(2042.39), volume: Some(1100) } |
| 112641 → 112659 | 46130058 / ETHUSD | 12 Nov 15:00:39.217 | filled / Filled { position_id: Some(262007168), price: Some(2042.71), volume: Some(1124) } |
| 112966 → 112971 | 46130058 / XAGUSD | 12 Nov 15:18:52.347 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 112967 → 112970 | 47790949 / XAGUSD | 12 Nov 15:18:52.351 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 113405 → 113406 | 47790949 / XAGUSD | 12 Nov 15:36:32.672 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 113407 → 113408 | 46130058 / XAGUSD | 12 Nov 15:36:32.978 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 115418 → 115427 | 49236051 / BTCUSD | 12 Nov 17:26:00.773 | filled / Filled { position_id: Some(262076304), price: Some(73317.19), volume: Some(1) } |
| 115423 → 115432 | 47790949 / BTCUSD | 12 Nov 17:26:01.936 | filled / Filled { position_id: Some(262076346), price: Some(73317.93), volume: Some(45) } |
| 115461 → 115462 | 46130058 / BTCUSD | 12 Nov 17:26:33.526 | refused / ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction |
| 116279 → 116284 | 46130058 / XAUUSD | 12 Nov 18:07:13.411 | filled / Filled { position_id: Some(262101929), price: Some(3942.25), volume: Some(500) } |
| 116281 → 116288 | 47790949 / XAUUSD | 12 Nov 18:07:16.190 | filled / Filled { position_id: Some(262101991), price: Some(3942.25), volume: Some(500) } |
| 116503 → 116505 | 46130058 / ETHUSD | 12 Nov 18:16:32.521 | refused / ENTRY_EXPIRED: entry dispatch deadline elapsed |
| 117462 → 117465 | 49236051 / SpotCrude | 12 Nov 19:16:34.708 | filled / Filled { position_id: Some(262132524), price: Some(86.239), volume: Some(700) } |
| 117472 → 117478 | 46130058 / SpotCrude | 12 Nov 19:16:42.842 | filled / Filled { position_id: Some(262132606), price: Some(86.229), volume: Some(23000) } |


## §9. New York 22:30 and crypto 20:00 readiness

All four actors are connected and unpaused, policy unchanged, with no unresolved S3 opening attempts in the health snapshot. NY 22:30 traditional preparations are **24/24 complete**; crypto 20:00 preparations are **8/8 complete** and their native opening references 8/8 are verified. Every preparation has metadata, spread/commission readiness, causal macro context, no source/macro error, no missing marks and no table error.

These full NY tables are independently keyed to 22:30. Their success does not prove that the previously identified missing 420/450-minute marks in the 15:00 London table were repaired. New NY opening quotes cannot exist yet at the20:28/20:29 snapshot. At dispatch, fresh quotes, human/pair ownership, turn confirmation, macro/peer direction, size and margin must still pass. Base readiness is not a promise of a trade.


| Account | Symbol | Prepared session SGT | Base ready | Full table ready | Missing marks |
| --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 43002148 | ETHUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 43002148 | NAS100 | 12 Nov 22:30:00.000 | True | True | 0 |
| 43002148 | SpotBrent | 12 Nov 22:30:00.000 | True | True | 0 |
| 43002148 | SpotCrude | 12 Nov 22:30:00.000 | True | True | 0 |
| 43002148 | US500 | 12 Nov 22:30:00.000 | True | True | 0 |
| 43002148 | XAGUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 43002148 | XAUUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 46130058 | BTCUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 46130058 | ETHUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 46130058 | NAS100 | 12 Nov 22:30:00.000 | True | True | 0 |
| 46130058 | SpotBrent | 12 Nov 22:30:00.000 | True | True | 0 |
| 46130058 | SpotCrude | 12 Nov 22:30:00.000 | True | True | 0 |
| 46130058 | US500 | 12 Nov 22:30:00.000 | True | True | 0 |
| 46130058 | XAGUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 46130058 | XAUUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 47790949 | BTCUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 47790949 | ETHUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 47790949 | NAS100 | 12 Nov 22:30:00.000 | True | True | 0 |
| 47790949 | SpotBrent | 12 Nov 22:30:00.000 | True | True | 0 |
| 47790949 | SpotCrude | 12 Nov 22:30:00.000 | True | True | 0 |
| 47790949 | US500 | 12 Nov 22:30:00.000 | True | True | 0 |
| 47790949 | XAGUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 47790949 | XAUUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 49236051 | BTCUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 49236051 | ETHUSD | 12 Nov 20:00:00.000 | True | True | 0 |
| 49236051 | NAS100 | 12 Nov 22:30:00.000 | True | True | 0 |
| 49236051 | SpotBrent | 12 Nov 22:30:00.000 | True | True | 0 |
| 49236051 | SpotCrude | 12 Nov 22:30:00.000 | True | True | 0 |
| 49236051 | US500 | 12 Nov 22:30:00.000 | True | True | 0 |
| 49236051 | XAGUSD | 12 Nov 22:30:00.000 | True | True | 0 |
| 49236051 | XAUUSD | 12 Nov 22:30:00.000 | True | True | 0 |


## §10. Conclusions and bounded next work

- Maintain the separation between cash accounting, price-path observations and deployable evidence. The window lost money despite two BTC targets; the day remains positive because earlier silver gains are included.
- Complete the two BTC follow-ups after 23:40:43 SGT in the next read-only audit. Retain this report's partial paths and capture horizon for comparison.
- If further research is authorised, freeze a matched offline comparison of source-window deadlines versus a specified remaining-horizon rule. Include the ETH paths that later touched TP **and** the oil paths that worsened. Preserve entries, costs, BE/TP, human/pair occupancy and holdouts; do not tune only to these three correlated ETH account copies.
- The opening-retry durable-success receipt gap and incomplete per-arm live admission attribution remain engineering observability follow-ups. This audit implements neither. Investigate late-London mark coverage separately from the complete NY tables.

No production change is justified merely by the observed target touches. This audit did not change alpha, exit timing, 1R breakeven, 1.38R target, risk, margin, pair caps, human protections, owner settings or Railway trading deployment.

## §11. Verification, release boundary and Gemini reading instructions

Four focused analytical checks passed: signed price/pip/R arithmetic and stale endpoints; full cash/volume cohort reconciliation; complete/partial horizons and TP-touch classification; provenance, unique outcome joins and human ownership. All 18quote-export hashes and row counts reconcile (**321,795 rows**). Raw source queries and calculation scripts are embedded in the JSON for inspection. Further source reproduction requires authorised database access; hashes prove identity, not independent correctness.

This branch changes only the named Markdown, HTML and JSON files in `doc/`. Required CI is `check`, `docker`, `engineering`; merge evidence and the post-merge deployment/actor-continuity check are returned in the final review block. Production baseline is deployment `c0fedaca-0370-4cec-9234-f59f8c7cafce`, commit `aebf404dedf89dbcdb8cee78c95a582eea042d51`, image `sha256:631e668c1ca138d5810f22fdab1ba27f3b7f4f34c6a28fa544c7ad09c2a9fad3`. The report path is outside Railway trading watch patterns.

**Gemini:** read the Markdown and JSON together at the merged commit. Inspect `ledger`, `post_close_summary`, `human_protection`, `opening_audit`, `telemetry` and `preparations`. Distinguish19:45 activity from20:28 follow-ups and20:29 snapshots; verify manual gold's broker-order link; do not count TP re-touches after profitable exits as rescued losses; do not promote three copies of one ETH path to independent edge evidence. Review proposed work as offline research/observability tasks, not deployment approval. **Financial pass: false.**


| Quote file | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 49236051-SpotCrude-2026-11-12-052959.csv.gz | 16214 | 6117b2f4d6057d54e62a4738c56c36a74ce39385c66d01525a53723b2a874a9f |
| 49236051-SpotCrude-2026-11-12-092959.csv.gz | 6634 | 4eb0ef0751d4eb847b335691c46ffccdab91dd7e904e366a99f9df263a49b3e9 |
| 46130058-SpotCrude-2026-11-12-053214.csv.gz | 16257 | b6126b6646fbe54edbb6d110587f7525f69be42d4bcdf7f9028346977e8d1f40 |
| 46130058-SpotCrude-2026-11-12-093214.csv.gz | 3276 | 8bd51c0b2bd4b1e2d628db1bae59ccf96da05901dc479b31cd5cb5c70b7ad2d9 |
| 47790949-SpotCrude-2026-11-12-053214.csv.gz | 16257 | 5fbfd2a52d12337fb744ecf99aeb5f94ed2d9dc306a598866a759ff7cec59b10 |
| 47790949-SpotCrude-2026-11-12-093214.csv.gz | 3276 | f3ddf6b20826d17963afc673e19f200a71390887cbd96da899dfac82be3a65be |
| 49236051-ETHUSD-2026-11-12-070030.csv.gz | 36528 | bbeb4e269472f4f6d659c6a48a459f0d8fe5d742fc7cad6b6f8c84c852a149a0 |
| 49236051-ETHUSD-2026-11-12-110030.csv.gz | 11682 | 1ff2f5bd48d167436de78a76d8d3b65e90cacde480183b02949b1016d3f6b318 |
| 47790949-ETHUSD-2026-11-12-070030.csv.gz | 36561 | 48e1d85953ed4716fb9616c039fc318eac70062ce3337f5afd9ff8e88cb1bd8a |
| 47790949-ETHUSD-2026-11-12-110030.csv.gz | 11692 | 02529368fa5895d5224ea4be769e92feb3c2a79f9a8bced8055ef630b6a0c0fe |
| 46130058-ETHUSD-2026-11-12-070033.csv.gz | 36520 | 2705a25046a23181cb709b5c42133ab669764c685b8b463de18dd7d030c5cc90 |
| 46130058-ETHUSD-2026-11-12-110033.csv.gz | 11678 | cb3d0b158e4a339d22af02a6da1d19e1b945c08240978ee0a8ac12a07080bddd |
| 49236051-BTCUSD-2026-11-12-092555.csv.gz | 41678 | 9f1a2ba4df7310b3d66b15ade3549b3c29d12ec1620bd52ab5a317fd9769479a |
| 47790949-BTCUSD-2026-11-12-092556.csv.gz | 41676 | 5632cca65711130e2639a8bf93d843df7599ed8eb039e4b5e62d83baf0fc50b9 |
| 46130058-XAUUSD-2026-11-12-100707.csv.gz | 14011 | 31763e96963aa26ed0873a95aa311bb990d35078402d92fe2ff4832a99a70423 |
| 47790949-XAUUSD-2026-11-12-100710.csv.gz | 14032 | f86df5665d16d1a91c9e691eeea29955ae9a527866a9854ff9f00534d23c00ea |
| 49236051-SpotCrude-2026-11-12-111629.csv.gz | 1916 | 6c4de393590a8c573cb378f7f29a57ae450e76e9ce632d9027dc9b1f726ffe90 |
| 46130058-SpotCrude-2026-11-12-111637.csv.gz | 1907 | 75bfc68ec64c97d52a1a72bdce66b131a20eca87af4348c15b24fd69b8d45152 |



| Source | SHA256 |
| --- | --- |
| opening-ticks.json | 4d6d84db54a96b7af9925b852a6d4b543cdc09cfd44d4a2f33ac17c0f5dc961c |
| preflight.json | 4bb30e05aceafe212f0ab40799cf8fb1346bb7f06f8ac109757bbe40bd095dcd |
| monitor.json | 999d2cf12397ae7b37d4dfde5586e43dbae805d3ffa603ec8f4fb1ec273d2d7b |
| human.json | cf303fca4fd8ce288a635df8c74daa979638f2451413b56ec033275236ae7957 |
| health.json | 87987f76cc088a8b8abe916982f1dfae53f93118771368a56f1c09d1c0e21a41 |
| activity.json | 3636ff3aa5bb3743eae7c6cb0cc6ec51e7671b0df6d3ebb2fe5be8f6168d1c74 |
| evidence.json | aca21d3aa2c13984827f86d7efdbacf327e142c99ffea991a5908b9814d5c93e |
| deployment-before.json | 6cab90a4e75ee1ed9ad74d89b5cb42a8c91d062e0a39cbe9e231c04b4049604a |
| runtime.json | dac806ac6fb1835dd28191fcfcf44781c18cbdcfa23a752e819752a3c03b41eb |
