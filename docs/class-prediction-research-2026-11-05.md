# All-class predictive research: first-hour continuation versus reversal

№18,257 §§1–6 · 05-11-2026, 19:46 SGT · by <Codex (GPT-6), effort: not exposed>.

Owner requests more predictive experiments and an interim50% result for each
asset class. This program makes the target measurable; it cannot guarantee it.
The original73%/1:1.38 goal per strategy/account remains unchanged.

## Concrete change

The bounded owner research endpoint previously accepted BTC/NatGas strategy
experiments and seven FX diagnostic pairs only. Two new research modes accept
exact original20 symbols across indices,FX,metals,energy,crypto. They use explicit
symbol/session mappings, including Sydney's UTC-date rollover and exchange DST.
Only these new modes expand symbol eligibility. Existing S3, opening-range and
FX diagnostic semantics are retained. Production's17-symbol universe is not
restored to20; removed names are research-only.

A single decision is made at minute60 after thirteen equally scheduled five-minute
fresh midpoint samples. First-to-last change predicts either continuation or
reversal. Absolute displacement / total sampled absolute displacement must be
at least0.5. Fresh tick midpoint range through the decision fixes originalR.
No future prices enter the features. Next strictly later fresh quote before the
five-second mark deadline is the explicit entry proxy, with direction/cost
rechecked. Missing samples are missing evidence, not zero movement.

Admission uses the causal median sampled spread, explicit commission scenario
and unchanged24bp per side. Both candidates retain existing Q1.38 target planning,
minimum native-volume constraints and fixed-stop/original-R-trailing comparisons.
Positions close at hour4 with a bounded five-second tail; absent exit coverage
remains censored. Same-millisecond quote grouping, receipt-versus-broker clocks,
historical contracts, real slippage, financing and account sizing remain unproven.
Research scenarios never become qualified account performance automatically.

These are competing causal price hypotheses, not a claim of universal economic
support. Crypto research motivates testing momentum at appropriate horizons;
FX technical-trading evidence warns that costs can erase results; exchange hours
and EIA calendars motivate distinct market context, not a direction signal.
Neither candidate includes economic surprise vintages, weather or signed order
flow. The source links and limits are frozen in the JSON plan. A failed screen
calls for a documented causal redesign, not parameter hunting on evaluation dates.

## Frozen batch and gates

`class-prediction-plan-2026-11-05.json` fixes all20 symbols, account49236051,
volumes, two development dates,14 later evaluation dates, candidate definitions,
costs and sample gates. Forty initial source episodes produce80paired candidate
results. Historical baseline/bar exposure is disclosed; later dates are separate
time evaluation, not claimed pristine unseen history.

The collector permits at most40new jobs per invocation, one active broker study
at a time, <=200000ticks/120pages per job, four-hour windows and the existing
research seal. Source failures are preserved without automatic retries. A saved
plan hash prevents silent plan changes. No raw ticks are inserted into Postgres;
existing endpoint keeps only its latest job in memory and the caller saves bounded
source JSON locally. No model API, allowance consumption or new scheduled worker.

The offline Rust example verifies original-source SHA256 and exact equality with
the deployed momentum study, then evaluates reversal on the same source. The
Python scorecard reports every class, symbol, missing episode, censored position,
net win rate, meanR, PF and Wilson interval. It never adds unlike price units
across instruments. Every symbol must have positive net price-unit evidence and
>=50%net wins before its class can pass the preliminary screen. Development
requires >=4closed scenarios per class; this is only a small screening gate.
Evaluation requires >=20per class,>=5per symbol,three weeks,positive expectancy
and PF>1, complete coverage and no censoring. Only a class passing development
opens its evaluation; the choice is frozen by development meanR. No financial
PASS or deployment authority is granted by this scorecard.

## Run

```sh
cargo build -p jiaoyi-core --example class_prediction_study --locked
python3 ops/research/class_research.py collect --stage development --output /tmp/class-research
python3 ops/research/class_research.py analyse --stage development --output /tmp/class-research --binary target/debug/examples/class_prediction_study
```

Uses privately configured `JIAOYI_UI_PASSWORD`; never put it in a request/file.
Subsequent validation collection is automatically refused unless a development
class passes. Results include HTML, JSON, source hashes and explicit failure
reasons. The CLI reads the frozen repository plan and does not change settings.

## Verification

Three focused Rust tests cover all20 local sessions/mode boundaries, causal
features with adversarial future prices, and missing samples/contract/cost/
censoring failures. Three Python tests cover class/member gates, loss/censor/
duplicate handling and UTC/DST dates. All37core research tests passed locally;
workspace Clippy with denied warnings passed. Required exact-head CI and deployed
broker/offline parity are recorded on the PR before economic conclusions.
