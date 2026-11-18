# Wednesday Morning NY & Asia Review — №18,484

**Activity:** [17 November 2026 18:00:00, 18 November 2026 07:48:00) Asia/Singapore. **Read-only production/database/broker audit.**

[Measured JSON and evidence](WEDNESDAY_MORNING_NY_ASIA_REVIEW-18484-2026-11-18.json) · [Standalone HTML](WEDNESDAY_MORNING_NY_ASIA_REVIEW-18484-2026-11-18.html)

## §1. Assessment

- **Window S3 cash: −$446.46 from seven closes, zero positive-net wins.** Two BTC initial stops lost $313.84; two ETH entry-price breakeven stops lost $5.53 after execution; two NAS100 and one Silver scheduled exits lost $127.09. No S3 position was open at the cutoff or in the separately timed owner snapshot. [1,3]
- **Full Tuesday S3 cash: −$1,333.75. Wednesday 00:00→07:48: −$132.62. Post-PR #214 cumulative closing cash: +$1,268.20.** The latest-20 counts are live **3/20**, demos **5/20, 6/20, 12/20**. None meets **15/20 (75%)**. Financial pass remains false. [3]
- **The 120-second pre-close buffer executed on all three remaining traditional positions:** NAS100 filled at **05:57:01.559 / 05:57:01.582**, Silver at **05:57:02.600 SGT**. There were zero recorded rejection/error events or repeated S3 close attempts during **[05:59,07:05)**. No Gold S3 position required a Wednesday deadline close. [2]
- **Live cash is −$76.17**, exactly **$227.43 − $280.50 external NatGas close − $23.10 S1 AUS200 close**, with no deposit/withdrawal in the interval. Live net floating P&L is **+$2,490.72**, giving **$2,414.55 equity** at **07:50:54.699491 SGT**. Negative cash is not negative equity. [4]
- Live assessed current margin is **255.40%**, pending projection **73.48%**, and stressed projection **unverified**. A pending external NatGas order has no stop; that incomplete liability is not treated as zero. These distinct checks do not establish a blanket new-entry pass. **Zero recorded S3 interference on human inventory.** [4]
- **66 arms → 24 consumed triggers/intents/outcomes → 7 fills + 17 terminal refusals**: twelve peer-below-24 bp and five opposing-peer refusals. Missing individual outcomes and unknown entry-guard receipts are zero. Maximum confirmation-to-guard latency is **24.173 ms**. [5]
- **Nine excursion rows: six complete and three partial.** Both carried ETH tracks from №18,483 reached their original TP prices around **17 Nov 18:07:20 SGT**, after that report's price horizon. The two new ETH BE exits did not reach their original TPs. This updates observed paths, not realised profit. [6]
- Wednesday **07:00 Asian: 20 source quotes, 18 exact operational references, two human/pair-withheld live metals**. **08:00 Crypto 8/8** and **09:00 Brent 4/4** base/full preparation are present. **12:00 Crypto has no retained preparation yet**. **15:00 London has 20/24 base preparations and 0/24 complete sigma tables**: four Brent receipts are absent, while the other symbols lack late 420/450-minute coefficients. [7]

## §2. Frozen clock, anchor and cash continuity

The mandatory first clock read was **17 Nov 23:48:46.120250638 UTC / 18 Nov 07:48:46.120250638 SGT**. The directive therefore freezes the activity cutoff to the current floored minute, **07:48:00 SGT**. Its optional 08:05 cutoff does not apply. Later workflow completion does not extend this window or turn 08:00 preparation into a captured opening.

The exact handoff anchor is [№18,483 JSON](TUESDAY_ASIA_LONDON_REVIEW-18483-2026-11-17.json), merged in **PR #239**, commit `5de6d9a75ac4839309d8f302ee71eaa945f65e1f`, SHA256 `77c9b3329a1cbd58574bb6ab21f22992e4192b241138dc48103dec76489dee50`. Its cutoff was **17 Nov 18:00:00 SGT**, cumulative S3 cash **+$1,714.66**, Tuesday-to-cutoff S3 cash **−$1,019.91**, live balance **$227.43**, and no open S3 positions. Its two incomplete ETH excursion rows are **263516658 / 263516659**; their old closing cash is excluded from this window.

The **post-close price horizon is 18 Nov 07:48:47 SGT**. Owner book/margin response capture is **18 Nov 07:49:11.848926 SGT** and health capture **18 Nov 07:49:41.372807 SGT**. Account money timestamps appear in §4. These current observations are not an atomic historical mark at 07:48. All cash inclusion uses broker execution timestamps and half-open intervals. [3,4]


| Account | Window closes | Window S3 USD | Full Tuesday S3 USD | Wednesday 00:00→07:48 S3 USD | Post-PR214 S3 USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | 0.00 | -0.01 | 0 |
| 46130058 | 3 | -206.51 | -676.93 | -47.55 | 533.32 | 0 |
| 47790949 | 4 | -239.95 | -656.82 | -85.07 | 708.45 | 0 |
| 49236051 | 0 | 0.00 | 0.00 | 0.00 | 26.44 | 0 |


The bridges reconcile exactly:

- **Cumulative:** $1,714.66 − $446.46 = **$1,268.20**.
- **Two-day split:** prior Tuesday −$1,019.91 plus this window −$446.46 = full Tuesday −$1,333.75 plus Wednesday −$132.62 = **−$1,466.37**.
- The new Tuesday evening BTC pair accounts for **−$313.84**. Wednesday contributes the remaining **−$132.62**.

Cash is broker gross plus signed swap minus absolute allocated closing commission and conversion fee, using recorded money digits and decimal arithmetic. Closing deals are unique by account/deal ID. Opening commission is not charged twice. All account cash totals were reconciled independently through retained broker closing deals and the existing account monitor.

The cumulative interval remains **[10 Nov 09:42, 18 Nov 07:48) SGT**. It includes **+$412.88** from three positions entered before its start; it is a mixed-entry closing-cash cohort, not an isolated return attributable to PR #214. The repeated window query produced identical parsed evidence. Successful retained history fetches extend past cutoff on all four accounts, while the history worker's rolling `complete=false` flag remains disclosed. [3]

## §3. Wednesday 05:57 exits and daily rollover

The three positions below retained policy `native_preclose_v18455_120s`, original **Session.close 05:59**, and exit-only deadline **05:57**. The inspected helper matches production commit `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`. No original session close, entry band or risk setting was changed. [2]


| Account / position | Symbol | Close trigger SGT | Send SGT | Broker fill SGT | Fill | 05:57 ask / age seconds | Fill minus deadline ask | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 263762252 | NAS100 | 18 Nov 05:57:00.906268 | 18 Nov 05:57:01.184664 | 18 Nov 05:57:01.559000 | 27,019.200 | 27,025.300 / 0.060173 | -6.100 | -44.13 / -0.343050 |
| 47790949 / 263762285 | NAS100 | 18 Nov 05:57:00.873792 | 18 Nov 05:57:01.166300 | 18 Nov 05:57:01.582000 | 27,019.200 | 27,025.300 / 0.055587 | -6.100 | -43.41 / -0.337453 |
| 47790949 / 264006264 | XAGUSD | 18 Nov 05:57:00.873792 | 18 Nov 05:57:02.034739 | 18 Nov 05:57:02.600000 | 47.406 | 47.408 / 0.276480 | -0.002 | -39.55 / -0.266509 |

| Position | Send→broker fill ms | Send→local receipt ms | Local receipt SGT | Stored deadline / session close SGT |
| --- | --- | --- | --- | --- |
| 263762252 | 374.336 | 499 | 18 Nov 05:57:01.684656 | 18 Nov 05:57:00.000000 / 18 Nov 05:59:00.000000 |
| 263762285 | 415.700 | 545 | 18 Nov 05:57:01.711462 | 18 Nov 05:57:00.000000 / 18 Nov 05:59:00.000000 |
| 264006264 | 565.261 | 699 | 18 Nov 05:57:02.733858 | 18 Nov 05:57:00.000000 / 18 Nov 05:59:00.000000 |


All three were short, so the deadline comparison uses the last recorded **ask at or before 05:57:00**, not a later substituted price. NAS100 asks were 27,025.3, aged 60.173/55.587 ms; fills at 27,019.2 were 6.1 points lower. Silver's 47.408 ask was aged 276.480 ms; its 47.406 fill was 0.002 lower. These comparisons include price movement between deadline and fill; they are not a pure slippage estimate.

Across **[18 Nov 05:59,07:05) SGT**, the read-only query found **zero rejection/error or explicit MARKET_CLOSED events, zero S3 intents, and zero S3 session-end attempts**. All three positions were already closed. This verifies the execution-hygiene repair on NAS100 and Silver. The two Tuesday Gold exits were already covered by the anchor and are not counted again. Human Gold on Live remains external and protected.

## §4. Four-account money, margin and inventory

All four account actors were connected and unpaused on the expected production commit. The demo money observations are around **07:50:16 SGT**. Live money below uses a dedicated bounded, read-only broker P&L snapshot at **07:50:54.699491**, bracketed by identical position/order books and stable balances. Margin values are from the earlier, independently timed owner assessment; they are not recomputed by summing position margins under the broker's MAX aggregation. [4]


| Account | Money snapshot SGT | Balance USD | Net floating USD | Equity USD | Assessed margin % | Pending projected % | Pending stressed % | S3 / S1 / S2 / external | Pending |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 18 Nov 07:50:54.699491 | -76.17 | 2,490.72 | 2,414.55 | 255.40 | 73.48 | Unverified / N/A | 0 / 0 / 0 / 4 | 5 |
| 46130058 | 18 Nov 07:50:16.034000 | 24,924.26 | 138.10 | 25,062.36 | 14,995.89 | Unverified / N/A | Unverified / N/A | 0 / 5 / 0 / 0 | 0 |
| 47790949 | 18 Nov 07:50:16.093000 | 24,774.92 | 132.31 | 24,907.23 | 1,863.34 | Unverified / N/A | Unverified / N/A | 0 / 5 / 0 / 0 | 0 |
| 49236051 | 18 Nov 07:50:16.077000 | 821.21 | -0.78 | 820.43 | 94,303.45 | Unverified / N/A | Unverified / N/A | 0 / 1 / 0 / 0 | 0 |


Live's assessed current margin **255.40%** passes the 250% floor at that snapshot, unlike the earlier 115.82% reading. Pending projected margin **73.48%** breaches it. The stressed result is **unknown**, not zero and not a pass: the external NatGas pending order **402420853** has no recorded stop, and the observer requires complete marks/stops/conversion to qualify stress. Current policy and ownership gates remain separate. Demo pending projections are N/A because they have no observed pending entries.


| Account / ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | Current SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 56,284.20000 | Unverified / N/A |
| 43002148 / 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 56,323.40000 | Unverified / N/A |
| 43002148 / 657180931 | NVDA.US | human_external / position | buy | 400 / 4.0000 | 160.82500 | 149.66000 | Unverified / N/A |
| 43002148 / 658574004 | XAUUSD | human_external / position | buy | 3000 / 0.3000 | 3,841.86000 | 3,824.00000 | Unverified / N/A |
| 43002148 / 402440509 | NatGas | human_external / pending_entry | buy | 500000 / 0.5000 | 2.94200 | 2.86700 | Unverified / N/A |
| 43002148 / 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 / 402420853 | NatGas | human_external / pending_entry | buy | 500000 / 0.5000 | 2.85600 | Unverified / N/A | Unverified / N/A |
| 43002148 / 401496419 | Cocoa | human_external / pending_entry | buy | 300 / 3.0000 | 3,653.70000 | 3,057.70000 | 5,904.00000 |
| 43002148 / 402061476 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | Unverified / N/A |
| 46130058 / 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.68733 | Unverified / N/A |
| 46130058 / 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.11656 | Unverified / N/A |
| 46130058 / 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.60000 | Unverified / N/A |
| 46130058 / 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56234 | Unverified / N/A |
| 46130058 / 263178401 | USDCAD | s1 / position | buy | 1100000 / 0.1100 | 1.43006 | 1.41242 | Unverified / N/A |
| 47790949 / 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.68728 | Unverified / N/A |
| 47790949 / 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.11651 | Unverified / N/A |
| 47790949 / 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.60000 | Unverified / N/A |
| 47790949 / 262976216 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55202 | 0.56229 | Unverified / N/A |
| 47790949 / 263178410 | USDCAD | s1 / position | buy | 1100000 / 0.1100 | 1.43006 | 1.41247 | Unverified / N/A |
| 49236051 / 262796763 | ETHUSD | s1 / position | sell | 1 / 0.0100 | 1,746.87000 | 2,026.76000 | Unverified / N/A |


## §5. Live cash bridge, per-position P&L and human protection

**Live cash bridge: $227.43 − $280.50 external NatGas − $23.10 S1 AUS200 = −$76.17.** No deposit, withdrawal or dividend balance operation was recorded within this activity interval. The preceding $200 deposit and $0.04 dividend were already included in the anchor and are not added again. [4]


| Ownership | Position | Symbol | Broker close SGT | Fill | Gross USD | Swap USD | Commission / conversion USD | Net USD |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| External | 658793840 | NatGas | 17 Nov 21:51:47.245000 | 2.95300 | -210.00 | -70.50 | 0.00 | -280.50 |
| S1 | 656795267 | AUS200 | 18 Nov 03:57:36.813000 | 8,831.20000 | -22.50 | -0.60 | 0.00 | -23.10 |


NatGas **658793840** closed externally at **17 Nov 21:51:47.245**, 2.953: −$210.00 gross and −$70.50 swap. The retained S3/bot intent audit contains no command against it. Live S1 AUS200 **656795267** closed at **18 Nov 03:57:36.813**, 8,831.2: −$22.50 gross and −$0.60 swap. Its broker fill is a **closing protective order, type 4**, with stored stop **8,831.6**, labelled `jiaoyi-s1`. This was not an S3 trade or a new S3 instruction.

The four remaining live positions match the owner book and all are human/external. The broker supplied five additional zero P&L rows linked to unfilled pending entries; those relationships were verified and excluded from the open-position total. No nonzero or unexplained mark was discarded.


| Position | Symbol / owner | Gross floating USD | Recorded swap USD | Recorded commission USD | Net floating USD |
| --- | --- | --- | --- | --- | --- |
| 649992251 | JPN225 / human_external | 645.19 | -18.49 | 0.00 | 626.70 |
| 649772867 | JPN225 / human_external | 1,750.11 | -55.44 | 0.00 | 1,694.67 |
| 657180931 | NVDA.US / human_external | -23.86 | -0.45 | -0.08 | -24.39 |
| 658574004 | XAUUSD / human_external | 240.60 | -45.81 | -1.05 | 193.74 |


The live net marks sum to **+$2,490.72**. Adding cash **−$76.17** gives **$2,414.55 equity**, excluding unverified credit and prospective closing commission. The two JPN225 positions supply **+$2,321.37** of net floating P&L. Gold's **+$240.60** gross is reduced by **−$45.81 swap and −$1.05 commission** to **+$193.74 net**. These are recorded broker amounts.

**Zero recorded human interference:** there were **zero live program intents** in the activity window. Current human Gold remains label-isolated; XAUUSD yields to the human and XAGUSD is blocked by the sibling reservation. NatGas/JPN225 remain reserved. The human Gold SL changed from 3,830.0 to 3,824.0 without a retained bot instruction; it is not attributed to the bot. No S1 or S3 position is currently open on Live.

S1 is kept separate on the demos too. Their AUS200 positions closed for **−$130.66** and **−$124.81**, explaining the difference between each balance change and its S3 cash loss. Five S1 positions remain on each large demo. Demo 49236051 retains S1 ETH short **262796763**, with stop **2,026.76** after the recorded 06:27:29.075843 amendment; this remains external to S3 and constrains its crypto pair.


| Account | S1 closed position IDs | S1 closing cash USD |
| --- | --- | --- |
| 43002148 | 656795267 | -23.10 |
| 46130058 | 260823431 | -130.66 |
| 47790949 | 260743517 | -124.81 |
| 49236051 | None | 0.00 |


## §6. Complete overnight S3 ledger and latest-20 assessment

Seven new S3 positions opened and all seven closed before cutoff. There was no carried-in open S3 position. Entry labels, native volumes, original protection, broker deals and exit receipts reconcile. Original R is the actual fill-to-initial-SL distance; cash-based net R uses that original risk and recorded conversion. [1]


| Account / position | Symbol / dir | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit reason | Net USD / R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 263599092 | BTCUSD / sell | 38 / 0.3800 | 17 Nov 19:18:23.464000 / 64,384.59000 | 17 Nov 19:41:19.198000 / 64,792.18000 | 64,772.49000 / 63,849.28000 | initial stop | -154.88 / -1.050732 |
| 46130058 / 263599093 | BTCUSD / sell | 39 / 0.3900 | 17 Nov 19:18:23.484000 / 64,384.59000 | 17 Nov 19:41:19.202000 / 64,792.18000 | 64,772.49000 / 63,849.28000 | initial stop | -158.96 / -1.050760 |
| 46130058 / 263762252 | NAS100 / sell | 30 / 0.3000 | 18 Nov 00:03:06.041000 / 26,872.10000 | 18 Nov 05:57:01.559000 / 27,019.20000 | 27,300.90000 / 26,280.30000 | session end | -44.13 / -0.343050 |
| 47790949 / 263762285 | NAS100 / sell | 30 / 0.3000 | 18 Nov 00:03:07.537000 / 26,874.50000 | 18 Nov 05:57:01.582000 / 27,019.20000 | 27,303.30000 / 26,282.70000 | session end | -43.41 / -0.337453 |
| 47790949 / 263791928 | ETHUSD / sell | 1406 / 14.0600 | 18 Nov 00:31:25.947000 / 1,775.89000 | 18 Nov 00:58:34.357000 / 1,776.04000 | 1,786.54000 / 1,761.19000 | BE stop with slippage | -2.11 / -0.014091 |
| 46130058 / 263791927 | ETHUSD / sell | 1426 / 14.2600 | 18 Nov 00:31:25.877000 / 1,777.83000 | 18 Nov 00:58:47.455000 / 1,778.07000 | 1,788.38000 / 1,763.27000 | BE stop with slippage | -3.42 / -0.022733 |
| 47790949 / 264006264 | XAGUSD / sell | 35000 / 0.0700 | 18 Nov 04:36:24.338000 / 47.29300 | 18 Nov 05:57:02.600000 / 47.40600 | 47.71700 / 46.70700 | session end | -39.55 / -0.266509 |

| Position | Observed held MFE / MAE R | BE amendment receipt SGT | Stored deadline SGT | Exit policy |
| --- | --- | --- | --- | --- |
| 263599092 | 0.241557 / 1.047693 | None | 17 Nov 20:00:00.000000 | native_preclose_v18455_120s |
| 263599093 | 0.241557 / 1.047693 | None | 17 Nov 20:00:00.000000 | native_preclose_v18455_120s |
| 263762252 | 0.376866 / 0.789179 | None | 18 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 263762285 | 0.382463 / 0.783582 | None | 18 Nov 05:57:00.000000 | native_preclose_v18455_120s |
| 263791928 | 1.014085 / 0.783099 | 18 Nov 00:55:14.125619 | 18 Nov 04:00:00.000000 | native_preclose_v18455_120s |
| 263791927 | 1.207583 / 0.606635 | 18 Nov 00:55:04.498113 | 18 Nov 04:00:00.000000 | native_preclose_v18455_120s |
| 264006264 | 0.320755 / 0.886792 | None | 18 Nov 05:57:00.000000 | native_preclose_v18455_120s |


**BTC:** the 19:18 Tuesday short pair stopped at 19:41 for **−$313.84**, about −1.051 R each. Their observed favourable excursion stayed near 0.242 R; they never activated entry-price BE.

**ETH:** the 00:31 Wednesday pair activated the unchanged +1 R→entry BE rule at **00:55:04.498113 / 00:55:14.125619** and closed at **00:58:47.455 / 00:58:34.357** for **−$3.42 / −$2.11**. Those are BE-classified exits but negative-net cash outcomes, so they are not counted as net wins or exact-zero trades.

**NAS100/Silver:** scheduled exits lost **−$127.09**. These positions were closed before maintenance as intended. Seven account positions share four setup groups, so the seven outcomes are not seven independent market hypotheses. There was no +1.38 R target completion in this window.


| Account | Latest20 wins / count | Win rate | Rolling Net R | Realised payoff (R) | Profit factor (R) | Assessment SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | -5.967682 | 3.4277 | 0.6049 | 18 Nov 07:42:12.804214 |
| 46130058 | 5/20 | 25.00% | -4.247828 | 1.6042 | 0.5347 | 18 Nov 07:45:20.031444 |
| 47790949 | 6/20 | 30.00% | -4.259420 | 1.2495 | 0.5355 | 18 Nov 07:45:26.483740 |
| 49236051 | 12/20 | 60.00% | 6.204701 | 1.3690 | 2.0534 | 18 Nov 07:46:29.893457 |


The large demos fell from **6/20→5/20** and **7/20→6/20** net wins. Live 3/20 and small-demo 12/20 are unchanged. The rolling Net R can move differently from win count as older trades leave the window; it is reported directly rather than inferred from win percentage. No account meets **15/20 (75%)**, and neither large demo has positive rolling Net R. **Financial pass=false.**

## §7. Mandatory four-hour post-close excursions

The horizon is frozen at **18 Nov 07:48:47 SGT**. There are **nine rows**, consisting of **seven current-window closes plus two prior partial follow-ups**. Six have complete four-hour clocks; the NAS100/Silver deadline exits have **1.8623–1.8626 hours** observed. Their full clocks finish around **09:57:01–09:57:03 SGT**. [6]

All nine are short, so executable-side movement uses the **ask**, signed from the actual exit fill and divided by the original price R. Actual broker pip sizes in this cohort are **BTC/ETH 0.1, NAS100 1.0, XAGUSD 0.01**. Gold uses 0.1 and Crude/Brent 0.01 when present; neither is in this excursion cohort. No FX pip convention is guessed. Favourable/adverse columns show **price movement / pips / original R**; TP/SL crossings use the original absolute price levels.


| Position / symbol / scope | Exit / original SL / TP | Hours / status | Max favourable price / pips / R | Max adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 263599092 / BTCUSD / current | 64,792.180 / 64,772.490 / 63,849.280 | 4.000000 COMPLETE | 336.89000 / 3,368.90 / 0.868497 | 629.69000 / 6,296.90 / 1.623331 | -1,251.30 / -0.322583 | -176.80 / -0.045579 | 2,923.50 / 0.753674 | False / True |
| 263599093 / BTCUSD / current | 64,792.180 / 64,772.490 / 63,849.280 | 4.000000 COMPLETE | 336.89000 / 3,368.90 / 0.868497 | 629.69000 / 6,296.90 / 1.623331 | -1,251.30 / -0.322583 | -179.80 / -0.046352 | 2,904.30 / 0.748724 | False / True |
| 263762252 / NAS100 / current | 27,019.200 / 27,300.900 / 26,280.300 | 1.862623 PARTIAL | 0.00000 / 0.00 / 0.000000 | 509.70000 / 509.70 / 1.188666 | stale_asof_quote | not_elapsed_at_capture | not_elapsed_at_capture | False / True |
| 263762285 / NAS100 / current | 27,019.200 / 27,303.300 / 26,282.700 | 1.862616 PARTIAL | 0.00000 / 0.00 / 0.000000 | 509.70000 / 509.70 / 1.188666 | stale_asof_quote | not_elapsed_at_capture | not_elapsed_at_capture | False / True |
| 263791928 / ETHUSD / current | 1,776.040 / 1,786.540 / 1,761.190 | 4.000000 COMPLETE | 6.18000 / 61.80 / 0.580282 | 30.24000 / 302.40 / 2.839437 | -106.70 / -1.001878 | -133.30 / -1.251643 | -194.30 / -1.824413 | False / True |
| 263791927 / ETHUSD / current | 1,778.070 / 1,788.380 / 1,763.270 | 4.000000 COMPLETE | 8.21000 / 82.10 / 0.778199 | 28.21000 / 282.10 / 2.673934 | -82.60 / -0.782938 | -110.80 / -1.050237 | -171.60 / -1.626540 | False / True |
| 264006264 / XAGUSD / current | 47.406 / 47.717 / 46.707 | 1.862333 PARTIAL | 0.00500 / 0.50 / 0.011792 | 0.23900 / 23.90 / 0.563679 | stale_asof_quote | not_elapsed_at_capture | not_elapsed_at_capture | False / False |
| 263516658 / ETHUSD / prior follow-up | 1,833.030 / 1,832.850 / 1,797.410 | 4.000000 COMPLETE | 49.07000 / 490.70 / 3.295500 | 2.63000 / 26.30 / 0.176629 | 27.50 / 0.184688 | 181.80 / 1.220954 | 422.10 / 2.834788 | True / True |
| 263516659 / ETHUSD / prior follow-up | 1,833.040 / 1,832.840 / 1,797.400 | 4.000000 COMPLETE | 49.08000 / 490.80 / 3.296172 | 2.62000 / 26.20 / 0.175957 | 27.60 / 0.185359 | 181.90 / 1.221625 | 422.20 / 2.835460 | True / True |


**Prior ETH follow-ups 263516658 /263516659:** both are now complete. Their original TPs were first crossed at **17 Nov 18:07:20.512865 /18:07:20.504169 SGT**, after the prior audit's 18:00:15 horizon and before their 19:02:31 endpoints. Maximum recovery from exit was about 3.296 R. The original stops had already been crossed; observing a later target does not establish an executable winning hold with those stops unchanged. Their realised −$153.86/−$154.82 remains solely in the prior cash window.

**New BTC stops:** neither original TP was crossed within 4 h. Favourable recovery was 0.8685 R from exit, while adverse movement reached 1.6233 R. **New ETH BE exits:** neither original target was crossed; adverse movement reached 2.674–2.839 R from the exit and both original stops were subsequently crossed. The BE exits therefore preceded substantial observed adverse movement on these two paths; this is not an independent profitability test.

**NAS100/Silver partials:** NAS100 moved adversely 1.1887 R after the scheduled exits and crossed its original SL after reopen. Silver moved adversely 0.5637 R, without an original SL or TP crossing yet. The 1 h endpoints fall at 06:57 during maintenance; last quotes are about 58 minutes old, so movement is **unavailable**, not zero. The 2 h/4 h endpoints are unelapsed at the frozen horizon.


| Excursion scope | Rows | Complete4h | Partial | Non-BE initial stops later crossing original TP | BE stops later crossing original TP |
| --- | --- | --- | --- | --- | --- |
| Current activity | 7 | 4 | 3 | 0 | 0 |
| Prior follow-ups | 2 | 2 | 0 | 2 | 0 |
| Combined path audit | 9 | 6 | 3 | 2 | 0 |


There are 18 fresh elapsed checkpoints, three stale maintenance checkpoints and six future checkpoints. The overnight traditional gap is about 61–62 minutes. Complete clock duration does not imply continuous executable quotes. No interpolation or hypothetical fill has been used, and non-BE stop counts explicitly exclude BE exits.

## §8. Execution/veto funnel and telemetry integrity


| Account | Arms | Consumed / persisted / outcomes | Fills | Terminal refusals | Deep retracement invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 21 | 0 / 0 / 0 | 0 | 0 | 24 |
| 46130058 | 19 | 12 / 12 / 12 | 3 | 9 | 5 |
| 47790949 | 19 | 12 / 12 / 12 | 4 | 8 | 5 |
| 49236051 | 7 | 0 / 0 / 0 | 0 | 0 | 8 |

| Terminal refusal reason | Count |
| --- | --- |
| ENTRY_PEER_HOLD_BELOW_24BPS: peer no longer holds 24 basis points in the impulse direction | 12 |
| ENTRY_PEER_CONFLICT: fresh peer opposes the selected impulse | 5 |


All 24 consumed IDs join to an individual terminal outcome and an opening intent created between consumption and outcome. **7 fills +17 refusals =24**. Missing individual outcomes=0 and unknown guard receipts=0. There are **42 >61.8% invalidations** and **178 pair-reservation diagnostic records**. Diagnostic repetition is not counted as independent rejected trades. The full counted activity starts at 18:00; Tuesday 16:00 crypto is included only as source-session context for later activity. [5]

Maximum confirmation-to-final-guard latency is **24.173 ms**. This is not broker send-to-fill latency. Canonical `refusals_all_stages_complete=false` is retained: a complete consumed-trigger outcome ledger does not prove exhaustive coverage of deduplicated idle/pre-arm diagnostics.


| Account | Symbol | Gate in separately timed current health snapshot |
| --- | --- | --- |
| 43002148 | BTCUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -9.485%, 20d: -20.480%) |
| 43002148 | ETHUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -9.701%, 20d: -23.177%) |
| 43002148 | NAS100 | Waiting: opening impulse below 48 bps |
| 43002148 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.243%, 20d: -16.074%) |
| 43002148 | SpotCrude | Waiting: opening impulse below 48 bps |
| 43002148 | US500 | Waiting: opening impulse below 48 bps |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -9.485%, 20d: -20.509%) |
| 46130058 | ETHUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -9.701%, 20d: -23.177%) |
| 46130058 | NAS100 | Waiting: opening impulse below 48 bps |
| 46130058 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.243%, 20d: -16.074%) |
| 46130058 | SpotCrude | Waiting: opening impulse below 48 bps |
| 46130058 | US500 | Waiting: opening impulse below 48 bps |
| 46130058 | XAGUSD | Latest assessment: impulse_below_48bps (2026-11-17 23:49:06.493341872 UTC) |
| 46130058 | XAUUSD | Waiting: opening impulse below 48 bps |
| 47790949 | BTCUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -9.485%, 20d: -20.509%) |
| 47790949 | ETHUSD | Vetoed: 5d/20d macro drift opposes impulse (5d: -9.701%, 20d: -23.177%) |
| 47790949 | NAS100 | Waiting: opening impulse below 48 bps |
| 47790949 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.243%, 20d: -16.074%) |
| 47790949 | SpotCrude | Waiting: opening impulse below 48 bps |
| 47790949 | US500 | Waiting: opening impulse below 48 bps |
| 47790949 | XAGUSD | Latest assessment: impulse_below_48bps (2026-11-17 23:49:06.493350822 UTC) |
| 47790949 | XAUUSD | Waiting: opening impulse below 48 bps |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Waiting: opening impulse below 48 bps |
| 49236051 | SpotBrent | Vetoed: 5d/20d macro drift opposes impulse (5d: 1.243%, 20d: -16.074%) |
| 49236051 | SpotCrude | Waiting: opening impulse below 48 bps |
| 49236051 | US500 | Waiting: opening impulse below 48 bps |
| 49236051 | XAGUSD | Latest assessment: impulse_below_48bps (2026-11-17 23:49:06.492745372 UTC) |
| 49236051 | XAUUSD | Waiting: opening impulse below 48 bps |


## §9. Overnight openings and Wednesday readiness

Across the 76 requested source-session slots there are **58 exact operational quote proofs, seven recovered-midpoint proofs and eleven reference-withheld slots**. The eight Tuesday 16:00 slots are inherited session context; reopening the evidence does not recount prior activity or cash. Quotes and operational evidence are separate from permission to enter. [7]


| Session open SGT | Expected/source slots | Exact operational | Recovered midpoint only | Reference withheld | Ownership-constrained slots |
| --- | --- | --- | --- | --- | --- |
| 17 Nov 16:00:00.000000 | 8 / 8 | 7 | 0 | 1 | 2 |
| 17 Nov 20:00:00.000000 | 8 / 8 | 6 | 0 | 2 | 2 |
| 17 Nov 22:30:00.000000 | 24 / 24 | 15 | 7 | 2 | 2 |
| 18 Nov 00:00:00.000000 | 8 / 8 | 6 | 0 | 2 | 2 |
| 18 Nov 04:00:00.000000 | 8 / 8 | 6 | 0 | 2 | 2 |
| 18 Nov 07:00:00.000000 | 20 / 20 | 18 | 0 | 2 | 2 |


Wednesday 07:00 Asian has **20/20 source quotes, 18 exact operational references and two live metals withheld by human/pair locks**. Metals' versioned broker observation begins 07:01 under the 07:00 session identity. SpotCrude's first observations arrive 6.914–6.982 s after 07:00, inside the 15 s capture policy. No later mid-session price has been relabelled as an opening. Recovered NY midpoint evidence certifies the numerical opening, not immutable full bid/ask timestamp storage.


| Account | Symbol | Session SGT | Observation start SGT | First recorded quote SGT | Delay seconds | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.130666 | 0.130666 | exact_quote_receipt |
| 43002148 | BTCUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.079430 | 0.079430 | exact_quote_receipt |
| 43002148 | BTCUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.021584 | 0.021584 | exact_quote_receipt |
| 43002148 | BTCUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.437881 | 0.437881 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.130664 | 0.130664 | exact_quote_receipt |
| 43002148 | ETHUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.079414 | 0.079414 | exact_quote_receipt |
| 43002148 | ETHUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.021585 | 0.021585 | exact_quote_receipt |
| 43002148 | ETHUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.158401 | 0.158401 | exact_quote_receipt |
| 49236051 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.043538 | 0.043538 | exact_quote_receipt |
| 49236051 | BTCUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.068865 | 0.068865 | ownership_locked |
| 49236051 | BTCUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.195771 | 0.195771 | ownership_locked |
| 49236051 | BTCUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.421973 | 0.421973 | ownership_locked |
| 49236051 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.105227 | 0.105227 | ownership_locked |
| 49236051 | ETHUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.068856 | 0.068856 | ownership_locked |
| 49236051 | ETHUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.195770 | 0.195770 | ownership_locked |
| 49236051 | ETHUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.221334 | 0.221334 | ownership_locked |
| 47790949 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.010058 | 0.010058 | exact_quote_receipt |
| 47790949 | BTCUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.111419 | 0.111419 | exact_quote_receipt |
| 47790949 | BTCUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.037324 | 0.037324 | exact_quote_receipt |
| 47790949 | BTCUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.414359 | 0.414359 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.131905 | 0.131905 | exact_quote_receipt |
| 46130058 | ETHUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.029910 | 0.029910 | exact_quote_receipt |
| 46130058 | ETHUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.194264 | 0.194264 | exact_quote_receipt |
| 46130058 | ETHUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.221258 | 0.221258 | exact_quote_receipt |
| 46130058 | BTCUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.023301 | 0.023301 | exact_quote_receipt |
| 46130058 | BTCUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.094164 | 0.094164 | exact_quote_receipt |
| 46130058 | BTCUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.194265 | 0.194265 | exact_quote_receipt |
| 46130058 | BTCUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.407875 | 0.407875 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.000000 | 17 Nov 16:00:00.136749 | 0.136749 | exact_quote_receipt |
| 47790949 | ETHUSD | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.000000 | 17 Nov 20:00:00.055428 | 0.055428 | exact_quote_receipt |
| 47790949 | ETHUSD | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.000000 | 18 Nov 00:00:00.037323 | 0.037323 | exact_quote_receipt |
| 47790949 | ETHUSD | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.000000 | 18 Nov 04:00:00.223179 | 0.223179 | exact_quote_receipt |
| 43002148 | NAS100 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.100866 | 0.100866 | exact_quote_receipt |
| 49236051 | SpotBrent | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.277292 | 0.277292 | exact_quote_receipt |
| 43002148 | XAUUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.100859 | 0.100859 | ownership_locked |
| 43002148 | XAGUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.151116 | 0.151116 | ownership_locked |
| 43002148 | US500 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.100867 | 0.100867 | exact_quote_receipt |
| 49236051 | NAS100 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.111753 | 0.111753 | exact_quote_receipt |
| 49236051 | US500 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.334419 | 0.334419 | exact_quote_receipt |
| 49236051 | XAGUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.169127 | 0.169127 | exact_quote_receipt |
| 49236051 | SpotCrude | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.559837 | 0.559837 | recovered_midpoint_only |
| 43002148 | SpotCrude | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.730299 | 0.730299 | recovered_midpoint_only |
| 49236051 | XAUUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.111748 | 0.111748 | recovered_midpoint_only |
| 43002148 | SpotBrent | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.195241 | 0.195241 | exact_quote_receipt |
| 46130058 | NAS100 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.096660 | 0.096660 | exact_quote_receipt |
| 46130058 | US500 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.314915 | 0.314915 | exact_quote_receipt |
| 47790949 | XAUUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.103397 | 0.103397 | recovered_midpoint_only |
| 46130058 | XAUUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.096655 | 0.096655 | recovered_midpoint_only |
| 46130058 | SpotBrent | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.205333 | 0.205333 | exact_quote_receipt |
| 47790949 | XAGUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.161265 | 0.161265 | exact_quote_receipt |
| 47790949 | US500 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.326689 | 0.326689 | exact_quote_receipt |
| 47790949 | SpotCrude | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.531192 | 0.531192 | recovered_midpoint_only |
| 46130058 | SpotCrude | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.559773 | 0.559773 | recovered_midpoint_only |
| 47790949 | NAS100 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.103403 | 0.103403 | exact_quote_receipt |
| 47790949 | SpotBrent | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.199538 | 0.199538 | exact_quote_receipt |
| 46130058 | XAGUSD | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.000000 | 17 Nov 22:30:00.205328 | 0.205328 | exact_quote_receipt |
| 43002148 | NAS100 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.487631 | 1.487631 | exact_quote_receipt |
| 43002148 | XAUUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.586507 | 0.586507 | ownership_locked |
| 43002148 | XAGUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.586506 | 0.586506 | ownership_locked |
| 43002148 | US500 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.487631 | 1.487631 | exact_quote_receipt |
| 49236051 | NAS100 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.528242 | 1.528242 | exact_quote_receipt |
| 49236051 | US500 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.528241 | 1.528241 | exact_quote_receipt |
| 49236051 | XAGUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.614491 | 0.614491 | exact_quote_receipt |
| 49236051 | SpotCrude | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:06.914446 | 6.914446 | exact_quote_receipt |
| 43002148 | SpotCrude | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:06.973651 | 6.973651 | exact_quote_receipt |
| 49236051 | XAUUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.614491 | 0.614491 | exact_quote_receipt |
| 46130058 | NAS100 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.506507 | 1.506507 | exact_quote_receipt |
| 46130058 | US500 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.506507 | 1.506507 | exact_quote_receipt |
| 47790949 | XAUUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.645946 | 0.645946 | exact_quote_receipt |
| 46130058 | XAUUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.670121 | 0.670121 | exact_quote_receipt |
| 47790949 | XAGUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.645945 | 0.645945 | exact_quote_receipt |
| 47790949 | US500 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.535263 | 1.535263 | exact_quote_receipt |
| 47790949 | SpotCrude | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:06.981894 | 6.981894 | exact_quote_receipt |
| 46130058 | SpotCrude | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:06.942312 | 6.942312 | exact_quote_receipt |
| 47790949 | NAS100 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:00.000000 | 18 Nov 07:00:01.535264 | 1.535264 | exact_quote_receipt |
| 46130058 | XAGUSD | 18 Nov 07:00:00.000000 | 18 Nov 07:01:00.000000 | 18 Nov 07:01:00.670120 | 0.670120 | exact_quote_receipt |

| Wednesday session SGT | Expected slots | Retained prep receipts | Base ready | Full marks ready | Missing pre-cutoff receipts |
| --- | --- | --- | --- | --- | --- |
| 18 Nov 07:00:00.000000 | 20 | 20 | 20 | 20 | None |
| 18 Nov 08:00:00.000000 | 8 | 8 | 8 | 8 | None |
| 18 Nov 09:00:00.000000 | 4 | 4 | 4 | 4 | None |
| 18 Nov 12:00:00.000000 | 8 | 0 | 0 | 0 | 43002148/BTCUSD, 43002148/ETHUSD, 46130058/BTCUSD, 46130058/ETHUSD, 47790949/BTCUSD, 47790949/ETHUSD, 49236051/BTCUSD, 49236051/ETHUSD |
| 18 Nov 15:00:00.000000 | 24 | 20 | 20 | 0 | 43002148/SpotBrent, 46130058/SpotBrent, 47790949/SpotBrent, 49236051/SpotBrent |


**08:00 Crypto 8/8** and **09:00 Brent 4/4** have base inputs and full marks ready before cutoff; their future openings are not yet verified by this audit. Wednesday Brent's native opening is 09:00.

**12:00 Crypto:** no retained pre-cutoff receipt, so readiness is not yet verified for all eight slots. **15:00 London:**20 base receipts exist for the five Asian traditional symbols; Brent's four are not yet retained. Every available London table lacks 420/450-minute coefficients, with **12 same-slot endpoints;14 required**. Thus London is 20/24 base and 0/24 full-table ready. Earlier coefficients, including 120 m, are available on those 20 receipts.

The inspected preparation worker warms the **current and next** session per symbol. At 07:48, crypto's next window is 08:00 and Brent's next is 09:00, so 12:00 Crypto and 15:00 Brent lie beyond that immediate pair. Their absent receipts are recorded as **not yet observed**, not mislabeled completed or diagnosed as a failed collection. The late London coefficient shortage is a separate, measured limitation requiring a later readiness check. No production repair is made in this audit.

## §10. Assessment and next handoff

Execution hygiene and the individual-trigger ledger are functioning in the observed interval: scheduled exits completed before maintenance, and all 24 consumed triggers have outcomes. Trading performance has not qualified: seven current closes were net losses, and large-demo rolling wins are 5/20 and 6/20. This is insufficient evidence to claim an edge or authorize a new exit rule.

The post-close evidence is mixed. The older ETH stopped pair later reached its targets, while the new BTC pair did not and the newer ETH BE pair subsequently moved against the trade. These are paired account copies and selected observed paths, not a clean independent strategy comparison. Their accounting must not be pooled with realised profits or treated as proof that holding all stops would help.

The next activity window starts exactly **18 Nov 07:48:00 SGT**. Carry only the three partial excursion IDs **263762252,263762285,264006264** until their four-hour endpoints around **09:57:01–09:57:03**. Verify the 08:00 and 09:00 actual openings, then 12:00 Crypto preparation and 15:00 London coverage. Keep Live's negative cash, positive floating equity, passing current-margin assessment, failing pending projection and unverified stress as separate facts; no human position/order is modified by this report.

## §11. Evidence, checks, publication and Gemini handoff

Sources: **[1]** retained broker deals, S3 orders/fills, guards and original protection; **[2]** stored exit deadlines, compiled helper, exact deadline quotes, broker/local fill timestamps and rollover queries; **[3]** repeatable-read activity/day/cumulative monitor queries and the hashed merged anchor; **[4]** owner inventory/margins, recorded cashflows and bounded read-only broker per-position P&L; **[5]** consumed-ID→intent→terminal-outcome joins; **[6]**17 bounded copies of existing PostgreSQL tick segments, 554,304 rows; **[7]** opening quote proofs and pre-cutoff preparation receipts.

Seven focused checks passed: half-open periods and cash bridges; anchor/input/quote hash reconciliation; pip/R arithmetic, prior-track immutability and missing endpoints; causal trigger outcomes; live cash/P&L/ownership; honest future preparation denominators; and the three actual pre-close executions/no retry storm. Source queries and calculation/test code are embedded in the JSON for inspection. Private quote files remain local with published hashes and exact read-only extraction queries.

The dedicated broker snapshot sent **seven frames: two authentication frames and five read-only account requests**. It sent **zero orders, subscriptions, token refreshes or historical market-data requests**. There were no production/database mutations. Raw credentials and private trader metadata are excluded from publication. Source contents were treated as evidence, not instructions.

Only the three requested files under `doc/` change. Production code, strategy, risk, configuration, protected paths and Railway deployment remain untouched. Required CI is **check, docker, engineering**. The baseline Trading deployment is `3d8b18d5-4d9f-4007-8fff-22eb25d5ea73`, production commit `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`; Railway watch patterns exclude these docs. The merged file hashes and post-merge deployment/account-session continuity are verified separately and returned with the actual PR and merge SHA.

**Gemini instruction:** read this Markdown and matching JSON. Reconcile −$446.46 window, −$1,333.75 full Tuesday, −$132.62 Wednesday and +$1,268.20 cumulative S3 cash; separate the external NatGas and S1 AUS200 cash bridge; verify the three 05:57 buffered exits, 24 individual outcomes, two completed prior ETH TP crossings and three new partial tracks. Preserve the 07:48 activity cutoff, 07:48:47 price horizon and later money snapshots. Assess 15/20 qualification honestly, and distinguish absent future preparations from measured late London coefficient shortages. This is a read-only audit, not authorization to change trading rules.


| Tick export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-BTCUSD-2026-11-17-111817.csv.gz | 58778 | 29588761b27508ac03e851b892232ce6b4cb1896823b116ac48c73a7c3d031dd |
| 46130058-BTCUSD-2026-11-17-151817.csv.gz | 6003 | 82045c35ad479001c149f03b607834b3dc16848471359df26c07e854dca3a096 |
| 46130058-ETHUSD-2026-11-17-070225.csv.gz | 45927 | 9662a29a1bb3f9a5b818181c1a514e6e258039f154524a630af9b5beb2a54083 |
| 46130058-ETHUSD-2026-11-17-110225.csv.gz | 16 | e614de6609d17ff693bab852d9fbe477d29a1074ebf3cdf3270dabe10504833a |
| 46130058-ETHUSD-2026-11-17-163119.csv.gz | 51045 | b2e291b6d24067cc9c66a5ea64f811e825011a9d51c3ffeb155756ce066f9304 |
| 46130058-ETHUSD-2026-11-17-203119.csv.gz | 5779 | 1508bda292f023cdd82d7f66ebf0c89e586ec1579983d1eeba242337033ef95c |
| 46130058-NAS100-2026-11-17-160300.csv.gz | 63127 | e173f1d91f4cf13156824b8fb416a47115340b33454a0d8780ee8563a8c93070 |
| 46130058-NAS100-2026-11-17-200300.csv.gz | 40397 | f4b5e743da89dfb2ed466840c4930d11eaa074e7133c77c59cc1adde46fb5dc1 |
| 47790949-BTCUSD-2026-11-17-111817.csv.gz | 58779 | 9cc03f939c032861e12934511b6d39e04eecb1ad440bdd86e2353779f0371a5d |
| 47790949-BTCUSD-2026-11-17-151817.csv.gz | 6002 | 73336c82b7b5f79f0a9c7f8d872540f73d28e53e3d365223b86ac69816e09827 |
| 47790949-ETHUSD-2026-11-17-070225.csv.gz | 45926 | 5858b0e573cc005496cdcd636285cbda6368815b8e3da4040df836070e63829e |
| 47790949-ETHUSD-2026-11-17-110225.csv.gz | 16 | 645b85f573e84a0be4b4364a6635b530aa5ffb68c281741daaa4c4696037301e |
| 47790949-ETHUSD-2026-11-17-163119.csv.gz | 51045 | 90fdad54c131cccdd56982dd128560dfbd920478c7a2f40943a5cea8dc7e369b |
| 47790949-ETHUSD-2026-11-17-203119.csv.gz | 5743 | 2c8c5c9d022540ff9fa43d5ed85b90fa5a56f959657264efda8546b7394c2e14 |
| 47790949-NAS100-2026-11-17-160301.csv.gz | 63130 | 2315e06f75c17a99ed8d6b431ff642c17dd2bc58afb9d60d36b4578ff4ac96c4 |
| 47790949-NAS100-2026-11-17-200301.csv.gz | 40389 | de1a9ce89fc575f353c1d2c4d9999f3917f1ebc53fac53af2a06502c7f1c18ce |
| 47790949-XAGUSD-2026-11-17-203618.csv.gz | 12202 | 44d34ad9393996d4856de108b1045401080072cc6568c01e37417864be93b166 |

| Frozen input | SHA256 |
| --- | --- |
| activity.json | 83360bfda5a6bef9c1102519c6d9820ffde8851d57eb6aa7f9e237bdac8d12b8 |
| anchor.json | 77c9b3329a1cbd58574bb6ab21f22992e4192b241138dc48103dec76489dee50 |
| capture.json | ac1f432781fc5f663a5a28b716dbb230bd0c7f0fa258c35efb28f752c19b4b3e |
| cumulative.json | 1c10b340cfb739baac1387fac46c6f70be7542ffcd24471dcac2107bdb7d20a9 |
| day.json | d232ad2f7fbd52968005d20b6083be2236083e56f6020522814161a26ccb7f7f |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| evidence.json | 999167bc12cc8d5e2ff2b2fa20e18ecba1245c81fc14426a05fc05da2e05b187 |
| health.json | 3a51e3c77924b61f8e9cc296ec1896b3fe5bfc6d093f02d02b683ded31ae2d5b |
| history-proof.json | 99994bb7fca24b2663a5436e84c8cdad2fad285d1c77d3453dd0e2e77c69de16 |
| human.json | 69a13a1fb803d89a1a9260b8486ab1345f86809bb534ccca098771bca7839741 |
| live-pnl.json | 4f7b39489742a210a1357fbae3a004a515c0da172ff55ca4a599a4265abffe88 |
| monitor-reconciled.json | d7abb7d8d83cf27dea98ae351e410fafbde68dad9a904a0360bd6ed045329d41 |
| monitor.json | d9f6ccf6274d893c341b410d44d8f3bfbb463cf09cfc793067be6bf1600be8f3 |
| opening-recovery.json | fbad570bf0e54ae06b01f9176a54ab2f89caa6c4ec4de6d47a7f9f59543b580b |
| opening-ticks.json | c89dc222b0333fd507a7f340ff7a456fd85cded95d7ff830cfdd6a86c9095194 |
| preflight.json | e9385b403cc67a0595c95edb0d1781c605f8c973c8afff1b93372d72a8d8f6a3 |
| quote-copy-receipts.json | aa6ddd941c5b4dc9c632a09d61d304c80078d9dded6ea6d09e060d832dfbb3e9 |
| rollover.json | 485566cf834f4c5c11549b765f644041215c6babe9deaa7523778c0dadac1c6a |
| runtime.json | 87655e65b009ac1d0c45a391afe0b5242c7f1bab84d88c5659657a388039237b |
| supplement.json | 8baa49d27501cc91dcfdd6086d36d8a7611fddc844a56acff9bb3feedee6ada9 |
| wednesday.json | f5e7aafb8b23203802366be56e9e8dbcc958c9e0cde60da392bd2074e1e7d828 |
