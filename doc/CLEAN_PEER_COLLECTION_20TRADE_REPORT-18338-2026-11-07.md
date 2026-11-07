# Peer collection and bounded twenty-close milestone evaluation

№18,338 §§1–10 · 07-11-2026 20:20:39 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**Measured S3 two-stage: 0/1 net wins; mean -1.752799R; PF(R) 0.000000.** Twenty closes are a milestone, never an invented denominator. Account goal achieved: **not established**. The invariant is 73% / reference risk:reward1:1.38 on latest20 per strategy/account (15/20=75%). No qualified improvement towards that account goal is demonstrated by this development run.

New peer tick reads: **35/35 complete**, plus 5 cached BTC slots. All80 eight-symbol/date slots remain, including **25 missing primary sources** outside the40-peer collection budget. Native replays: 55. May2025 oil/US500/BTC dates were already studied; this is not a clean validation batch.

## §2. Corrections to the supplied script

The supplied collector invocation had an unsupported `--symbol` and omitted its required `--output`; that collector also refuses these historical dates. Failed commands became `simulated_ok`, and all trade counts, class rows and controls were hard-coded. Those values were not executed as evidence. Even the invented16/22 is72.727%, below73%, and it does not describe the latest20. Its own win/loss arithmetic gives a different mean/PF from its final block. A1.024R loss is not24bp/side cost accounting: actual cost/R depends on prices and originalR. This replacement uses native fills, retains failures and never authorises Stage2 from a fabricated PASS.

## §3. Measured models at24bp/side

| Model | Entries | Closed | Wins | Net win rate | Mean net R | PF(R) | Censored |
| --- | --- | --- | --- | --- | --- | --- | --- |
| s3_two_stage | 5 | 1 | 0 | 0.000000 | -1.752799 | 0.000000 | 4 |
| macro_180m_two_stage | 24 | 23 | 1 | 0.043478 | -0.453806 | 0.001261 | 1 |
| production_s3_baseline | 5 | 2 | 0 | 0.000000 | -1.154226 | 0.000000 | 3 |
| always_long_control | 23 | 22 | 8 | 0.363636 | -0.633653 | 0.170993 | 1 |
| own_momentum_fixed_horizon | 23 | 22 | 4 | 0.181818 | -0.984557 | 0.046018 | 1 |

### Per-class results

| Class | Model | Closed | Wins | Mean R | PF(R) |
| --- | --- | --- | --- | --- | --- |
| crypto | s3_two_stage | 1 | 0 | -1.752799 | 0.000000 |
| crypto | macro_180m_two_stage | 15 | 1 | -0.319373 | 0.002743 |
| energy | s3_two_stage | 0 | 0 | unavailable | unavailable |
| energy | macro_180m_two_stage | 0 | 0 | unavailable | unavailable |
| equity_index | s3_two_stage | 0 | 0 | unavailable | unavailable |
| equity_index | macro_180m_two_stage | 8 | 0 | -0.705869 | 0.000000 |
| metal | s3_two_stage | 0 | 0 | unavailable | unavailable |
| metal | macro_180m_two_stage | 0 | 0 | unavailable | unavailable |

## §4. Latest20 research windows

```json
{
  "s3_two_stage": {
    "available_closes": 1,
    "window_size": 1,
    "wins": 0,
    "win_rate": 0.0,
    "enough_closes": false,
    "numerical_73_percent_threshold_met": false,
    "account_goal_satisfied": false,
    "scope": "pooled research only; not account outcomes",
    "trades": [
      {
        "at": "2025-05-15T01:11:15.957Z",
        "key": "ETHUSD-2025-05-15",
        "net_r": -1.7527990521327026
      }
    ]
  },
  "macro_180m_two_stage": {
    "available_closes": 23,
    "window_size": 20,
    "wins": 1,
    "win_rate": 0.05,
    "enough_closes": true,
    "numerical_73_percent_threshold_met": false,
    "account_goal_satisfied": false,
    "scope": "pooled research only; not account outcomes",
    "trades": [
      {
        "at": "2024-05-14T04:00:00.636Z",
        "key": "BTCUSD-2024-05-14",
        "net_r": -0.30595847568988177
      },
      {
        "at": "2024-05-15T04:00:00.453Z",
        "key": "ETHUSD-2024-05-15",
        "net_r": -0.07827051282050412
      },
      {
        "at": "2024-05-16T04:00:00.274Z",
        "key": "BTCUSD-2024-05-16",
        "net_r": -0.44386815144766145
      },
      {
        "at": "2024-05-17T04:00:02.810Z",
        "key": "ETHUSD-2024-05-17",
        "net_r": -0.7424503253796015
      },
      {
        "at": "2025-05-12T17:30:00.105Z",
        "key": "NAS100-2025-05-12",
        "net_r": -0.7752221933246577
      },
      {
        "at": "2025-05-12T17:30:02.211Z",
        "key": "US500-2025-05-12",
        "net_r": -1.067899567099559
      },
      {
        "at": "2025-05-13T04:00:00.015Z",
        "key": "BTCUSD-2025-05-13",
        "net_r": -0.1377721020278963
      },
      {
        "at": "2025-05-13T04:00:00.262Z",
        "key": "ETHUSD-2025-05-13",
        "net_r": -0.006179019927311236
      },
      {
        "at": "2025-05-13T17:30:00.098Z",
        "key": "NAS100-2025-05-13",
        "net_r": -0.2621417812313188
      },
      {
        "at": "2025-05-13T17:30:00.099Z",
        "key": "US500-2025-05-13",
        "net_r": -0.38554316163411273
      },
      {
        "at": "2025-05-14T04:00:00.004Z",
        "key": "ETHUSD-2025-05-14",
        "net_r": -0.23878757763975225
      },
      {
        "at": "2025-05-14T04:00:00.111Z",
        "key": "BTCUSD-2025-05-14",
        "net_r": -0.605304475706878
      },
      {
        "at": "2025-05-14T17:30:00.041Z",
        "key": "NAS100-2025-05-14",
        "net_r": -1.3493299999999886
      },
      {
        "at": "2025-05-14T17:30:00.191Z",
        "key": "US500-2025-05-14",
        "net_r": -1.4113324999999888
      },
      {
        "at": "2025-05-15T04:00:00.198Z",
        "key": "BTCUSD-2025-05-15",
        "net_r": -0.2350930481692693
      },
      {
        "at": "2025-05-15T04:00:00.198Z",
        "key": "ETHUSD-2025-05-15",
        "net_r": 0.013178476238368347
      },
      {
        "at": "2025-05-15T17:30:00.111Z",
        "key": "NAS100-2025-05-15",
        "net_r": -0.12530313986680391
      },
      {
        "at": "2025-05-15T17:30:00.376Z",
        "key": "US500-2025-05-15",
        "net_r": -0.27017723577236874
      },
      {
        "at": "2025-05-16T04:00:00.087Z",
        "key": "BTCUSD-2025-05-16",
        "net_r": -0.6767763018056179
      },
      {
        "at": "2025-05-16T04:00:00.460Z",
        "key": "ETHUSD-2025-05-16",
        "net_r": -0.17664196141478808
      }
    ]
  }
}
```

## §5. Frozen rules and data limits

The owner-selected dates and40 peer slots are fixed before reads. Five cachedBTC slots and15 primary sources are reused by hash;35 new peer tick jobs are permitted. Missing25 primary sources are not silently added to the budget. Completed M30 warm-up is collected separately and bounded. Old oil ticks end11UTC; the locked native oil observation requires07–17UTC, so these sources remain partial and cannot supply afternoon macro execution. Missing bars/quotes block decisions and unclosed positions stay censored. No interpolation, duplicate interval collection, same-day refit or sampling until favourable results.

S3 retains the strict spread+commission+two-sided cost reserve<R at decision/dispatch;48bp alone is not equivalent. The existing exit engine retains−1R until executable+1R, acknowledges an entry-price stop on a strictly later valid quote, and targets gross+1.38R. Broker bracket priority/gaps, native session deadlines and censoring remain. Peer confirmation affects macro only. Authorised pairs are symmetric Crude/Brent, US500/NAS100, Gold/Silver and BTC/ETH; missing peer observations never pass confirmation. S3 exits are tick-replayed; macro signals use six completed M30 bars and causal quotes.

Primary accounting uses actual bid/ask fills plus the assumed24bp per side; zero additional commission remains uncalibrated. Gross-flat exits can be net losses. Historical contract changes, swaps, latency, rejections and account portfolio capacity remain unverified. Current broker-minimum contracts are not proof of historical contract terms. Trade results are R scenarios, not account cash/NLV outcomes.

## §6. Evidence and next step

The [frozen plan](../docs/peer-collection-plan-18338.json) and [measured JSON](../docs/clean_peer_collection_results_18338.json) retain all80 slots, source/replay hashes, per-symbol/class/cohort outcomes, reasons and receipts. Raw sources remain private in the existing workspace; no database data were added. The native executable and source hashes were frozen before reads. No trading-service strategy, risk, worker permission or paid model API changed. Validation19Aug–5Sep2025 stays sealed.

Next: inspect measured failures/censoring and per-class net expectancy. Any proposal to collect the25 missing primaries or choose genuinely unused evaluation dates needs a separately bounded manifest; this batch does not silently expand. Fewer than20 closes is an insufficient-sample result, not a reason to invent trades. No Stage2 expansion or financial PASS follows automatically.
