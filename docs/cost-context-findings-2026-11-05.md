# Cost calibration and class-specific predictive context

№18,260 §§1–6 · 05-11-2026, 21:09 SGT · by <Codex (GPT-6), effort: not exposed>.

## Decision

No asset class has qualified the interim 50% net-win objective. The original
73% / 1:1.38 per strategy/account objective remains unchanged. The work adds
a reproducible offline investigation, not a new live rule. Production retains
its existing execution/risk settings and 17-name universe. All original 20
names remain in the research report, including missing observations.

There are two distinct unresolved economic questions: what execution actually
costs, and whether information available before entry predicts a sufficiently
large subsequent move. Changing stops or buying more model reasoning cannot
substitute for answering those questions. A measured price forecast can be
correct while the trade loses after costs.

## Actual execution evidence

Eight refreshed newest-500 account/strategy order views contain 1,079 distinct
records and 262 opens with a recorded fill price. Only 12 have the required
final-dispatch quote and actual-deal attribution; all are demo entries. There
are none for the live account or FX. These are selected bounded views, not a
complete or unbiased execution cohort. Each retained attributed entry is checked
for direction, price arithmetic, quote age and decision/dispatch/fill decomposition.

The same exports contain 51 filled market-close records and 207 broker exits.
Their legacy slippage is against a decision quote or an inferred SL/TP level,
respectively. Those benchmarks cannot be pooled with final-dispatch entry
slippage. These views do not certify full paired entry/exit/commission lifecycles.
Therefore the calibrated round-trip allowance remains **unknown**, rather than
zero or the median of 12 demo entries. The inherited 24bp/side allowance remains
an unchanged stress assumption; this study does not establish its accuracy.

The next calibration gate requires actual deal VWAP and quantity, direction,
timestamped entry and exit benchmarks, actual fee lifecycle and account/size/
regime coverage. Use existing receipts where present; mark missing fields.
Do not collect redundant per-tick telemetry or infer a new lower live limit
from an unrepresentative demo subset. Historical 2025 costs also cannot be
certified by a few November 2026 receipts.

## Economic mechanisms and source limits

Primary sources were retrieved on 5 November 2026. They motivate questions;
none proves the intraday CFD hypotheses below.

| Class | Grounded distinction | Bounded test and missing evidence |
|---|---|---|
| FX | [Lustig, Roussanov and Verdelhan, Common Risk Factors in Currency Markets](https://www.nber.org/papers/w14082) identifies common risk exposure and a carry-related slope factor. | Test an equal-weight USD peer return, excluding the target. This is a constructed price-context hypothesis, **not** a replication of the paper or evidence of an intraday lead. Interest rates, positioning and timestamped macro surprises are missing. |
| Crypto | [Liu, Tsyvinski and Wu, Common Risk Factors in Cryptocurrency](https://www.nber.org/papers/w25882) identifies cryptocurrency market, size and momentum factors. | Test the other cryptocurrency's first-hour return. Two CFDs cannot reproduce the paper's cross-sectional model. Funding, spot/futures basis and historical attention are missing. |
| Oil | [EIA: crude spot prices](https://www.eia.gov/finance/markets/crudeoil/spot_prices.php) explains common global pricing, persistent grade differences and supply/demand disruptions. | Test Brent/WTI peer returns. Co-movement does not establish a lead. Inventory expectations, supply news, curve and CFD roll/basis remain absent. |
| NatGas | [EIA: factors affecting natural-gas prices](https://www.eia.gov/energyexplained/natural-gas/factors-affecting-natural-gas-prices.php) identifies production, storage, imports/exports, weather and demand. | No substitution of crude oil for a gas predictor. Historical as-published weather forecasts, storage expectations and revisions are needed. Its missing predictor remains visible. |
| Metals | [World Gold Council: relevance of gold](https://www.gold.org/goldhub/research/relevance-of-gold-as-a-strategic-asset) discusses diverse demand and long-term portfolio roles. It is an industry source, not proof of an intraday trade. | Inverse USD return is an explicitly unproven proxy. The cited page does not establish that sign/horizon; real yields, policy surprises and silver's industrial component are missing. |
| Indices | [NYSE hours and calendars](https://www.nyse.com/markets/hours-calendars) establishes cash-session timing. | Compare only other index episodes with exactly matching opening windows. This tests common price context, not breadth, auction imbalance or an established lead/lag model. |

The source manifest records one excluded initial NBER URL whose title was
unrelated; the currency paper's identifier was corrected using Crossref metadata
before drawing any research conclusion. Inspect source contents, not URL labels.

## Frozen protocol and actual results

`cost-context-plan-2026-11-05.json` froze 40 source-file hashes and the protocol
before this diagnostic ran. These are the **same two already-inspected Tuesdays**,
5 and 12 August 2025; this is adaptive development, not independent validation.
No validation dates or new broker-history jobs were opened. Forty native Rust
verifications rechecked source digests and the original broker study exactly.

At hour one, compare own momentum, peer continuation, peer reversal and
always-long. Observe 30-, 60- and 180-minute outcomes. Peer features use only
same-start episodes and observations available by the decision, at most 60
seconds old and fresh when recorded. The target never contributes to its FX
factor. Both quote sides must be no more than six seconds old when a snapshot
forms. Entry uses the next strictly later fresh snapshot before the five-second
deadline; missing entry/exit observations remain missing. No interpolation.

Report midpoint directional accuracy separately from executable bid/ask returns
before additional fees and the unchanged 24bp/side stress return. Spread is
charged once by bid/ask. The fee level at which an individual outcome breaks
even is a diagnostic frontier, not a calibrated fee recommendation. Unknown
commission/funding prevents qualified net-return claims. Unlike price units
are not summed; class averages use equal-weight basis-point observations, not
account or portfolio P&L. All comparisons use identical observed episodes.

Three-hour peer continuation:

| Class | Observed / expected episodes | Direction correct | Winners after 24bp/side stress | Mean stress return |
|---|---:|---:|---:|---:|
| Indices | 11 / 14 | 7 / 11 | 1 / 11 | -47.01bp |
| FX | 12 / 12 | 5 / 12 | 0 / 12 | -52.39bp |
| Metals | 4 / 4 | 2 / 4 | 1 / 4 | -45.83bp |
| Energy | 3 / 6 | 3 / 3 | 2 / 3 | +5.72bp |
| Crypto | 4 / 4 | 3 / 4 | 2 / 4 | -0.73bp |

These are fixed-horizon **diagnostics**, without strategy SL/TP, margin, quantity,
trailing or account lifecycle reconstruction. They are not live trade results.
Energy's observed cases are oil only; NatGas is missing. Crypto's peer and own
signals make identical decisions on all four matched observations. The oil
peer and own signals also match on both mutually observed three-hour episodes;
different raw arm counts must not be presented as peer improvement. Both oil
winners occur on the same day, so they are especially weak independent evidence.

At 30/60 minutes, no FX, metals, crypto or energy arm has a stress winner; one
index observation survives at 60 minutes in continuation arms. This supports
investigating cost/horizon compatibility, not a universal failure of prediction.
There are 480 paired episode/variant/horizon records, not 480 independent trades.
Some directions are wrong even before fees; reducing fees alone cannot create
an edge. The observed three-hour energy result is too sparse to select/deploy.

The source windows also limit fundamental inference. FX, metals and energy
cover 07:00–11:00 UTC; crypto starts at midnight UTC; indices use local cash
opens. These two Tuesday energy windows do not study the usual Wednesday oil
or Thursday gas inventory-release periods. Future event research must use
actual dated release schedules and pre-release expectations, not assume that
the generic window or current calendar contains historical surprises.

## Keep, reject and next checks

Keep the receipt attribution, causal timestamp checks, per-symbol disclosures,
matched-control comparisons and the earlier proven replay fixes. Reject live
promotion of these diagnostic arms. No threshold, fee, lot size, target or stop
is changed to make a desired percentage appear.

Five gates remain before a financially supported promotion:

1. Representative paired execution and fee evidence; separately label live/demo,
   broker exits and market-order execution. Unknown fields fail qualification.
2. As-published class-specific information with publication and observation times,
   plus relevant session/event windows. Revised data and missing expectations
   must not masquerade as historical forecasts.
3. One frozen economic candidate tested against matched controls using the real
   strategy exit and quantity rules, not just fixed-horizon direction.
4. Separate-time evaluation after development qualifies: per-symbol results,
   costs, missingness, drawdowns and adequate independent dates. No retuning
   on the evaluation sample. The existing original-plan gates remain intact.
5. Demo shadow verification and applicable owner-authorised promotion. Report
   the original per-strategy/account target separately from class research.

No new subscription, model API, Railway service, database schema or automated
collector is introduced. The separate engineering worker's mutation deadline
and Telegram-incident mute remain unchanged. Costs/data cannot be truthfully
called calibrated or complete with the evidence currently available.

## Reproduce and verification

```sh
python3 -m unittest discover -s ops/research -p 'test_*.py'
python3 ops/research/cost_context.py \
  --sources /workspace/jiaoyi-class-prediction-evidence/development \
  --receipts /workspace/jiaoyi-cost-context-evidence \
  --binary /workspace/jiaoyi-anchor-fix-20261031/target/debug/examples/class_prediction_study \
  --output /workspace/jiaoyi-cost-context-evidence
```

Five new unit tests cover causal/stale quotes, peer exclusion/time/orientation,
executable cost arithmetic, conflicting/missing receipts and matched cohorts.
All 13 research unit tests pass. The report and JSON record source, plan,
receipt, verifier and analysis hashes. Required CI and merge/deployment receipts
belong to the PR closure. No claim of financial completion follows a green build.
