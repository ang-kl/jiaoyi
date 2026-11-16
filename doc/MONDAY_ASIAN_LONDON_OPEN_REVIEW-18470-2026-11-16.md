# Monday Asian & Early London Review — №18,470

**Activity:** [16 Nov 2026 07:15:00, 15:25:00) Asia/Singapore. **Read-only audit.** Production `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`; policy `s3_macro_pullback_owner_18409_v1`.

[Measured JSON and evidence](MONDAY_ASIAN_LONDON_OPEN_REVIEW-18470-2026-11-16.json) · [Standalone HTML](MONDAY_ASIAN_LONDON_OPEN_REVIEW-18470-2026-11-16.html)

## §1. Assessment

- **S3 window closing cash: +$47.99 from nine closes**, five positive-net and four negative-net. Eleven entries account for those nine closes plus **two still-open demo BTC shorts**. Full Monday-to-cutoff S3 cash is **−$12.26**; cumulative post-PR #214 closing cash is **+$2,745.99**. [1,2]
- **Live manual Gold cash is −$1,372.23, separate from S3.** The supplied **−$1,014.60** is exactly three losses; add an earlier **+$12.57** close and two later losses totalling **−$370.20**. Two recorded **$1,000 deposits** reconcile the balance from **−$906.87 to −$279.10**. These deposits are not trading profit. [1,5]
- **Live entry margin is constrained:** assessed current margin **242.97%**, pending-entry projection **136.98%**, both below 250%; stressed projection is **unverified**, not zero. Current crypto diagnostics also report stale/incomplete FX conversion evidence. There is **zero recorded live bot interference**. No resting Gold pending order remains in the observed book, but a new external **0.30-lot Gold long** still locks S3 Gold and its Silver pair. [5]
- **The two older Gold follow-ups are complete:** neither touched original TP in four hours; both crossed original SL after the actual close. Across all 11 post-close rows, **five clocks are complete and six remain partial** at the frozen **15:25:30 SGT** price horizon. No initial-stop or BE-stop path reached its original TP within the observed follow-up. [4]
- **Opening availability differs from entry permission:** crypto 08:00/12:00 has **16/16 source quotes, 12 exact operational references and four S1/pair-locked slots**. London has **24/24 source quotes, 23 exact operational references and one human-locked Gold slot**; the Silver reference exists but entry is pair-blocked. Brent's Monday 07:00 reference is verified on all four accounts. Crypto 16:00 preparation is **8/8 complete**. [6,7]
- **55 arms →17 consumed triggers →17 intents/outcomes →11 fills and six 24 bp peer refusals.** Missing individual outcomes and unknown entry-guard receipts are zero. Maximum confirmation-to-final-guard latency is **40.458 ms**. [6]
- **Financial pass=false:** latest20 net wins are **3/20, 11/20, 12/20, 11/20**, below the required 15/20. No production code, settings, entry/exit rules or Railway deployment changed.

## §2. Clock guard, continuity and cash periods

Initial `date -u` was **07:22:35 UTC /15:22:35 SGT**. The audit waited in bounded intervals, then verified **07:25:01.481 UTC /15:25:01.481 SGT** before collecting evidence. The activity boundary remains exactly **[07:15,15:25)**, with later snapshots timestamped separately.

This report continues directly from [№18,464](MONDAY_GOLD_REOPEN_AND_WEEKEND_CRYPTO_REVIEW-18464-2026-11-16.md), PR #235, merge `d78984ddeb6e47e9db0f77f4b104ead3f486f2fd`. **№18,466 was not executed.** No gap or duplicated closing deal is introduced. The two older Gold rows are price follow-ups only and add no cash to the current activity window. [1–3]


| Account | Window closes | Window S3 net USD | Monday 00:00→cutoff S3 USD | Post-PR214 closing cash USD | Open S3 at cutoff |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 0 | 0.00 | 0.00 | -0.01 | 0 |
| 46130058 | 4 | 21.83 | -22.59 | 1,368.12 | 1 |
| 47790949 | 4 | 20.02 | 4.19 | 1,352.92 | 1 |
| 49236051 | 1 | 6.14 | 6.14 | 24.96 | 0 |


**Period arithmetic:** previous cumulative **$2,698.00 +$47.99 =$2,745.99**. Monday before 07:15 contains **−$15.73** from two midnight crypto exits and **−$44.52** from Gold reopening; **−$60.25 +$47.99 =−$12.26**. The cumulative window **[10 Nov 09:42,16 Nov 15:25)** includes **+$412.88 from three positions entered before its start**. It is a mixed-entry closing-cash cohort, not a causal profit estimate for PR #214.

Broker closing-deal cash is reconciled independently with decimal arithmetic and unique `(account_id,deal_id)` keys. Successful retained history fetches through **15:26:14 SGT** cover beyond the cutoff for all four accounts; repeating the same frozen monitor query produced identical parsed results. `complete=false` on those successful fetches denotes the rolling, unsealed history window, not an assertion that it already includes all future activity. [1,2]

## §3. Account money, margin and complete inventory

Money marks are **15:26:16.042–15:26:16.499 SGT**. Books are approximately **15:25:14**, runtime/quote assessment is later, and health was captured at **15:25:53.877**. These are separate observations, not an atomic reconstruction of exact 15:25 account equity. Equity is recorded balance plus net unrealised P&L, excluding unverified credit and prospective closing costs. Margin percentages are the owner's assessment; aggregate margin remains unverified in the monetary snapshot. [5]


| Account | Balance USD | Floating USD | Equity USD | Assessed current margin % | Pending projected % | Pending stressed % | S3 / S1 / S2 / external | Pending entries |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | -279.10 | 3,041.39 | 2,762.29 | 242.97 | 136.98 | Unverified / N/A | 0 / 2 / 0 / 5 | 4 |
| 46130058 | 25,889.72 | 89.75 | 25,979.47 | 2,771.76 | Unverified / N/A | Unverified / N/A | 1 / 5 / 0 / 0 | 0 |
| 47790949 | 25,544.20 | 89.84 | 25,634.04 | 1,382.67 | Unverified / N/A | Unverified / N/A | 1 / 5 / 0 / 0 | 0 |
| 49236051 | 819.73 | -0.66 | 819.07 | 94,147.13 | Unverified / N/A | Unverified / N/A | 0 / 1 / 0 / 0 | 0 |


Live stressed pending margin is **unknown**: the endpoint requires complete broker marks, stops, conversion, margin projection and warning threshold evidence. The demos have no pending entries, so no current pending projection/stress number is emitted; those cells are not invented zeros. Historical candidate projections at the two still-open BTC entries are reported separately below and must not be mistaken for a current account snapshot.

Live current margin **242.97%** and pending projection **136.98%** independently fail the unchanged 250% entry floor. Current S3 crypto gate text also cites `conversion quote stale/incomplete/crossed for broker symbol 5`; the latest displayed gate is not a complete inventory of every concurrent constraint. These diagnostics do not authorise closing human inventory or lowering the floor.


| Account / ID | Symbol | Ownership / kind | Side | Native volume / lots | Entry or trigger | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / 649992251 | JPN225 | human_external / position | buy | 4000 / 0.4000 | 57,126.25000 | 59,000.00000 | Unverified / N/A |
| 43002148 / 649772867 | JPN225 | human_external / position | buy | 12000 / 1.2000 | 57,372.70000 | 58,500.00000 | Unverified / N/A |
| 43002148 / 637075128 | NatGas | human_external / position | buy | 150000 / 0.1500 | 2.52000 | 2.97900 | 3.87500 |
| 43002148 / 656513085 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,809.00000 | 25,813.70000 | Unverified / N/A |
| 43002148 / 656795267 | AUS200 | s1 / position | buy | 10 / 0.1000 | 9,165.30000 | 8,830.70000 | Unverified / N/A |
| 43002148 / 657180931 | NVDA.US | human_external / position | buy | 200 / 2.0000 | 165.22000 | 149.66000 | Unverified / N/A |
| 43002148 / 658574004 | XAUUSD | human_external / position | buy | 3000 / 0.3000 | 3,841.86000 | 3,830.00000 | Unverified / N/A |
| 43002148 / 400083120 | JPN225 | human_external / pending_entry | buy | 15000 / 1.5000 | 56,232.60000 | 55,679.50000 | Unverified / N/A |
| 43002148 / 401496419 | Cocoa | human_external / pending_entry | buy | 100 / 1.0000 | 4,973.20000 | 4,852.40000 | 5,904.00000 |
| 43002148 / 398403207 | NatGas | human_external / pending_entry | buy | 200000 / 0.2000 | 2.46700 | 2.38100 | 4.27500 |
| 43002148 / 400920027 | NatGas | human_external / pending_entry | buy | 100000 / 0.1000 | 2.58100 | 2.42300 | Unverified / N/A |
| 46130058 / 260823431 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,142.60000 | 8,830.80000 | Unverified / N/A |
| 46130058 / 261231902 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68521 | 0.69070 | Unverified / N/A |
| 46130058 / 261754023 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11005 | 1.12142 | Unverified / N/A |
| 46130058 / 261813392 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 46130058 / 262578251 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55271 | 0.56652 | Unverified / N/A |
| 46130058 / 263028399 | BTCUSD | s3 / position | sell | 25 / 0.2500 | 64,549.03000 | 65,152.09000 | 63,716.80000 |
| 47790949 / 260743517 | AUS200 | s1 / position | buy | 60 / 0.6000 | 9,126.30000 | 8,830.80000 | Unverified / N/A |
| 47790949 / 261231903 | AUDUSD | s1 / position | sell | 900000 / 0.0900 | 0.68526 | 0.69065 | Unverified / N/A |
| 47790949 / 261754015 | EURUSD | s1 / position | sell | 700000 / 0.0700 | 1.11010 | 1.12137 | Unverified / N/A |
| 47790949 / 261813391 | GER40 | s1 / position | buy | 10 / 0.1000 | 26,714.90000 | 25,806.50000 | Unverified / N/A |
| 47790949 / 262976216 | NZDUSD | s1 / position | sell | 1000000 / 0.1000 | 0.55202 | 0.56647 | Unverified / N/A |
| 47790949 / 263028398 | BTCUSD | s3 / position | sell | 25 / 0.2500 | 64,549.03000 | 65,152.09000 | 63,716.80000 |
| 49236051 / 262796763 | ETHUSD | s1 / position | sell | 1 / 0.0100 | 1,746.87000 | 2,027.27000 | Unverified / N/A |


## §4. Live manual Gold reconciliation and human protection

The complete retained live journal contains **no bot intent in this activity window**. The Gold closes are outside registered S3 ownership, and live S1 has no entry/close/trail fill in this window. The observed external positions and pending orders remain outside S3 authority. **Recorded interference=0** means no recorded bot command against human inventory; it does not claim knowledge of unrecorded outside actors. [1,5]


| External Gold position / closing deal | Closed SGT | Close price | Gross USD | Swap USD | Commission USD | Net USD |
| --- | --- | --- | --- | --- | --- | --- |
| 657187868 / 348204003 | 16 Nov 09:20:49.501 | 3,871.17 | 53.30 | -38.97 | -1.76 | 12.57 |
| 658504057 / 348212926 | 16 Nov 10:00:29.461 | 3,865.15 | -349.80 | 0.00 | -2.10 | -351.90 |
| 658513094 / 348225914 | 16 Nov 11:00:42.382 | 3,873.67 | -91.50 | 0.00 | -2.10 | -93.60 |
| 658512603 / 348250007 | 16 Nov 13:46:26.004 | 3,851.76 | -567.00 | 0.00 | -2.10 | -569.10 |
| 658568964 / 348256138 | 16 Nov 13:57:33.538 | 3,833.51 | -121.50 | 0.00 | -2.10 | -123.60 |
| 658571100 / 348258002 | 16 Nov 14:02:13.352 | 3,841.69 | -244.50 | 0.00 | -2.10 | -246.60 |


The three positions **658504057,658513094,658512603** account for the owner's **−$1,014.60**: **−$351.90−$93.60−$569.10**. The earlier long **657187868** closed for **+$12.57**, while later positions **658568964 /658571100** lost **−$123.60 /−$246.60**. Thus the complete cutoff total is **−$1,372.23**, not −$1,014.60. None is counted as S3 cash.


| Broker balance-history ID | Executed SGT | Operation | USD amount | Balance after USD |
| --- | --- | --- | --- | --- |
| 133942097 | 16 Nov 11:09:29.320 | BALANCE_DEPOSIT | 1,000.00 | -339.80 |
| 133959695 | 16 Nov 14:06:21.006 | BALANCE_DEPOSIT | 1,000.00 | -279.10 |


Both deposit receipts are `BALANCE_DEPOSIT`, with two decimal money digits. Their signed 64-bit balances decode to **−$339.80** and **−$279.10**, respectively. Reconciliation is exact: **−$906.87−$1,372.23+$2,000.00=−$279.10**. No profit is inferred from these balance increases.

The current external Gold position is **658574004**, long **0.30 lot at3,841.86**, opened **14:02:13.769 SGT**, SL **3,830.00**, no recorded TP. **No resting XAUUSD pending entry remains** in the 15:25 book. This establishes the current absence of the prior limit order, without inventing its cancellation cause. The open manual Gold position still triggers S3's ownership yield; Silver remains pair-blocked. Four other external positions—JPN225×2, NatGas and NVDA.US—and four external pending entries remain separately listed above.

## §5. Strategy 1 remains separate

Live S1 **GER40 656513085** and **AUS200 656795267** remain open. Their SLs are **25,813.7 /8,830.7**, unchanged from the preceding report; no live S1 entry, close or trailing amendment is recorded in this activity interval.

Demo 49236051's S1 **ETHUSD sell 262796763** remains open: entry **15 Nov 06:37:46.205 SGT at1,746.87**, native volume 1 (0.01 unit), latest observed SL **2,027.27**. No new S1 trail or close is recorded for it in this activity window; retained last trail is **15 Nov 09:40:46.833 SGT**. Its later account float is **−$0.66**, with only that position observed. S3 ETH yields to S1 ownership and BTC is pair-blocked; the generic yield diagnostic does not make it a human trade.

Demo 47790949 has a new S1 NZDUSD position **262976216**, listed separately from S3. The window contains **75 successful S1 trail amendments on 46130058**, **68 on 47790949**, and **one S1 filled entry on 47790949**. These events are not part of S3's 11-entry funnel or realised cash. [1,5]

## §6. Complete S3 ledger, risk geometry and open positions

All 11 current-window S3 entries are demo shorts. Nine closed and two remained open. Initial R is actual opening fill to original SL, scaled by actual volume and quote-to-deposit conversion. Later BE amendments do not reset R. TP classification uses protective closing receipts and target price; session-end classification uses a matching successful close intent. [1]


| Account / position | Symbol / side | Native volume / lots | Entry SGT / price | Exit SGT / price | Initial SL / TP | Exit classification | Net USD / net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 262978198 | BTCUSD / sell | 31 / 0.3100 | 16 Nov 08:51:02.243 / 64,902.40000 | 16 Nov 12:00:01.352 / 65,061.04000 | 65,397.06000 / 64,219.76000 | session end | -49.18 / -0.320716 |
| 46130058 / 262978205 | BTCUSD / sell | 31 / 0.3100 | 16 Nov 08:51:07.289 / 64,901.72000 | 16 Nov 12:00:01.403 / 65,059.53000 | 65,396.38000 / 64,219.08000 | session end | -48.92 / -0.319020 |
| 49236051 / 262986786 | SpotCrude / sell | 900 / 0.0900 | 16 Nov 09:46:35.497 / 88.46300 | 16 Nov 11:04:09.727 / 87.78100 | 88.95200 / 87.78800 | protective TP | 6.14 / 1.395137 |
| 46130058 / 262986789 | SpotCrude / sell | 32300 / 3.2300 | 16 Nov 09:46:37.410 / 88.46300 | 16 Nov 11:04:09.736 / 87.78100 | 88.94600 / 87.79600 | protective TP | 220.29 / 1.412034 |
| 47790949 / 262986807 | SpotCrude / sell | 31400 / 3.1400 | 16 Nov 09:47:00.351 / 88.46000 | 16 Nov 11:04:09.705 / 87.78100 | 88.94900 / 87.78500 | protective TP | 213.21 / 1.388574 |
| 46130058 / 263005199 | ETHUSD / sell | 1314 / 13.1400 | 16 Nov 12:14:34.625 / 1,817.84000 | 16 Nov 12:24:26.883 / 1,830.56000 | 1,829.78000 / 1,801.36000 | initial stop | -167.14 / -1.065322 |
| 47790949 / 263005201 | ETHUSD / sell | 1296 / 12.9600 | 16 Nov 12:14:40.112 / 1,818.05000 | 16 Nov 12:24:26.886 / 1,830.56000 | 1,829.99000 / 1,801.57000 | initial stop | -162.13 / -1.047741 |
| 47790949 / 263023705 | XAUUSD / sell | 400 / 0.0400 | 16 Nov 14:17:51.759 / 3,855.49000 | 16 Nov 15:00:01.194 / 3,850.89000 | 3,887.49000 / 3,811.33000 | session end | 18.12 / 0.141563 |
| 46130058 / 263024012 | XAUUSD / sell | 400 / 0.0400 | 16 Nov 14:18:46.648 / 3,855.32000 | 16 Nov 15:00:01.174 / 3,850.92000 | 3,887.33000 / 3,811.14000 | session end | 17.60 / 0.137457 |
| 46130058 / 263028399 | BTCUSD / sell | 25 / 0.2500 | 16 Nov 14:32:50.084 / 64,549.03000 | Open / unavailable / Unverified / N/A | 65,152.09000 / 63,716.80000 | OPEN at cutoff | Unverified / N/A / Unverified / N/A |
| 47790949 / 263028398 | BTCUSD / sell | 25 / 0.2500 | 16 Nov 14:32:50.047 / 64,549.03000 | Open / unavailable / Unverified / N/A | 65,152.09000 / 63,716.80000 | OPEN at cutoff | Unverified / N/A / Unverified / N/A |

| Position | Observed MFE / MAE R | Confirmed +1R BE amendment SGT | Registered deadline SGT |
| --- | --- | --- | --- |
| 262978198 | 0.843266 / 0.898395 | None observed | 16 Nov 12:00:00.000 |
| 262978205 | 0.841891 / 0.899770 | None observed | 16 Nov 12:00:00.000 |
| 262986786 | 1.394683 / 0.229039 | 16 Nov 10:52:39.474 | 16 Nov 15:00:00.000 |
| 262986789 | 1.412008 / 0.231884 | 16 Nov 10:52:39.473 | 16 Nov 15:00:00.000 |
| 262986807 | 1.388548 / 0.235174 | 16 Nov 10:52:39.616 | 16 Nov 15:00:00.000 |
| 263005199 | 0.000000 / 1.054439 | None observed | 16 Nov 16:00:00.000 |
| 263005201 | 0.000000 / 1.036851 | None observed | 16 Nov 16:00:00.000 |
| 263023705 | 0.531875 / 0.300625 | None observed | 16 Nov 15:00:00.000 |
| 263024012 | 0.525461 / 0.306779 | None observed | 16 Nov 15:00:00.000 |
| 263028399 | 0.348373 / 0.411767 | None observed | 16 Nov 16:00:00.000 |
| 263028398 | 0.348373 / 0.411767 | None observed | 16 Nov 16:00:00.000 |


The three Crude TP wins contribute **+$439.64** from one correlated market episode across three accounts. The two ETH initial stops contribute **−$329.27**; two BTC session ends contribute **−$98.10**; two Gold session ends contribute **+$35.72**. This reconciles to **+$47.99**, with five positive-net closes out of nine. Counting the copied Crude fills as three independent tests would overstate the evidence.

The earlier BTC shorts reached observed **+0.8433R /+0.8419R** before their 12:00 session ends, below the deployed 1R BE trigger. The ETH shorts had **zero positive observed MFE** before stopping. The two currently open BTC shorts reached **+0.3484R MFE /0.4118R MAE** by the cutoff and had no confirmed BE amendment. These observations do not change any exit rule.


| Open BTC position / account | Cutoff ask / quote SGT | Price-only gross USD / R | Historical entry projected / stressed margin % | Registered deadline |
| --- | --- | --- | --- | --- |
| 263028399 / 46130058 | 64,530.30 / 16 Nov 15:24:59.860 | 4.6825 / 0.031058 | 2,562.69 / 2,471.64 | 16 Nov 16:00:00.000 |
| 263028398 / 47790949 | 64,530.30 / 16 Nov 15:24:59.860 | 4.6825 / 0.031058 | 1,038.36 / 1,000.95 | 16 Nov 16:00:00.000 |


The two cutoff gross marks are approximately **+$4.6825 each** using fresh recorded ask **64,530.30** just before 15:25. They exclude fees/swap and are not broker-certified S3 net unrealised P&L. Both registered exits are **16:00 SGT**. No later outcome is substituted into this frozen cutoff ledger.

## §7. Latest20 and financial qualification


| Account | Net wins / count | Win rate | Realised payoff (R) | Profit factor (R) | Net R | Assessment recorded SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.00% | 3.4277 | 0.6049 | -5.9677 | 16 Nov 15:24:12.812 |
| 46130058 | 11/20 | 55.00% | 1.9021 | 2.3248 | 6.7337 | 16 Nov 15:22:18.608 |
| 47790949 | 12/20 | 60.00% | 1.5898 | 2.3848 | 6.8488 | 16 Nov 15:22:24.495 |
| 49236051 | 11/20 | 55.00% | 1.3583 | 1.8677 | 4.9736 | 16 Nov 15:19:27.645 |


The 73% target requires **15 positive-net closes out of 20 (75%)**, alongside the payoff requirement. No account meets that win count. Live includes older October closes; the two large demos have correlated setups. The configured 1.38R target is not a guaranteed net payoff, and positive cumulative cash is not financial qualification. **Financial pass remains false.**

## §8. Mandatory four-hour post-close excursions

There are **11 post-close paths**: nine current-window closes plus the two earlier Gold reopen closes. **Five clocks are complete**—three Crude TP exits and two older Gold exits—while **six current paths are partial**. The price horizon is frozen at **15:25:30 SGT**, regardless of later publication time. [4]

All are shorts, so the executable comparison side is **ask**. Movement is signed in the original direction from the **actual exit fill**, using original risk R. Favourable/adverse magnitudes are shown as **price / broker pips / R**. Broker pip size is **0.1 for BTC/ETH/Gold**, **0.01 for Crude**. Original TP/SL crossing tests retain absolute original levels; a movement of2R from a slipped stop fill need not reach the original TP.



### Current-window closes


| Position / symbol / exit | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 262978198 / BTCUSD / session end | 3.42462 PARTIAL | 836.27000 / 8,362.70 / 1.690596 | 97.50000 / 975.00 / 0.197105 | 2,796.90 / 0.565419 | 6,807.50 / 1.376198 | not_elapsed_at_capture | False / False |
| 262978205 / BTCUSD / session end | 3.42461 PARTIAL | 834.76000 / 8,347.60 / 1.687543 | 99.01000 / 990.10 / 0.200158 | 2,781.80 / 0.562366 | 6,792.40 / 1.373145 | not_elapsed_at_capture | False / False |
| 262986786 / SpotCrude / protective TP | 4.00000 complete clock | 0.03000 / 3.00 / 0.061350 | 1.86400 / 186.40 / 3.811861 | stale_asof_quote (7.278s old) | stale_asof_quote (10.373s old) | -150.40 / -3.075665 | True / True |
| 262986789 / SpotCrude / protective TP | 4.00000 complete clock | 0.03000 / 3.00 / 0.062112 | 1.86400 / 186.40 / 3.859213 | stale_asof_quote (7.298s old) | stale_asof_quote (10.422s old) | -150.40 / -3.113872 | True / True |
| 262986807 / SpotCrude / protective TP | 4.00000 complete clock | 0.03000 / 3.00 / 0.061350 | 1.86400 / 186.40 / 3.811861 | stale_asof_quote (7.229s old) | stale_asof_quote (10.351s old) | -150.40 / -3.075665 | True / True |
| 263005199 / ETHUSD / initial stop | 3.01753 PARTIAL | 27.34000 / 273.40 / 2.289782 | 0.00000 / 0.00 / 0.000000 | 80.60 / 0.675042 | 214.30 / 1.794807 | not_elapsed_at_capture | False / True |
| 263005201 / ETHUSD / initial stop | 3.01753 PARTIAL | 27.34000 / 273.40 / 2.289782 | 0.00000 / 0.00 / 0.000000 | 80.60 / 0.675042 | 214.30 / 1.794807 | not_elapsed_at_capture | False / True |
| 263023705 / XAUUSD / session end | 0.42467 PARTIAL | 3.21000 / 32.10 / 0.100313 | 5.81000 / 58.10 / 0.181562 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | False / False |
| 263024012 / XAUUSD / session end | 0.42467 PARTIAL | 3.21000 / 32.10 / 0.100281 | 5.81000 / 58.10 / 0.181506 | not_elapsed_at_capture | not_elapsed_at_capture | not_elapsed_at_capture | False / False |


### Completed older Gold follow-ups — earlier closing cash excluded


| Position / symbol / exit | Elapsed hours | Favourable price / pips / R | Adverse price / pips / R | 1h signed pips / R | 2h signed pips / R | 4h signed pips / R | Original TP / SL crossed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 262583136 / XAUUSD / session end | 4.00000 complete clock | 19.88000 / 198.80 / 1.285899 | 21.32000 / 213.20 / 1.379043 | -179.50 / -1.161061 | 56.80 / 0.367400 | 61.70 / 0.399094 | False / True |
| 262583980 / XAUUSD / session end | 4.00000 complete clock | 19.88000 / 198.80 / 1.282581 | 21.32000 / 213.20 / 1.375484 | -179.50 / -1.158065 | 56.80 / 0.366452 | 61.70 / 0.398065 | False / True |


The two old Gold shorts moved favourably by **19.88 price units /198.8 pips**, but also adversely by **21.32 /213.2 pips** from their actual reopen exits. Both crossed original SL; neither reached original TP within its complete four-hour window ending approximately 11:01. Their earlier **−$44.52** close cash stays in №18,464 and Monday's full-day total, not in this report's new activity-window cash.

The three Crude winners briefly extended by **0.03 price /3 pips**, then reversed enough to cross original SL within four hours. The ETH stop exits subsequently moved favourably by **27.34 price /273.4 pips /2.2898R** from the stop fill, but the lowest observed ask **1,803.22** still remained above original TP **1,801.36 /1,801.57**. The earlier BTC session-end paths likewise approached but had not touched original TP at capture. There is no claim that holding would produce a profitable, executable alternative trade.

**Coverage limitations:** complete clock is not continuous quote coverage. All three Crude 1h checkpoints have quotes **7.229–7.298 seconds** old; their 2h checkpoints are **10.351–10.422 seconds** old. Those six values remain null under the unchanged six-second freshness rule. The largest Crude inter-update gap is about **63.498 seconds**. Ten checkpoints have not elapsed and also remain null. No interpolation fills gaps; observed extrema cannot rule out a larger unrecorded excursion.

The remaining six four-hour horizons mature around **16:00 (BTC×2),16:24 (ETH×2),19:00 (Gold×2)**. Follow them in a later report without changing this cash interval or overwriting the frozen 15:25:30 horizon. The JSON retains exact extrema/crossing timestamps, endpoint ages, first/last quote times and maximum gaps.

## §9. Veto funnel and outcome integrity


| Account | New arms | Consumed / intents / outcomes | Filled entries | 24 bp peer refusals | Deep invalidations |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 21 | 0 / 0 / 0 | 0 | 0 | 19 |
| 46130058 | 13 | 7 / 7 / 7 | 5 | 2 | 5 |
| 47790949 | 13 | 7 / 7 / 7 | 5 | 2 | 5 |
| 49236051 | 8 | 3 / 3 / 3 | 1 | 2 | 5 |


The complete consumed-trigger join has **17 decisions,17 persisted opening intents,17 individual terminal outcomes**. Eleven filled; **six refused because the peer no longer held 24 bp in the selected direction**. Missing individual outcomes and unknown entry-guard receipts are both **zero**. There are **34 >61.8% invalidations** and **140 pair-reservation diagnostic records**. Deduplicated idle diagnostics are not independent missed trades or an exhaustive count of every pre-arm evaluation. [6]

Maximum observed **confirmation-to-final-guard latency is 40.458 ms**. That is distinct from touch-to-turn waiting and broker send-to-fill latency. The canonical monitor retains `refusals_all_stages_complete=false`; complete consumed-trigger evidence does not claim a complete census of every earlier idle gate.

Current post-cutoff core-symbol states:


| Account | Symbol | Current gate / reason |
| --- | --- | --- |
| 43002148 | BTCUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 5 |
| 43002148 | ETHUSD | Blocked: prepared admission unavailable; last refusal: margin buffer: conversion quote stale/incomplete/crossed for broker symbol 5 |
| 43002148 | NAS100 | Waiting: opening impulse below 48 bps |
| 43002148 | SpotBrent | Waiting: opening impulse below 48 bps |
| 43002148 | SpotCrude | Waiting: opening impulse below 48 bps |
| 43002148 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.005%, 20d: -1.236%) |
| 43002148 | XAGUSD | Blocked: pair reservation held by sibling symbol XAUUSD |
| 43002148 | XAUUSD | Blocked: yield_to_human_position_or_order |
| 46130058 | BTCUSD | Managing existing position or pending entry; no additional entry |
| 46130058 | ETHUSD | Blocked: pair reservation held by sibling symbol BTCUSD |
| 46130058 | NAS100 | Waiting: opening impulse below 48 bps |
| 46130058 | SpotBrent | Waiting: opening impulse below 48 bps |
| 46130058 | SpotCrude | Waiting: opening impulse below 48 bps |
| 46130058 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.005%, 20d: -1.236%) |
| 46130058 | XAGUSD | Waiting: opening impulse below 48 bps |
| 46130058 | XAUUSD | Waiting: opening impulse below 48 bps |
| 47790949 | BTCUSD | Managing existing position or pending entry; no additional entry |
| 47790949 | ETHUSD | Blocked: pair reservation held by sibling symbol BTCUSD |
| 47790949 | NAS100 | Waiting: opening impulse below 48 bps |
| 47790949 | SpotBrent | Waiting: opening impulse below 48 bps |
| 47790949 | SpotCrude | Waiting: opening impulse below 48 bps |
| 47790949 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.005%, 20d: -1.236%) |
| 47790949 | XAGUSD | Waiting: opening impulse below 48 bps |
| 47790949 | XAUUSD | Waiting: opening impulse below 48 bps |
| 49236051 | BTCUSD | Blocked: pair reservation held by sibling symbol ETHUSD |
| 49236051 | ETHUSD | Blocked: yield_to_human_position_or_order |
| 49236051 | NAS100 | Waiting: opening impulse below 48 bps |
| 49236051 | SpotBrent | Waiting: opening impulse below 48 bps |
| 49236051 | SpotCrude | Waiting: opening impulse below 48 bps |
| 49236051 | US500 | Vetoed: 5d/20d macro drift opposes impulse (5d: 3.005%, 20d: -1.236%) |
| 49236051 | XAGUSD | Waiting: opening impulse below 48 bps |
| 49236051 | XAUUSD | Waiting: opening impulse below 48 bps |


## §10. Opening references, Monday Brent and 16:00 readiness

All **44 requested source slots** have two-sided opening quotes inside 15 seconds: crypto 08:00/12:00×8=16, London traditional×24, and Monday Brent 07:00×4. Exact full-quote operational evidence exists for **39**; five are ownership-locked. Evidence matches bid/ask prices and independent side/observation timestamps, using session notes, immutable arm source context, order receipts, current health or the prior frozen Brent proof. It does not treat an early unavailable session note as proof that later recovery failed. [7]


| Account | Symbol | Window SGT | First two-sided quote SGT | Delay seconds | Verification | Ownership constraint |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.052 | 0.052703 | exact_quote_receipt | None recorded for this slot |
| 43002148 | BTCUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.099 | 0.099020 | exact_quote_receipt | None recorded for this slot |
| 43002148 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.393 | 0.393800 | exact_quote_receipt | None recorded for this slot |
| 43002148 | ETHUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.558 | 0.558045 | exact_quote_receipt | None recorded for this slot |
| 49236051 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.094 | 0.094728 | ownership_locked | external_s1_crypto_pair |
| 49236051 | BTCUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.108 | 0.108147 | ownership_locked | external_s1_crypto_pair |
| 49236051 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.541 | 0.541683 | ownership_locked | external_s1_crypto_pair |
| 49236051 | ETHUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.559 | 0.559929 | ownership_locked | external_s1_crypto_pair |
| 47790949 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.100 | 0.100846 | exact_quote_receipt | None recorded for this slot |
| 47790949 | BTCUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.062 | 0.062147 | exact_quote_receipt | None recorded for this slot |
| 46130058 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.538 | 0.538421 | exact_quote_receipt | None recorded for this slot |
| 46130058 | ETHUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.555 | 0.555757 | exact_quote_receipt | None recorded for this slot |
| 47790949 | ETHUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.607 | 0.607315 | exact_quote_receipt | None recorded for this slot |
| 47790949 | ETHUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.544 | 0.544365 | exact_quote_receipt | None recorded for this slot |
| 46130058 | BTCUSD | 16 Nov 08:00:00.000 | 16 Nov 08:00:00.111 | 0.111450 | exact_quote_receipt | None recorded for this slot |
| 46130058 | BTCUSD | 16 Nov 12:00:00.000 | 16 Nov 12:00:00.094 | 0.094452 | exact_quote_receipt | None recorded for this slot |
| 43002148 | NAS100 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.184 | 0.184084 | exact_quote_receipt | None recorded for this slot |
| 49236051 | SpotBrent | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.113 | 0.113527 | exact_quote_receipt | None recorded for this slot |
| 43002148 | XAUUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.184 | 0.184070 | ownership_locked | live_human_gold_or_pair |
| 43002148 | XAGUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.352 | 0.352551 | exact_quote_receipt | live_human_gold_or_pair |
| 43002148 | US500 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.017 | 0.017199 | exact_quote_receipt | None recorded for this slot |
| 49236051 | NAS100 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.182 | 0.182080 | exact_quote_receipt | None recorded for this slot |
| 49236051 | US500 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.016 | 0.016127 | exact_quote_receipt | None recorded for this slot |
| 49236051 | XAGUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.392 | 0.392320 | exact_quote_receipt | None recorded for this slot |
| 49236051 | SpotCrude | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.279 | 0.279591 | exact_quote_receipt | None recorded for this slot |
| 43002148 | SpotCrude | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.283 | 0.283699 | exact_quote_receipt | None recorded for this slot |
| 43002148 | SpotBrent | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.055 | 0.055805 | exact_quote_receipt | None recorded for this slot |
| 46130058 | NAS100 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.182 | 0.182043 | exact_quote_receipt | None recorded for this slot |
| 49236051 | XAUUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.182 | 0.182070 | exact_quote_receipt | None recorded for this slot |
| 46130058 | US500 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.016 | 0.016187 | exact_quote_receipt | None recorded for this slot |
| 47790949 | XAUUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.179 | 0.179734 | exact_quote_receipt | None recorded for this slot |
| 46130058 | XAUUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.182 | 0.182036 | exact_quote_receipt | None recorded for this slot |
| 46130058 | SpotBrent | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.055 | 0.055067 | exact_quote_receipt | None recorded for this slot |
| 47790949 | XAGUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.384 | 0.384995 | exact_quote_receipt | None recorded for this slot |
| 47790949 | US500 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.014 | 0.014744 | exact_quote_receipt | None recorded for this slot |
| 47790949 | SpotCrude | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.273 | 0.273857 | exact_quote_receipt | None recorded for this slot |
| 46130058 | SpotCrude | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.278 | 0.278191 | exact_quote_receipt | None recorded for this slot |
| 47790949 | NAS100 | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.179 | 0.179743 | exact_quote_receipt | None recorded for this slot |
| 47790949 | SpotBrent | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.108 | 0.108909 | exact_quote_receipt | None recorded for this slot |
| 46130058 | XAGUSD | 16 Nov 15:00:00.000 | 16 Nov 15:00:00.386 | 0.386704 | exact_quote_receipt | None recorded for this slot |
| 49236051 | SpotBrent | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693215 | exact_quote_receipt | None recorded for this slot |
| 43002148 | SpotBrent | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.662 | 6.662241 | exact_quote_receipt | None recorded for this slot |
| 46130058 | SpotBrent | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.693 | 6.693905 | exact_quote_receipt | None recorded for this slot |
| 47790949 | SpotBrent | 16 Nov 07:00:00.000 | 16 Nov 07:00:06.703 | 6.703147 | exact_quote_receipt | None recorded for this slot |


**Crypto 08:00/12:00:** source quotes16/16; operational full references12/16. The four small-demo BTC/ETH slots are locked by the continuously open S1 ETH position and pair reservation. This is intentional ownership separation, not evidence of four missing market quotes.

**London 15:00:** source quotes24/24; exact operational references23/24. Live Gold is human-locked with no operational reference. Live Silver **has** a verified reference but is pair-blocked, so reference availability must not be reported as permission to trade. London base preparation is **24/24**; full-session tables remain **0/24 complete** because 420/450-minute marks have **10 same-slot endpoints versus14 required**. Those late 22:00/22:30 marks are a preparation limitation, not a failure of the 15:00 price capture.

**Monday Brent:** the native session began at **Sunday 18:00 America/New_York =Monday 07:00 SGT**, not 09:00. All four 07:00 references are supported by the preceding frozen audit and rechecked source ticks.09:00 was the120-minute mark, not a second opening. The old Asian450/480-minute marks retain 6 endpoints versus14; the current London late marks are a separate10/14 issue.

**Crypto 16:00:** all **8/8** base preparations and full mark tables are ready. The future opening price is not known at the cutoff, and ownership, quote/FX freshness, risk, margin and pair guards still apply.


| Account | Crypto symbol | Target open SGT | Preparation recorded SGT | Base / full marks ready |
| --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:20.234 | True / True |
| 43002148 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:23.181 | True / True |
| 46130058 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:20.734 | True / True |
| 46130058 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:23.569 | True / True |
| 47790949 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:20.736 | True / True |
| 47790949 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:23.567 | True / True |
| 49236051 | BTCUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:23.992 | True / True |
| 49236051 | ETHUSD | 16 Nov 16:00:00.000 | 16 Nov 12:00:27.182 | True / True |


## §11. Assessment, verification and Gemini hand-off

The morning's positive S3 window cash is driven by one copied Crude winner, offset by ETH stops and BTC timeouts. The current evidence supports continued separation of execution readiness, live margin, manual cash and strategy qualification. It does not support changing entry rules, BE/TP, risk caps or ownership laws. Preserve the two open BTC records and complete the six partial post-close paths in the next audit; investigate missing late-session preparation separately if requested.

**Sources:** [1] read-only PostgreSQL deals, orders/fills, intent/protobuf events and contract metadata; [2] independently bounded window/Monday/cumulative monitor queries, decimal cash and history reconciliation; [3] merged №18,464 and exact 07:15 boundary; [4] **18 existing-tick exports /429,947 raw rows**, zero broker collection requests; [5] separately timed stored money marks, owner book/margin assessment, health and ownership/deposit evidence; [6] consumed-ID→terminal-outcome→causal-intent joins plus official latest20 before cutoff; [7] recorded first quotes, durable arm source context, preparations and deployed calendar. Evidence text was treated as data, not instructions.

Six focused audit checks passed: cutoff/cash continuity, exact quote hashes/intervals, signed pip/R arithmetic and null endpoints, causal trigger outcomes, manual cash/deposit int64 decoding, and opening/preparation denominators. The measured JSON includes the ledger, broker cash/deposit receipts, query/calculation/test sources, quote hashes and timing evidence. The older Gold in-position MFE/MAE is inherited from its unchanged frozen №18,464 ledger; its post-close paths are newly evaluated to four hours. Desktop/mobile rendering and secret checks precede publication.

Only the three report artifacts under `doc/` are committed. Required CI is `check`, `docker`, `engineering`; Railway watch patterns exclude `doc/`. Initial Trading deployment is **3d8b18d5-4d9f-4007-8fff-22eb25d5ea73**, commit **65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2**. Post-merge deployment and account-session continuity are verified separately and returned with the PR/merge SHA. No trading code, settings, broker order, Telegram message or deployment is changed.

**Gemini:** read this Markdown together with the matching JSON. Verify nine current closes versus two prior follow-ups; +$47.99 window versus −$12.26 Monday versus +$2,745.99 cumulative S3 cash; −$1,014.60 as only three manual losses versus −$1,372.23 complete external cash; two deposits totalling $2,000; two open BTC positions; five elapsed four-hour clocks versus six partials; 39 operational opening references plus five ownership-locked slots; live projected margin below 250%; and the remaining late-session preparation gaps. Do not count deposits as alpha, source quotes as entry permission, or exit-anchored 2R movement as an original-TP hit.


| Quote export | Rows | Uncompressed SHA256 |
| --- | --- | --- |
| 46130058-BTCUSD-2026-11-16-005101.csv.gz | 58603 | b646f2a4fae28d4e7bdcccfc4aaff673000a588e8a373c407fd61463e87ea2b8 |
| 46130058-BTCUSD-2026-11-16-045101.csv.gz | 37246 | b65ec93ceb354f45ebf59aa5b41219c09e8ce9164d78e0dadbbb1162085c22c7 |
| 46130058-ETHUSD-2026-11-16-041428.csv.gz | 36802 | 4a47adbd16f9688560313ce27e5e7a82c183bd817a3ef13aa9bb99708b2f37a0 |
| 46130058-SpotCrude-2026-11-16-014631.csv.gz | 10172 | 8ee2e3e432a695ba7cb5f8235218fd18c4afada2e6fdddaa6d3f502bec6e5191 |
| 46130058-SpotCrude-2026-11-16-054631.csv.gz | 4934 | 91e1aee643995b1655f969d802640633f74fda46bb3008d6ac8bd11859c95a6c |
| 46130058-XAUUSD-2026-11-15-230055.csv.gz | 47217 | 695b9286866fbd1cf9d4f1b99c82a41c84d0f70b52c285d844354ef5b994a402 |
| 46130058-XAUUSD-2026-11-16-030055.csv.gz | 14 | cdb98866fa6d33155ca9a8beb5fb85e82f4bb580fc0d97b871713b7bd8c26c1c |
| 46130058-XAUUSD-2026-11-16-061840.csv.gz | 12340 | 3227514540d0c5db4891607b25df4dad72ece4a44660bf153e1e906e113e2d61 |
| 47790949-BTCUSD-2026-11-16-005056.csv.gz | 58604 | f1772653fb2721da2dc60e7153ae616f39f384552e26d14ea6d67a0783559466 |
| 47790949-BTCUSD-2026-11-16-045056.csv.gz | 37266 | 21fde8e4870cc44004920ff73a38c1a6887c14c8f0188aeec70118fcf6e29a2e |
| 47790949-ETHUSD-2026-11-16-041434.csv.gz | 36786 | e5f1c553a389b4e22c304cb5e9005ac1ae668cd9ee261911f333ca12b8739c68 |
| 47790949-SpotCrude-2026-11-16-014654.csv.gz | 10166 | b667a5309282bfaf48e9693e18baf1b8690055434cb788bc014aac21fd9856ea |
| 47790949-SpotCrude-2026-11-16-054654.csv.gz | 4915 | f2d5d379d75dcb23dbef3dc81e17648bad5bd0655f10294789ef609afefa885f |
| 47790949-XAUUSD-2026-11-15-230055.csv.gz | 47217 | 0b2d8694c8b8a30d8cca4d788ff3f83b2d985f5c3171f496c58c7ddbe7b23831 |
| 47790949-XAUUSD-2026-11-16-030055.csv.gz | 14 | 6f7e900d2419c7a3301706c9de358a84dd9550c31117906484678d12e9c717f4 |
| 47790949-XAUUSD-2026-11-16-061745.csv.gz | 12545 | b3e7fe729cd08c6c9f956bc95c5daaf3766e401de69667903802c43e3913c13a |
| 49236051-SpotCrude-2026-11-16-014629.csv.gz | 10172 | dfcc5413635ddde7cc8d7aa42e569a97bfd430e8d00efac6bf41435758413c3c |
| 49236051-SpotCrude-2026-11-16-054629.csv.gz | 4934 | 91e3f3c4858f2ec4c7189cbae11647936627226803fdbe685a52d7a95a4779ad |

| Source | SHA256 |
| --- | --- |
| opening-ticks.json | 8f568302af00803428e01b894e0e126cef4e165d826036fb9d0ac1af565401d3 |
| preflight.json | 8047520f72fc423e22acdb8b0aee4c397ff47496be197f4547b9c01b3105dc27 |
| monitor.json | 95caf7b7626e0ef88d419e21ceefb931788e4e9a058f473b0217b4a222c17b6d |
| cumulative.json | 04f8f906e688089b4c8a6308946cdc9a743830c40483a4e08ce50deb8e97c893 |
| human.json | fdc352c40f7dcbdc271a1880750bc97a20d05e26d6cf56fd77ac44fcad4d8857 |
| health.json | b284d3db0079aee88361a9ed24537e2ba2cb104fb6b14afc4d8d850dfb6cb00d |
| supplement.json | fc249f2603498c78588473f8a630ceac109f292de73b13dbcc6632129f6a309b |
| deployment.json | d54198b58431a0420a7644b2193e5882685a159c3bb08e674d4bfa78aa937d1b |
| activity.json | f3ccff4a38101aa0d432585bc14daf3096aef90a8fe3c7504403c51b94e7972c |
| evidence.json | 992b9ab08036f2d459443bacb1a05ffef2d03675bd76f5115da7c06f6133756a |
| day.json | a35da6f23ee02799a4948786ceeb51fbb55786c70037b3fa053239ad0aa7883e |
| opening-recovery.json | e8bfb6efc731614cb1d06ed77f6de462db0031761ab7e868a6fb47b03563e218 |
| monitor-reconciled.json | 24ec103e6817503c2d81e2ca4f5c3eddf951a88bbfcc0e1e9b747158cb5465b9 |
| runtime.json | 298489deb7bbb1fedc6bc838fa0923eef4623baa398912d630e68d12efaff97f |
| history-proof.json | 0e78dcae01cd9b7239bbbd8c65337eef5adf8ab196d864bd9d701daa41ccf4e5 |
