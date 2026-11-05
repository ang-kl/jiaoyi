# Major FX: research before setup

№18,245 §§1–7 · 5 November 2026 · by <Codex (GPT-6), effort: not exposed>.

The owner explicitly rejected assuming the BTC/NatGas setup was suitable for FX. No opening-range strategy or cheaper cost sensitivity is promoted or scored. This change introduces a no-trade market-structure diagnostic for seven major pairs, not an FX edge.

## External evidence

- [BIS 2025 survey](https://www.bis.org/statistics/rpfx25_fx.htm): Global OTC FX turnover and currency concentration; not broker-specific volume or a directional signal.
- [Neely and Weller, Intraday Technical Trading in the Foreign Exchange Market](https://doi.org/10.20955/wp.1999.016): Tested technical rules lacked excess returns after realistic costs/hours. Historical sample does not prove all modern strategies fail.
- [Breedon and Ranaldo, Intraday Patterns in FX Returns and Order Flow](https://doi.org/10.1111/jmcb.12032): Published abstract supports time-of-day/order-flow effects. Full paper not retrieved; no modern net profitability or identical cTrader execution asserted.
- [Andersen et al., Micro Effects of Macro Announcements](https://www.nber.org/papers/w8959): Surprises versus expectations cause conditional mean jumps; a calendar alone is not a direction predictor.
- [Menkhoff et al., Currency Momentum Strategies](https://www.bis.org/publ/work366.htm): Cross-sectional momentum evidence, costs and limits to arbitrage. Does not validate a four-hour opening-range breakout.
- [BIS, FX execution algorithms and market functioning](https://www.bis.org/publ/mktc13.htm): Fragmented liquidity and execution risk require suitable data and transaction-cost measurement.
- [Pepperstone costs and fees](https://pepperstone.com/en/trading/costs-and-fees/): Account/platform-specific spread, commission and NY-close funding. Public page entity may differ from owner; current generic wording is not historical account calibration.
- [FX Global Code](https://www.globalfxc.org/fx-global-code/): Good execution, disclosure and risk practices; no return forecast or strategy validation.

Retrieved 5 November 2026. Neely/Weller full working paper read; Ranaldo/Breedon published abstract only. Other cited publication summaries and broker page inspected. Historical results are not current CFD guarantees. Incorrect-title search hits were rejected.

## Decision

Do not transfer the existing entry band, two-hour opening range, four-hour exit, TP geometry or cost assumptions as a validated FX system. First observe Tokyo 09:00, London 08:00 and New York 08:00 local-time four-hour windows with DST. These are explicit diagnostic conventions, not optimal exchange-open claims. Use two frozen development dates (8 and 18 September 2025), seven pairs, 42 bounded jobs. No entry/SL/TP candidate is selected. The JSON freezes scope before broker outcomes.

The existing 24bp-per-side reserve is compared with observed movement and spread, excluding unknown commission. No 1/3bp replacement is activated. Request commission=0 means not measured in this diagnostic. Current account summaries contain few executions and no USDCHF cost observations, and public Pepperstone fee wording/entity cannot certify historical account charges. Tick quotes do not supply consolidated volume, signed order flow or historical economic surprises. Seven USD pairs share currency risk.

## Six gates

1. Confirm exact symbol, quote quality, units, commission conversion and costs per account.
2. Acquire as-of event timing/consensus and broader session/regime history; do not retrospectively relabel using revised data.
3. Choose one economically motivated FX hypothesis, freeze controls/cost scenarios/splits before outcomes.
4. Replay costs/latency/gaps and compare no-skill/unchanged-rule controls. Preserve failed trials.
5. Separate-time validation per instrument with uncertainty, net expectancy/PF and adequate samples; interim >=50% alone insufficient.
6. Demo shadow qualification and portfolio USD exposure/margin checks before any proposed live strategy change.

The 50% interim research target and original 73% / 1:1.38 per-strategy/account goal remain unverified. No trading, risk, registration or worker setting is changed. Existing baseline replay outputs must remain identical. Final CI/deployment/data receipts belong on this PR; raw broker data stays outside Git.
