# Real-position costs and clean reservation — №18,372

№18,372 §§1–10 · 2026-11-08T19:03:25.743847+08:00 · by Codex (GPT-6), effort not exposed · base PR #198 / `9e640cd92c6b1ebef6a033c7f4c050e36ea7b59d`.

## §1. TLDR and progress towards the goal

**The 260 statement-matched positions lost $15,357.91 net, with 36/260 net wins (13.85%).** Gross fill-to-fill P&L was $-15,166.28. Recorded commission and financing reduced it by a further $191.63. The gross loss already existed; removing those booked charges would not repair it. Spread and slippage can already be embedded in gross P&L, so this does not show that all execution costs were small.

**Engineering gates advance from 4/12 to 6/12:** G6 passes for the 260 reconciled actual positions, and G9 passes for the explicitly versioned local pre-collection reservation. The requested current/historical-proxy verifier is implemented, but original G5 historical broker authority remains unproved. G7 has honest component calibration, not a complete round-trip execution-cost model.

**Reservation:** 160 episodes in four calendar weeks; zero planning refusals; 45 verified M30 source files reused; 332 bounded M30 request intervals and 160 tick episodes still needed. Nothing was collected. Tick collection correctly remains refused until preparation is decoded.

**Goal progress:** zero new independent trades and no new strategy-return experiment. The project target remains at least 15 net wins in each strategy/account's latest 20 trades (75%, the first attainable count at or above 73%), with reference 1:1.38 R:R. The pooled historic win rate below is descriptive, not a strategy/account qualification result. Financial pass remains false; validation remains sealed; live rules are unchanged.

## §2. Denominators, dates and exact money reconciliation

These are positions, not 271 independent deal messages. The 271 quantity-complete position lifecycles include partial closes and real netting reversals. Exactly 260 have every closing fill matched to retained statements: 264 closing fills. All 264 dated lot conversions inferred from statement lots and broker closed native quantity agree with current contract lot sizes. This establishes the observed quantity conversion at those trades; it does not certify all historic contract terms.

| Evidence scope | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD |
| --- | --- | --- | --- | --- | --- | --- | --- |
| All quantity-complete | 271 | 38 | 14.02% | -15,959.99 | -179.76 | -13.47 | -16,153.22 |
| Fully statement matched | 260 | 36 | 13.85% | -15,166.28 | -179.76 | -11.87 | -15,357.91 |
| Other complete, not fully statement matched | 11 | 2 | 18.18% | -793.71 | 0.00 | -1.60 | -795.31 |

Matched data range: first entry `2026-10-25T08:33:48.729000Z`, last exit `2026-11-04T03:01:14.042000Z`. The all-271 ledger extends through `2026-11-06T14:12:49.366000Z`. Money is USD as recorded in the retained account statements. Demo dollars and live dollars are shown separately; their descriptive sum is not the owner's investable portfolio return. The additional 11 positions are retained rather than silently discarded or certified as statement-matched.

| Statement-matched account | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 26 | 3 | 11.54% | -250.81 | -0.16 | -5.20 | -256.17 | 5.36 | 0.0000 | 2026-10-25T08:33:49.142000Z → 2026-11-04T03:01:14.042000Z |
| 46130058 | 85 | 13 | 15.29% | -6,110.40 | 0.00 | 7.43 | -6,102.97 | -7.43 | 0.0000 | 2026-10-25T08:33:48.930000Z → 2026-11-03T20:00:00.873000Z |
| 47790949 | 81 | 12 | 14.81% | -8,622.22 | -169.24 | -14.10 | -8,805.56 | 183.34 | 0.0000 | 2026-10-25T08:33:48.788000Z → 2026-11-03T20:00:00.728000Z |
| 49236051 | 68 | 8 | 11.76% | -182.85 | -10.36 | 0.00 | -193.21 | 10.36 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-10-29T14:30:55.896000Z |

| All quantity-complete account | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 29 | 3 | 10.34% | -815.18 | -0.16 | -5.20 | -820.54 | 5.36 | 0.0000 | 2026-10-25T08:33:49.142000Z → 2026-11-06T03:59:06.262000Z |
| 46130058 | 87 | 13 | 14.94% | -6,228.39 | 0.00 | 6.63 | -6,221.76 | -6.63 | 0.0000 | 2026-10-25T08:33:48.930000Z → 2026-11-06T14:12:49.340000Z |
| 47790949 | 83 | 12 | 14.46% | -8,737.44 | -169.24 | -14.90 | -8,921.58 | 184.14 | 0.0000 | 2026-10-25T08:33:48.788000Z → 2026-11-06T14:12:49.366000Z |
| 49236051 | 72 | 10 | 13.89% | -178.98 | -10.36 | 0.00 | -189.34 | 10.36 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-11-05T15:02:00.355000Z |

### Matched positions by symbol

| Symbol | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| AUDUSD | 10 | 2 | 20.00% | -487.05 | -14.10 | 0.00 | -501.15 | 14.10 | 0.9956 | 2026-10-26T09:17:58.514000Z → 2026-10-27T20:00:03.442000Z |
| AUS200 | 17 | 6 | 35.29% | 181.32 | 0.00 | 0.00 | 181.32 | 0.00 | 0.0000 | 2026-10-25T23:30:03.751000Z → 2026-10-27T05:00:01Z |
| BTCUSD | 42 | 2 | 4.76% | -3,990.09 | 0.00 | -4.32 | -3,994.41 | 4.32 | 0.0000 | 2026-10-25T09:12:34.155000Z → 2026-11-04T03:01:14.042000Z |
| ETHUSD | 64 | 0 | 0.00% | -5,186.35 | 0.00 | 0.00 | -5,186.35 | 0.00 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-10-29T14:30:55.896000Z |
| EURUSD | 7 | 3 | 42.86% | -114.46 | -6.96 | 0.00 | -121.42 | 6.96 | 0.6263 | 2026-10-26T07:30:17.633000Z → 2026-10-27T12:10:53.215000Z |
| GBPUSD | 6 | 3 | 50.00% | 321.02 | -22.54 | 0.00 | 298.48 | 22.54 | 0.5239 | 2026-10-26T07:30:10.195000Z → 2026-10-27T11:23:44.006000Z |
| GER40 | 4 | 4 | 100.00% | 796.08 | 0.00 | 0.00 | 796.08 | 0.00 | 0.0000 | 2026-10-26T10:49:17.785000Z → 2026-10-27T16:30:01.765000Z |
| JPN225 | 17 | 0 | 0.00% | -2,567.92 | 0.00 | 0.00 | -2,567.92 | 0.00 | 0.0000 | 2026-10-26T00:30:05.480000Z → 2026-10-29T03:37:44.359000Z |
| NAS100 | 3 | 0 | 0.00% | -43.58 | 0.00 | 0.00 | -43.58 | 0.00 | 0.0000 | 2026-10-26T16:31:54.777000Z → 2026-10-27T20:00:02.087000Z |
| NZDUSD | 6 | 3 | 50.00% | 84.37 | -36.34 | 0.00 | 48.03 | 36.34 | 1.2306 | 2026-10-26T08:01:42.951000Z → 2026-10-27T14:01:16.686000Z |
| NatGas | 16 | 0 | 0.00% | -2,611.10 | 0.00 | -14.10 | -2,625.20 | 14.10 | 0.0000 | 2026-10-26T07:31:20.296000Z → 2026-10-30T13:09:54.569000Z |
| SpotBrent | 14 | 1 | 7.14% | -991.26 | 0.00 | 6.55 | -984.71 | -6.55 | 0.0000 | 2026-10-26T13:58:14.656000Z → 2026-11-03T13:29:49.889000Z |
| SpotCrude | 15 | 1 | 6.67% | -1,105.18 | 0.00 | 0.00 | -1,105.18 | 0.00 | 0.0000 | 2026-10-26T07:31:06.684000Z → 2026-11-03T20:00:00.873000Z |
| UK100 | 1 | 0 | 0.00% | -3.25 | 0.00 | 0.00 | -3.25 | 0.00 | 0.0000 | 2026-10-27T08:58:35.780000Z → 2026-10-27T10:39:32.826000Z |
| US500 | 3 | 1 | 33.33% | -165.71 | 0.00 | 0.00 | -165.71 | 0.00 | 0.0000 | 2026-10-26T14:13:54.452000Z → 2026-10-27T14:30:22.874000Z |
| USDCAD | 4 | 0 | 0.00% | -512.65 | -33.46 | 0.00 | -546.11 | 33.46 | 0.3499 | 2026-10-26T07:30:04.385000Z → 2026-10-27T07:42:16.521000Z |
| USDJPY | 9 | 4 | 44.44% | 960.79 | -61.72 | 0.00 | 899.07 | 61.72 | 0.7010 | 2026-10-26T07:30:04.468000Z → 2026-10-27T12:34:05.127000Z |
| XAGUSD | 14 | 4 | 28.57% | 412.85 | 0.00 | 0.00 | 412.85 | 0.00 | 0.0000 | 2026-10-26T07:30:21.935000Z → 2026-10-28T13:27:03.766000Z |
| XAUUSD | 8 | 2 | 25.00% | -144.11 | -4.64 | 0.00 | -148.75 | 4.64 | 0.0872 | 2026-10-26T08:01:00.475000Z → 2026-10-27T20:00:04.024000Z |

### All 271 quantity-complete positions by symbol

| Symbol | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| AUDUSD | 10 | 2 | 20.00% | -487.05 | -14.10 | 0.00 | -501.15 | 14.10 | 0.9956 | 2026-10-26T09:17:58.514000Z → 2026-10-27T20:00:03.442000Z |
| AUS200 | 17 | 6 | 35.29% | 181.32 | 0.00 | 0.00 | 181.32 | 0.00 | 0.0000 | 2026-10-25T23:30:03.751000Z → 2026-10-27T05:00:01Z |
| BTCUSD | 42 | 2 | 4.76% | -3,990.09 | 0.00 | -4.32 | -3,994.41 | 4.32 | 0.0000 | 2026-10-25T09:12:34.155000Z → 2026-11-04T03:01:14.042000Z |
| ETHUSD | 65 | 1 | 1.54% | -5,174.85 | 0.00 | 0.00 | -5,174.85 | 0.00 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-11-05T15:02:00.355000Z |
| EURUSD | 7 | 3 | 42.86% | -114.46 | -6.96 | 0.00 | -121.42 | 6.96 | 0.6263 | 2026-10-26T07:30:17.633000Z → 2026-10-27T12:10:53.215000Z |
| GBPUSD | 6 | 3 | 50.00% | 321.02 | -22.54 | 0.00 | 298.48 | 22.54 | 0.5239 | 2026-10-26T07:30:10.195000Z → 2026-10-27T11:23:44.006000Z |
| GER40 | 6 | 4 | 66.67% | 572.67 | 0.00 | -1.60 | 571.07 | 1.60 | 0.0000 | 2026-10-26T10:49:17.785000Z → 2026-11-06T14:12:49.366000Z |
| JPN225 | 20 | 0 | 0.00% | -3,132.29 | 0.00 | 0.00 | -3,132.29 | 0.00 | 0.0000 | 2026-10-26T00:30:05.480000Z → 2026-11-06T03:59:06.262000Z |
| NAS100 | 3 | 0 | 0.00% | -43.58 | 0.00 | 0.00 | -43.58 | 0.00 | 0.0000 | 2026-10-26T16:31:54.777000Z → 2026-10-27T20:00:02.087000Z |
| NZDUSD | 6 | 3 | 50.00% | 84.37 | -36.34 | 0.00 | 48.03 | 36.34 | 1.2306 | 2026-10-26T08:01:42.951000Z → 2026-10-27T14:01:16.686000Z |
| NatGas | 19 | 1 | 5.26% | -2,620.10 | 0.00 | -14.10 | -2,634.20 | 14.10 | 0.0000 | 2026-10-26T07:31:20.296000Z → 2026-11-04T19:13:44.481000Z |
| SpotBrent | 15 | 1 | 6.67% | -995.21 | 0.00 | 6.55 | -988.66 | -6.55 | 0.0000 | 2026-10-26T13:58:14.656000Z → 2026-11-03T13:29:49.889000Z |
| SpotCrude | 16 | 1 | 6.25% | -1,109.66 | 0.00 | 0.00 | -1,109.66 | 0.00 | 0.0000 | 2026-10-26T07:31:06.684000Z → 2026-11-03T20:00:00.873000Z |
| UK100 | 1 | 0 | 0.00% | -3.25 | 0.00 | 0.00 | -3.25 | 0.00 | 0.0000 | 2026-10-27T08:58:35.780000Z → 2026-10-27T10:39:32.826000Z |
| US500 | 3 | 1 | 33.33% | -165.71 | 0.00 | 0.00 | -165.71 | 0.00 | 0.0000 | 2026-10-26T14:13:54.452000Z → 2026-10-27T14:30:22.874000Z |
| USDCAD | 4 | 0 | 0.00% | -512.65 | -33.46 | 0.00 | -546.11 | 33.46 | 0.3499 | 2026-10-26T07:30:04.385000Z → 2026-10-27T07:42:16.521000Z |
| USDJPY | 9 | 4 | 44.44% | 960.79 | -61.72 | 0.00 | 899.07 | 61.72 | 0.7010 | 2026-10-26T07:30:04.468000Z → 2026-10-27T12:34:05.127000Z |
| XAGUSD | 14 | 4 | 28.57% | 412.85 | 0.00 | 0.00 | 412.85 | 0.00 | 0.0000 | 2026-10-26T07:30:21.935000Z → 2026-10-28T13:27:03.766000Z |
| XAUUSD | 8 | 2 | 25.00% | -144.11 | -4.64 | 0.00 | -148.75 | 4.64 | 0.0872 | 2026-10-26T08:01:00.475000Z → 2026-10-27T20:00:04.024000Z |

Actual money uses Decimal broker moneyDigits, signed swap/conversion and the broker closing commission total once. It does not charge opening commission again or append hypothetical spread/slippage to already realised broker net P&L. Gross + commission + swap + conversion fee = net is checked. Conversion fees are zero in this retained set; that is an observation, not a permanent fee assumption.

## §3. What the cost figures mean

`Booked fee/financing drag = gross − net`. Per-position price drag divides each closing money charge by its recorded quote/deposit conversion and then by total native units. Basis points divide total money drag by the sum of closing-reference entry price × closed native units × recorded conversion. Credits produce negative drag. This fixed denominator is a descriptive normalisation, not the margin, account equity, R-risk, or an independent midpoint benchmark.

Price-unit medians are shown only within a symbol. Money sums can be aggregated; incompatible price units cannot. Full round-trip execution cost remains null, not zero, because matching causal benchmarks for both legs and accepted resting-order provenance are absent. A zero median booked fee does not imply zero trading cost.

| Symbol | Matched positions | Median booked fee/financing price units | Median booked fee/financing bp | Full execution-cost median bp |
| --- | --- | --- | --- | --- |
| AUDUSD | 10 | 0.00007000 | 0.9956 | Not identifiable |
| AUS200 | 17 | 0.00000000 | 0.0000 | Not identifiable |
| BTCUSD | 42 | 0.00000000 | 0.0000 | Not identifiable |
| ETHUSD | 64 | 0.00000000 | 0.0000 | Not identifiable |
| EURUSD | 7 | 0.00007000 | 0.6263 | Not identifiable |
| GBPUSD | 6 | 0.00007000 | 0.5239 | Not identifiable |
| GER40 | 4 | 0.00000000 | 0.0000 | Not identifiable |
| JPN225 | 17 | 0.00000000 | 0.0000 | Not identifiable |
| NAS100 | 3 | 0.00000000 | 0.0000 | Not identifiable |
| NZDUSD | 6 | 0.00007000 | 1.2306 | Not identifiable |
| NatGas | 16 | 0.00000000 | 0.0000 | Not identifiable |
| SpotBrent | 14 | 0.00000000 | 0.0000 | Not identifiable |
| SpotCrude | 15 | 0.00000000 | 0.0000 | Not identifiable |
| UK100 | 1 | 0.00000000 | 0.0000 | Not identifiable |
| US500 | 3 | 0.00000000 | 0.0000 | Not identifiable |
| USDCAD | 4 | 0.00004906 | 0.3499 | Not identifiable |
| USDJPY | 9 | 0.01104656 | 0.7010 | Not identifiable |
| XAGUSD | 14 | 0.00000000 | 0.0000 | Not identifiable |
| XAUUSD | 8 | 0.03500000 | 0.0872 | Not identifiable |

The largest losses in this matched set include ETHUSD (0/64 wins, −$5,186.35), BTCUSD (2/42, −$3,994.41), NatGas (0/16, −$2,625.20) and JPN225 (0/17, −$2,567.92). These are observed outcomes, not causal proof of one defective signal. Strategy labels include S3, S1 and unlabelled/mixed positions; this is not a backtest of the frozen macro-pullback candidate. No reverse-direction edge or profitable repair is inferred from these losses.

## §4. Entry and resting-level exit evidence

All 17 captured entry dispatch observations are retained below, with account and symbol attribution. They are not automatically 17 of the 260 positions: 5 overlap that matched position set. Positive signed entry deviation is adverse under the dispatch-side convention; negative is favourable. Entry deviation alone is not round-trip cost.

| Account | Symbol | Position | UTC observation | Side | Signed bp | Dispatch spread price units | In matched260 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 | BTCUSD | 256217469 | 2026-11-02T13:30:04.639000Z | entry_sell | 0.0180 | 17.25000000 | True |
| 46130058 | BTCUSD | 256468373 | 2026-11-02T18:30:04.837000Z | entry_sell | -0.6623 | 15.00000000 | True |
| 49236051 | SpotBrent | 256977862 | 2026-11-03T10:00:05.021000Z | entry_sell | 0.9149 | 0.03200000 | False |
| 47790949 | SpotBrent | 257083550 | 2026-11-03T13:00:04.852000Z | entry_sell | -1.3776 | 0.03200000 | True |
| 49236051 | SpotCrude | 257128578 | 2026-11-03T14:00:04.860000Z | entry_sell | -0.6300 | 0.02800000 | False |
| 47790949 | SpotCrude | 257188563 | 2026-11-03T15:30:04.720000Z | entry_sell | 2.4745 | 0.02800000 | True |
| 46130058 | SpotCrude | 257230036 | 2026-11-03T16:30:09.775000Z | entry_sell | -0.0000 | 0.02800000 | True |
| 49236051 | NatGas | 257968264 | 2026-11-04T17:00:04.888000Z | entry_buy | 0.0000 | 0.00400000 | False |
| 47790949 | NatGas | 257991393 | 2026-11-04T17:30:04.767000Z | entry_buy | 0.0000 | 0.00300000 | False |
| 46130058 | NatGas | 257991398 | 2026-11-04T17:30:04.964000Z | entry_buy | 3.7383 | 0.00300000 | False |
| 46130058 | GER40 | 258055230 | 2026-11-04T19:00:23.414000Z | entry_buy | -0.3755 | 0.90000000 | False |
| 47790949 | GER40 | 258058043 | 2026-11-04T19:04:10.657000Z | entry_buy | 0.0000 | 0.90000000 | False |
| 49236051 | ETHUSD | 258721368 | 2026-11-05T13:00:06.932000Z | entry_sell | 4.4360 | 2.52000000 | False |
| 47790949 | AUDUSD | 259680976 | 2026-11-06T16:31:22.811000Z | entry_sell | 0.4362 | 0.00001000 | False |
| 46130058 | AUDUSD | 259681011 | 2026-11-06T16:31:26.157000Z | entry_sell | 0.1454 | 0.00011000 | False |
| 47790949 | NZDUSD | 259681023 | 2026-11-06T16:31:26.986000Z | entry_sell | -0.0000 | 0.00002000 | False |
| 46130058 | NZDUSD | 259681080 | 2026-11-06T16:31:30.600000Z | entry_sell | -0.0000 | 0.00012000 | False |

**Exit reference found:** 143 deal/recorded-level pairs, of which 137 belong to the matched260 positions. 65 broker-exit rows lack usable recorded-level/close detail and remain explicit exclusions. The stored row's fill and volume are checked against its raw cTrader deal. Absolute `|fill − recorded level|` and signed exit-side deviation are computed; adverse is positive. Full precision observations and raw deal/order hashes are in JSON.

**Critical provenance finding:** `fills.rs:exit_level` selects the nearer stored SL/TP, while `strategy3.rs:broker_exit` can fall back from report fields to local owned-position levels. Consequently “stop_loss” or “take_profit” here is an inferred nearest-level label. It is not proof that a broker resting order triggered, that the level was the last broker-accepted amendment, or that the deviation is pure execution slippage. Server-triggered exits need accepted order/amendment history and trigger/fill context; inventing a client dispatch timestamp would be wrong.

| Account | Symbol | Inferred level label | Fills | Median signed bp | Median absolute bp | Max absolute bp |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | AUS200 | stop_loss | 1 | 0.5674 | 0.5674 | 0.5674 |
| 43002148 | BTCUSD | stop_loss | 1 | 3.4197 | 3.4197 | 3.4197 |
| 43002148 | ETHUSD | stop_loss | 6 | 0.8317 | 0.8317 | 4.7875 |
| 43002148 | SpotCrude | stop_loss | 1 | 1.5588 | 1.5588 | 1.5588 |
| 46130058 | AUDUSD | stop_loss | 3 | -0.0000 | 0.0000 | 0.1424 |
| 46130058 | AUS200 | stop_loss | 1 | 1.2484 | 1.2484 | 1.2484 |
| 46130058 | BTCUSD | stop_loss | 8 | 1.0729 | 1.0729 | 4.3649 |
| 46130058 | ETHUSD | stop_loss | 7 | 1.5734 | 2.6312 | 6.2067 |
| 46130058 | EURUSD | stop_loss | 2 | -0.1345 | 0.1345 | 0.2690 |
| 46130058 | GBPUSD | stop_loss | 1 | 0.0751 | 0.0751 | 0.0751 |
| 46130058 | JPN225 | stop_loss | 4 | 0.3318 | 0.6593 | 0.7274 |
| 46130058 | NatGas | stop_loss | 3 | 3.7608 | 3.7608 | 7.4738 |
| 46130058 | SpotBrent | stop_loss | 3 | 1.0368 | 1.0368 | 6.5556 |
| 46130058 | SpotCrude | stop_loss | 3 | -0.6832 | 2.2365 | 8.8581 |
| 46130058 | USDCAD | stop_loss | 2 | 0.1783 | 0.1783 | 0.2853 |
| 46130058 | USDJPY | stop_loss | 1 | 0.0000 | 0.0000 | 0.0000 |
| 46130058 | XAGUSD | stop_loss | 5 | -0.0000 | 0.5682 | 1.3256 |
| 46130058 | XAGUSD | take_profit | 1 | -0.5544 | 0.5544 | 0.5544 |
| 46130058 | XAUUSD | stop_loss | 3 | 0.0749 | 0.0749 | 0.1500 |
| 47790949 | AUS200 | stop_loss | 1 | 1.2484 | 1.2484 | 1.2484 |
| 47790949 | BTCUSD | stop_loss | 8 | 1.4849 | 1.4849 | 4.3649 |
| 47790949 | ETHUSD | stop_loss | 6 | 1.3038 | 2.2918 | 6.2067 |
| 47790949 | GBPUSD | stop_loss | 1 | 0.5254 | 0.5254 | 0.5254 |
| 47790949 | JPN225 | stop_loss | 3 | 0.0468 | 0.6168 | 0.7274 |
| 47790949 | NZDUSD | stop_loss | 1 | -0.5269 | 0.5269 | 0.5269 |
| 47790949 | NatGas | stop_loss | 3 | 3.8285 | 3.8285 | 7.4738 |
| 47790949 | SpotBrent | stop_loss | 3 | 0.3457 | 0.3457 | 6.5556 |
| 47790949 | SpotCrude | stop_loss | 3 | 0.3897 | 0.3897 | 3.2303 |
| 47790949 | US500 | stop_loss | 1 | 0.3898 | 0.3898 | 0.3898 |
| 47790949 | USDCAD | stop_loss | 1 | -0.0000 | 0.0000 | 0.0000 |
| 47790949 | USDJPY | stop_loss | 2 | -0.0000 | 0.1267 | 0.1267 |
| 47790949 | XAGUSD | stop_loss | 4 | 0.5684 | 0.5684 | 0.7541 |
| 47790949 | XAGUSD | take_profit | 1 | -0.0000 | 0.0000 | 0.0000 |
| 47790949 | XAUUSD | stop_loss | 2 | 0.2998 | 0.2998 | 0.3501 |
| 49236051 | AUDUSD | stop_loss | 3 | 0.1424 | 0.1424 | 0.1425 |
| 49236051 | AUS200 | stop_loss | 1 | 1.2484 | 1.2484 | 1.2484 |
| 49236051 | BTCUSD | stop_loss | 6 | 0.7061 | 0.7061 | 1.3669 |
| 49236051 | ETHUSD | stop_loss | 7 | 0.4229 | 0.8599 | 6.2928 |
| 49236051 | ETHUSD | take_profit | 1 | -5.2551 | 5.2551 | 5.2551 |
| 49236051 | EURUSD | stop_loss | 2 | -0.0448 | 0.0448 | 0.0897 |
| 49236051 | GBPUSD | stop_loss | 1 | 0.3002 | 0.3002 | 0.3002 |
| 49236051 | NZDUSD | stop_loss | 2 | 0.0878 | 0.0878 | 0.1757 |
| 49236051 | NatGas | stop_loss | 7 | -0.0000 | 0.0000 | 11.4635 |
| 49236051 | SpotBrent | stop_loss | 5 | 1.9972 | 1.9972 | 6.5556 |
| 49236051 | SpotCrude | stop_loss | 5 | 0.1040 | 0.5846 | 8.8581 |
| 49236051 | UK100 | stop_loss | 1 | 0.0935 | 0.0935 | 0.0935 |
| 49236051 | US500 | stop_loss | 1 | 0.2599 | 0.2599 | 0.2599 |
| 49236051 | USDCAD | stop_loss | 1 | -0.0000 | 0.0000 | 0.0000 |
| 49236051 | USDJPY | stop_loss | 2 | -0.0634 | 0.1901 | 0.2535 |
| 49236051 | XAGUSD | stop_loss | 1 | 0.3711 | 0.3711 | 0.3711 |
| 49236051 | XAUUSD | stop_loss | 1 | 0.2749 | 0.2749 | 0.2749 |

## §5. Frozen temporal calibration and held-out verification

The plan was frozen before calculation with SHA256 `1759843cdc9073c0059ecfc55ad48e167239430ce61fec4ce24b1192abc254fa`. For each component, the earliest 70% of distinct observation dates form training; remaining dates form a chronological holdout. Fee positions crossing the cutoff are explicitly boundary-excluded. Each account/environment/symbol/purpose bucket is separate. No demo estimate is borrowed by live. At least three training observations and two held-out observations are required; otherwise there is no fitted prediction. The only fitted estimator is the training arithmetic mean, fixed without inspecting holdout values.

| Component | Observations | UTC cutoff | Train | Held-out | Boundary exclusions | Fitted buckets |
| --- | --- | --- | --- | --- | --- | --- |
| entry_dispatch_slippage | 17 | 2026-11-05T00:00:00Z | 12 | 5 | 0 | 0 |
| exit_level_deviation_proxy | 143 | 2026-11-03T00:00:00Z | 136 | 7 | 0 | 0 |
| historical_spread | 1710 | 2025-05-13T00:00:00Z | 1024 | 686 | 0 | 7 |
| recorded_fee_financing | 260 | 2026-11-01T00:00:00Z | 249 | 10 | 1 | 2 |

| Component | Account / symbol | Train / held-out | Train mean bp | Held-out MAE bp | Bias bp | Daily blocks | Normal 95% half-width bp |
| --- | --- | --- | --- | --- | --- | --- | --- |
| historical_spread | 49236051 / BTCUSD | 47 / 55 | 3.6776 | 1.3449 | -1.3358 | 11 | 0.5986 |
| historical_spread | 49236051 / NAS100 | 25 / 10 | 0.6137 | 0.1482 | -0.1482 | 2 | 0.0014 |
| historical_spread | 49236051 / SpotBrent | 291 / 93 | 3.6987 | 0.7166 | -0.2645 | 16 | 0.1364 |
| historical_spread | 49236051 / SpotCrude | 291 / 100 | 3.5488 | 0.6050 | -0.0996 | 16 | 0.2084 |
| historical_spread | 49236051 / US500 | 172 / 77 | 0.7876 | 0.1547 | -0.1547 | 16 | 0.0159 |
| historical_spread | 49236051 / XAGUSD | 23 / 10 | 8.7272 | 3.3600 | -3.3600 | 2 | 0.5359 |
| historical_spread | 49236051 / XAUUSD | 175 / 35 | 0.7264 | 0.3386 | -0.3386 | 7 | 0.0190 |
| recorded_fee_financing | 46130058 / BTCUSD | 12 / 2 | 0.0000 | 0.2726 | -0.2726 | 2 | 0.5343 |
| recorded_fee_financing | 47790949 / BTCUSD | 11 / 2 | 0.0000 | 0.0000 | 0.0000 | 2 | 0.0000 |

The nine fitted component buckets comprise two BTCUSD demo fee/financing buckets and seven historical spread buckets, all on their own account. There are no fitted dispatch or exit-reference buckets at the declared minimum sample sizes. All sparse buckets, observation IDs, cutoff exclusions, frozen predictions and individual holdout errors are retained in JSON. The historical spreads come from 1,710 deduplicated causal quote checkpoints, not the 2026 execution events.

Uncertainty is reported as daily block errors plus a normal 95% half-width around daily mean error where at least two days exist. These are diagnostics: blocks may be dependent; some have only two days; zero error on two zero-fee observations is weak evidence. These records were already inspected in earlier work, so the temporal split is retrospective, not a pristine independent evaluation. No calibrated component is silently inserted into live costs or historical strategy admission. The assumed 2/3.74/24 bp scenarios retain their prior labels.

## §6. Two authority tiers and complete on-disk scenarios

**Tier A — current authority:** parsed authenticated account/symbol snapshots, raw hashes, broker IANA schedule zone, current contract fields and captured holiday records. Their authority is at capture time. Display precision is not independent minimum-tick-size evidence.

**Tier B — historical replay proxy:** `historical_iana_plus_current_broker_spec`. Exact frozen native-calendar metadata is compared with the archived Phase4 engine, including US index 09:30 New York DST behaviour. Required native M30 timestamps and fresh opening/closing quote boundaries must exist; missing bars are not manufactured or declared holidays. London DST conversion is covered by the existing verifier fixtures. Unknown historical broker holidays, maintenance, funding and contract multipliers stay unknown. The current snapshot is not backdated.

The requested practical proxy policy passes for the 37 reused native-complete scenarios. This does not turn the original RFC G5 requirement for dated authoritative examples into a pass. Any future promotion requiring that authority remains blocked. The candidate, native sessions, risk geometry and cost assumptions are unchanged.

37 on-disk scenarios have complete native preparation and fresh tick boundaries; all were already used by the candidate. **Untouched complete scenarios: 0.** Additional explicit side-gap, same-timestamp price ambiguity and closing coverage checks qualify 2 under the existing observed-path criterion (US500 9–10 April 2025). A >6-second update gap may reflect market inactivity; it remains insufficient evidence for full quote-path reconstruction under this declared guard. Neither qualified reused path supplies a new independent trade or proves broker execution parity.

| Episode | Symbol | Date | Candidate already used | Proxy prep/boundaries | Side gaps >6s | Ambiguous side times | Observed path qualified |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cdb6706f8187637f3fe3352d | SpotBrent | 2024-11-04 | True | QUALIFIED_PROXY_REPLAY | 1411 | 0 | False |
| 3609573a2a98577036cc248d | SpotBrent | 2024-11-05 | True | QUALIFIED_PROXY_REPLAY | 1401 | 0 | False |
| 7e1f8994fecf745690603eeb | SpotBrent | 2024-11-06 | True | QUALIFIED_PROXY_REPLAY | 1240 | 0 | False |
| a92fb0ae4794ae5899abbb1d | SpotBrent | 2025-04-07 | True | QUALIFIED_PROXY_REPLAY | 378 | 0 | False |
| f5233c2f0a824dfd2f998e89 | SpotBrent | 2025-04-08 | True | QUALIFIED_PROXY_REPLAY | 854 | 0 | False |
| c3a0d383d87130df509791fa | SpotBrent | 2025-04-09 | True | QUALIFIED_PROXY_REPLAY | 599 | 0 | False |
| 1f6ef75ca564ffa87d523f97 | SpotBrent | 2025-04-10 | True | QUALIFIED_PROXY_REPLAY | 746 | 0 | False |
| 70e3acf0ef22a39ed2354106 | SpotBrent | 2025-04-11 | True | QUALIFIED_PROXY_REPLAY | 621 | 0 | False |
| a451c5daee55ffdac04ccd9a | SpotCrude | 2024-11-04 | True | QUALIFIED_PROXY_REPLAY | 1420 | 0 | False |
| e060f88cb7dc74c96c07b68b | SpotCrude | 2024-11-05 | True | QUALIFIED_PROXY_REPLAY | 1430 | 0 | False |
| 910dc7f401c979f077122690 | SpotCrude | 2024-11-06 | True | QUALIFIED_PROXY_REPLAY | 1117 | 0 | False |
| 77b10f0f75d5383bb011c4b2 | SpotCrude | 2025-04-07 | True | QUALIFIED_PROXY_REPLAY | 281 | 0 | False |
| 2b977b08318352c8c79f2cbc | SpotCrude | 2025-04-08 | True | QUALIFIED_PROXY_REPLAY | 786 | 0 | False |
| b8444e851d499207d0449237 | SpotCrude | 2025-04-09 | True | QUALIFIED_PROXY_REPLAY | 466 | 0 | False |
| 9e626ae09c741e2673fd8c78 | SpotCrude | 2025-04-10 | True | QUALIFIED_PROXY_REPLAY | 620 | 0 | False |
| ce5a75c11b08e52a205e5350 | SpotCrude | 2025-04-11 | True | QUALIFIED_PROXY_REPLAY | 494 | 0 | False |
| 4d04e030932184cbabc7f97b | US500 | 2024-11-05 | True | QUALIFIED_PROXY_REPLAY | 438 | 0 | False |
| 8d7090e5c928d51baf5f5d28 | US500 | 2024-11-06 | True | QUALIFIED_PROXY_REPLAY | 238 | 0 | False |
| f35c2955c68672466ebc4e06 | US500 | 2024-11-07 | True | QUALIFIED_PROXY_REPLAY | 882 | 0 | False |
| b8faf0a449b13f21ebd8db35 | US500 | 2024-11-08 | True | QUALIFIED_PROXY_REPLAY | 1026 | 0 | False |
| 31ac96bb6c2db5e56d25c188 | US500 | 2025-04-07 | True | QUALIFIED_PROXY_REPLAY | 8 | 0 | False |
| 5b35427f494f9de75bb0ddd1 | US500 | 2025-04-08 | True | QUALIFIED_PROXY_REPLAY | 4 | 0 | False |
| 7be5dc89274ab1082ba3e7ba | US500 | 2025-04-09 | True | QUALIFIED_PROXY_REPLAY | 0 | 0 | True |
| 5ff6b8a3b787709ae2dcc9a0 | US500 | 2025-04-10 | True | QUALIFIED_PROXY_REPLAY | 0 | 0 | True |
| acdc1e0257e10eefaf512a7f | US500 | 2025-04-11 | True | QUALIFIED_PROXY_REPLAY | 2 | 0 | False |
| caca0aae14aacd9f05311212 | XAUUSD | 2024-11-04 | True | QUALIFIED_PROXY_REPLAY | 37 | 0 | False |
| b92b7b077dd282ac80b2c042 | XAUUSD | 2024-11-05 | True | QUALIFIED_PROXY_REPLAY | 42 | 0 | False |
| 877ac3424c0551b6f7772d6a | XAUUSD | 2024-11-06 | True | QUALIFIED_PROXY_REPLAY | 5 | 0 | False |
| 569183452c60779cd669e2eb | XAUUSD | 2024-11-07 | True | QUALIFIED_PROXY_REPLAY | 15 | 0 | False |
| f741e94dbeb081106be380a1 | XAUUSD | 2024-11-08 | True | QUALIFIED_PROXY_REPLAY | 15 | 0 | False |
| ae86f219cd744d0fe05a351e | XAUUSD | 2025-04-07 | True | QUALIFIED_PROXY_REPLAY | 4 | 0 | False |
| 44e4ecb72fbb53cc14fc14da | XAUUSD | 2025-04-08 | True | QUALIFIED_PROXY_REPLAY | 57 | 0 | False |
| 3da7750ba2a191ca753fb2e9 | XAUUSD | 2025-04-09 | True | QUALIFIED_PROXY_REPLAY | 10 | 0 | False |
| 46371ae1ac4c72c2b4aa725b | XAUUSD | 2025-04-10 | True | QUALIFIED_PROXY_REPLAY | 5 | 0 | False |
| d9c1023f5de634fed865d7b3 | XAUUSD | 2025-04-11 | True | QUALIFIED_PROXY_REPLAY | 1 | 0 | False |
| phase5-XAUUSD-2023-10-20 | XAUUSD | 2023-10-20 | True | QUALIFIED_PROXY_REPLAY | 255 | 0 | False |
| phase5-XAGUSD-2023-10-20 | XAGUSD | 2023-10-20 | True | QUALIFIED_PROXY_REPLAY | 1162 | 0 | False |

## §7. Versioned reservation with zero local planning refusals

The deterministic search covers 5872 symbol/date definitions from 2023-01-02 to before 2025-10-24. It excludes the sealed 19 August–5 September 2025 interval and protected preparation dependencies, all locally inspected evaluation-price intervals, failed/spent missing preparation, and development-label embargo overlaps. 7 full eight-symbol weeks survive. Selection uses minimum distinct missing preparation slots, then date, with no new price outcomes inspected and no optimisation of trade returns.

| Week starts | Required cache slots reused | Missing distinct native slots within week | Episodes |
| --- | --- | --- | --- |
| 2023-08-21 | 1421 | 7833 | 40 |
| 2024-11-11 | 2786 | 6468 | 40 |
| 2024-11-18 | 2331 | 6923 | 40 |
| 2024-11-25 | 1876 | 7378 | 40 |

These are non-overlapping calendar weeks, not four independent regimes. Three November weeks are adjacent. Their causal warm-up may include earlier selected weeks. The entire candidate/batch must remain frozen: no refitting, selection or adaptive stopping after seeing any week's returns. The final November week contains Thanksgiving/early-close dates; historical CFD dealing hours remain unverified. If later collection fails coverage, preserve the failure; do not silently switch dates or count an empty path as a success.

The owner-authorised transition changes only reservation policy: `rebuild-offline-18368-v1` → `reservation-only-18372-v1`. Verified causal preparation may be reused, while evaluation intervals must be unseen in the retained registry and missing preparation must be unspent. The exact 160 jobs and source hashes are frozen, with one candidate and at most one trial. The old strict guard and collection pause are preserved. This is a new registered plan, not a relabelling of the previously refused reservation.

| Item | Measured/planned count |
| --- | --- |
| new_m30_segments_needed | 332 |
| new_tick_episodes_needed | 160 |
| planned_episodes | 160 |
| refusal_count | 0 |
| reused_verified_m30_segments | 45 |

Reused count means 45 distinct verified source files, not 45 job references. The 332 prospective M30 requests deduplicate overlapping dependencies, span at most seven days each, and use interval-only mode without count=500. Request intervals are half-open; any future broker adapter must preserve the exact boundary semantics. All 160 tick windows still need collection. Exact episode slots are encoded as half-open 30-minute runs with counts and SHA256; expansion is verified against the full native slot list. Deduplicated request intervals, reused source paths/hashes and the transition/proof are in JSON.

**State distinction:** preparation planning passes; tick-stage refusal is `preparation_must_be_decoded_before_ticks`; evaluation-ready is false; collection authority is false. A broker may return insufficient history or a holiday path may fail. The reservation certifies local provenance and request partitioning, not future data availability. Older unavailable non-ledger references remain a scope limitation; no claim is made about unknown external research performed elsewhere.

## §8. All twelve original RFC gates

| Gate | Original acceptance condition | Status | Evidence / remaining requirement |
| --- | --- | --- | --- |
| G1 | every result links to source, code, calendar, contract and cost versions. | PASSED | Versioned identity binds frozen hypothesis, unchanged entry/exit engine, native calendar, contract inventory, cost assumptions, spent intervals and source receipts. Unknown historical contract authority remains explicitly unknown. |
| G2 | baseline replay reproduces frozen decisions/fills and refusing paths before a migration is accepted. | PASSED | All159+35+2 native outputs reproduce the original receipt bytes, twice, including no-entry/refusal paths. |
| G3 | the runner makes zero network calls in offline mode, preserves all denominators and cannot accept arbitrary result constants. | PASSED | Single offline command; inherited Linux seccomp plus Python audit guard; native output computation only; all544 planned definitions and failures retained. |
| G4 | an experiment cannot claim full-path qualification without verified lookback, holding-period and closing-tail coverage; interrupted receipts remain visible. | PASSED | Preflight refuses missing45-day dependencies, unspent-but-undecoded preparation, wrong native windows, stale boundaries, gaps, unverified receipts and incomplete continuation. Full-path eligibility is stricter than a recorded-scenario tier. |
| G5 | targeted DST/early-close/rollover fixtures match dated authoritative examples; unknown historical terms remain unknown. | BLOCKED_HISTORICAL_AUTHORITY | Tier A parses captured current broker specs. Tier B verifies frozen native IANA sessions, decoded preparation and actual quote boundaries for 37 reused scenarios, with historical_iana_plus_current_broker_spec provenance. Requested proxy engineering policy passes; original historical-authority gate does not. Remaining: Historical broker/venue early-close, maintenance, funding and contract terms remain unknown. Current 2026 snapshots do not backdate authority to 2023–2025. |
| G6 | quantities, fees, swaps, conversion and statement money reconcile with no double counting. | PASSED | 260 positions / 264 closing fills reconcile statement money, signed fees, swaps, conversion and closed quantity. All 264 dated observed lot conversions agree with the current snapshot. No extra spread/slippage debit is applied to actual P&L. Scope: these real 2026 account observations, not historical replay contracts. |
| G7 | cost estimates have held-out error/uncertainty reports and explicit sparse-bucket fallbacks. | PARTIAL_COMPONENT_CALIBRATION | Date-ordered retrospective splits, held-out errors, daily-block diagnostics and sparse fallbacks exist for all four components. Two fee/financing and seven historical-spread buckets fit; entry and exit-reference buckets remain sparse. No full round-trip execution-cost model is asserted. Remaining: Matched causal reference/dispatch quotes and accepted resting-order provenance for both legs, sufficient account-specific independent cost observations, and stable held-out errors. Retrospective component splits are not fresh independent alpha evidence. |
| G8 | selected adversarial fixtures and recorded-event replays establish bracket priority, amend timing, gaps, partial fills, restart/reconciliation and research/live decision parity. | BLOCKED_EXTERNAL_EVIDENCE | 143 actual deal/recorded-level pairs have checked signed and absolute deviations. Source code shows closest-level inference and local fallback; these are not acknowledgement, accepted-level or trigger receipts. Remaining: Actual send/accept/amend/cancel/fill/restart sequences and frozen-policy shadow decision parity. |
| G9 | split/overlap and trial-budget audit passes before opening evaluation. | PASSED | Owner-authorised reservation-only state transition preserves the candidate, registers 160 exact native-window jobs, allows only verified causal preparation reuse, and enforces unseen local evaluation intervals and a one-candidate/one-trial ceiling. Local planning refusal count is zero. Missing preparation still blocks the tick stage. |
| G10 | net expectancy, payoff/tail metrics, controls, per-class requirements and predeclared uncertainty criteria pass; failures remain published. | BLOCKED_FINANCIAL_EVIDENCE | Receipt-linked trade evaluation separates classes/accounts, requires resolution, predeclared controls, tail and block uncertainty criteria; no aggregate win constants are accepted. Remaining: Independent qualified outcomes under verified costs and predeclared quantitative criteria. The frozen pilot explicitly cannot grant financial promotion. |
| G11 | shadow/canary execution and money reconcile within registered tolerances. | BLOCKED_EXTERNAL_EVIDENCE | Shadow/canary receipt parser checks authorization, account/policy identity, sends, acknowledgements, partial fills, restart book, decision divergence/latency and money. Remaining: Actual shadow/canary decision and broker receipts within registered tolerances; no orders or canary launched here. |
| G12 | owner risk rules, monitoring, rollback and per-strategy/account scorecards work under a complete trading lifecycle. | BLOCKED_EXTERNAL_EVIDENCE | Owner policy guard retains one minimum live position,250% entry margin floor, risk/protection/rollback checks and separate latest20 scorecards per account/strategy. Remaining: Complete observed lifecycle evidence for monitoring/protection/rollback and qualified per-account/strategy outcomes. CI and a synthetic operational checklist cannot prove this. |

G6's pass is deliberately about the 260 observed account positions. Missing historical contract authority belongs to G5 and missing full expected-cost estimation to G7; they do not invalidate actual reconciled statement arithmetic. G9 passes the local pre-evaluation split/trial audit, while data capture, path qualification and financial tests remain separate barriers. CI success is not a financial pass.

## §9. Verification, failure prevention and next bounded step

Six focused regression tests cover money/quantity versus full-cost distinctions; signed exit-level joins; held-out contamination and account separation; proxy calendar boundaries; spent-gap versus verified-prep reuse; and registered-transition/immutable-job/trial protection. The complete research suite passes 105 tests with one existing skip. Frozen Phase4 metadata and engine receipts are checked, and eligible local market source hashes are rechecked after the run. Analysis processes enforce an inherited Linux seccomp network denial as well as Python socket denial. No broker call, order, new backtest result, protected-date read or live-rule edit occurs here.

Reproduce with the canonical offline CLI: `python3 ops/research/rebuild.py cost-reservation`. It requires the retained local receipt files and archived native executable; absence is a refusal, never a fabricated replacement. It calculates the forensic ledger, verifies the reservation and renders this report without invoking collection or a new strategy arm.

The next steps are finite and evidence-dependent:

- Review this cost decomposition and reservation. Do not lower live costs or declare the historical proxy to be broker-certified.
- If price collection is later authorised, activate a reviewed collection state for this exact reservation; collect/decode all missing M30 preparation before ticks. Fail closed on any missing/spent or holiday-conflicting dependency.
- Collect all reserved tick windows as one frozen batch, then verify boundaries, side ages, ambiguity and full holding coverage before replay. Do not replace failed dates after looking at returns.
- Capture accepted bracket/amendment/trigger context and full causal execution benchmarks through the bounded evidence path before promoting exit-level proxies into a cost model or claiming G8 parity.
- Run one unchanged candidate against its matched controls, report all failures and account/class uncertainty, and require financial/shadow/lifecycle gates before any live migration. More trades or a lower assumed charge is not proof of an edge.

## §10. Audit references and account/symbol appendix

Primary sources are the retained authenticated broker deal/symbol receipts and statement matches from PR #198, the Phase1 checkpoint manifest, the 544-definition/616-M30 unified inventory, the Phase4 frozen candidate and the spent registry. Result SHA256: `bb2e74111247be087d1cc224769c6d35d72d3c26f19b4fe82c617d4a7b7d5c3f`. Frozen analysis plan: [JSON](../docs/rebuild_rfc_260deals_analysis_plan_18372.json). Full ledger, all component observations/predictions, exact reservation and source hashes: [result JSON](../docs/rebuild_rfc_260deals_and_clean_reservation_18372.json).

| Input | SHA256 |
| --- | --- |
| docs/rebuild_phase1_unified_replay_manifest_18355.json | 622e0d1d3ab3a718d6ecc94bd0de342089d22f66beaf73b0858e30842a6e5a29 |
| docs/rebuild_phase4_forensic_audit_and_hypothesis_freeze_18362.json | 2581b38797f917f78e7b09a4716fa38104cf12ffda2bf20a18416c8c9d05275a |
| docs/rebuild_rfc_gates_5_to_9_audit_18370.json | e37b00490dd05cf1caf530d8675211a81ed4316a3633716445709e7824a42522 |
| docs/rebuild_rfc_gates_5_to_9_evidence_18370.json | b84058c8c2af8a634c5d3dd385ec223c074bdb9a430ac2418d5df96afa76c878 |
| docs/rebuild_rfc_offline_gates_completion_18368.json | 31d6f174a0fe9821c9f83e4e91cc55acead8de2b958ee6ee3e2d3caf3b99c673 |
| docs/rebuild_rfc_registry_18368.json | 6bd36a36f3f4150d10819781039ce402682355e6b88c4858e4a1fca3e26152e5 |

### Matched260 account × symbol

| Account / symbol | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / AUDUSD | 1 | 1 | 100.00% | 0.14 | -0.08 | 0.00 | 0.06 | 0.08 | 1.1385 | 2026-10-27T09:07:54.883000Z → 2026-10-27T20:00:00.401000Z |
| 43002148 / AUS200 | 2 | 0 | 0.00% | -2.40 | 0.00 | 0.00 | -2.40 | 0.00 | 0.0000 | 2026-10-26T00:05:45.746000Z → 2026-10-27T01:32:44.545000Z |
| 43002148 / BTCUSD | 5 | 1 | 20.00% | -150.65 | 0.00 | -5.20 | -155.85 | 5.20 | 0.0000 | 2026-10-29T21:30:04.969000Z → 2026-11-04T03:01:14.042000Z |
| 43002148 / ETHUSD | 15 | 0 | 0.00% | -1.49 | 0.00 | 0.00 | -1.49 | 0.00 | 0.0000 | 2026-10-25T08:33:49.142000Z → 2026-10-29T02:49:48.246000Z |
| 43002148 / NatGas | 1 | 0 | 0.00% | -98.80 | 0.00 | 0.00 | -98.80 | 0.00 | 0.0000 | 2026-10-29T23:37:46.127000Z → 2026-10-30T13:09:54.569000Z |
| 43002148 / SpotCrude | 1 | 0 | 0.00% | -0.37 | 0.00 | 0.00 | -0.37 | 0.00 | 0.0000 | 2026-10-27T07:50:41.197000Z → 2026-10-27T09:07:12.776000Z |
| 43002148 / USDJPY | 1 | 1 | 100.00% | 2.76 | -0.08 | 0.00 | 2.68 | 0.08 | 0.8022 | 2026-10-26T07:30:04.468000Z → 2026-10-26T20:00:00.802000Z |
| 46130058 / AUDUSD | 4 | 1 | 25.00% | -388.93 | 0.00 | 0.00 | -388.93 | 0.00 | 0.0000 | 2026-10-26T09:17:58.514000Z → 2026-10-27T20:00:03.442000Z |
| 46130058 / AUS200 | 5 | 2 | 40.00% | 83.75 | 0.00 | 0.00 | 83.75 | 0.00 | 0.0000 | 2026-10-25T23:30:03.751000Z → 2026-10-27T05:00:01.090000Z |
| 46130058 / BTCUSD | 14 | 1 | 7.14% | -1,777.11 | 0.00 | 0.88 | -1,776.23 | -0.88 | 0.0000 | 2026-10-25T09:12:34.348000Z → 2026-11-03T00:00:01.055000Z |
| 46130058 / ETHUSD | 17 | 0 | 0.00% | -2,527.78 | 0.00 | 0.00 | -2,527.78 | 0.00 | 0.0000 | 2026-10-25T08:33:48.930000Z → 2026-10-29T02:49:48.135000Z |
| 46130058 / EURUSD | 3 | 1 | 33.33% | -258.42 | 0.00 | 0.00 | -258.42 | 0.00 | 0.0000 | 2026-10-26T07:30:17.908000Z → 2026-10-27T11:17:37.895000Z |
| 46130058 / GBPUSD | 2 | 1 | 50.00% | 230.10 | 0.00 | 0.00 | 230.10 | 0.00 | 0.0000 | 2026-10-26T07:30:10.423000Z → 2026-10-27T11:17:38.387000Z |
| 46130058 / GER40 | 2 | 2 | 100.00% | 286.71 | 0.00 | 0.00 | 286.71 | 0.00 | 0.0000 | 2026-10-26T10:49:17.785000Z → 2026-10-27T16:30:01.620000Z |
| 46130058 / JPN225 | 9 | 0 | 0.00% | -1,274.52 | 0.00 | 0.00 | -1,274.52 | 0.00 | 0.0000 | 2026-10-26T00:30:05.480000Z → 2026-10-29T03:37:44.359000Z |
| 46130058 / NAS100 | 2 | 0 | 0.00% | -40.38 | 0.00 | 0.00 | -40.38 | 0.00 | 0.0000 | 2026-10-26T16:31:54.777000Z → 2026-10-27T20:00:02.087000Z |
| 46130058 / NZDUSD | 1 | 1 | 100.00% | 62.70 | 0.00 | 0.00 | 62.70 | 0.00 | 0.0000 | 2026-10-26T10:27:44.961000Z → 2026-10-26T20:00:01.889000Z |
| 46130058 / NatGas | 2 | 0 | 0.00% | -312.00 | 0.00 | 0.00 | -312.00 | 0.00 | 0.0000 | 2026-10-27T07:51:55.458000Z → 2026-10-27T14:44:26.565000Z |
| 46130058 / SpotBrent | 5 | 0 | 0.00% | -647.67 | 0.00 | 6.55 | -641.12 | -6.55 | 0.0000 | 2026-10-27T07:57:14.289000Z → 2026-11-03T13:29:49.889000Z |
| 46130058 / SpotCrude | 4 | 0 | 0.00% | -509.11 | 0.00 | 0.00 | -509.11 | 0.00 | 0.0000 | 2026-10-27T07:46:34.707000Z → 2026-11-03T20:00:00.873000Z |
| 46130058 / USDCAD | 2 | 0 | 0.00% | -328.51 | 0.00 | 0.00 | -328.51 | 0.00 | 0.0000 | 2026-10-26T07:30:04.398000Z → 2026-10-27T07:42:16.521000Z |
| 46130058 / USDJPY | 2 | 1 | 50.00% | 1,213.09 | 0.00 | 0.00 | 1,213.09 | 0.00 | 0.0000 | 2026-10-26T07:30:14.145000Z → 2026-10-27T07:33:51.148000Z |
| 46130058 / XAGUSD | 7 | 2 | 28.57% | 228.95 | 0.00 | 0.00 | 228.95 | 0.00 | 0.0000 | 2026-10-26T07:30:23.678000Z → 2026-10-28T13:27:03.766000Z |
| 46130058 / XAUUSD | 4 | 1 | 25.00% | -151.27 | 0.00 | 0.00 | -151.27 | 0.00 | 0.0000 | 2026-10-26T08:03:01.261000Z → 2026-10-27T20:00:04.024000Z |
| 47790949 / AUDUSD | 1 | 0 | 0.00% | -80.00 | -11.20 | 0.00 | -91.20 | 11.20 | 0.9953 | 2026-10-27T11:24:29.853000Z → 2026-10-27T20:00:02.936000Z |
| 47790949 / AUS200 | 5 | 2 | 40.00% | 98.02 | 0.00 | 0.00 | 98.02 | 0.00 | 0.0000 | 2026-10-25T23:30:03.829000Z → 2026-10-27T05:00:01.129000Z |
| 47790949 / BTCUSD | 13 | 0 | 0.00% | -2,017.82 | 0.00 | 0.00 | -2,017.82 | 0.00 | 0.0000 | 2026-10-25T09:12:34.281000Z → 2026-11-02T13:38:12.736000Z |
| 47790949 / ETHUSD | 16 | 0 | 0.00% | -2,576.41 | 0.00 | 0.00 | -2,576.41 | 0.00 | 0.0000 | 2026-10-25T08:33:48.788000Z → 2026-10-28T12:02:51.314000Z |
| 47790949 / EURUSD | 1 | 1 | 100.00% | 147.90 | -5.96 | 0.00 | 141.94 | 5.96 | 0.6263 | 2026-10-26T11:19:47.361000Z → 2026-10-26T20:00:01.277000Z |
| 47790949 / GBPUSD | 2 | 1 | 50.00% | 81.92 | -21.70 | 0.00 | 60.22 | 21.70 | 0.5249 | 2026-10-26T08:10:41.942000Z → 2026-10-27T11:23:44.006000Z |
| 47790949 / GER40 | 2 | 2 | 100.00% | 509.37 | 0.00 | 0.00 | 509.37 | 0.00 | 0.0000 | 2026-10-26T10:55:18.928000Z → 2026-10-27T16:30:01.765000Z |
| 47790949 / JPN225 | 8 | 0 | 0.00% | -1,293.40 | 0.00 | 0.00 | -1,293.40 | 0.00 | 0.0000 | 2026-10-26T00:30:05.499000Z → 2026-10-28T05:24:12.361000Z |
| 47790949 / NAS100 | 1 | 0 | 0.00% | -3.20 | 0.00 | 0.00 | -3.20 | 0.00 | 0.0000 | 2026-10-27T17:00:01.469000Z → 2026-10-27T20:00:01.857000Z |
| 47790949 / NZDUSD | 2 | 1 | 50.00% | 26.22 | -34.58 | 0.00 | -8.36 | 34.58 | 1.2289 | 2026-10-26T08:01:42.951000Z → 2026-10-27T14:01:16.686000Z |
| 47790949 / NatGas | 7 | 0 | 0.00% | -2,176.40 | 0.00 | -14.10 | -2,190.50 | 14.10 | 0.0000 | 2026-10-26T07:31:20.296000Z → 2026-10-29T14:30:06.013000Z |
| 47790949 / SpotBrent | 4 | 1 | 25.00% | -322.89 | 0.00 | 0.00 | -322.89 | 0.00 | 0.0000 | 2026-10-27T07:46:23.045000Z → 2026-11-03T13:27:27.882000Z |
| 47790949 / SpotCrude | 5 | 0 | 0.00% | -579.34 | 0.00 | 0.00 | -579.34 | 0.00 | 0.0000 | 2026-10-26T07:31:06.684000Z → 2026-11-03T20:00:00.728000Z |
| 47790949 / US500 | 1 | 0 | 0.00% | -164.22 | 0.00 | 0.00 | -164.22 | 0.00 | 0.0000 | 2026-10-27T14:04:02.624000Z → 2026-10-27T14:30:22.874000Z |
| 47790949 / USDCAD | 1 | 0 | 0.00% | -178.75 | -32.48 | 0.00 | -211.23 | 32.48 | 0.6997 | 2026-10-26T07:30:04.385000Z → 2026-10-26T08:00:23.606000Z |
| 47790949 / USDJPY | 3 | 1 | 33.33% | -293.61 | -58.76 | 0.00 | -352.37 | 58.76 | 0.7010 | 2026-10-26T13:33:16.149000Z → 2026-10-27T12:34:05.088000Z |
| 47790949 / XAGUSD | 6 | 2 | 33.33% | 189.50 | 0.00 | 0.00 | 189.50 | 0.00 | 0.0000 | 2026-10-26T07:30:21.935000Z → 2026-10-27T20:00:02.414000Z |
| 47790949 / XAUUSD | 3 | 1 | 33.33% | 10.89 | -4.56 | 0.00 | 6.33 | 4.56 | 0.1753 | 2026-10-26T08:01:00.475000Z → 2026-10-27T20:00:01.270000Z |
| 49236051 / AUDUSD | 4 | 0 | 0.00% | -18.26 | -2.82 | 0.00 | -21.08 | 2.82 | 1.0042 | 2026-10-26T09:28:03.740000Z → 2026-10-27T20:00:02.257000Z |
| 49236051 / AUS200 | 5 | 2 | 40.00% | 1.95 | 0.00 | 0.00 | 1.95 | 0.00 | 0.0000 | 2026-10-25T23:30:03.913000Z → 2026-10-27T05:00:01Z |
| 49236051 / BTCUSD | 10 | 0 | 0.00% | -44.51 | 0.00 | 0.00 | -44.51 | 0.00 | 0.0000 | 2026-10-25T09:12:34.155000Z → 2026-10-28T12:05:13.948000Z |
| 49236051 / ETHUSD | 16 | 0 | 0.00% | -80.67 | 0.00 | 0.00 | -80.67 | 0.00 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-10-29T14:30:55.896000Z |
| 49236051 / EURUSD | 3 | 1 | 33.33% | -3.94 | -1.00 | 0.00 | -4.94 | 1.00 | 0.6373 | 2026-10-26T07:30:17.633000Z → 2026-10-27T12:10:53.215000Z |
| 49236051 / GBPUSD | 2 | 1 | 50.00% | 9.00 | -0.84 | 0.00 | 8.16 | 0.84 | 0.5248 | 2026-10-26T07:30:10.195000Z → 2026-10-27T11:23:43.929000Z |
| 49236051 / NZDUSD | 3 | 1 | 33.33% | -4.55 | -1.76 | 0.00 | -6.31 | 1.76 | 1.2308 | 2026-10-26T08:03:02.815000Z → 2026-10-27T14:00:03.538000Z |
| 49236051 / NatGas | 6 | 0 | 0.00% | -23.90 | 0.00 | 0.00 | -23.90 | 0.00 | 0.0000 | 2026-10-26T10:28:20.565000Z → 2026-10-27T19:16:47.474000Z |
| 49236051 / SpotBrent | 5 | 0 | 0.00% | -20.70 | 0.00 | 0.00 | -20.70 | 0.00 | 0.0000 | 2026-10-26T13:58:14.656000Z → 2026-10-28T10:07:20.356000Z |
| 49236051 / SpotCrude | 5 | 1 | 20.00% | -16.36 | 0.00 | 0.00 | -16.36 | 0.00 | 0.0000 | 2026-10-27T07:46:34.608000Z → 2026-10-28T19:59:59.127000Z |
| 49236051 / UK100 | 1 | 0 | 0.00% | -3.25 | 0.00 | 0.00 | -3.25 | 0.00 | 0.0000 | 2026-10-27T08:58:35.780000Z → 2026-10-27T10:39:32.826000Z |
| 49236051 / US500 | 2 | 1 | 50.00% | -1.49 | 0.00 | 0.00 | -1.49 | 0.00 | 0.0000 | 2026-10-26T14:13:54.452000Z → 2026-10-27T14:30:22.848000Z |
| 49236051 / USDCAD | 1 | 0 | 0.00% | -5.39 | -0.98 | 0.00 | -6.37 | 0.98 | 0.6997 | 2026-10-26T07:30:04.399000Z → 2026-10-26T08:00:23.628000Z |
| 49236051 / USDJPY | 3 | 1 | 33.33% | 38.55 | -2.88 | 0.00 | 35.67 | 2.88 | 0.7019 | 2026-10-26T07:30:13.943000Z → 2026-10-27T12:34:05.127000Z |
| 49236051 / XAGUSD | 1 | 0 | 0.00% | -5.60 | 0.00 | 0.00 | -5.60 | 0.00 | 0.0000 | 2026-10-26T07:30:23.374000Z → 2026-10-26T10:44:52.374000Z |
| 49236051 / XAUUSD | 1 | 0 | 0.00% | -3.73 | -0.08 | 0.00 | -3.81 | 0.08 | 0.1998 | 2026-10-27T07:30:29.832000Z → 2026-10-27T07:55:51.915000Z |

### All271 account × symbol

| Account / symbol | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / AUDUSD | 1 | 1 | 100.00% | 0.14 | -0.08 | 0.00 | 0.06 | 0.08 | 1.1385 | 2026-10-27T09:07:54.883000Z → 2026-10-27T20:00:00.401000Z |
| 43002148 / AUS200 | 2 | 0 | 0.00% | -2.40 | 0.00 | 0.00 | -2.40 | 0.00 | 0.0000 | 2026-10-26T00:05:45.746000Z → 2026-10-27T01:32:44.545000Z |
| 43002148 / BTCUSD | 5 | 1 | 20.00% | -150.65 | 0.00 | -5.20 | -155.85 | 5.20 | 0.0000 | 2026-10-29T21:30:04.969000Z → 2026-11-04T03:01:14.042000Z |
| 43002148 / ETHUSD | 15 | 0 | 0.00% | -1.49 | 0.00 | 0.00 | -1.49 | 0.00 | 0.0000 | 2026-10-25T08:33:49.142000Z → 2026-10-29T02:49:48.246000Z |
| 43002148 / JPN225 | 3 | 0 | 0.00% | -564.37 | 0.00 | 0.00 | -564.37 | 0.00 | 0.0000 | 2026-11-06T02:47:00.853000Z → 2026-11-06T03:59:06.262000Z |
| 43002148 / NatGas | 1 | 0 | 0.00% | -98.80 | 0.00 | 0.00 | -98.80 | 0.00 | 0.0000 | 2026-10-29T23:37:46.127000Z → 2026-10-30T13:09:54.569000Z |
| 43002148 / SpotCrude | 1 | 0 | 0.00% | -0.37 | 0.00 | 0.00 | -0.37 | 0.00 | 0.0000 | 2026-10-27T07:50:41.197000Z → 2026-10-27T09:07:12.776000Z |
| 43002148 / USDJPY | 1 | 1 | 100.00% | 2.76 | -0.08 | 0.00 | 2.68 | 0.08 | 0.8022 | 2026-10-26T07:30:04.468000Z → 2026-10-26T20:00:00.802000Z |
| 46130058 / AUDUSD | 4 | 1 | 25.00% | -388.93 | 0.00 | 0.00 | -388.93 | 0.00 | 0.0000 | 2026-10-26T09:17:58.514000Z → 2026-10-27T20:00:03.442000Z |
| 46130058 / AUS200 | 5 | 2 | 40.00% | 83.75 | 0.00 | 0.00 | 83.75 | 0.00 | 0.0000 | 2026-10-25T23:30:03.751000Z → 2026-10-27T05:00:01.090000Z |
| 46130058 / BTCUSD | 14 | 1 | 7.14% | -1,777.11 | 0.00 | 0.88 | -1,776.23 | -0.88 | 0.0000 | 2026-10-25T09:12:34.348000Z → 2026-11-03T00:00:01.055000Z |
| 46130058 / ETHUSD | 17 | 0 | 0.00% | -2,527.78 | 0.00 | 0.00 | -2,527.78 | 0.00 | 0.0000 | 2026-10-25T08:33:48.930000Z → 2026-10-29T02:49:48.135000Z |
| 46130058 / EURUSD | 3 | 1 | 33.33% | -258.42 | 0.00 | 0.00 | -258.42 | 0.00 | 0.0000 | 2026-10-26T07:30:17.908000Z → 2026-10-27T11:17:37.895000Z |
| 46130058 / GBPUSD | 2 | 1 | 50.00% | 230.10 | 0.00 | 0.00 | 230.10 | 0.00 | 0.0000 | 2026-10-26T07:30:10.423000Z → 2026-10-27T11:17:38.387000Z |
| 46130058 / GER40 | 3 | 2 | 66.67% | 175.12 | 0.00 | -0.80 | 174.32 | 0.80 | 0.0000 | 2026-10-26T10:49:17.785000Z → 2026-11-06T14:12:49.340000Z |
| 46130058 / JPN225 | 9 | 0 | 0.00% | -1,274.52 | 0.00 | 0.00 | -1,274.52 | 0.00 | 0.0000 | 2026-10-26T00:30:05.480000Z → 2026-10-29T03:37:44.359000Z |
| 46130058 / NAS100 | 2 | 0 | 0.00% | -40.38 | 0.00 | 0.00 | -40.38 | 0.00 | 0.0000 | 2026-10-26T16:31:54.777000Z → 2026-10-27T20:00:02.087000Z |
| 46130058 / NZDUSD | 1 | 1 | 100.00% | 62.70 | 0.00 | 0.00 | 62.70 | 0.00 | 0.0000 | 2026-10-26T10:27:44.961000Z → 2026-10-26T20:00:01.889000Z |
| 46130058 / NatGas | 3 | 0 | 0.00% | -318.40 | 0.00 | 0.00 | -318.40 | 0.00 | 0.0000 | 2026-10-27T07:51:55.458000Z → 2026-11-04T19:13:44.466000Z |
| 46130058 / SpotBrent | 5 | 0 | 0.00% | -647.67 | 0.00 | 6.55 | -641.12 | -6.55 | 0.0000 | 2026-10-27T07:57:14.289000Z → 2026-11-03T13:29:49.889000Z |
| 46130058 / SpotCrude | 4 | 0 | 0.00% | -509.11 | 0.00 | 0.00 | -509.11 | 0.00 | 0.0000 | 2026-10-27T07:46:34.707000Z → 2026-11-03T20:00:00.873000Z |
| 46130058 / USDCAD | 2 | 0 | 0.00% | -328.51 | 0.00 | 0.00 | -328.51 | 0.00 | 0.0000 | 2026-10-26T07:30:04.398000Z → 2026-10-27T07:42:16.521000Z |
| 46130058 / USDJPY | 2 | 1 | 50.00% | 1,213.09 | 0.00 | 0.00 | 1,213.09 | 0.00 | 0.0000 | 2026-10-26T07:30:14.145000Z → 2026-10-27T07:33:51.148000Z |
| 46130058 / XAGUSD | 7 | 2 | 28.57% | 228.95 | 0.00 | 0.00 | 228.95 | 0.00 | 0.0000 | 2026-10-26T07:30:23.678000Z → 2026-10-28T13:27:03.766000Z |
| 46130058 / XAUUSD | 4 | 1 | 25.00% | -151.27 | 0.00 | 0.00 | -151.27 | 0.00 | 0.0000 | 2026-10-26T08:03:01.261000Z → 2026-10-27T20:00:04.024000Z |
| 47790949 / AUDUSD | 1 | 0 | 0.00% | -80.00 | -11.20 | 0.00 | -91.20 | 11.20 | 0.9953 | 2026-10-27T11:24:29.853000Z → 2026-10-27T20:00:02.936000Z |
| 47790949 / AUS200 | 5 | 2 | 40.00% | 98.02 | 0.00 | 0.00 | 98.02 | 0.00 | 0.0000 | 2026-10-25T23:30:03.829000Z → 2026-10-27T05:00:01.129000Z |
| 47790949 / BTCUSD | 13 | 0 | 0.00% | -2,017.82 | 0.00 | 0.00 | -2,017.82 | 0.00 | 0.0000 | 2026-10-25T09:12:34.281000Z → 2026-11-02T13:38:12.736000Z |
| 47790949 / ETHUSD | 16 | 0 | 0.00% | -2,576.41 | 0.00 | 0.00 | -2,576.41 | 0.00 | 0.0000 | 2026-10-25T08:33:48.788000Z → 2026-10-28T12:02:51.314000Z |
| 47790949 / EURUSD | 1 | 1 | 100.00% | 147.90 | -5.96 | 0.00 | 141.94 | 5.96 | 0.6263 | 2026-10-26T11:19:47.361000Z → 2026-10-26T20:00:01.277000Z |
| 47790949 / GBPUSD | 2 | 1 | 50.00% | 81.92 | -21.70 | 0.00 | 60.22 | 21.70 | 0.5249 | 2026-10-26T08:10:41.942000Z → 2026-10-27T11:23:44.006000Z |
| 47790949 / GER40 | 3 | 2 | 66.67% | 397.55 | 0.00 | -0.80 | 396.75 | 0.80 | 0.0000 | 2026-10-26T10:55:18.928000Z → 2026-11-06T14:12:49.366000Z |
| 47790949 / JPN225 | 8 | 0 | 0.00% | -1,293.40 | 0.00 | 0.00 | -1,293.40 | 0.00 | 0.0000 | 2026-10-26T00:30:05.499000Z → 2026-10-28T05:24:12.361000Z |
| 47790949 / NAS100 | 1 | 0 | 0.00% | -3.20 | 0.00 | 0.00 | -3.20 | 0.00 | 0.0000 | 2026-10-27T17:00:01.469000Z → 2026-10-27T20:00:01.857000Z |
| 47790949 / NZDUSD | 2 | 1 | 50.00% | 26.22 | -34.58 | 0.00 | -8.36 | 34.58 | 1.2289 | 2026-10-26T08:01:42.951000Z → 2026-10-27T14:01:16.686000Z |
| 47790949 / NatGas | 8 | 0 | 0.00% | -2,179.80 | 0.00 | -14.10 | -2,193.90 | 14.10 | 0.0000 | 2026-10-26T07:31:20.296000Z → 2026-11-04T19:13:44.481000Z |
| 47790949 / SpotBrent | 4 | 1 | 25.00% | -322.89 | 0.00 | 0.00 | -322.89 | 0.00 | 0.0000 | 2026-10-27T07:46:23.045000Z → 2026-11-03T13:27:27.882000Z |
| 47790949 / SpotCrude | 5 | 0 | 0.00% | -579.34 | 0.00 | 0.00 | -579.34 | 0.00 | 0.0000 | 2026-10-26T07:31:06.684000Z → 2026-11-03T20:00:00.728000Z |
| 47790949 / US500 | 1 | 0 | 0.00% | -164.22 | 0.00 | 0.00 | -164.22 | 0.00 | 0.0000 | 2026-10-27T14:04:02.624000Z → 2026-10-27T14:30:22.874000Z |
| 47790949 / USDCAD | 1 | 0 | 0.00% | -178.75 | -32.48 | 0.00 | -211.23 | 32.48 | 0.6997 | 2026-10-26T07:30:04.385000Z → 2026-10-26T08:00:23.606000Z |
| 47790949 / USDJPY | 3 | 1 | 33.33% | -293.61 | -58.76 | 0.00 | -352.37 | 58.76 | 0.7010 | 2026-10-26T13:33:16.149000Z → 2026-10-27T12:34:05.088000Z |
| 47790949 / XAGUSD | 6 | 2 | 33.33% | 189.50 | 0.00 | 0.00 | 189.50 | 0.00 | 0.0000 | 2026-10-26T07:30:21.935000Z → 2026-10-27T20:00:02.414000Z |
| 47790949 / XAUUSD | 3 | 1 | 33.33% | 10.89 | -4.56 | 0.00 | 6.33 | 4.56 | 0.1753 | 2026-10-26T08:01:00.475000Z → 2026-10-27T20:00:01.270000Z |
| 49236051 / AUDUSD | 4 | 0 | 0.00% | -18.26 | -2.82 | 0.00 | -21.08 | 2.82 | 1.0042 | 2026-10-26T09:28:03.740000Z → 2026-10-27T20:00:02.257000Z |
| 49236051 / AUS200 | 5 | 2 | 40.00% | 1.95 | 0.00 | 0.00 | 1.95 | 0.00 | 0.0000 | 2026-10-25T23:30:03.913000Z → 2026-10-27T05:00:01Z |
| 49236051 / BTCUSD | 10 | 0 | 0.00% | -44.51 | 0.00 | 0.00 | -44.51 | 0.00 | 0.0000 | 2026-10-25T09:12:34.155000Z → 2026-10-28T12:05:13.948000Z |
| 49236051 / ETHUSD | 17 | 1 | 5.88% | -69.17 | 0.00 | 0.00 | -69.17 | 0.00 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-11-05T15:02:00.355000Z |
| 49236051 / EURUSD | 3 | 1 | 33.33% | -3.94 | -1.00 | 0.00 | -4.94 | 1.00 | 0.6373 | 2026-10-26T07:30:17.633000Z → 2026-10-27T12:10:53.215000Z |
| 49236051 / GBPUSD | 2 | 1 | 50.00% | 9.00 | -0.84 | 0.00 | 8.16 | 0.84 | 0.5248 | 2026-10-26T07:30:10.195000Z → 2026-10-27T11:23:43.929000Z |
| 49236051 / NZDUSD | 3 | 1 | 33.33% | -4.55 | -1.76 | 0.00 | -6.31 | 1.76 | 1.2308 | 2026-10-26T08:03:02.815000Z → 2026-10-27T14:00:03.538000Z |
| 49236051 / NatGas | 7 | 1 | 14.29% | -23.10 | 0.00 | 0.00 | -23.10 | 0.00 | 0.0000 | 2026-10-26T10:28:20.565000Z → 2026-11-04T18:44:52.698000Z |
| 49236051 / SpotBrent | 6 | 0 | 0.00% | -24.65 | 0.00 | 0.00 | -24.65 | 0.00 | 0.0000 | 2026-10-26T13:58:14.656000Z → 2026-11-03T10:22:06.689000Z |
| 49236051 / SpotCrude | 6 | 1 | 16.67% | -20.84 | 0.00 | 0.00 | -20.84 | 0.00 | 0.0000 | 2026-10-27T07:46:34.608000Z → 2026-11-03T14:06:48.366000Z |
| 49236051 / UK100 | 1 | 0 | 0.00% | -3.25 | 0.00 | 0.00 | -3.25 | 0.00 | 0.0000 | 2026-10-27T08:58:35.780000Z → 2026-10-27T10:39:32.826000Z |
| 49236051 / US500 | 2 | 1 | 50.00% | -1.49 | 0.00 | 0.00 | -1.49 | 0.00 | 0.0000 | 2026-10-26T14:13:54.452000Z → 2026-10-27T14:30:22.848000Z |
| 49236051 / USDCAD | 1 | 0 | 0.00% | -5.39 | -0.98 | 0.00 | -6.37 | 0.98 | 0.6997 | 2026-10-26T07:30:04.399000Z → 2026-10-26T08:00:23.628000Z |
| 49236051 / USDJPY | 3 | 1 | 33.33% | 38.55 | -2.88 | 0.00 | 35.67 | 2.88 | 0.7019 | 2026-10-26T07:30:13.943000Z → 2026-10-27T12:34:05.127000Z |
| 49236051 / XAGUSD | 1 | 0 | 0.00% | -5.60 | 0.00 | 0.00 | -5.60 | 0.00 | 0.0000 | 2026-10-26T07:30:23.374000Z → 2026-10-26T10:44:52.374000Z |
| 49236051 / XAUUSD | 1 | 0 | 0.00% | -3.73 | -0.08 | 0.00 | -3.81 | 0.08 | 0.1998 | 2026-10-27T07:30:29.832000Z → 2026-10-27T07:55:51.915000Z |

### Strategy labels within matched260

| Account / observed labels | Positions | Wins | Win rate | Gross USD | Commission USD | Swap USD | Net USD | Booked drag USD | Median booked drag bp | UTC entry → exit range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 / ['jiaoyi-s1', 'unlabelled'] | 1 | 0 | 0.00% | -98.80 | 0.00 | 0.00 | -98.80 | 0.00 | 0.0000 | 2026-10-29T23:37:46.127000Z → 2026-10-30T13:09:54.569000Z |
| 43002148 / ['jiaoyi-s3'] | 21 | 3 | 14.29% | -0.82 | -0.16 | 0.00 | -0.98 | 0.16 | 0.0000 | 2026-10-25T08:33:49.142000Z → 2026-10-29T22:24:36.655000Z |
| 43002148 / ['unlabelled'] | 4 | 0 | 0.00% | -151.19 | 0.00 | -5.20 | -156.39 | 5.20 | 1.9190 | 2026-10-30T18:52:23.294000Z → 2026-11-04T03:01:14.042000Z |
| 46130058 / ['jiaoyi-s1'] | 1 | 0 | 0.00% | -129.79 | 0.00 | 6.55 | -123.24 | -6.55 | -46.2916 | 2026-10-30T01:43:24.928000Z → 2026-11-03T13:29:49.889000Z |
| 46130058 / ['jiaoyi-s3'] | 84 | 13 | 15.48% | -5,980.61 | 0.00 | 0.88 | -5,979.73 | -0.88 | 0.0000 | 2026-10-25T08:33:48.930000Z → 2026-11-03T20:00:00.873000Z |
| 47790949 / ['jiaoyi-s3'] | 78 | 12 | 15.38% | -6,985.22 | -169.24 | 0.00 | -7,154.46 | 169.24 | 0.0000 | 2026-10-25T08:33:48.788000Z → 2026-11-03T20:00:00.728000Z |
| 47790949 / ['unlabelled'] | 3 | 0 | 0.00% | -1,637.00 | 0.00 | -14.10 | -1,651.10 | 14.10 | 0.0000 | 2026-10-28T00:29:28.114000Z → 2026-10-29T14:30:06.013000Z |
| 49236051 / ['jiaoyi-s3'] | 68 | 8 | 11.76% | -182.85 | -10.36 | 0.00 | -193.21 | 10.36 | 0.0000 | 2026-10-25T08:33:48.729000Z → 2026-10-29T14:30:55.896000Z |

The JSON contains the full 271-position ledger, not merely these aggregates. No ZIP, duplicate tick archive, new service or paid model route is introduced.
