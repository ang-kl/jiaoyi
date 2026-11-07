# Phase 1 — Contract, cost evidence and unified replay inventory

№18,355 §§1–8 · 08-11-2026 07:32:36 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed> · Repository: `ang-kl/jiaoyi`

## §1. TL;DR and goal progress

**Phase 1 engineering is implemented and verified. It does not establish a trading edge or a calibrated execution-cost model.** The manifest contains **384 distinct tick episode definitions across 40 symbols**, plus **377 verified M30 source files**. **379 episodes are outside the protected dependency intervals**; this eligibility is a seal check, not a claim of complete preparation or financial quality.

The audit records **6 legacy tick receipts with protected preparation, representing 5 canonical episodes plus a duplicate receipt**. It excludes their protected dependencies from this phase's calculations. Their existence means **historically untouched validation is not established**. Quarantine prevents new use here; it cannot undo earlier collection or exposure. No strategy outcome was evaluated on the protected interval in this task.

The retained evidence provides **13 attributable entry slippage observations, 77 unique actual broker deals, 1053 unique order records and 1492 distinct statement closing-deal IDs**. It provides **0 fully qualified round-trip execution-cost samples**. Recorded money fields are useful evidence, but do not replace missing dispatch quotes on both legs.

**Verification:** 2691 frozen local files match; **887 collection-ledger references match**, all **159** prior frozen episode receipts remain represented, four focused tests pass, and two independent builds produce byte-identical JSON. There are **530 other historical references whose exact revision cannot be resolved in this frozen local catalog**. These include retired temporary exports and code/publication references; they are neither silently verified nor all described as missing market prices.

**Goal progress:** zero independent trade observations and zero new broker requests. The per-strategy/per-account goal remains at least 73% net wins over the latest 20 trades with reference 1:1.38 R:R; at least 15/20 wins are needed on an exact 20-trade window. This phase makes provenance and cost gaps inspectable. It does not improve or qualify the reported win percentage. Live strategy, account authority, lot limits and risk settings are unchanged.

Read the [cost/contract inventory](../docs/rebuild_phase1_cost_and_contract_inventory_18355.json), [unified manifest](../docs/rebuild_phase1_unified_replay_manifest_18355.json), [calendar registry](../docs/rebuild_phase1_session_calendars_18355.json), [frozen scope](../docs/rebuild_phase1_scope_18355.json), and [verification record](../docs/rebuild_phase1_verification_18355.json). Sections 6–8 contain the complete spread and receipt tables; full hashes remain in the JSON.

## §2. Contracts and empirical spread evidence

### 2.1 Contract inventory: all cached symbols

The following table reports the research account's recorded contract identities. Native volume is in **cents of units**, following the existing cTrader schema; it is not lots. Scenario volume is the quantity stored in an experiment and can differ from broker minimum volume. The code price quantum is `10^-digits`; it is a representational grid, not independent proof of a broker's economically tradable minimum tick or money value per point.

| Symbol | Class | Cache account | Symbol ID | Scenario volume | Minimum volume | Volume step | Digits | Code price quantum | Money/tick value | Broker spec snapshots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ADAUSD | crypto | 49236051 | 10095 | 100 | 100 | 100 | 4 | 0.0001 | Unverified | 0 |
| AUDJPY | fx | 49236051 | 11 | 100000 | 100000 | 100000 | 3 | 0.001 | Unverified | 0 |
| AUDUSD | fx | 49236051 | 5 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| AUS200 | equity_index | 49236051 | 10000 | 10 | 10 | 10 | 1 | 0.1 | Unverified | 0 |
| AVAXUSD | crypto | 49236051 | 10738 | 100 | 100 | 100 | 3 | 0.001 | Unverified | 0 |
| BCHUSD | crypto | 49236051 | 10063 | 1 | 1 | 1 | 2 | 0.01 | Unverified | 0 |
| BTCUSD | crypto | 49236051 | 10028 | 1 | 1 | 1 | 2 | 0.01 | Unverified | 2 |
| CADJPY | fx | 49236051 | 15 | 100000 | 100000 | 100000 | 3 | 0.001 | Unverified | 0 |
| CHFJPY | fx | 49236051 | 13 | 100000 | 100000 | 100000 | 3 | 0.001 | Unverified | 0 |
| DOGEUSD | crypto | 49236051 | 10094 | 100 | 100 | 100 | 5 | 1e-05 | Unverified | 0 |
| DOTUSD | crypto | 49236051 | 10072 | 100 | 100 | 100 | 3 | 0.001 | Unverified | 0 |
| ETHUSD | crypto | 49236051 | 10029 | 1 | 1 | 1 | 2 | 0.01 | Unverified | 0 |
| EURAUD | fx | 49236051 | 14 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| EURGBP | fx | 49236051 | 9 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| EURJPY | fx | 49236051 | 3 | 100000 | 100000 | 100000 | 3 | 0.001 | Unverified | 0 |
| EURUSD | fx | 49236051 | 1 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 2 |
| GBPJPY | fx | 49236051 | 7 | 100000 | 100000 | 100000 | 3 | 0.001 | Unverified | 0 |
| GBPUSD | fx | 49236051 | 2 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| GER40 | equity_index | 49236051 | 10003 | 10 | 10 | 10 | 1 | 0.1 | Unverified | 0 |
| JPN225 | equity_index | 49236051 | 10006 | 1000 | 1000 | 1000 | 1 | 0.1 | Unverified | 0 |
| LINKUSD | crypto | 49236051 | 10073 | 100 | 100 | 100 | 3 | 0.001 | Unverified | 0 |
| LTCUSD | crypto | 49236051 | 10030 | 1 | 1 | 1 | 2 | 0.01 | Unverified | 0 |
| NAS100 | equity_index | 49236051 | 10014 | 10 | 10 | 10 | 1 | 0.1 | Unverified | 0 |
| NZDUSD | fx | 49236051 | 12 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| NatGas | energy | 49236051 | 10055 | 10000 | 10000 | 10000 | 3 | 0.001 | Unverified | 4 |
| SOLUSD | crypto | 49236051 | 10737 | 100 | 100 | 100 | 2 | 0.01 | Unverified | 0 |
| SpotBrent | energy | 49236051 | 10053 | 100 | 100 | 100 | 3 | 0.001 | Unverified | 2 |
| SpotCrude | energy | 49236051 | 10054 | 100 | 100 | 100 | 3 | 0.001 | Unverified | 1 |
| UK100 | equity_index | 49236051 | 10011 | 10 | 10 | 10 | 1 | 0.1 | Unverified | 0 |
| US30 | equity_index | 49236051 | 10015 | 10 | 10 | 10 | 1 | 0.1 | Unverified | 0 |
| US500 | equity_index | 49236051 | 10013 | 10 | 10 | 10 | 1 | 0.1 | Unverified | 0 |
| USDCAD | fx | 49236051 | 8 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| USDCHF | fx | 49236051 | 6 | 100000 | 100000 | 100000 | 5 | 1e-05 | Unverified | 0 |
| USDJPY | fx | 49236051 | 4 | 100000 | 100000 | 100000 | 3 | 0.001 | Unverified | 0 |
| XAGUSD | metal | 49236051 | 42 | 5000 | 5000 | 5000 | 3 | 0.001 | Unverified | 0 |
| XAUUSD | metal | 49236051 | 41 | 100 | 100 | 100 | 2 | 0.01 | Unverified | 0 |
| XLMUSD | crypto | 49236051 | 10074 | 1000 | 1000 | 1000 | 4 | 0.0001 | Unverified | 0 |
| XPDUSD | metal | 49236051 | 73 | 100 | 100 | 100 | 2 | 0.01 | Unverified | 0 |
| XPTUSD | metal | 49236051 | 74 | 100 | 100 | 100 | 2 | 0.01 | Unverified | 0 |
| XRPUSD | crypto | 49236051 | 10076 | 1000 | 1000 | 100 | 4 | 0.0001 | Unverified | 0 |

**Account identity is mandatory.** A numeric symbol ID is not global. For example, cached demo account 49236051 uses ID 10054 for SpotCrude, whereas retained live account 43002148 metadata maps 10054 to NatGas. All contract/deal joins use account plus symbol ID; the inventory retains explicit symbol-name resolution. The recorded current full specifications below supplement the cache contract table and must not be transferred between accounts or backdated to 2023–2025.

Tick/point money value remains **unverified_requires_broker_spec** where a dated contract, quantity denomination and quote-to-deposit conversion are unavailable. A known decimal price grid does not establish a dollar tick value. Current full snapshots retain lot size, volume steps, commission types, swap fields and measurement units where the broker supplied them. Missing fields are not filled with zero.

### 2.2 Spread measurement definition

For each eligible episode, take the first recorded non-crossed quote at or after the checkpoint within five seconds. Each side must be at most six seconds old. Group same-millisecond updates using stable within-side order; cross-side ordering is not invented. A missing quote remains missing. Off-grid, non-positive or unordered source prices fail validation. The exact price spread is `(ask_e5 − bid_e5)/100000`; basis points are `20000 × (ask_e5 − bid_e5)/(ask_e5 + bid_e5)`, using the contemporaneous mid-price denominator.

Quantiles use **type-7 linear interpolation with exact rational arithmetic**. Both exact fractions and decimal display values are published. The checkpoints are 0, 30, 60, 180 and 240 minutes from (a) the **coded strategy-session open**, and separately (b) the **registered experiment-window open**. These anchors differ for oil and some diagnostic sessions; they are never mixed into one distribution.

Identical tick streams and duplicate checkpoint observations are deduplicated. The final diagnostic found no conflicting quote observations at a shared account/symbol/checkpoint. These are cross-episode checkpoint distributions, **not time-weighted whole-day spread estimates**. Dates were selected for earlier research and are not a random market sample. Inherited repeat flags remain visible. A zero sample count means unavailable, not zero spread.

Core-symbol open checkpoints, using the coded strategy session:

| Symbol | Available open samples | Eligible episode definitions | Median spread: price | Median spread: bp |
| --- | --- | --- | --- | --- |
| SpotCrude | 46 | 62 | 0.02 | 3.2281529 |
| SpotBrent | 46 | 62 | 0.03 | 3.696557 |
| US500 | 51 | 51 | 0.4 | 0.6981412 |
| NAS100 | 7 | 7 | 1.3 | 0.57477541 |
| XAUUSD | 42 | 42 | 0.17 | 0.67006297 |
| XAGUSD | 6 | 7 | 0.0215 | 9.1936918 |
| BTCUSD | 29 | 30 | 17 | 2.5163004 |
| ETHUSD | 2 | 2 | 3 | 10.877088 |

The “eligible episode definitions” column includes distinct source windows; overlapping definitions can share a deduplicated observation. It is not automatically the denominator for a missing-quote percentage. Sections 6 and 7 publish all requested quantiles at all five checkpoints for all 40 symbols.

### 2.3 Slippage and cost inventory: all cached symbols

| Symbol | Attributable entry observations | Min entry slip bp | Median entry slip bp | Max entry slip bp | Current contract snapshots | Round-trip calibrated |
| --- | --- | --- | --- | --- | --- | --- |
| ADAUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| AUDJPY | 0 | Not verified | Not verified | Not verified | 0 | No |
| AUDUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| AUS200 | 0 | Not verified | Not verified | Not verified | 0 | No |
| AVAXUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| BCHUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| BTCUSD | 2 | -0.66231973 | -0.32213954 | 0.018040647 | 2 | No |
| CADJPY | 0 | Not verified | Not verified | Not verified | 0 | No |
| CHFJPY | 0 | Not verified | Not verified | Not verified | 0 | No |
| DOGEUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| DOTUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| ETHUSD | 1 | 4.4359817 | 4.4359817 | 4.4359817 | 0 | No |
| EURAUD | 0 | Not verified | Not verified | Not verified | 0 | No |
| EURGBP | 0 | Not verified | Not verified | Not verified | 0 | No |
| EURJPY | 0 | Not verified | Not verified | Not verified | 0 | No |
| EURUSD | 0 | Not verified | Not verified | Not verified | 2 | No |
| GBPJPY | 0 | Not verified | Not verified | Not verified | 0 | No |
| GBPUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| GER40 | 2 | -0.37551633 | -0.18775817 | 0 | 0 | No |
| JPN225 | 0 | Not verified | Not verified | Not verified | 0 | No |
| LINKUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| LTCUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| NAS100 | 0 | Not verified | Not verified | Not verified | 0 | No |
| NZDUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| NatGas | 3 | 0 | 0 | 3.7383178 | 4 | No |
| SOLUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| SpotBrent | 2 | -1.3775749 | -0.2313351 | 0.91490471 | 2 | No |
| SpotCrude | 3 | -0.6299874 | 0 | 2.4744819 | 1 | No |
| UK100 | 0 | Not verified | Not verified | Not verified | 0 | No |
| US30 | 0 | Not verified | Not verified | Not verified | 0 | No |
| US500 | 0 | Not verified | Not verified | Not verified | 0 | No |
| USDCAD | 0 | Not verified | Not verified | Not verified | 0 | No |
| USDCHF | 0 | Not verified | Not verified | Not verified | 0 | No |
| USDJPY | 0 | Not verified | Not verified | Not verified | 0 | No |
| XAGUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| XAUUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| XLMUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| XPDUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| XPTUSD | 0 | Not verified | Not verified | Not verified | 0 | No |
| XRPUSD | 0 | Not verified | Not verified | Not verified | 0 | No |

The 13 attributed entry records contain dispatch and fill prices, side and a reported slippage measure. Their signed arithmetic was independently checked. They are retained normalised execution evidence; some original exports referenced by the older audit are no longer in the local catalog. All 13 are demo observations. They do not calibrate live fills, exits or historical market conditions. The 24 bp stress reserve and 2 / 3.74 bp sensitivities remain assumptions, not measured all-in costs.

## §3. Actual account evidence and calendar authority

### 3.1 Broker deals and statements

The 77 account-scoped deal IDs cover 38 positions. Closing `ProtoOADeal` details can supply gross profit, commission, swap, conversion fee, closed volume and money scaling. The inventory preserves the signed original fields. It does not subtract the opening fee a second time when the recorded closing commission convention already represents both legs. Opening/closing volume mismatches, partial positions and missing dispatch quotes remain explicit.

| Account | Position | Broker symbol IDs | Open/close deal records | Opening/closed native volume | Closing money records | Missing cost evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 635265179 | 10027 | 1/1 | 1/1 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 43002148 | 635314805 | 10027 | 1/1 | 1/1 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 43002148 | 636072448 | 10027 | 1/1 | 1/1 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 43002148 | 637075128 | 10054 | 1/0 | 150000/0 | 0 | opening_and_closing_volume, settled_money, entry_dispatch_quote, exit_dispatch_quote |
| 43002148 | 637097369 | 10026 | 1/1 | 1/1 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 43002148 | 637255862 | 10054 | 4/2 | 200000/210000 | 2 | opening_and_closing_volume, entry_dispatch_quote, exit_dispatch_quote |
| 43002148 | 639274336 | 10026 | 2/1 | 8/8 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 240732790 | 10741 | 0/1 | 0/470 | 1 | opening_and_closing_volume, entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253467077 | 10006 | 1/1 | 7000/7000 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253617508 | 10028 | 1/1 | 33/33 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253617536 | 10029 | 1/1 | 740/740 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253636518 | 10029 | 1/1 | 710/710 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253636546 | 10028 | 1/1 | 29/29 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253636577 | 10053 | 1/1 | 23100/23100 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253718036 | 42 | 1/1 | 45000/45000 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 253876523 | 10054 | 1/1 | 8700/8700 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 254219899 | 10029 | 1/1 | 982/982 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 254247242 | 10006 | 1/1 | 6000/6000 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 254331404 | 10028 | 1/1 | 35/35 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 254522444 | 1 | 1/0 | 800000/0 | 0 | opening_and_closing_volume, settled_money, entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 254894097 | 10055 | 1/0 | 30000/0 | 0 | opening_and_closing_volume, settled_money, entry_dispatch_quote, exit_dispatch_quote |
| 46130058 | 254961198 | 10053 | 1/0 | 1200/0 | 0 | opening_and_closing_volume, settled_money, entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253424956 | 10055 | 1/1 | 3000000/3000000 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253438245 | 10055 | 1/2 | 100000/200000 | 2 | opening_and_closing_volume, entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253438330 | 10055 | 1/1 | 100000/100000 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253467076 | 10006 | 1/1 | 7000/7000 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253617507 | 10028 | 1/1 | 36/36 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253617535 | 10029 | 1/1 | 806/806 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253636517 | 10029 | 1/1 | 774/774 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253636545 | 10028 | 1/1 | 32/32 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 47790949 | 253636576 | 10053 | 1/1 | 25100/25100 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 253617506 | 10028 | 1/1 | 1/1 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 253617531 | 10029 | 1/1 | 24/24 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 253636514 | 10029 | 1/1 | 23/23 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 253636560 | 10053 | 1/1 | 700/700 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 253650945 | 10028 | 1/1 | 1/1 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 253876505 | 10054 | 1/1 | 200/200 | 1 | entry_dispatch_quote, exit_dispatch_quote |
| 49236051 | 254486522 | 10029 | 1/1 | 23/23 | 1 | entry_dispatch_quote, exit_dispatch_quote |

The 15 retained statement CSV files yield the following deduplicated closing-deal evidence. These totals describe the supplied overlapping statement exports, **not a new daily account performance report or a complete position ledger**. Commission totals are recorded facts across those deal IDs; they are not a universal per-symbol rate. Statement conflicts found: **0**.

| Account | Unique closing deal IDs | Recorded commission sum USD | Dispatch samples |
| --- | --- | --- | --- |
| 43002148 | 27 | -0.16 | 0 |
| 46130058 | 760 | -1798.05 | 0 |
| 47790949 | 616 | -2796.85 | 0 |
| 49236051 | 89 | -10.82 | 0 |

The material missing evidence is a joined lifecycle: account and contract version, opening/closing order and deal IDs, decision and dispatch bid/ask and timestamps for both legs, direction, requested/filled quantities, acknowledgement/fill times, settled fees, financing and quote-to-deposit conversion. Aggregate dashboard medians, synthetic fixtures and replay scenarios cannot fill those gaps. The JSON separately records implementation/test-file classifications and aggregate sources.

### 3.2 Coded strategy calendars

The Rust adapter invokes the existing `sessions::rule_for` and `session_on`. It does not reproduce those rules in a separate Python timezone table. The following sessions are **code facts**; they are not a historical broker/reference-exchange calendar certification.

| Symbols | IANA zone | Strategy open | Strategy close | Calendar rule | 45-day code-slot counts across observed dates | Historical broker count |
| --- | --- | --- | --- | --- | --- | --- |
| NAS100, US30, US500 | America/New_York | 09:30:00 | 16:00:00 | Mon–Fri | 403, 416, 429 | Not verified |
| AUDJPY, AUDUSD, CADJPY, CHFJPY, EURAUD, EURGBP, EURJPY, EURUSD, GBPJPY, GBPUSD, NZDUSD, NatGas, SpotBrent, SpotCrude, USDCAD, USDCHF, USDJPY, XAGUSD, XAUUSD, XPDUSD, XPTUSD | UTC | 07:00:00 | 20:00:00 | Mon–Fri | 806, 832, 858 | Not verified |
| UK100 | Europe/London | 08:00:00 | 16:30:00 | Mon–Fri | 527 | Not verified |
| AUS200 | Australia/Sydney | 10:00:00 | 16:00:00 | Mon–Fri | 372 | Not verified |
| GER40 | Europe/Berlin | 09:00:00 | 17:30:00 | Mon–Fri | 527 | Not verified |
| JPN225 | Asia/Tokyo | 09:00:00 | 15:30:00 | Mon–Fri | 341 | Not verified |
| ADAUSD, AVAXUSD, BCHUSD, BTCUSD, DOGEUSD, DOTUSD, ETHUSD, LINKUSD, LTCUSD, SOLUSD, XLMUSD, XRPUSD | UTC | 00:00:00 | 24:00:00 | Every day | 2160 | Not verified |

The 45-day counts enumerate complete M30 slots in the 45 calendar dates preceding each episode under that coded strategy rule, excluding coded lunch breaks. The count varies with weekday composition; no universal 1,400- or 2,160-bar rule is imposed. Historical holidays and early closes are not implemented by that existing calendar. Accordingly **complete_calendar is never claimed**. `complete_session_aware` means every required code-session timestamp was found in the verified local M30 union. It does not certify historical broker tradability.

Current preparation statuses: **39 complete_session_aware; 345 missing_segments**. A missing count here is relative to this declared 45-day rule; it is not proof that every strategy requiring a shorter lookback is unusable. A later experiment must declare its actual dependency interval.

### 3.3 Recorded broker schedules, maintenance gaps and fee fields

The cTrader schema defines weekly intervals as half-open seconds from Sunday 00:00 in `scheduleTimeZone`. The registry preserves the exact supplied intervals and derives their complement without changing endpoints. Gaps are recorded closed intervals; their business reason is not guessed. In particular, a crypto UTC strategy day does **not** mean its CFD broker has no maintenance breaks.

| Account | Symbol | ID | Broker IANA zone | Lot size: native cents | Recorded weekly closed intervals | Precise commission field | Commission type | Swap long | Swap short | Source file |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | BTCUSD | 10026 | America/New_York | 100 | Sunday 16:59:00 → Sunday 17:01:00; Monday 16:59:00 → Monday 17:01:00; Tuesday 16:59:00 → Tuesday 17:01:00; Wednesday 16:59:00 → Wednesday 17:01:00; Thursday 16:59:00 → Thursday 17:01:00; Friday 16:55:00 → Friday 17:01:00; Saturday 16:59:00 → Saturday 17:01:00; Saturday 23:59:59 → next Sunday 00:00:00 | 0 | 2 | -14.0 | 2.0 | broker-43002148-symbol-details.json |
| 43002148 | NatGas | 10054 | America/New_York | 1000000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 18:00:00; Tuesday 16:59:00 → Tuesday 18:00:00; Wednesday 16:59:00 → Wednesday 18:00:00; Thursday 16:59:00 → Thursday 18:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 2 | -1.4 | 1.26 | broker-43002148-symbol-details.json |
| 46130058 | EURUSD | 1 | America/New_York | 10000000 | Sunday 00:00:00 → Sunday 17:01:00; Monday 16:59:00 → Monday 17:01:00; Tuesday 16:59:00 → Tuesday 17:01:00; Wednesday 16:59:00 → Wednesday 17:01:00; Thursday 16:59:00 → Thursday 17:01:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | -0.73 | 0.15 | broker-46130058-symbol-details.json |
| 46130058 | SpotBrent | 10053 | America/New_York | 10000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 20:00:00; Tuesday 16:59:00 → Tuesday 20:00:00; Wednesday 16:59:00 → Wednesday 20:00:00; Thursday 16:59:00 → Thursday 20:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | 13.63 | -15.0 | broker-46130058-symbol-details.json |
| 46130058 | NatGas | 10055 | America/New_York | 1000000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 18:00:00; Tuesday 16:59:00 → Tuesday 18:00:00; Wednesday 16:59:00 → Wednesday 18:00:00; Thursday 16:59:00 → Thursday 18:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | -1.4 | 1.26 | broker-46130058-symbol-details.json |
| 47790949 | SpotCrude | 10054 | America/New_York | 10000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 18:00:00; Tuesday 16:59:00 → Tuesday 18:00:00; Wednesday 16:59:00 → Wednesday 18:00:00; Thursday 16:59:00 → Thursday 18:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | 3.3 | -4.17 | broker-47790949-symbol-details.json |
| 43002148 | BTCUSD | 10026 | America/New_York | 100 | Sunday 16:59:00 → Sunday 17:01:00; Monday 16:59:00 → Monday 17:01:00; Tuesday 16:59:00 → Tuesday 17:01:00; Wednesday 16:59:00 → Wednesday 17:01:00; Thursday 16:59:00 → Thursday 17:01:00; Friday 16:55:00 → Friday 17:01:00; Saturday 16:59:00 → Saturday 17:01:00; Saturday 23:59:59 → next Sunday 00:00:00 | 0 | 2 | -14.0 | 2.0 | broker-43002148-current-symbol-details.json |
| 43002148 | NatGas | 10054 | America/New_York | 1000000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 18:00:00; Tuesday 16:59:00 → Tuesday 18:00:00; Wednesday 16:59:00 → Wednesday 18:00:00; Thursday 16:59:00 → Thursday 18:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 2 | -1.4 | 1.26 | broker-43002148-current-symbol-details.json |
| 46130058 | EURUSD | 1 | America/New_York | 10000000 | Sunday 00:00:00 → Sunday 17:01:00; Monday 16:59:00 → Monday 17:01:00; Tuesday 16:59:00 → Tuesday 17:01:00; Wednesday 16:59:00 → Wednesday 17:01:00; Thursday 16:59:00 → Thursday 17:01:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | -0.73 | 0.15 | broker-46130058-current-symbol-details.json |
| 46130058 | SpotBrent | 10053 | America/New_York | 10000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 20:00:00; Tuesday 16:59:00 → Tuesday 20:00:00; Wednesday 16:59:00 → Wednesday 20:00:00; Thursday 16:59:00 → Thursday 20:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | 13.63 | -15.0 | broker-46130058-current-symbol-details.json |
| 46130058 | NatGas | 10055 | America/New_York | 1000000 | Sunday 00:00:00 → Sunday 18:00:00; Monday 16:59:00 → Monday 18:00:00; Tuesday 16:59:00 → Tuesday 18:00:00; Wednesday 16:59:00 → Wednesday 18:00:00; Thursday 16:59:00 → Thursday 18:00:00; Friday 16:55:00 → next Sunday 00:00:00 | 0 | 1 | -1.4 | 1.26 | broker-46130058-current-symbol-details.json |

Raw commission fields require their recorded type and scale. `preciseTradingCommissionRate` is scaled by 10^8 for non-percentage types and 10^5 for percentage-of-value type, according to the retained protocol. Swap fields also require their calculation type, charge schedule, rollover-day rules and dated instrument terms. A recorded zero commission on one contract snapshot or deal is not a default for another account or period.

The requested New York/NYMEX and London reference opens must remain separate from these CFD schedules. The registry records the requested IANA references but marks undocumented exact reference hours and historical maintenance terms **unverified_requires_broker_spec**. No guessed NYMEX, London, holiday or rollover window was inserted into live code.

## §4. Manifest, verification and acceptance gates

The builder scans a frozen catalog of local evidence directories and this repository's document/log artifacts, plus supplied statement CSVs. Other Git worktrees are excluded as duplicated history; credentials, build outputs and unrelated scratch projects are excluded. Source files remain in their existing cache locations. Only manifests, facts and hashes are published; raw ticks are not duplicated into Git or Postgres.

The manifest retains **10 duplicate tick receipts**, **28 failed/unqualified sources**, and **0 conflicting M30 observations**. Protocol statuses across canonical episodes are: **299 clean_first_read**, **20 inherited_repeat_interval**, and **65 unverified_receipt_history**. The third status deliberately avoids inventing a clean-first-read history where older API captures lack that evidence.

Across eligible episodes, **252** have some later cached M30 bars and **203** have a contiguous M30 bar beginning at their window boundary. A later bar months away is not a 72-hour continuation. Each row therefore also supplies the first available timestamp, a within-72-hour flag and the contiguous count. Financing, missing session intervals and closing-quote tails remain separate requirements.

**Checks completed:** four focused Python tests cover altered/missing receipts, exact quantile/bps arithmetic and checkpoint freshness, protected windows/nested preparation, and account-scoped identities. The compiled Rust adapter passed checks around US DST, the US/UK mismatch week and a crypto weekend. Formatting and warnings-denied Clippy passed. Two independent inventory builds produce identical bytes for all three data artifacts. Every frozen file was rehashed before and after each build. All 887 retained collection-ledger references reconcile exactly; all 159 prior frozen episode identities are present. No broker or database call is made by the builder.

The catalog also resolves 4,164 historical references in total and retains 530 unresolved exact revisions. A basename such as `costs.json` is not treated as a corrupt file merely because a different dated export with that basename exists. Old versioned code/publication references are not treated as current-market receipts. **This is not a claim that 100% of all historical artifacts remain locally available.**

Acceptance against the twelve gates in the №18,353 RFC:

| RFC gate | Phase 1 status | Evidence / remaining requirement |
| --- | --- | --- |
| G1 | Partial | Current inventory links source/code/calendar/contract identities and records unknown cost authority. Older unresolved references prevent a claim of complete historical provenance. |
| G2 | Passed for inventory identity only | All 159 prior frozen sources are represented. PR #190’s 159-path parity remains prior evidence; this phase does not migrate or rerun the strategy kernel. |
| G3 | Partial | Deterministic offline freeze/build and verified episode loader exist. The consolidated replay/scoring/publication command is Phase 2 scope. |
| G4 | Partial | Preparation slots, protected dependencies and contiguous M30 tails are indexed. Missing data remain missing; full holding/closing-tail qualification is still required per experiment. |
| G5 | Partial | Rust strategy sessions and four DST examples checked; dated broker/reference holidays and historical maintenance terms remain unverified. |
| G6 | Partial | Actual deal/statement money and volume coverage inventoried. Complete matched entry/exit dispatch-cost reconciliation remains absent. |
| G7 | Not satisfied | No calibrated round-trip model or independent cost prediction-error validation. |
| G8 | Deferred | Shared multi-day execution kernel and research/live parity migration are later work. |
| G9 | Not satisfied for promotion | Current analysis excludes protected dependencies; legacy protected preparation means historical unseen status cannot be asserted. A future split/exposure audit is required. |
| G10 | Not satisfied | No new strategy experiment or positive quantitative qualification in Phase 1. |
| G11 | Deferred | No new shadow/canary deployment or reconciliation trial. |
| G12 | Deferred | Existing live risk rules remain in force; a rebuild lifecycle/rollback acceptance exercise was not performed. |

**Phase 1 completion means the requested inventory, registry, builders, focused verification and report exist. It does not mean all twelve rebuild gates passed.** Engineering gaps and unavailable economic evidence are recorded separately, so successful CI cannot promote the strategy.

Reproduce with the frozen local sources and this version of the builder:

```text
cargo build -p jiaoyi-core --example replay_inventory_metadata --locked --offline
python3 ops/research/replay_inventory.py build --scope docs/rebuild_phase1_scope_18355.json --output /tmp/phase1-inventory
python3 -m unittest discover -s ops/research -p test_replay_inventory.py
```

A clean clone can inspect the published facts and hashes. Rerunning requires the exact source cache paths in the scope file; the builder neither downloads substitutes nor treats missing files as zero observations. Future consumers can use `load_verified_episode()` to resolve one canonical ID and recheck its hash and protected dependencies. The prior CODEX log is read through an immutable local snapshot, allowing this task's journal to append without changing its input evidence.

## §5. Exact proposed Phase 2 scope

**Build one offline coverage-and-replay command consuming this manifest, starting with the existing four-symbol two-stage baseline.** This consolidates already tested mechanisms before another hypothesis or cost change. It is the next proposed phase, not work silently performed by Phase 1.

1. **Typed experiment contract:** explicit strategy/kernel version, account and contract identity, calendar version, price-risk definition, cost assumption versus calibrated model, entry window, lookback, maximum holding time and closing-quote tail. Preserve the original 159-episode baseline as a migration fixture.
2. **Coverage resolver:** select verified source slices by account, symbol and half-open interval; reject contradictory quotes/bars, protected dependencies and unacknowledged provenance flags. Return a complete coverage receipt or an exact missing-interval list. Preserve actual calendar gaps without synthesising prices.
3. **Unified offline runner:** one command resolves a frozen contract, loads the verified cache, invokes the existing native replay, scores strict and forced-close outcomes separately, and emits a stable report/receipt. Old date-specific scripts become compatibility wrappers after equivalence is demonstrated. No new strategy tuning is part of this migration.
4. **Cost-evidence loader:** expose the already retained account-scoped orders, deals and statement references through a typed join. Keep aggregate medians and synthetic replay costs separate. Emit the precise missing capture fields for each incomplete round trip; do not derive a lower production reserve from the backtest.
5. **Finite acceptance:** exact 159-episode decision/fill/outcome parity, deterministic restart/idempotence, one deliberate gap/protected-dependency failure, zero offline network access, and unchanged live rule/risk paths. The broad G3/G4/G6/G8 status improves only to the extent those properties are actually demonstrated.

No Phase 2 collection, historical broker-spec retrieval, paid model service, risk change or live deployment is implied by this Phase 1 PR. A later data-collection proposal should name the smallest missing intervals and contract/receipt fields, rather than request another broad historical batch. Before any independent validation, the existing protected-preparation exposure must be accounted for and an appropriate genuinely unseen evaluation scope agreed.

The operational invariants remain the account-level goals, serial/paragraph traceability and SGT timestamps, preserving proven fixes, and **“Time is Money”**. Preparation means resolving the evidence needed for the next decision; neither extra activity nor another target tweak substitutes for an edge.

## §6. Complete spread quantiles — coded strategy-session anchor

Every quantile cell below is **price units / basis points**. Exact rational values, opportunity counts and per-episode quote timestamps are in the JSON. “Not verified” with n=0 means no eligible fresh observation.

| Symbol | Minutes | n | Min: price / bp | p10 | p25 | p50 median | p75 | p90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ADAUSD | 0 | 2 | 0.0042 / 59.669108 | 0.00422 / 60.866987 | 0.00425 / 62.663806 | 0.0043 / 65.658505 | 0.00435 / 68.653203 | 0.00438 / 70.450022 | 0.0044 / 71.647902 |
| ADAUSD | 30 | 2 | 0.0042 / 59.09213 | 0.00422 / 60.368546 | 0.00425 / 62.283169 | 0.0043 / 65.474209 | 0.00435 / 68.665248 | 0.00438 / 70.579872 | 0.0044 / 71.856287 |
| ADAUSD | 60 | 2 | 0.0042 / 59.307184 | 0.00422 / 60.563324 | 0.00425 / 62.447534 | 0.0043 / 65.587884 | 0.00435 / 68.728233 | 0.00438 / 70.612443 | 0.0044 / 71.868583 |
| ADAUSD | 180 | 2 | 0.0041 / 58.339963 | 0.00413 / 59.450999 | 0.004175 / 61.117554 | 0.00425 / 63.895144 | 0.004325 / 66.672735 | 0.00437 / 68.33929 | 0.0044 / 69.450326 |
| ADAUSD | 240 | 2 | 0.0041 / 57.871893 | 0.00413 / 59.060458 | 0.004175 / 60.843307 | 0.00425 / 63.814721 | 0.004325 / 66.786136 | 0.00437 / 68.568984 | 0.0044 / 69.75755 |
| AUDJPY | 0 | 2 | 0.005 / 0.52968627 | 0.0052 / 0.54917953 | 0.0055 / 0.57841943 | 0.006 / 0.6271526 | 0.0065 / 0.67588576 | 0.0068 / 0.70512566 | 0.007 / 0.72461893 |
| AUDJPY | 30 | 2 | 0.006 / 0.62060405 | 0.0064 / 0.6644579 | 0.007 / 0.73023867 | 0.008 / 0.83987329 | 0.009 / 0.9495079 | 0.0096 / 1.0152887 | 0.01 / 1.0591425 |
| AUDJPY | 60 | 2 | 0.006 / 0.63535765 | 0.0064 / 0.67506261 | 0.007 / 0.73462006 | 0.008 / 0.83388246 | 0.009 / 0.93314486 | 0.0096 / 0.9927023 | 0.01 / 1.0324073 |
| AUDJPY | 180 | 2 | 0.007 / 0.72355534 | 0.0071 / 0.73586485 | 0.00725 / 0.75432912 | 0.0075 / 0.78510289 | 0.00775 / 0.81587667 | 0.0079 / 0.83434093 | 0.008 / 0.84665044 |
| AUDJPY | 240 | 2 | 0.007 / 0.74082305 | 0.0074 / 0.78050463 | 0.008 / 0.84002699 | 0.009 / 0.93923092 | 0.01 / 1.0384348 | 0.0106 / 1.0979572 | 0.011 / 1.1376388 |
| AUDUSD | 0 | 4 | 1e-05 / 0.15103345 | 1e-05 / 0.15134489 | 1e-05 / 0.15181206 | 1e-05 / 0.15225473 | 2e-05 / 0.30635986 | 3.8e-05 / 0.58341945 | 5e-05 / 0.76812585 |
| AUDUSD | 30 | 4 | 0 / 0 | 3e-06 / 0.045704187 | 7.5e-06 / 0.11426047 | 2e-05 / 0.3040731 | 3.5e-05 / 0.53389539 | 4.4e-05 / 0.67446907 | 5e-05 / 0.76818486 |
| AUDUSD | 60 | 4 | 1e-05 / 0.15055139 | 1e-05 / 0.15103244 | 1e-05 / 0.15175402 | 1e-05 / 0.1529479 | 1.25e-05 / 0.19123467 | 1.7e-05 / 0.25872345 | 2e-05 / 0.30371596 |
| AUDUSD | 180 | 4 | 0 / 0 | 3e-06 / 0.045090745 | 7.5e-06 / 0.11272686 | 1e-05 / 0.15111311 | 1e-05 / 0.15200179 | 1e-05 / 0.1521423 | 1e-05 / 0.15223597 |
| AUDUSD | 240 | 4 | 0 / 0 | 3e-06 / 0.045077872 | 7.5e-06 / 0.11269468 | 1e-05 / 0.15125356 | 1e-05 / 0.15266526 | 1e-05 / 0.15341713 | 1e-05 / 0.15391838 |
| AUS200 | 0 | 2 | 1.8 / 2.0896701 | 1.8 / 2.0906793 | 1.8 / 2.0921931 | 1.8 / 2.094716 | 1.8 / 2.097239 | 1.8 / 2.0987528 | 1.8 / 2.099762 |
| AUS200 | 30 | 1 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 |
| AUS200 | 60 | 2 | 1 / 1.1607257 | 1 / 1.1612658 | 1 / 1.1620759 | 1 / 1.163426 | 1 / 1.1647762 | 1 / 1.1655863 | 1 / 1.1661264 |
| AUS200 | 180 | 0 | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified |
| AUS200 | 240 | 1 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 |
| AVAXUSD | 0 | 1 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 |
| AVAXUSD | 30 | 1 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 |
| AVAXUSD | 60 | 1 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 |
| AVAXUSD | 180 | 2 | 0.051 / 25.899547 | 0.0515 / 26.067616 | 0.05225 / 26.31972 | 0.0535 / 26.739893 | 0.05475 / 27.160066 | 0.0555 / 27.41217 | 0.056 / 27.58024 |
| AVAXUSD | 240 | 2 | 0.052 / 25.685717 | 0.0524 / 25.938785 | 0.053 / 26.318387 | 0.054 / 26.951057 | 0.055 / 27.583728 | 0.0556 / 27.96333 | 0.056 / 28.216398 |
| BCHUSD | 0 | 2 | 1.74 / 34.226366 | 1.74 / 34.344345 | 1.74 / 34.521313 | 1.74 / 34.81626 | 1.74 / 35.111207 | 1.74 / 35.288175 | 1.74 / 35.406153 |
| BCHUSD | 30 | 2 | 1.7 / 33.454689 | 1.706 / 33.66456 | 1.715 / 33.979367 | 1.73 / 34.504045 | 1.745 / 35.028723 | 1.754 / 35.34353 | 1.76 / 35.553401 |
| BCHUSD | 60 | 1 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 |
| BCHUSD | 180 | 1 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 |
| BCHUSD | 240 | 2 | 1.67 / 33.58708 | 1.677 / 33.607603 | 1.6875 / 33.638388 | 1.705 / 33.689697 | 1.7225 / 33.741005 | 1.733 / 33.77179 | 1.74 / 33.792313 |
| BTCUSD | 0 | 29 | 10 / 1.2025232 | 15 / 1.2288486 | 15 / 1.2873551 | 17 / 2.5163004 | 30.85 / 3.2159095 | 33.05 / 4.7416155 | 40.73 / 5.8206481 |
| BTCUSD | 30 | 29 | 14 / 1.2183302 | 15 / 1.2697566 | 15 / 1.2948846 | 18 / 2.6381744 | 31 / 3.3682354 | 34.032 / 4.8937615 | 64.38 / 6.1660619 |
| BTCUSD | 60 | 28 | 15 / 1.2002544 | 15 / 1.2308634 | 15 / 1.2930435 | 17 / 2.5725725 | 28.0325 / 3.1857263 | 33.349 / 3.4522574 | 33.61 / 5.7747593 |
| BTCUSD | 180 | 29 | 11 / 1.2063327 | 15 / 1.2321142 | 15 / 1.2850582 | 16.77 / 2.6565568 | 23 / 3.2072982 | 35.034 / 3.6428168 | 39.57 / 5.7875467 |
| BTCUSD | 240 | 27 | 11 / 1.2059259 | 15 / 1.2326203 | 15 / 1.2850351 | 16.77 / 2.646159 | 27.485 / 3.2772282 | 34.316 / 3.5165149 | 36.4 / 5.8093278 |
| CADJPY | 0 | 2 | 0.007 / 0.65859728 | 0.0072 / 0.67642851 | 0.0075 / 0.70317535 | 0.008 / 0.74775343 | 0.0085 / 0.7923315 | 0.0088 / 0.81907834 | 0.009 / 0.83690957 |
| CADJPY | 30 | 2 | 0.007 / 0.65889485 | 0.0071 / 0.66735135 | 0.00725 / 0.6800361 | 0.0075 / 0.70117736 | 0.00775 / 0.72231862 | 0.0079 / 0.73500337 | 0.008 / 0.74345988 |
| CADJPY | 60 | 2 | 0.008 / 0.74252142 | 0.0082 / 0.76238692 | 0.0085 / 0.79218518 | 0.009 / 0.84184894 | 0.0095 / 0.89151271 | 0.0098 / 0.92131097 | 0.01 / 0.94117647 |
| CADJPY | 180 | 2 | 0.008 / 0.74301795 | 0.008 / 0.74393453 | 0.008 / 0.74530939 | 0.008 / 0.74760082 | 0.008 / 0.74989225 | 0.008 / 0.75126711 | 0.008 / 0.75218368 |
| CADJPY | 240 | 2 | 0.007 / 0.65014373 | 0.0072 / 0.66968046 | 0.0075 / 0.69898556 | 0.008 / 0.74782738 | 0.0085 / 0.79666921 | 0.0088 / 0.82597431 | 0.009 / 0.84551104 |
| CHFJPY | 0 | 2 | 0.009 / 0.49347922 | 0.0097 / 0.5308282 | 0.01075 / 0.58685167 | 0.0125 / 0.68022412 | 0.01425 / 0.77359657 | 0.0153 / 0.82962004 | 0.016 / 0.86696902 |
| CHFJPY | 30 | 2 | 0.008 / 0.43872638 | 0.0082 / 0.44898592 | 0.0085 / 0.46437523 | 0.009 / 0.49002409 | 0.0095 / 0.51567294 | 0.0098 / 0.53106226 | 0.01 / 0.5413218 |
| CHFJPY | 60 | 2 | 0.009 / 0.49364162 | 0.0094 / 0.51463425 | 0.01 / 0.54612319 | 0.011 / 0.59860476 | 0.012 / 0.65108633 | 0.0126 / 0.68257527 | 0.013 / 0.7035679 |
| CHFJPY | 180 | 2 | 0.011 / 0.59521072 | 0.011 / 0.59599352 | 0.011 / 0.59716773 | 0.011 / 0.59912474 | 0.011 / 0.60108175 | 0.011 / 0.60225596 | 0.011 / 0.60303877 |
| CHFJPY | 240 | 2 | 0.009 / 0.4868378 | 0.0095 / 0.51489789 | 0.01025 / 0.55698802 | 0.0115 / 0.62713824 | 0.01275 / 0.69728845 | 0.0135 / 0.73937859 | 0.014 / 0.76743867 |
| DOGEUSD | 0 | 2 | 0.00118 / 59.391987 | 0.001187 / 60.719174 | 0.0011975 / 62.709955 | 0.001215 / 66.027922 | 0.0012325 / 69.34589 | 0.001243 / 71.33667 | 0.00125 / 72.663857 |
| DOGEUSD | 30 | 2 | 0.00117 / 58.706942 | 0.001178 / 60.12679 | 0.00119 / 62.256563 | 0.00121 / 65.806185 | 0.00123 / 69.355806 | 0.001242 / 71.485579 | 0.00125 / 72.905427 |
| DOGEUSD | 60 | 2 | 0.00117 / 58.848678 | 0.001178 / 60.264146 | 0.00119 / 62.387348 | 0.00121 / 65.926018 | 0.00123 / 69.464688 | 0.001242 / 71.58789 | 0.00125 / 73.003358 |
| DOGEUSD | 180 | 2 | 0.00118 / 58.607331 | 0.001184 / 59.793326 | 0.00119 / 61.572318 | 0.0012 / 64.537305 | 0.00121 / 67.502292 | 0.001216 / 69.281284 | 0.00122 / 70.467279 |
| DOGEUSD | 240 | 2 | 0.00119 / 58.143796 | 0.001193 / 59.297659 | 0.0011975 / 61.028455 | 0.001205 / 63.913113 | 0.0012125 / 66.797772 | 0.001217 / 68.528567 | 0.00122 / 69.682431 |
| DOTUSD | 0 | 2 | 0.03 / 75.301205 | 0.03 / 76.607609 | 0.03 / 78.567214 | 0.03 / 81.833224 | 0.03 / 85.099233 | 0.03 / 87.058839 | 0.03 / 88.365243 |
| DOTUSD | 30 | 2 | 0.03 / 74.812968 | 0.03 / 76.178619 | 0.03 / 78.227095 | 0.03 / 81.641223 | 0.03 / 85.05535 | 0.03 / 87.103827 | 0.03 / 88.469478 |
| DOTUSD | 60 | 1 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 |
| DOTUSD | 180 | 1 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 |
| DOTUSD | 240 | 2 | 0.03 / 72.393822 | 0.03 / 73.936642 | 0.03 / 76.25087 | 0.03 / 80.107918 | 0.03 / 83.964966 | 0.03 / 86.279195 | 0.03 / 87.822014 |
| ETHUSD | 0 | 2 | 3 / 10.087017 | 3 / 10.245032 | 3 / 10.482053 | 3 / 10.877088 | 3 / 11.272124 | 3 / 11.509145 | 3 / 11.667159 |
| ETHUSD | 30 | 2 | 3 / 10.264254 | 3.006 / 10.407337 | 3.015 / 10.621962 | 3.03 / 10.97967 | 3.045 / 11.337378 | 3.054 / 11.552003 | 3.06 / 11.695086 |
| ETHUSD | 60 | 2 | 3 / 10.076616 | 3 / 10.239805 | 3 / 10.484588 | 3 / 10.89256 | 3 / 11.300533 | 3 / 11.545316 | 3 / 11.708505 |
| ETHUSD | 180 | 2 | 3 / 9.9934709 | 3 / 10.158047 | 3 / 10.404911 | 3 / 10.816351 | 3 / 11.227791 | 3 / 11.474654 | 3 / 11.63923 |
| ETHUSD | 240 | 2 | 3 / 9.8704666 | 3 / 10.047253 | 3 / 10.312432 | 3 / 10.754397 | 3 / 11.196362 | 3 / 11.461541 | 3 / 11.638327 |
| EURAUD | 0 | 2 | 4e-05 / 0.22496935 | 5.4e-05 / 0.3021297 | 7.5e-05 / 0.41787024 | 0.00011 / 0.61077112 | 0.000145 / 0.80367201 | 0.000166 / 0.91941254 | 0.00018 / 0.9965729 |
| EURAUD | 30 | 2 | 5e-05 / 0.27701061 | 5.1e-05 / 0.28303162 | 5.25e-05 / 0.29206314 | 5.5e-05 / 0.30711568 | 5.75e-05 / 0.32216821 | 5.9e-05 / 0.33119973 | 6e-05 / 0.33722074 |
| EURAUD | 60 | 2 | 7e-05 / 0.38765163 | 7.1e-05 / 0.39388161 | 7.25e-05 / 0.40322656 | 7.5e-05 / 0.41880149 | 7.75e-05 / 0.43437642 | 7.9e-05 / 0.44372138 | 8e-05 / 0.44995135 |
| EURAUD | 180 | 2 | 4e-05 / 0.22147536 | 4.4e-05 / 0.24427442 | 5e-05 / 0.27847301 | 6e-05 / 0.33547067 | 7e-05 / 0.39246832 | 7.6e-05 / 0.42666692 | 8e-05 / 0.44946598 |
| EURAUD | 240 | 2 | 4e-05 / 0.22159314 | 4e-05 / 0.22189943 | 4e-05 / 0.22235886 | 4e-05 / 0.22312457 | 4e-05 / 0.22389028 | 4e-05 / 0.22434971 | 4e-05 / 0.224656 |
| EURGBP | 0 | 2 | 3e-05 / 0.34611458 | 3.1e-05 / 0.35782193 | 3.25e-05 / 0.37538296 | 3.5e-05 / 0.40465135 | 3.75e-05 / 0.43391974 | 3.9e-05 / 0.45148077 | 4e-05 / 0.46318812 |
| EURGBP | 30 | 2 | 1e-05 / 0.11543812 | 1e-05 / 0.11547984 | 1e-05 / 0.11554243 | 1e-05 / 0.11564675 | 1e-05 / 0.11575107 | 1e-05 / 0.11581366 | 1e-05 / 0.11585539 |
| EURGBP | 60 | 2 | 1e-05 / 0.11538484 | 1.2e-05 / 0.13859532 | 1.5e-05 / 0.17341105 | 2e-05 / 0.23143726 | 2.5e-05 / 0.28946347 | 2.8e-05 / 0.32427919 | 3e-05 / 0.34748968 |
| EURGBP | 180 | 2 | 1e-05 / 0.11598304 | 1.2e-05 / 0.13898422 | 1.5e-05 / 0.17348599 | 2e-05 / 0.23098893 | 2.5e-05 / 0.28849188 | 2.8e-05 / 0.32299364 | 3e-05 / 0.34599482 |
| EURGBP | 240 | 2 | 2e-05 / 0.23067253 | 2e-05 / 0.23080255 | 2e-05 / 0.2309976 | 2e-05 / 0.23132267 | 2e-05 / 0.23164774 | 2e-05 / 0.23184278 | 2e-05 / 0.23197281 |
| EURJPY | 0 | 2 | 0.009 / 0.52397323 | 0.0095 / 0.55369072 | 0.01025 / 0.59826694 | 0.0115 / 0.67256066 | 0.01275 / 0.74685437 | 0.0135 / 0.7914306 | 0.014 / 0.82114808 |
| EURJPY | 30 | 2 | 0.007 / 0.40693771 | 0.007 / 0.40731832 | 0.007 / 0.40788925 | 0.007 / 0.4088408 | 0.007 / 0.40979234 | 0.007 / 0.41036327 | 0.007 / 0.41074389 |
| EURJPY | 60 | 2 | 0.009 / 0.52772769 | 0.0095 / 0.55624818 | 0.01025 / 0.59902892 | 0.0115 / 0.67033014 | 0.01275 / 0.74163137 | 0.0135 / 0.78441211 | 0.014 / 0.8129326 |
| EURJPY | 180 | 2 | 0.014 / 0.81303174 | 0.0142 / 0.82548607 | 0.0145 / 0.84416757 | 0.015 / 0.87530341 | 0.0155 / 0.90643924 | 0.0158 / 0.92512075 | 0.016 / 0.93757508 |
| EURJPY | 240 | 2 | 0.011 / 0.64493622 | 0.0112 / 0.65595748 | 0.0115 / 0.67248937 | 0.012 / 0.70004252 | 0.0125 / 0.72759566 | 0.0128 / 0.74412755 | 0.013 / 0.75514881 |
| EURUSD | 0 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| EURUSD | 30 | 4 | 0 / 0 | 3e-06 / 0.025371159 | 7.5e-06 / 0.063427897 | 1e-05 / 0.084848056 | 1e-05 / 0.085160966 | 1e-05 / 0.085224659 | 1e-05 / 0.085267121 |
| EURUSD | 60 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| EURUSD | 180 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| EURUSD | 240 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 2.5e-06 / 0.021376931 | 7e-06 / 0.059855406 | 1e-05 / 0.085507723 |
| GBPJPY | 0 | 2 | 0.006 / 0.30392519 | 0.0064 / 0.32399593 | 0.007 / 0.35410203 | 0.008 / 0.40427886 | 0.009 / 0.45445569 | 0.0096 / 0.48456179 | 0.01 / 0.50463253 |
| GBPJPY | 30 | 2 | 0.006 / 0.30387748 | 0.0061 / 0.30874027 | 0.00625 / 0.31603447 | 0.0065 / 0.32819145 | 0.00675 / 0.34034844 | 0.0069 / 0.34764264 | 0.007 / 0.35250543 |
| GBPJPY | 60 | 2 | 0.005 / 0.25312162 | 0.0052 / 0.26303659 | 0.0055 / 0.27790903 | 0.006 / 0.30269645 | 0.0065 / 0.32748386 | 0.0068 / 0.3423563 | 0.007 / 0.35227127 |
| GBPJPY | 180 | 2 | 0.005 / 0.25177565 | 0.005 / 0.2518596 | 0.005 / 0.25198554 | 0.005 / 0.25219542 | 0.005 / 0.25240531 | 0.005 / 0.25253124 | 0.005 / 0.2526152 |
| GBPJPY | 240 | 2 | 0.005 / 0.25181242 | 0.0052 / 0.26201554 | 0.0055 / 0.27732022 | 0.006 / 0.30282802 | 0.0065 / 0.32833582 | 0.0068 / 0.34364051 | 0.007 / 0.35384363 |
| GBPUSD | 0 | 4 | 1e-05 / 0.073974619 | 1.3e-05 / 0.095915457 | 1.75e-05 / 0.12882671 | 2e-05 / 0.14786669 | 2.25e-05 / 0.16656198 | 2.7e-05 / 0.1988528 | 3e-05 / 0.22038001 |
| GBPUSD | 30 | 4 | 1e-05 / 0.07342117 | 1e-05 / 0.073589996 | 1e-05 / 0.073843235 | 1.5e-05 / 0.11046627 | 2e-05 / 0.14729849 | 2e-05 / 0.14792828 | 2e-05 / 0.14834814 |
| GBPUSD | 60 | 4 | 0 / 0 | 3e-06 / 0.022005347 | 7.5e-06 / 0.055013368 | 1e-05 / 0.073427719 | 1.5e-05 / 0.11075196 | 2.4e-05 / 0.17779777 | 3e-05 / 0.22249498 |
| GBPUSD | 180 | 4 | 1e-05 / 0.073284501 | 1.3e-05 / 0.09538503 | 1.75e-05 / 0.12853582 | 2.5e-05 / 0.18451314 | 3e-05 / 0.22217876 | 3e-05 / 0.22236849 | 3e-05 / 0.22249498 |
| GBPUSD | 240 | 4 | 0 / 0 | 3e-06 / 0.022173195 | 7.5e-06 / 0.055432987 | 1e-05 / 0.074024076 | 2.5e-05 / 0.18381342 | 5.2e-05 / 0.38123007 | 7e-05 / 0.51284118 |
| GER40 | 0 | 2 | 2 / 0.83355214 | 2 / 0.83410397 | 2 / 0.83493173 | 2 / 0.83631131 | 2 / 0.83769089 | 2 / 0.83851864 | 2 / 0.83907048 |
| GER40 | 30 | 2 | 0.9 / 0.37448586 | 0.9 / 0.37474885 | 0.9 / 0.37514334 | 0.9 / 0.37580082 | 0.9 / 0.3764583 | 0.9 / 0.37685279 | 0.9 / 0.37711578 |
| GER40 | 60 | 2 | 0.9 / 0.37369751 | 0.9 / 0.37404044 | 0.9 / 0.37455484 | 0.9 / 0.37541217 | 0.9 / 0.37626951 | 0.9 / 0.37678391 | 0.9 / 0.37712684 |
| GER40 | 180 | 2 | 0.9 / 0.37361529 | 0.9 / 0.37383621 | 0.9 / 0.37416758 | 0.9 / 0.37471988 | 0.9 / 0.37527217 | 0.9 / 0.37560355 | 0.9 / 0.37582446 |
| GER40 | 240 | 2 | 0.9 / 0.37459965 | 0.9 / 0.37468685 | 0.9 / 0.37481766 | 0.9 / 0.37503567 | 0.9 / 0.37525368 | 0.9 / 0.37538448 | 0.9 / 0.37547169 |
| JPN225 | 0 | 2 | 7 / 1.7619726 | 7 / 1.7634714 | 7 / 1.7657197 | 7 / 1.7694668 | 7 / 1.7732139 | 7 / 1.7754621 | 7 / 1.7769609 |
| JPN225 | 30 | 2 | 7 / 1.7630821 | 7 / 1.7644023 | 7 / 1.7663827 | 7 / 1.7696833 | 7 / 1.7729839 | 7 / 1.7749643 | 7 / 1.7762846 |
| JPN225 | 60 | 2 | 7 / 1.7646377 | 7 / 1.7659151 | 7 / 1.7678313 | 7 / 1.7710249 | 7 / 1.7742185 | 7 / 1.7761347 | 7 / 1.7774121 |
| JPN225 | 180 | 2 | 7 / 1.7670877 | 7 / 1.7679736 | 7 / 1.7693024 | 7 / 1.7715172 | 7 / 1.7737319 | 7 / 1.7750607 | 7 / 1.7759466 |
| JPN225 | 240 | 2 | 7 / 1.7663074 | 7 / 1.7667547 | 7 / 1.7674256 | 7 / 1.7685438 | 7 / 1.7696619 | 7 / 1.7703328 | 7 / 1.7707801 |
| LINKUSD | 0 | 0 | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified |
| LINKUSD | 30 | 2 | 0.037 / 23.537644 | 0.037 / 23.931031 | 0.037 / 24.52111 | 0.037 / 25.504576 | 0.037 / 26.488042 | 0.037 / 27.078121 | 0.037 / 27.471508 |
| LINKUSD | 60 | 0 | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified |
| LINKUSD | 180 | 1 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 |
| LINKUSD | 240 | 1 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 |
| LTCUSD | 0 | 2 | 0.31 / 32.743597 | 0.31 / 33.016756 | 0.31 / 33.426496 | 0.31 / 34.109396 | 0.31 / 34.792296 | 0.31 / 35.202036 | 0.31 / 35.475196 |
| LTCUSD | 30 | 1 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 |
| LTCUSD | 60 | 2 | 0.31 / 32.684907 | 0.31 / 32.96353 | 0.31 / 33.381464 | 0.31 / 34.078022 | 0.31 / 34.774579 | 0.31 / 35.192514 | 0.31 / 35.471137 |
| LTCUSD | 180 | 2 | 0.31 / 32.414911 | 0.311 / 32.819728 | 0.3125 / 33.426953 | 0.315 / 34.438996 | 0.3175 / 35.451039 | 0.319 / 36.058264 | 0.32 / 36.463081 |
| LTCUSD | 240 | 2 | 0.31 / 33.051023 | 0.311 / 33.280899 | 0.3125 / 33.625715 | 0.315 / 34.200407 | 0.3175 / 34.7751 | 0.319 / 35.119915 | 0.32 / 35.349792 |
| NAS100 | 0 | 7 | 1 / 0.56919084 | 1 / 0.57006617 | 1 / 0.57083647 | 1.3 / 0.57477541 | 1.6 / 0.82759651 | 1.9 / 1.0832251 | 1.9 / 1.0892091 |
| NAS100 | 30 | 7 | 1 / 0.43932678 | 1 / 0.43941829 | 1 / 0.50464156 | 1 / 0.56994346 | 1 / 0.57397896 | 1 / 0.57519104 | 1 / 0.57640874 |
| NAS100 | 60 | 7 | 1 / 0.43903553 | 1 / 0.43926935 | 1 / 0.50361963 | 1 / 0.56848221 | 1 / 0.57430914 | 1 / 0.57634641 | 1 / 0.57667134 |
| NAS100 | 180 | 7 | 1 / 0.43756208 | 1 / 0.43920585 | 1 / 0.50363613 | 1 / 0.56943393 | 1 / 0.57394963 | 1 / 0.57644985 | 1 / 0.57715394 |
| NAS100 | 240 | 7 | 1 / 0.43687391 | 1 / 0.43917624 | 1 / 0.50389551 | 1 / 0.57112506 | 1 / 0.57502452 | 1 / 0.57686507 | 1 / 0.57769408 |
| NZDUSD | 0 | 4 | 2e-05 / 0.33435311 | 2e-05 / 0.33547906 | 2e-05 / 0.33716798 | 2e-05 / 0.33865965 | 3.25e-05 / 0.54549185 | 5.5e-05 / 0.91679372 | 7e-05 / 1.1643283 |
| NZDUSD | 30 | 4 | 2e-05 / 0.33424136 | 2e-05 / 0.33549179 | 2e-05 / 0.33736745 | 3e-05 / 0.50684866 | 4.25e-05 / 0.714437 | 4.7e-05 / 0.78490548 | 5e-05 / 0.83188447 |
| NZDUSD | 60 | 4 | 2e-05 / 0.33387309 | 2e-05 / 0.33495648 | 2e-05 / 0.33658157 | 2e-05 / 0.33792689 | 2.25e-05 / 0.37866569 | 2.7e-05 / 0.45119901 | 3e-05 / 0.49955456 |
| NZDUSD | 180 | 4 | 2e-05 / 0.33343892 | 2e-05 / 0.33356416 | 2e-05 / 0.33375201 | 2e-05 / 0.3357701 | 2e-05 / 0.33791394 | 2e-05 / 0.33832814 | 2e-05 / 0.33860427 |
| NZDUSD | 240 | 4 | 2e-05 / 0.33343892 | 2e-05 / 0.33457574 | 2e-05 / 0.33628097 | 2e-05 / 0.3379593 | 2.25e-05 / 0.37915015 | 2.7e-05 / 0.45197789 | 3e-05 / 0.50052973 |
| NatGas | 0 | 8 | 0.004 / 13.020833 | 0.004 / 13.662348 | 0.00475 / 14.470112 | 0.005 / 15.908001 | 0.005 / 16.492304 | 0.005 / 16.624756 | 0.005 / 16.714023 |
| NatGas | 30 | 6 | 0.004 / 13.333333 | 0.0045 / 14.43909 | 0.005 / 15.732287 | 0.005 / 16.307904 | 0.005 / 16.421916 | 0.005 / 16.529255 | 0.005 / 16.603022 |
| NatGas | 60 | 8 | 0.004 / 11.719895 | 0.004 / 12.233278 | 0.004 / 12.818922 | 0.004 / 13.003923 | 0.004 / 13.293524 | 0.0043 / 14.264743 | 0.005 / 16.406891 |
| NatGas | 180 | 9 | 0.004 / 12.464942 | 0.004 / 12.86576 | 0.004 / 13.162224 | 0.004 / 13.34668 | 0.005 / 16.178612 | 0.005 / 17.193206 | 0.005 / 17.461149 |
| NatGas | 240 | 10 | 0.004 / 12.414649 | 0.004 / 12.865617 | 0.00425 / 13.952055 | 0.005 / 16.178749 | 0.005 / 16.614978 | 0.005 / 17.375048 | 0.005 / 17.473353 |
| SOLUSD | 0 | 2 | 0.37 / 22.949294 | 0.372 / 23.220914 | 0.375 / 23.628344 | 0.38 / 24.307394 | 0.385 / 24.986443 | 0.388 / 25.393873 | 0.39 / 25.665493 |
| SOLUSD | 30 | 2 | 0.38 / 23.519218 | 0.381 / 23.743169 | 0.3825 / 24.079095 | 0.385 / 24.638972 | 0.3875 / 25.198849 | 0.389 / 25.534776 | 0.39 / 25.758727 |
| SOLUSD | 60 | 2 | 0.38 / 23.491592 | 0.381 / 23.722566 | 0.3825 / 24.069027 | 0.385 / 24.646461 | 0.3875 / 25.223895 | 0.389 / 25.570356 | 0.39 / 25.80133 |
| SOLUSD | 180 | 2 | 0.37 / 23.037284 | 0.371 / 23.166246 | 0.3725 / 23.359688 | 0.375 / 23.682092 | 0.3775 / 24.004497 | 0.379 / 24.197939 | 0.38 / 24.326901 |
| SOLUSD | 240 | 2 | 0.37 / 22.732711 | 0.371 / 22.894692 | 0.3725 / 23.137663 | 0.375 / 23.542615 | 0.3775 / 23.947567 | 0.379 / 24.190538 | 0.38 / 24.352519 |
| SpotBrent | 0 | 46 | 0.02 / 2.4059282 | 0.02 / 2.6099165 | 0.02 / 2.8853631 | 0.03 / 3.696557 | 0.03 / 4.5740089 | 0.04 / 4.7880658 | 0.04 / 4.955647 |
| SpotBrent | 30 | 42 | 0.02 / 2.4174735 | 0.02 / 2.6543048 | 0.02 / 2.8981314 | 0.03 / 3.7339779 | 0.03 / 4.5662381 | 0.04 / 4.789174 | 0.04 / 4.9679567 |
| SpotBrent | 60 | 52 | 0.02 / 2.4050602 | 0.02 / 2.6344038 | 0.02 / 2.9046625 | 0.02 / 3.1999328 | 0.03 / 4.32915 | 0.04 / 4.6754409 | 0.04 / 4.9608711 |
| SpotBrent | 180 | 47 | 0.02 / 2.4157215 | 0.02 / 2.628007 | 0.02 / 3.0123884 | 0.03 / 3.6974044 | 0.03 / 4.3517842 | 0.04 / 4.7175521 | 0.04 / 4.898959 |
| SpotBrent | 240 | 50 | 0.02 / 2.4225393 | 0.02 / 2.655323 | 0.02 / 3.0138248 | 0.03 / 3.6787428 | 0.03 / 4.3471081 | 0.04 / 4.6636211 | 0.04 / 4.9329935 |
| SpotCrude | 0 | 46 | 0.02 / 2.4926156 | 0.02 / 2.7542843 | 0.02 / 2.9159429 | 0.02 / 3.2281529 | 0.03 / 3.7807325 | 0.03 / 4.291507 | 0.03 / 5.1600475 |
| SpotCrude | 30 | 45 | 0.02 / 2.5060458 | 0.02 / 2.7789437 | 0.02 / 2.9938327 | 0.03 / 3.7427017 | 0.03 / 4.1703737 | 0.03 / 4.831481 | 0.03 / 5.1547277 |
| SpotCrude | 60 | 52 | 0.02 / 2.4913736 | 0.02 / 2.7343931 | 0.02 / 3.0072213 | 0.02 / 3.3308412 | 0.03 / 3.910438 | 0.03 / 4.2150141 | 0.03 / 4.8567266 |
| SpotCrude | 180 | 49 | 0.02 / 2.5022833 | 0.02 / 2.7831543 | 0.02 / 2.9724749 | 0.02 / 3.2991323 | 0.03 / 3.8118957 | 0.03 / 4.1746639 | 0.03 / 5.2531125 |
| SpotCrude | 240 | 50 | 0.02 / 2.5082459 | 0.02 / 2.784729 | 0.02 / 3.0611365 | 0.03 / 3.7036572 | 0.03 / 4.0883196 | 0.03 / 4.7685651 | 0.03 / 5.2302167 |
| UK100 | 0 | 2 | 1.8 / 2.0425764 | 1.81 / 2.0510811 | 1.825 / 2.0638382 | 1.85 / 2.0851001 | 1.875 / 2.106362 | 1.89 / 2.1191191 | 1.9 / 2.1276238 |
| UK100 | 30 | 2 | 1.1 / 1.2282337 | 1.11 / 1.2416888 | 1.125 / 1.2618715 | 1.15 / 1.2955092 | 1.175 / 1.3291469 | 1.19 / 1.3493295 | 1.2 / 1.3627846 |
| UK100 | 60 | 2 | 1.1 / 1.2273293 | 1.1 / 1.229356 | 1.1 / 1.2323962 | 1.1 / 1.2374631 | 1.1 / 1.24253 | 1.1 / 1.2455702 | 1.1 / 1.247597 |
| UK100 | 180 | 2 | 1 / 1.1336198 | 1.01 / 1.1428211 | 1.025 / 1.1566232 | 1.05 / 1.1796267 | 1.075 / 1.2026301 | 1.09 / 1.2164322 | 1.1 / 1.2256336 |
| UK100 | 240 | 2 | 1.1 / 1.225224 | 1.1 / 1.2272142 | 1.1 / 1.2301994 | 1.1 / 1.2351748 | 1.1 / 1.2401502 | 1.1 / 1.2431355 | 1.1 / 1.2451256 |
| US30 | 0 | 2 | 2.6 / 0.58070193 | 2.6 / 0.58128674 | 2.6 / 0.58216394 | 2.6 / 0.58362595 | 2.6 / 0.58508795 | 2.6 / 0.58596516 | 2.6 / 0.58654996 |
| US30 | 30 | 2 | 2 / 0.44664392 | 2 / 0.44706163 | 2 / 0.4476882 | 2 / 0.44873249 | 2 / 0.44977677 | 2 / 0.45040334 | 2 / 0.45082106 |
| US30 | 60 | 2 | 2 / 0.44779394 | 2 / 0.4481292 | 2 / 0.44863208 | 2 / 0.44947021 | 2 / 0.45030834 | 2 / 0.45081122 | 2 / 0.45114648 |
| US30 | 180 | 2 | 2 / 0.45044436 | 2 / 0.45046781 | 2 / 0.45050298 | 2 / 0.4505616 | 2 / 0.45062022 | 2 / 0.45065539 | 2 / 0.45067883 |
| US30 | 240 | 2 | 2 / 0.44994882 | 2 / 0.4500334 | 2 / 0.45016027 | 2 / 0.45037173 | 2 / 0.45058318 | 2 / 0.45071006 | 2 / 0.45079464 |
| US500 | 0 | 51 | 0.4 / 0.59160221 | 0.4 / 0.62751004 | 0.4 / 0.65758165 | 0.4 / 0.6981412 | 0.4 / 0.81762806 | 0.4 / 0.87892771 | 0.4 / 1.0073029 |
| US500 | 30 | 50 | 0.4 / 0.59223287 | 0.4 / 0.62405261 | 0.4 / 0.65566627 | 0.4 / 0.69181837 | 0.4 / 0.81787472 | 0.4 / 0.88970269 | 0.4 / 1.0111479 |
| US500 | 60 | 51 | 0.4 / 0.5926804 | 0.4 / 0.62713618 | 0.4 / 0.65656752 | 0.4 / 0.69449267 | 0.4 / 0.8166682 | 0.4 / 0.87742388 | 0.4 / 1.0098205 |
| US500 | 180 | 47 | 0.4 / 0.5930055 | 0.4 / 0.62962695 | 0.4 / 0.65933913 | 0.4 / 0.6995453 | 0.4 / 0.81778092 | 0.4 / 0.91884294 | 0.4 / 1.0049241 |
| US500 | 240 | 50 | 0.4 / 0.59200497 | 0.4 / 0.62477531 | 0.4 / 0.65887354 | 0.4 / 0.69584086 | 0.4 / 0.81938762 | 0.4 / 0.88766211 | 0.4 / 0.99972508 |
| USDCAD | 0 | 4 | 2e-05 / 0.14464872 | 2e-05 / 0.1447456 | 2e-05 / 0.14489092 | 2.5e-05 / 0.18202405 | 3.75e-05 / 0.2742353 | 5.1e-05 / 0.37352122 | 6e-05 / 0.43971184 |
| USDCAD | 30 | 4 | 2e-05 / 0.14509576 | 2e-05 / 0.14539849 | 2e-05 / 0.14585257 | 3e-05 / 0.21951356 | 4.25e-05 / 0.31014654 | 4.7e-05 / 0.34115024 | 5e-05 / 0.36181937 |
| USDCAD | 60 | 4 | 1e-05 / 0.072562358 | 1e-05 / 0.07272564 | 1e-05 / 0.072970562 | 1.5e-05 / 0.10893526 | 2.25e-05 / 0.16345103 | 2.7e-05 / 0.19708789 | 3e-05 / 0.21951246 |
| USDCAD | 180 | 4 | 1e-05 / 0.072370159 | 1.3e-05 / 0.094212802 | 1.75e-05 / 0.12697677 | 2e-05 / 0.1456873 | 2e-05 / 0.14621059 | 2e-05 / 0.14623753 | 2e-05 / 0.14625549 |
| USDCAD | 240 | 4 | 1e-05 / 0.07318153 | 1.3e-05 / 0.0947893 | 1.75e-05 / 0.12720095 | 2.5e-05 / 0.18124623 | 3e-05 / 0.2178162 | 3e-05 / 0.21877231 | 3e-05 / 0.21940971 |
| USDCHF | 0 | 4 | 0 / 0 | 3e-06 / 0.03766502 | 7.5e-06 / 0.09416255 | 2e-05 / 0.25076263 | 3.5e-05 / 0.43918147 | 4.4e-05 / 0.55295277 | 5e-05 / 0.62880031 |
| USDCHF | 30 | 4 | 1e-05 / 0.12666004 | 1.3e-05 / 0.16391507 | 1.75e-05 / 0.21979761 | 2.5e-05 / 0.31377994 | 3.25e-05 / 0.40823651 | 3.7e-05 / 0.46497269 | 4e-05 / 0.50279681 |
| USDCHF | 60 | 4 | 0 / 0 | 3e-06 / 0.037612603 | 7.5e-06 / 0.094031507 | 1e-05 / 0.12551319 | 1.5e-05 / 0.18841146 | 2.4e-05 / 0.30138024 | 3e-05 / 0.37669276 |
| USDCHF | 180 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 5e-06 / 0.062737225 | 1.25e-05 / 0.15681828 | 1.7e-05 / 0.21323716 | 2e-05 / 0.25084975 |
| USDCHF | 240 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 5e-06 / 0.0627908 | 1.25e-05 / 0.15704686 | 1.7e-05 / 0.21368434 | 2e-05 / 0.25144265 |
| USDJPY | 0 | 4 | 0.001 / 0.067836852 | 0.001 / 0.068171957 | 0.001 / 0.068674613 | 0.0015 / 0.10238496 | 0.00225 / 0.15259616 | 0.0027 / 0.18280036 | 0.003 / 0.20293649 |
| USDJPY | 30 | 4 | 0.001 / 0.067959021 | 0.001 / 0.068251343 | 0.001 / 0.068689826 | 0.0015 / 0.10219995 | 0.002 / 0.13554673 | 0.002 / 0.13569117 | 0.002 / 0.13578747 |
| USDJPY | 60 | 4 | 0 / 0 | 0.0003 / 0.020334503 | 0.00075 / 0.050836256 | 0.0015 / 0.10276136 | 0.00225 / 0.15419654 | 0.0027 / 0.18381644 | 0.003 / 0.20356303 |
| USDJPY | 180 | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 0.001 / 0.067895116 | 0.002 / 0.13579254 | 0.002 / 0.13579669 | 0.002 / 0.13579945 |
| USDJPY | 240 | 4 | 0.001 / 0.067907795 | 0.0013 / 0.088294941 | 0.00175 / 0.11887566 | 0.002 / 0.13669219 | 0.00225 / 0.15388193 | 0.0027 / 0.18333444 | 0.003 / 0.20296944 |
| XAGUSD | 0 | 6 | 0.02 / 5.4613473 | 0.02 / 6.1973232 | 0.02025 / 7.4250644 | 0.0215 / 9.1936918 | 0.02275 / 9.6127265 | 0.025 / 9.958837 | 0.027 / 10.263046 |
| XAGUSD | 30 | 6 | 0.017 / 5.1259707 | 0.0185 / 5.4340735 | 0.02 / 6.1700543 | 0.0205 / 8.2059269 | 0.021 / 9.2328588 | 0.022 / 9.8492846 | 0.023 / 10.374146 |
| XAGUSD | 60 | 7 | 0.018 / 4.874357 | 0.018 / 4.9093767 | 0.0185 / 6.4152888 | 0.019 / 8.4169491 | 0.02 / 8.9909925 | 0.0208 / 9.2744459 | 0.022 / 9.6347552 |
| XAGUSD | 180 | 7 | 0.017 / 4.6423354 | 0.0176 / 4.9292094 | 0.018 / 6.489674 | 0.018 / 7.8829815 | 0.019 / 8.1089603 | 0.02 / 8.4010184 | 0.02 / 8.7919817 |
| XAGUSD | 240 | 7 | 0.017 / 4.6903668 | 0.017 / 5.5656164 | 0.0175 / 6.8109039 | 0.018 / 7.8564881 | 0.0205 / 8.2038634 | 0.0228 / 8.9212101 | 0.024 / 9.8898629 |
| XAUUSD | 0 | 42 | 0.07 / 0.2627626 | 0.12 / 0.3646983 | 0.15 / 0.51413313 | 0.17 / 0.67006297 | 0.19 / 0.85951231 | 0.247 / 1.0590035 | 0.29 / 1.4998668 |
| XAUUSD | 30 | 42 | 0.06 / 0.26597749 | 0.12 / 0.36302065 | 0.14 / 0.51380564 | 0.155 / 0.60473712 | 0.17 / 0.83676581 | 0.22 / 1.0088849 | 0.32 / 1.2002094 |
| XAUUSD | 60 | 42 | 0.11 / 0.32795243 | 0.13 / 0.41652955 | 0.14 / 0.53093238 | 0.17 / 0.66477338 | 0.18 / 0.83214952 | 0.229 / 0.9355908 | 0.35 / 1.197689 |
| XAUUSD | 180 | 42 | 0.09 / 0.33241516 | 0.121 / 0.40855009 | 0.1325 / 0.47918976 | 0.155 / 0.60597886 | 0.1775 / 0.81763791 | 0.2 / 0.97736766 | 0.23 / 1.187486 |
| XAUUSD | 240 | 42 | 0.06 / 0.29512599 | 0.111 / 0.35734031 | 0.13 / 0.47931461 | 0.16 / 0.61878399 | 0.18 / 0.76563971 | 0.209 / 0.85728891 | 0.23 / 1.0718524 |
| XLMUSD | 0 | 1 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 |
| XLMUSD | 30 | 1 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 |
| XLMUSD | 60 | 2 | 0.0016 / 44.070513 | 0.00166 / 45.97261 | 0.00175 / 48.825755 | 0.0019 / 53.580998 | 0.00205 / 58.33624 | 0.00214 / 61.189386 | 0.0022 / 63.091483 |
| XLMUSD | 180 | 1 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 |
| XLMUSD | 240 | 2 | 0.0016 / 42.0042 | 0.00165 / 44.175744 | 0.001725 / 47.433059 | 0.00185 / 52.861917 | 0.001975 / 58.290775 | 0.00205 / 61.54809 | 0.0021 / 63.719634 |
| XPDUSD | 0 | 2 | 4.09 / 36.921188 | 4.537 / 40.318214 | 5.2075 / 45.413752 | 6.325 / 53.906316 | 7.4425 / 62.39888 | 8.113 / 67.494418 | 8.56 / 70.891443 |
| XPDUSD | 30 | 2 | 3.85 / 34.674532 | 4.67 / 41.030942 | 5.9 / 50.565558 | 7.95 / 66.456583 | 10 / 82.347609 | 11.23 / 91.882224 | 12.05 / 98.238634 |
| XPDUSD | 60 | 2 | 3.93 / 35.303153 | 4.733 / 41.461756 | 5.9375 / 50.699659 | 7.945 / 66.096165 | 9.9525 / 81.492671 | 11.157 / 90.730575 | 11.96 / 96.889177 |
| XPDUSD | 180 | 2 | 3.85 / 34.214467 | 4.661 / 40.492546 | 5.8775 / 49.909664 | 7.905 / 65.604861 | 9.9325 / 81.300058 | 11.149 / 90.717177 | 11.96 / 96.995256 |
| XPDUSD | 240 | 2 | 4.42 / 39.361314 | 5.079 / 44.370039 | 6.0675 / 51.883126 | 7.715 / 64.404938 | 9.3625 / 76.926751 | 10.351 / 84.439838 | 11.01 / 89.448563 |
| XPTUSD | 0 | 2 | 3.95 / 29.196649 | 5.442 / 39.822071 | 7.68 / 55.760205 | 11.41 / 82.323762 | 15.14 / 108.88732 | 17.378 / 124.82545 | 18.87 / 135.45087 |
| XPTUSD | 30 | 2 | 2.6 / 19.149469 | 3.096 / 22.625435 | 3.84 / 27.839383 | 5.08 / 36.529297 | 6.32 / 45.219211 | 7.064 / 50.433159 | 7.56 / 53.909125 |
| XPTUSD | 60 | 1 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 |
| XPTUSD | 180 | 2 | 2.54 / 18.514604 | 3.03 / 21.997242 | 3.765 / 27.221198 | 4.99 / 35.927792 | 6.215 / 44.634387 | 6.95 / 49.858343 | 7.44 / 53.340981 |
| XPTUSD | 240 | 2 | 2.5 / 18.285815 | 2.997 / 21.822938 | 3.7425 / 27.128623 | 4.985 / 35.971431 | 6.2275 / 44.814239 | 6.973 / 50.119924 | 7.47 / 53.657047 |
| XRPUSD | 0 | 2 | 0.0063 / 26.434979 | 0.00642 / 26.564918 | 0.0066 / 26.759826 | 0.0069 / 27.084673 | 0.0072 / 27.409519 | 0.00738 / 27.604428 | 0.0075 / 27.734366 |
| XRPUSD | 30 | 2 | 0.0063 / 26.408916 | 0.00642 / 26.550648 | 0.0066 / 26.763247 | 0.0069 / 27.117578 | 0.0072 / 27.471909 | 0.00738 / 27.684508 | 0.0075 / 27.826241 |
| XRPUSD | 60 | 2 | 0.0063 / 26.341206 | 0.00642 / 26.489341 | 0.0066 / 26.711543 | 0.0069 / 27.08188 | 0.0072 / 27.452217 | 0.00738 / 27.674419 | 0.0075 / 27.822554 |
| XRPUSD | 180 | 2 | 0.0059 / 25.950606 | 0.00608 / 26.021981 | 0.00635 / 26.129044 | 0.0068 / 26.307482 | 0.00725 / 26.48592 | 0.00752 / 26.592983 | 0.0077 / 26.664358 |
| XRPUSD | 240 | 2 | 0.0059 / 25.976885 | 0.00608 / 26.009023 | 0.00635 / 26.05723 | 0.0068 / 26.137574 | 0.00725 / 26.217919 | 0.00752 / 26.266125 | 0.0077 / 26.298263 |

## §7. Complete spread quantiles — registered experiment-window anchor

The alternative anchor is reported separately; it does not redefine the coded session open.

| Symbol | Minutes | n | Min: price / bp | p10 | p25 | p50 median | p75 | p90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ADAUSD | 0 | 2 | 0.0042 / 59.669108 | 0.00422 / 60.866987 | 0.00425 / 62.663806 | 0.0043 / 65.658505 | 0.00435 / 68.653203 | 0.00438 / 70.450022 | 0.0044 / 71.647902 |
| ADAUSD | 30 | 2 | 0.0042 / 59.09213 | 0.00422 / 60.368546 | 0.00425 / 62.283169 | 0.0043 / 65.474209 | 0.00435 / 68.665248 | 0.00438 / 70.579872 | 0.0044 / 71.856287 |
| ADAUSD | 60 | 2 | 0.0042 / 59.307184 | 0.00422 / 60.563324 | 0.00425 / 62.447534 | 0.0043 / 65.587884 | 0.00435 / 68.728233 | 0.00438 / 70.612443 | 0.0044 / 71.868583 |
| ADAUSD | 180 | 2 | 0.0041 / 58.339963 | 0.00413 / 59.450999 | 0.004175 / 61.117554 | 0.00425 / 63.895144 | 0.004325 / 66.672735 | 0.00437 / 68.33929 | 0.0044 / 69.450326 |
| ADAUSD | 240 | 2 | 0.0041 / 57.871893 | 0.00413 / 59.060458 | 0.004175 / 60.843307 | 0.00425 / 63.814721 | 0.004325 / 66.786136 | 0.00437 / 68.568984 | 0.0044 / 69.75755 |
| AUDJPY | 0 | 2 | 0.005 / 0.52968627 | 0.0052 / 0.54917953 | 0.0055 / 0.57841943 | 0.006 / 0.6271526 | 0.0065 / 0.67588576 | 0.0068 / 0.70512566 | 0.007 / 0.72461893 |
| AUDJPY | 30 | 2 | 0.006 / 0.62060405 | 0.0064 / 0.6644579 | 0.007 / 0.73023867 | 0.008 / 0.83987329 | 0.009 / 0.9495079 | 0.0096 / 1.0152887 | 0.01 / 1.0591425 |
| AUDJPY | 60 | 2 | 0.006 / 0.63535765 | 0.0064 / 0.67506261 | 0.007 / 0.73462006 | 0.008 / 0.83388246 | 0.009 / 0.93314486 | 0.0096 / 0.9927023 | 0.01 / 1.0324073 |
| AUDJPY | 180 | 2 | 0.007 / 0.72355534 | 0.0071 / 0.73586485 | 0.00725 / 0.75432912 | 0.0075 / 0.78510289 | 0.00775 / 0.81587667 | 0.0079 / 0.83434093 | 0.008 / 0.84665044 |
| AUDJPY | 240 | 2 | 0.007 / 0.74082305 | 0.0074 / 0.78050463 | 0.008 / 0.84002699 | 0.009 / 0.93923092 | 0.01 / 1.0384348 | 0.0106 / 1.0979572 | 0.011 / 1.1376388 |
| AUDUSD | 0 | 8 | 0 / 0 | 7e-06 / 0.10522439 | 1e-05 / 0.15085522 | 1e-05 / 0.15225473 | 1.25e-05 / 0.19028472 | 2.9e-05 / 0.44286151 | 5e-05 / 0.76812585 |
| AUDUSD | 30 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 5e-06 / 0.076173645 | 3e-05 / 0.45250178 | 3.6e-05 / 0.54951469 | 5e-05 / 0.76818486 |
| AUDUSD | 60 | 8 | 0 / 0 | 7e-06 / 0.10538598 | 1e-05 / 0.15073529 | 1e-05 / 0.1529479 | 2.5e-05 / 0.37815839 | 4.3e-05 / 0.64847293 | 5e-05 / 0.75810988 |
| AUDUSD | 180 | 8 | 0 / 0 | 0 / 0 | 7.5e-06 / 0.11272686 | 1e-05 / 0.15154227 | 1e-05 / 0.15200179 | 1e-05 / 0.15227215 | 1e-05 / 0.15235658 |
| AUDUSD | 240 | 8 | 0 / 0 | 7e-06 / 0.1051817 | 1e-05 / 0.15046994 | 1e-05 / 0.15147887 | 1e-05 / 0.15230094 | 1e-05 / 0.15289829 | 1e-05 / 0.15391838 |
| AUS200 | 0 | 2 | 1.8 / 2.0896701 | 1.8 / 2.0906793 | 1.8 / 2.0921931 | 1.8 / 2.094716 | 1.8 / 2.097239 | 1.8 / 2.0987528 | 1.8 / 2.099762 |
| AUS200 | 30 | 1 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 | 1 / 1.1624798 |
| AUS200 | 60 | 2 | 1 / 1.1607257 | 1 / 1.1612658 | 1 / 1.1620759 | 1 / 1.163426 | 1 / 1.1647762 | 1 / 1.1655863 | 1 / 1.1661264 |
| AUS200 | 180 | 0 | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified |
| AUS200 | 240 | 1 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 | 1 / 1.1643612 |
| AVAXUSD | 0 | 1 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 | 0.053 / 28.946722 |
| AVAXUSD | 30 | 1 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 | 0.055 / 25.781049 |
| AVAXUSD | 60 | 1 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 | 0.055 / 25.876873 |
| AVAXUSD | 180 | 2 | 0.051 / 25.899547 | 0.0515 / 26.067616 | 0.05225 / 26.31972 | 0.0535 / 26.739893 | 0.05475 / 27.160066 | 0.0555 / 27.41217 | 0.056 / 27.58024 |
| AVAXUSD | 240 | 2 | 0.052 / 25.685717 | 0.0524 / 25.938785 | 0.053 / 26.318387 | 0.054 / 26.951057 | 0.055 / 27.583728 | 0.0556 / 27.96333 | 0.056 / 28.216398 |
| BCHUSD | 0 | 2 | 1.74 / 34.226366 | 1.74 / 34.344345 | 1.74 / 34.521313 | 1.74 / 34.81626 | 1.74 / 35.111207 | 1.74 / 35.288175 | 1.74 / 35.406153 |
| BCHUSD | 30 | 2 | 1.7 / 33.454689 | 1.706 / 33.66456 | 1.715 / 33.979367 | 1.73 / 34.504045 | 1.745 / 35.028723 | 1.754 / 35.34353 | 1.76 / 35.553401 |
| BCHUSD | 60 | 1 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 | 1.71 / 33.696241 |
| BCHUSD | 180 | 1 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 | 1.69 / 33.836205 |
| BCHUSD | 240 | 2 | 1.67 / 33.58708 | 1.677 / 33.607603 | 1.6875 / 33.638388 | 1.705 / 33.689697 | 1.7225 / 33.741005 | 1.733 / 33.77179 | 1.74 / 33.792313 |
| BTCUSD | 0 | 29 | 10 / 1.2025232 | 15 / 1.2288486 | 15 / 1.2873551 | 17 / 2.5163004 | 30.85 / 3.2159095 | 33.05 / 4.7416155 | 40.73 / 5.8206481 |
| BTCUSD | 30 | 29 | 14 / 1.2183302 | 15 / 1.2697566 | 15 / 1.2948846 | 18 / 2.6381744 | 31 / 3.3682354 | 34.032 / 4.8937615 | 64.38 / 6.1660619 |
| BTCUSD | 60 | 28 | 15 / 1.2002544 | 15 / 1.2308634 | 15 / 1.2930435 | 17 / 2.5725725 | 28.0325 / 3.1857263 | 33.349 / 3.4522574 | 33.61 / 5.7747593 |
| BTCUSD | 180 | 29 | 11 / 1.2063327 | 15 / 1.2321142 | 15 / 1.2850582 | 16.77 / 2.6565568 | 23 / 3.2072982 | 35.034 / 3.6428168 | 39.57 / 5.7875467 |
| BTCUSD | 240 | 27 | 11 / 1.2059259 | 15 / 1.2326203 | 15 / 1.2850351 | 16.77 / 2.646159 | 27.485 / 3.2772282 | 34.316 / 3.5165149 | 36.4 / 5.8093278 |
| CADJPY | 0 | 2 | 0.007 / 0.65859728 | 0.0072 / 0.67642851 | 0.0075 / 0.70317535 | 0.008 / 0.74775343 | 0.0085 / 0.7923315 | 0.0088 / 0.81907834 | 0.009 / 0.83690957 |
| CADJPY | 30 | 2 | 0.007 / 0.65889485 | 0.0071 / 0.66735135 | 0.00725 / 0.6800361 | 0.0075 / 0.70117736 | 0.00775 / 0.72231862 | 0.0079 / 0.73500337 | 0.008 / 0.74345988 |
| CADJPY | 60 | 2 | 0.008 / 0.74252142 | 0.0082 / 0.76238692 | 0.0085 / 0.79218518 | 0.009 / 0.84184894 | 0.0095 / 0.89151271 | 0.0098 / 0.92131097 | 0.01 / 0.94117647 |
| CADJPY | 180 | 2 | 0.008 / 0.74301795 | 0.008 / 0.74393453 | 0.008 / 0.74530939 | 0.008 / 0.74760082 | 0.008 / 0.74989225 | 0.008 / 0.75126711 | 0.008 / 0.75218368 |
| CADJPY | 240 | 2 | 0.007 / 0.65014373 | 0.0072 / 0.66968046 | 0.0075 / 0.69898556 | 0.008 / 0.74782738 | 0.0085 / 0.79666921 | 0.0088 / 0.82597431 | 0.009 / 0.84551104 |
| CHFJPY | 0 | 2 | 0.009 / 0.49347922 | 0.0097 / 0.5308282 | 0.01075 / 0.58685167 | 0.0125 / 0.68022412 | 0.01425 / 0.77359657 | 0.0153 / 0.82962004 | 0.016 / 0.86696902 |
| CHFJPY | 30 | 2 | 0.008 / 0.43872638 | 0.0082 / 0.44898592 | 0.0085 / 0.46437523 | 0.009 / 0.49002409 | 0.0095 / 0.51567294 | 0.0098 / 0.53106226 | 0.01 / 0.5413218 |
| CHFJPY | 60 | 2 | 0.009 / 0.49364162 | 0.0094 / 0.51463425 | 0.01 / 0.54612319 | 0.011 / 0.59860476 | 0.012 / 0.65108633 | 0.0126 / 0.68257527 | 0.013 / 0.7035679 |
| CHFJPY | 180 | 2 | 0.011 / 0.59521072 | 0.011 / 0.59599352 | 0.011 / 0.59716773 | 0.011 / 0.59912474 | 0.011 / 0.60108175 | 0.011 / 0.60225596 | 0.011 / 0.60303877 |
| CHFJPY | 240 | 2 | 0.009 / 0.4868378 | 0.0095 / 0.51489789 | 0.01025 / 0.55698802 | 0.0115 / 0.62713824 | 0.01275 / 0.69728845 | 0.0135 / 0.73937859 | 0.014 / 0.76743867 |
| DOGEUSD | 0 | 2 | 0.00118 / 59.391987 | 0.001187 / 60.719174 | 0.0011975 / 62.709955 | 0.001215 / 66.027922 | 0.0012325 / 69.34589 | 0.001243 / 71.33667 | 0.00125 / 72.663857 |
| DOGEUSD | 30 | 2 | 0.00117 / 58.706942 | 0.001178 / 60.12679 | 0.00119 / 62.256563 | 0.00121 / 65.806185 | 0.00123 / 69.355806 | 0.001242 / 71.485579 | 0.00125 / 72.905427 |
| DOGEUSD | 60 | 2 | 0.00117 / 58.848678 | 0.001178 / 60.264146 | 0.00119 / 62.387348 | 0.00121 / 65.926018 | 0.00123 / 69.464688 | 0.001242 / 71.58789 | 0.00125 / 73.003358 |
| DOGEUSD | 180 | 2 | 0.00118 / 58.607331 | 0.001184 / 59.793326 | 0.00119 / 61.572318 | 0.0012 / 64.537305 | 0.00121 / 67.502292 | 0.001216 / 69.281284 | 0.00122 / 70.467279 |
| DOGEUSD | 240 | 2 | 0.00119 / 58.143796 | 0.001193 / 59.297659 | 0.0011975 / 61.028455 | 0.001205 / 63.913113 | 0.0012125 / 66.797772 | 0.001217 / 68.528567 | 0.00122 / 69.682431 |
| DOTUSD | 0 | 2 | 0.03 / 75.301205 | 0.03 / 76.607609 | 0.03 / 78.567214 | 0.03 / 81.833224 | 0.03 / 85.099233 | 0.03 / 87.058839 | 0.03 / 88.365243 |
| DOTUSD | 30 | 2 | 0.03 / 74.812968 | 0.03 / 76.178619 | 0.03 / 78.227095 | 0.03 / 81.641223 | 0.03 / 85.05535 | 0.03 / 87.103827 | 0.03 / 88.469478 |
| DOTUSD | 60 | 1 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 | 0.03 / 74.943792 |
| DOTUSD | 180 | 1 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 | 0.03 / 73.565473 |
| DOTUSD | 240 | 2 | 0.03 / 72.393822 | 0.03 / 73.936642 | 0.03 / 76.25087 | 0.03 / 80.107918 | 0.03 / 83.964966 | 0.03 / 86.279195 | 0.03 / 87.822014 |
| ETHUSD | 0 | 2 | 3 / 10.087017 | 3 / 10.245032 | 3 / 10.482053 | 3 / 10.877088 | 3 / 11.272124 | 3 / 11.509145 | 3 / 11.667159 |
| ETHUSD | 30 | 2 | 3 / 10.264254 | 3.006 / 10.407337 | 3.015 / 10.621962 | 3.03 / 10.97967 | 3.045 / 11.337378 | 3.054 / 11.552003 | 3.06 / 11.695086 |
| ETHUSD | 60 | 2 | 3 / 10.076616 | 3 / 10.239805 | 3 / 10.484588 | 3 / 10.89256 | 3 / 11.300533 | 3 / 11.545316 | 3 / 11.708505 |
| ETHUSD | 180 | 2 | 3 / 9.9934709 | 3 / 10.158047 | 3 / 10.404911 | 3 / 10.816351 | 3 / 11.227791 | 3 / 11.474654 | 3 / 11.63923 |
| ETHUSD | 240 | 2 | 3 / 9.8704666 | 3 / 10.047253 | 3 / 10.312432 | 3 / 10.754397 | 3 / 11.196362 | 3 / 11.461541 | 3 / 11.638327 |
| EURAUD | 0 | 2 | 4e-05 / 0.22496935 | 5.4e-05 / 0.3021297 | 7.5e-05 / 0.41787024 | 0.00011 / 0.61077112 | 0.000145 / 0.80367201 | 0.000166 / 0.91941254 | 0.00018 / 0.9965729 |
| EURAUD | 30 | 2 | 5e-05 / 0.27701061 | 5.1e-05 / 0.28303162 | 5.25e-05 / 0.29206314 | 5.5e-05 / 0.30711568 | 5.75e-05 / 0.32216821 | 5.9e-05 / 0.33119973 | 6e-05 / 0.33722074 |
| EURAUD | 60 | 2 | 7e-05 / 0.38765163 | 7.1e-05 / 0.39388161 | 7.25e-05 / 0.40322656 | 7.5e-05 / 0.41880149 | 7.75e-05 / 0.43437642 | 7.9e-05 / 0.44372138 | 8e-05 / 0.44995135 |
| EURAUD | 180 | 2 | 4e-05 / 0.22147536 | 4.4e-05 / 0.24427442 | 5e-05 / 0.27847301 | 6e-05 / 0.33547067 | 7e-05 / 0.39246832 | 7.6e-05 / 0.42666692 | 8e-05 / 0.44946598 |
| EURAUD | 240 | 2 | 4e-05 / 0.22159314 | 4e-05 / 0.22189943 | 4e-05 / 0.22235886 | 4e-05 / 0.22312457 | 4e-05 / 0.22389028 | 4e-05 / 0.22434971 | 4e-05 / 0.224656 |
| EURGBP | 0 | 2 | 3e-05 / 0.34611458 | 3.1e-05 / 0.35782193 | 3.25e-05 / 0.37538296 | 3.5e-05 / 0.40465135 | 3.75e-05 / 0.43391974 | 3.9e-05 / 0.45148077 | 4e-05 / 0.46318812 |
| EURGBP | 30 | 2 | 1e-05 / 0.11543812 | 1e-05 / 0.11547984 | 1e-05 / 0.11554243 | 1e-05 / 0.11564675 | 1e-05 / 0.11575107 | 1e-05 / 0.11581366 | 1e-05 / 0.11585539 |
| EURGBP | 60 | 2 | 1e-05 / 0.11538484 | 1.2e-05 / 0.13859532 | 1.5e-05 / 0.17341105 | 2e-05 / 0.23143726 | 2.5e-05 / 0.28946347 | 2.8e-05 / 0.32427919 | 3e-05 / 0.34748968 |
| EURGBP | 180 | 2 | 1e-05 / 0.11598304 | 1.2e-05 / 0.13898422 | 1.5e-05 / 0.17348599 | 2e-05 / 0.23098893 | 2.5e-05 / 0.28849188 | 2.8e-05 / 0.32299364 | 3e-05 / 0.34599482 |
| EURGBP | 240 | 2 | 2e-05 / 0.23067253 | 2e-05 / 0.23080255 | 2e-05 / 0.2309976 | 2e-05 / 0.23132267 | 2e-05 / 0.23164774 | 2e-05 / 0.23184278 | 2e-05 / 0.23197281 |
| EURJPY | 0 | 2 | 0.009 / 0.52397323 | 0.0095 / 0.55369072 | 0.01025 / 0.59826694 | 0.0115 / 0.67256066 | 0.01275 / 0.74685437 | 0.0135 / 0.7914306 | 0.014 / 0.82114808 |
| EURJPY | 30 | 2 | 0.007 / 0.40693771 | 0.007 / 0.40731832 | 0.007 / 0.40788925 | 0.007 / 0.4088408 | 0.007 / 0.40979234 | 0.007 / 0.41036327 | 0.007 / 0.41074389 |
| EURJPY | 60 | 2 | 0.009 / 0.52772769 | 0.0095 / 0.55624818 | 0.01025 / 0.59902892 | 0.0115 / 0.67033014 | 0.01275 / 0.74163137 | 0.0135 / 0.78441211 | 0.014 / 0.8129326 |
| EURJPY | 180 | 2 | 0.014 / 0.81303174 | 0.0142 / 0.82548607 | 0.0145 / 0.84416757 | 0.015 / 0.87530341 | 0.0155 / 0.90643924 | 0.0158 / 0.92512075 | 0.016 / 0.93757508 |
| EURJPY | 240 | 2 | 0.011 / 0.64493622 | 0.0112 / 0.65595748 | 0.0115 / 0.67248937 | 0.012 / 0.70004252 | 0.0125 / 0.72759566 | 0.0128 / 0.74412755 | 0.013 / 0.75514881 |
| EURUSD | 0 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| EURUSD | 30 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 1e-05 / 0.084848056 | 1e-05 / 0.085298612 | 1.6e-05 / 0.13593372 | 3e-05 / 0.25386187 |
| EURUSD | 60 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 2.5e-06 / 0.021141381 | 1.6e-05 / 0.13586828 | 3e-05 / 0.25557472 |
| EURUSD | 180 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 3e-06 / 0.025382106 | 1e-05 / 0.084607022 |
| EURUSD | 240 | 7 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 4e-06 / 0.034203089 | 1e-05 / 0.085507723 |
| GBPJPY | 0 | 2 | 0.006 / 0.30392519 | 0.0064 / 0.32399593 | 0.007 / 0.35410203 | 0.008 / 0.40427886 | 0.009 / 0.45445569 | 0.0096 / 0.48456179 | 0.01 / 0.50463253 |
| GBPJPY | 30 | 2 | 0.006 / 0.30387748 | 0.0061 / 0.30874027 | 0.00625 / 0.31603447 | 0.0065 / 0.32819145 | 0.00675 / 0.34034844 | 0.0069 / 0.34764264 | 0.007 / 0.35250543 |
| GBPJPY | 60 | 2 | 0.005 / 0.25312162 | 0.0052 / 0.26303659 | 0.0055 / 0.27790903 | 0.006 / 0.30269645 | 0.0065 / 0.32748386 | 0.0068 / 0.3423563 | 0.007 / 0.35227127 |
| GBPJPY | 180 | 2 | 0.005 / 0.25177565 | 0.005 / 0.2518596 | 0.005 / 0.25198554 | 0.005 / 0.25219542 | 0.005 / 0.25240531 | 0.005 / 0.25253124 | 0.005 / 0.2526152 |
| GBPJPY | 240 | 2 | 0.005 / 0.25181242 | 0.0052 / 0.26201554 | 0.0055 / 0.27732022 | 0.006 / 0.30282802 | 0.0065 / 0.32833582 | 0.0068 / 0.34364051 | 0.007 / 0.35384363 |
| GBPUSD | 0 | 8 | 0 / 0 | 7e-06 / 0.051358619 | 1e-05 / 0.07340056 | 1e-05 / 0.074043424 | 2e-05 / 0.14748872 | 2.3e-05 / 0.17014985 | 3e-05 / 0.22038001 |
| GBPUSD | 30 | 8 | 1e-05 / 0.073387224 | 1e-05 / 0.073410986 | 1e-05 / 0.07377722 | 1e-05 / 0.074057416 | 2e-05 / 0.14689277 | 2e-05 / 0.14736847 | 2e-05 / 0.14834814 |
| GBPUSD | 60 | 8 | 0 / 0 | 0 / 0 | 7.5e-06 / 0.055013368 | 1e-05 / 0.073444675 | 1.5e-05 / 0.11119474 | 4.8e-05 / 0.35511221 | 9e-05 / 0.66455241 |
| GBPUSD | 180 | 8 | 0 / 0 | 0 / 0 | 7.5e-06 / 0.054963376 | 1e-05 / 0.073968153 | 2.25e-05 / 0.16573304 | 3e-05 / 0.22219984 | 3e-05 / 0.22249498 |
| GBPUSD | 240 | 8 | 0 / 0 | 0 / 0 | 7.5e-06 / 0.055063047 | 1e-05 / 0.073861822 | 1.5e-05 / 0.11117577 | 4.2e-05 / 0.30945574 | 7e-05 / 0.51284118 |
| GER40 | 0 | 2 | 2 / 0.83355214 | 2 / 0.83410397 | 2 / 0.83493173 | 2 / 0.83631131 | 2 / 0.83769089 | 2 / 0.83851864 | 2 / 0.83907048 |
| GER40 | 30 | 2 | 0.9 / 0.37448586 | 0.9 / 0.37474885 | 0.9 / 0.37514334 | 0.9 / 0.37580082 | 0.9 / 0.3764583 | 0.9 / 0.37685279 | 0.9 / 0.37711578 |
| GER40 | 60 | 2 | 0.9 / 0.37369751 | 0.9 / 0.37404044 | 0.9 / 0.37455484 | 0.9 / 0.37541217 | 0.9 / 0.37626951 | 0.9 / 0.37678391 | 0.9 / 0.37712684 |
| GER40 | 180 | 2 | 0.9 / 0.37361529 | 0.9 / 0.37383621 | 0.9 / 0.37416758 | 0.9 / 0.37471988 | 0.9 / 0.37527217 | 0.9 / 0.37560355 | 0.9 / 0.37582446 |
| GER40 | 240 | 2 | 0.9 / 0.37459965 | 0.9 / 0.37468685 | 0.9 / 0.37481766 | 0.9 / 0.37503567 | 0.9 / 0.37525368 | 0.9 / 0.37538448 | 0.9 / 0.37547169 |
| JPN225 | 0 | 2 | 7 / 1.7619726 | 7 / 1.7634714 | 7 / 1.7657197 | 7 / 1.7694668 | 7 / 1.7732139 | 7 / 1.7754621 | 7 / 1.7769609 |
| JPN225 | 30 | 2 | 7 / 1.7630821 | 7 / 1.7644023 | 7 / 1.7663827 | 7 / 1.7696833 | 7 / 1.7729839 | 7 / 1.7749643 | 7 / 1.7762846 |
| JPN225 | 60 | 2 | 7 / 1.7646377 | 7 / 1.7659151 | 7 / 1.7678313 | 7 / 1.7710249 | 7 / 1.7742185 | 7 / 1.7761347 | 7 / 1.7774121 |
| JPN225 | 180 | 2 | 7 / 1.7670877 | 7 / 1.7679736 | 7 / 1.7693024 | 7 / 1.7715172 | 7 / 1.7737319 | 7 / 1.7750607 | 7 / 1.7759466 |
| JPN225 | 240 | 2 | 7 / 1.7663074 | 7 / 1.7667547 | 7 / 1.7674256 | 7 / 1.7685438 | 7 / 1.7696619 | 7 / 1.7703328 | 7 / 1.7707801 |
| LINKUSD | 0 | 0 | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified |
| LINKUSD | 30 | 2 | 0.037 / 23.537644 | 0.037 / 23.931031 | 0.037 / 24.52111 | 0.037 / 25.504576 | 0.037 / 26.488042 | 0.037 / 27.078121 | 0.037 / 27.471508 |
| LINKUSD | 60 | 0 | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified | Not verified / Not verified |
| LINKUSD | 180 | 1 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 | 0.037 / 23.316634 |
| LINKUSD | 240 | 1 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 | 0.038 / 23.503216 |
| LTCUSD | 0 | 2 | 0.31 / 32.743597 | 0.31 / 33.016756 | 0.31 / 33.426496 | 0.31 / 34.109396 | 0.31 / 34.792296 | 0.31 / 35.202036 | 0.31 / 35.475196 |
| LTCUSD | 30 | 1 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 | 0.31 / 32.643606 |
| LTCUSD | 60 | 2 | 0.31 / 32.684907 | 0.31 / 32.96353 | 0.31 / 33.381464 | 0.31 / 34.078022 | 0.31 / 34.774579 | 0.31 / 35.192514 | 0.31 / 35.471137 |
| LTCUSD | 180 | 2 | 0.31 / 32.414911 | 0.311 / 32.819728 | 0.3125 / 33.426953 | 0.315 / 34.438996 | 0.3175 / 35.451039 | 0.319 / 36.058264 | 0.32 / 36.463081 |
| LTCUSD | 240 | 2 | 0.31 / 33.051023 | 0.311 / 33.280899 | 0.3125 / 33.625715 | 0.315 / 34.200407 | 0.3175 / 34.7751 | 0.319 / 35.119915 | 0.32 / 35.349792 |
| NAS100 | 0 | 7 | 1 / 0.56919084 | 1 / 0.57006617 | 1 / 0.57083647 | 1.3 / 0.57477541 | 1.6 / 0.82759651 | 1.9 / 1.0832251 | 1.9 / 1.0892091 |
| NAS100 | 30 | 7 | 1 / 0.43932678 | 1 / 0.43941829 | 1 / 0.50464156 | 1 / 0.56994346 | 1 / 0.57397896 | 1 / 0.57519104 | 1 / 0.57640874 |
| NAS100 | 60 | 7 | 1 / 0.43903553 | 1 / 0.43926935 | 1 / 0.50361963 | 1 / 0.56848221 | 1 / 0.57430914 | 1 / 0.57634641 | 1 / 0.57667134 |
| NAS100 | 180 | 7 | 1 / 0.43756208 | 1 / 0.43920585 | 1 / 0.50363613 | 1 / 0.56943393 | 1 / 0.57394963 | 1 / 0.57644985 | 1 / 0.57715394 |
| NAS100 | 240 | 7 | 1 / 0.43687391 | 1 / 0.43917624 | 1 / 0.50389551 | 1 / 0.57112506 | 1 / 0.57502452 | 1 / 0.57686507 | 1 / 0.57769408 |
| NZDUSD | 0 | 8 | 2e-05 / 0.33435311 | 2e-05 / 0.33610732 | 2e-05 / 0.33698259 | 2e-05 / 0.33838394 | 2e-05 / 0.33932825 | 3.5e-05 / 0.58707023 | 7e-05 / 1.1643283 |
| NZDUSD | 30 | 8 | 2e-05 / 0.33424136 | 2e-05 / 0.33617313 | 2e-05 / 0.33805736 | 2.5e-05 / 0.42313509 | 4.25e-05 / 0.714437 | 5.3e-05 / 0.88731082 | 6e-05 / 1.016639 |
| NZDUSD | 60 | 8 | 2e-05 / 0.33387309 | 2e-05 / 0.336401 | 2e-05 / 0.33814815 | 3e-05 / 0.50440082 | 5e-05 / 0.8454447 | 5.3e-05 / 0.89705604 | 6e-05 / 1.010084 |
| NZDUSD | 180 | 8 | 2e-05 / 0.33343892 | 2e-05 / 0.33373114 | 2e-05 / 0.33618931 | 2e-05 / 0.33814405 | 2e-05 / 0.33930702 | 2.6e-05 / 0.44034596 | 4e-05 / 0.67447939 |
| NZDUSD | 240 | 8 | 2e-05 / 0.33343892 | 2e-05 / 0.33574969 | 2e-05 / 0.33710625 | 2e-05 / 0.33810158 | 2e-05 / 0.33935784 | 2.3e-05 / 0.38838786 | 3e-05 / 0.50052973 |
| NatGas | 0 | 8 | 0.004 / 13.020833 | 0.004 / 13.662348 | 0.00475 / 14.470112 | 0.005 / 15.908001 | 0.005 / 16.492304 | 0.005 / 16.624756 | 0.005 / 16.714023 |
| NatGas | 30 | 6 | 0.004 / 13.333333 | 0.0045 / 14.43909 | 0.005 / 15.732287 | 0.005 / 16.307904 | 0.005 / 16.421916 | 0.005 / 16.529255 | 0.005 / 16.603022 |
| NatGas | 60 | 8 | 0.004 / 11.719895 | 0.004 / 12.233278 | 0.004 / 12.818922 | 0.004 / 13.003923 | 0.004 / 13.293524 | 0.0043 / 14.264743 | 0.005 / 16.406891 |
| NatGas | 180 | 9 | 0.004 / 12.464942 | 0.004 / 12.86576 | 0.004 / 13.162224 | 0.004 / 13.34668 | 0.005 / 16.178612 | 0.005 / 17.193206 | 0.005 / 17.461149 |
| NatGas | 240 | 10 | 0.004 / 12.414649 | 0.004 / 12.865617 | 0.00425 / 13.952055 | 0.005 / 16.178749 | 0.005 / 16.614978 | 0.005 / 17.375048 | 0.005 / 17.473353 |
| SOLUSD | 0 | 2 | 0.37 / 22.949294 | 0.372 / 23.220914 | 0.375 / 23.628344 | 0.38 / 24.307394 | 0.385 / 24.986443 | 0.388 / 25.393873 | 0.39 / 25.665493 |
| SOLUSD | 30 | 2 | 0.38 / 23.519218 | 0.381 / 23.743169 | 0.3825 / 24.079095 | 0.385 / 24.638972 | 0.3875 / 25.198849 | 0.389 / 25.534776 | 0.39 / 25.758727 |
| SOLUSD | 60 | 2 | 0.38 / 23.491592 | 0.381 / 23.722566 | 0.3825 / 24.069027 | 0.385 / 24.646461 | 0.3875 / 25.223895 | 0.389 / 25.570356 | 0.39 / 25.80133 |
| SOLUSD | 180 | 2 | 0.37 / 23.037284 | 0.371 / 23.166246 | 0.3725 / 23.359688 | 0.375 / 23.682092 | 0.3775 / 24.004497 | 0.379 / 24.197939 | 0.38 / 24.326901 |
| SOLUSD | 240 | 2 | 0.37 / 22.732711 | 0.371 / 22.894692 | 0.3725 / 23.137663 | 0.375 / 23.542615 | 0.3775 / 23.947567 | 0.379 / 24.190538 | 0.38 / 24.352519 |
| SpotBrent | 0 | 59 | 0.02 / 2.4059282 | 0.02 / 2.4505204 | 0.02 / 2.6644854 | 0.02 / 3.3534541 | 0.03 / 4.4221049 | 0.04 / 4.664022 | 0.04 / 4.8944631 |
| SpotBrent | 30 | 54 | 0.02 / 2.4174735 | 0.02 / 2.4560721 | 0.02 / 2.748751 | 0.025 / 3.4818377 | 0.03 / 3.9880595 | 0.035 / 4.531178 | 0.04 / 4.6373586 |
| SpotBrent | 60 | 61 | 0.02 / 2.4050602 | 0.02 / 2.4566705 | 0.02 / 2.6680896 | 0.02 / 3.0771598 | 0.03 / 3.8340618 | 0.035 / 4.333006 | 0.04 / 4.6438191 |
| SpotBrent | 180 | 58 | 0.02 / 2.4157215 | 0.02 / 2.4414683 | 0.02 / 2.7034915 | 0.0225 / 3.1213526 | 0.03 / 3.9986163 | 0.035 / 4.2253187 | 0.035 / 4.6385775 |
| SpotBrent | 240 | 61 | 0.02 / 2.4222459 | 0.02 / 2.6106252 | 0.02 / 3.0112772 | 0.03 / 3.6378629 | 0.03 / 4.0427142 | 0.035 / 4.3581847 | 0.035 / 4.9313717 |
| SpotCrude | 0 | 60 | 0.02 / 2.4926156 | 0.02 / 2.5518049 | 0.02 / 2.9445791 | 0.02 / 3.2734355 | 0.03 / 3.8678454 | 0.03 / 4.112925 | 0.03 / 4.9919297 |
| SpotCrude | 30 | 57 | 0.02 / 2.5060458 | 0.02 / 2.5600328 | 0.02 / 3.0086499 | 0.03 / 3.697131 | 0.03 / 4.1866696 | 0.03 / 4.8579581 | 0.04 / 5.4805782 |
| SpotCrude | 60 | 62 | 0.02 / 2.4913736 | 0.02 / 2.5910988 | 0.02 / 2.9049245 | 0.025 / 3.259692 | 0.03 / 3.9658392 | 0.03 / 4.4396639 | 0.03 / 4.9869508 |
| SpotCrude | 180 | 61 | 0.02 / 2.5022833 | 0.02 / 2.5688121 | 0.02 / 2.9689448 | 0.025 / 3.2153824 | 0.03 / 3.8729167 | 0.03 / 4.4962007 | 0.04 / 5.5686264 |
| SpotCrude | 240 | 60 | 0.02 / 2.5079313 | 0.02 / 2.5652269 | 0.02 / 2.9333343 | 0.02 / 3.1532664 | 0.025 / 3.351814 | 0.03 / 4.213788 | 0.03 / 5.2302167 |
| UK100 | 0 | 2 | 1.8 / 2.0425764 | 1.81 / 2.0510811 | 1.825 / 2.0638382 | 1.85 / 2.0851001 | 1.875 / 2.106362 | 1.89 / 2.1191191 | 1.9 / 2.1276238 |
| UK100 | 30 | 2 | 1.1 / 1.2282337 | 1.11 / 1.2416888 | 1.125 / 1.2618715 | 1.15 / 1.2955092 | 1.175 / 1.3291469 | 1.19 / 1.3493295 | 1.2 / 1.3627846 |
| UK100 | 60 | 2 | 1.1 / 1.2273293 | 1.1 / 1.229356 | 1.1 / 1.2323962 | 1.1 / 1.2374631 | 1.1 / 1.24253 | 1.1 / 1.2455702 | 1.1 / 1.247597 |
| UK100 | 180 | 2 | 1 / 1.1336198 | 1.01 / 1.1428211 | 1.025 / 1.1566232 | 1.05 / 1.1796267 | 1.075 / 1.2026301 | 1.09 / 1.2164322 | 1.1 / 1.2256336 |
| UK100 | 240 | 2 | 1.1 / 1.225224 | 1.1 / 1.2272142 | 1.1 / 1.2301994 | 1.1 / 1.2351748 | 1.1 / 1.2401502 | 1.1 / 1.2431355 | 1.1 / 1.2451256 |
| US30 | 0 | 2 | 2.6 / 0.58070193 | 2.6 / 0.58128674 | 2.6 / 0.58216394 | 2.6 / 0.58362595 | 2.6 / 0.58508795 | 2.6 / 0.58596516 | 2.6 / 0.58654996 |
| US30 | 30 | 2 | 2 / 0.44664392 | 2 / 0.44706163 | 2 / 0.4476882 | 2 / 0.44873249 | 2 / 0.44977677 | 2 / 0.45040334 | 2 / 0.45082106 |
| US30 | 60 | 2 | 2 / 0.44779394 | 2 / 0.4481292 | 2 / 0.44863208 | 2 / 0.44947021 | 2 / 0.45030834 | 2 / 0.45081122 | 2 / 0.45114648 |
| US30 | 180 | 2 | 2 / 0.45044436 | 2 / 0.45046781 | 2 / 0.45050298 | 2 / 0.4505616 | 2 / 0.45062022 | 2 / 0.45065539 | 2 / 0.45067883 |
| US30 | 240 | 2 | 2 / 0.44994882 | 2 / 0.4500334 | 2 / 0.45016027 | 2 / 0.45037173 | 2 / 0.45058318 | 2 / 0.45071006 | 2 / 0.45079464 |
| US500 | 0 | 51 | 0.4 / 0.59160221 | 0.4 / 0.62751004 | 0.4 / 0.65758165 | 0.4 / 0.6981412 | 0.4 / 0.81762806 | 0.4 / 0.87892771 | 0.4 / 1.0073029 |
| US500 | 30 | 50 | 0.4 / 0.59223287 | 0.4 / 0.62405261 | 0.4 / 0.65566627 | 0.4 / 0.69181837 | 0.4 / 0.81787472 | 0.4 / 0.88970269 | 0.4 / 1.0111479 |
| US500 | 60 | 51 | 0.4 / 0.5926804 | 0.4 / 0.62713618 | 0.4 / 0.65656752 | 0.4 / 0.69449267 | 0.4 / 0.8166682 | 0.4 / 0.87742388 | 0.4 / 1.0098205 |
| US500 | 180 | 47 | 0.4 / 0.5930055 | 0.4 / 0.62962695 | 0.4 / 0.65933913 | 0.4 / 0.6995453 | 0.4 / 0.81778092 | 0.4 / 0.91884294 | 0.4 / 1.0049241 |
| US500 | 240 | 50 | 0.4 / 0.59200497 | 0.4 / 0.62477531 | 0.4 / 0.65887354 | 0.4 / 0.69584086 | 0.4 / 0.81938762 | 0.4 / 0.88766211 | 0.4 / 0.99972508 |
| USDCAD | 0 | 8 | 2e-05 / 0.14464872 | 2e-05 / 0.1448314 | 2e-05 / 0.14495616 | 2.5e-05 / 0.18150758 | 3.25e-05 / 0.23658256 | 4.6e-05 / 0.33428418 | 6e-05 / 0.43971184 |
| USDCAD | 30 | 8 | 2e-05 / 0.14452538 | 2e-05 / 0.14475692 | 2e-05 / 0.14503586 | 2e-05 / 0.14565034 | 4.25e-05 / 0.31014654 | 7.4e-05 / 0.53617643 | 0.00013 / 0.94300958 |
| USDCAD | 60 | 8 | 1e-05 / 0.072562358 | 1e-05 / 0.072943348 | 1.75e-05 / 0.12684957 | 2e-05 / 0.14516218 | 3.25e-05 / 0.23689756 | 4.3e-05 / 0.31108347 | 5e-05 / 0.36248826 |
| USDCAD | 180 | 8 | 1e-05 / 0.072344505 | 1e-05 / 0.072362463 | 1.75e-05 / 0.12676063 | 2e-05 / 0.1451874 | 2e-05 / 0.14621059 | 2.6e-05 / 0.18913357 | 4e-05 / 0.28918241 |
| USDCAD | 240 | 8 | 1e-05 / 0.072569204 | 1e-05 / 0.072997832 | 1.75e-05 / 0.12682431 | 2e-05 / 0.14508324 | 3e-05 / 0.21697049 | 3e-05 / 0.21792243 | 3e-05 / 0.21940971 |
| USDCHF | 0 | 8 | 0 / 0 | 0 / 0 | 7.5e-06 / 0.09416255 | 1e-05 / 0.12625974 | 3e-05 / 0.37550243 | 3.6e-05 / 0.45182272 | 5e-05 / 0.62880031 |
| USDCHF | 30 | 8 | 1e-05 / 0.12570157 | 1e-05 / 0.1263725 | 1.75e-05 / 0.21979761 | 2.5e-05 / 0.31453542 | 3.25e-05 / 0.40823651 | 4.3e-05 / 0.54206822 | 5e-05 / 0.63370151 |
| USDCHF | 60 | 8 | 0 / 0 | 0 / 0 | 7.5e-06 / 0.094031507 | 1e-05 / 0.12602905 | 1.5e-05 / 0.18927422 | 3.3e-05 / 0.41468911 | 4e-05 / 0.50334726 |
| USDCHF | 180 | 7 | 0 / 0 | 0 / 0 | 0 / 0 | 1e-05 / 0.12547445 | 1e-05 / 0.12609158 | 1.4e-05 / 0.17606354 | 2e-05 / 0.25084975 |
| USDCHF | 240 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 5e-06 / 0.0627908 | 1.25e-05 / 0.15737683 | 2e-05 / 0.25198519 | 2e-05 / 0.25325111 |
| USDJPY | 0 | 8 | 0.001 / 0.067667926 | 0.001 / 0.067786175 | 0.001 / 0.067843756 | 0.0015 / 0.10238496 | 0.00225 / 0.15269723 | 0.003 / 0.20247769 | 0.003 / 0.20293649 |
| USDJPY | 30 | 8 | 0.001 / 0.067841455 | 0.001 / 0.067923751 | 0.001 / 0.06802596 | 0.0015 / 0.10211702 | 0.002 / 0.13554673 | 0.0023 / 0.15566688 | 0.003 / 0.20205218 |
| USDJPY | 60 | 8 | 0 / 0 | 0 / 0 | 0.00075 / 0.050745794 | 0.0015 / 0.10130354 | 0.00225 / 0.15419654 | 0.0048 / 0.32529648 | 0.009 / 0.6093412 |
| USDJPY | 180 | 8 | 0 / 0 | 0 / 0 | 0 / 0 | 0.0005 / 0.033764734 | 0.00125 / 0.084618954 | 0.002 / 0.135793 | 0.002 / 0.13579945 |
| USDJPY | 240 | 8 | 0.001 / 0.067907795 | 0.001 / 0.067974385 | 0.00175 / 0.11833702 | 0.002 / 0.13568774 | 0.00225 / 0.15374532 | 0.003 / 0.20258694 | 0.003 / 0.20296944 |
| XAGUSD | 0 | 6 | 0.02 / 5.4613473 | 0.02 / 6.1973232 | 0.02025 / 7.4250644 | 0.0215 / 9.1936918 | 0.02275 / 9.6127265 | 0.025 / 9.958837 | 0.027 / 10.263046 |
| XAGUSD | 30 | 6 | 0.017 / 5.1259707 | 0.0185 / 5.4340735 | 0.02 / 6.1700543 | 0.0205 / 8.2059269 | 0.021 / 9.2328588 | 0.022 / 9.8492846 | 0.023 / 10.374146 |
| XAGUSD | 60 | 7 | 0.018 / 4.874357 | 0.018 / 4.9093767 | 0.0185 / 6.4152888 | 0.019 / 8.4169491 | 0.02 / 8.9909925 | 0.0208 / 9.2744459 | 0.022 / 9.6347552 |
| XAGUSD | 180 | 7 | 0.017 / 4.6423354 | 0.0176 / 4.9292094 | 0.018 / 6.489674 | 0.018 / 7.8829815 | 0.019 / 8.1089603 | 0.02 / 8.4010184 | 0.02 / 8.7919817 |
| XAGUSD | 240 | 7 | 0.017 / 4.6903668 | 0.017 / 5.5656164 | 0.0175 / 6.8109039 | 0.018 / 7.8564881 | 0.0205 / 8.2038634 | 0.0228 / 8.9212101 | 0.024 / 9.8898629 |
| XAUUSD | 0 | 42 | 0.07 / 0.2627626 | 0.12 / 0.3646983 | 0.15 / 0.51413313 | 0.17 / 0.67006297 | 0.19 / 0.85951231 | 0.247 / 1.0590035 | 0.29 / 1.4998668 |
| XAUUSD | 30 | 42 | 0.06 / 0.26597749 | 0.12 / 0.36302065 | 0.14 / 0.51380564 | 0.155 / 0.60473712 | 0.17 / 0.83676581 | 0.22 / 1.0088849 | 0.32 / 1.2002094 |
| XAUUSD | 60 | 42 | 0.11 / 0.32795243 | 0.13 / 0.41652955 | 0.14 / 0.53093238 | 0.17 / 0.66477338 | 0.18 / 0.83214952 | 0.229 / 0.9355908 | 0.35 / 1.197689 |
| XAUUSD | 180 | 42 | 0.09 / 0.33241516 | 0.121 / 0.40855009 | 0.1325 / 0.47918976 | 0.155 / 0.60597886 | 0.1775 / 0.81763791 | 0.2 / 0.97736766 | 0.23 / 1.187486 |
| XAUUSD | 240 | 42 | 0.06 / 0.29512599 | 0.111 / 0.35734031 | 0.13 / 0.47931461 | 0.16 / 0.61878399 | 0.18 / 0.76563971 | 0.209 / 0.85728891 | 0.23 / 1.0718524 |
| XLMUSD | 0 | 1 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 | 0.0021 / 44.619144 |
| XLMUSD | 30 | 1 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 | 0.0022 / 44.751831 |
| XLMUSD | 60 | 2 | 0.0016 / 44.070513 | 0.00166 / 45.97261 | 0.00175 / 48.825755 | 0.0019 / 53.580998 | 0.00205 / 58.33624 | 0.00214 / 61.189386 | 0.0022 / 63.091483 |
| XLMUSD | 180 | 1 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 | 0.0022 / 43.255997 |
| XLMUSD | 240 | 2 | 0.0016 / 42.0042 | 0.00165 / 44.175744 | 0.001725 / 47.433059 | 0.00185 / 52.861917 | 0.001975 / 58.290775 | 0.00205 / 61.54809 | 0.0021 / 63.719634 |
| XPDUSD | 0 | 2 | 4.09 / 36.921188 | 4.537 / 40.318214 | 5.2075 / 45.413752 | 6.325 / 53.906316 | 7.4425 / 62.39888 | 8.113 / 67.494418 | 8.56 / 70.891443 |
| XPDUSD | 30 | 2 | 3.85 / 34.674532 | 4.67 / 41.030942 | 5.9 / 50.565558 | 7.95 / 66.456583 | 10 / 82.347609 | 11.23 / 91.882224 | 12.05 / 98.238634 |
| XPDUSD | 60 | 2 | 3.93 / 35.303153 | 4.733 / 41.461756 | 5.9375 / 50.699659 | 7.945 / 66.096165 | 9.9525 / 81.492671 | 11.157 / 90.730575 | 11.96 / 96.889177 |
| XPDUSD | 180 | 2 | 3.85 / 34.214467 | 4.661 / 40.492546 | 5.8775 / 49.909664 | 7.905 / 65.604861 | 9.9325 / 81.300058 | 11.149 / 90.717177 | 11.96 / 96.995256 |
| XPDUSD | 240 | 2 | 4.42 / 39.361314 | 5.079 / 44.370039 | 6.0675 / 51.883126 | 7.715 / 64.404938 | 9.3625 / 76.926751 | 10.351 / 84.439838 | 11.01 / 89.448563 |
| XPTUSD | 0 | 2 | 3.95 / 29.196649 | 5.442 / 39.822071 | 7.68 / 55.760205 | 11.41 / 82.323762 | 15.14 / 108.88732 | 17.378 / 124.82545 | 18.87 / 135.45087 |
| XPTUSD | 30 | 2 | 2.6 / 19.149469 | 3.096 / 22.625435 | 3.84 / 27.839383 | 5.08 / 36.529297 | 6.32 / 45.219211 | 7.064 / 50.433159 | 7.56 / 53.909125 |
| XPTUSD | 60 | 1 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 | 2.5 / 18.348759 |
| XPTUSD | 180 | 2 | 2.54 / 18.514604 | 3.03 / 21.997242 | 3.765 / 27.221198 | 4.99 / 35.927792 | 6.215 / 44.634387 | 6.95 / 49.858343 | 7.44 / 53.340981 |
| XPTUSD | 240 | 2 | 2.5 / 18.285815 | 2.997 / 21.822938 | 3.7425 / 27.128623 | 4.985 / 35.971431 | 6.2275 / 44.814239 | 6.973 / 50.119924 | 7.47 / 53.657047 |
| XRPUSD | 0 | 2 | 0.0063 / 26.434979 | 0.00642 / 26.564918 | 0.0066 / 26.759826 | 0.0069 / 27.084673 | 0.0072 / 27.409519 | 0.00738 / 27.604428 | 0.0075 / 27.734366 |
| XRPUSD | 30 | 2 | 0.0063 / 26.408916 | 0.00642 / 26.550648 | 0.0066 / 26.763247 | 0.0069 / 27.117578 | 0.0072 / 27.471909 | 0.00738 / 27.684508 | 0.0075 / 27.826241 |
| XRPUSD | 60 | 2 | 0.0063 / 26.341206 | 0.00642 / 26.489341 | 0.0066 / 26.711543 | 0.0069 / 27.08188 | 0.0072 / 27.452217 | 0.00738 / 27.674419 | 0.0075 / 27.822554 |
| XRPUSD | 180 | 2 | 0.0059 / 25.950606 | 0.00608 / 26.021981 | 0.00635 / 26.129044 | 0.0068 / 26.307482 | 0.00725 / 26.48592 | 0.00752 / 26.592983 | 0.0077 / 26.664358 |
| XRPUSD | 240 | 2 | 0.0059 / 25.976885 | 0.00608 / 26.009023 | 0.00635 / 26.05723 | 0.0068 / 26.137574 | 0.00725 / 26.217919 | 0.00752 / 26.266125 | 0.0077 / 26.298263 |

## §8. Complete tick, M30 and failed-source index

Receipt hashes are abbreviated to 12 characters in these tables; the complete SHA256, file paths and source claims are in the unified JSON. A canonical tick-stream ID is not an independent trade or validation sample. Identical tick aliases retain their own preparation/seal status.

### 8.1 Canonical tick episodes

| Episode ID | Symbol | Date | Cohort | Source UTC window | Coded UTC session | Tick SHA256 prefix | M30 prep | Missing code slots | Later M30 | Contiguous bars | Protocol | Seal |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| d1a69a2dc803a87c5097b45c | ADAUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 1480c4aca9b5 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 082a6549c9d765be3ea185f0 | ADAUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 205e9813bb61 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 89062e8ea535c6e11e35605c | AUDJPY | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 0af1b4830e9a | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| ff659acf6369cf424d3be3f0 | AUDJPY | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 4f78bf090db5 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 3b5f4cd3aec1c1aeec26d1c1 | AUDUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e7c031a19d07 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 5b27681e214647e03fd2d057 | AUDUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 5e8b7f11f2c1 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| a99521db57837133c65c7bf8 | AUS200 | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–06:00 | 8373638ebab2 | missing_segments | 372 | False | 0 | clean_first_read | outside_protected_intervals |
| 17e9233fca4846dcd3b20771 | AUS200 | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–06:00 | e034daf823ba | missing_segments | 372 | False | 0 | clean_first_read | outside_protected_intervals |
| 6d3f76cfb8e5d6be83456b15 | AVAXUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 95b0eae01ae6 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 3434d4494f742203c4176189 | AVAXUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | e142c4c9c5f8 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 6a8ef9c5180a2a3a33e355cf | BCHUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 87eb89863c81 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| ca49f91ad05c674370c5e7d4 | BCHUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 16f391ee5291 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| a3281b32cf61af3a41b4fb3a | BTCUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 117c6759d19f | missing_segments | 2160 | True | 0 | clean_first_read | outside_protected_intervals |
| ae497d730dbeacf465872233 | BTCUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 5dcd99cab23c | missing_segments | 2160 | True | 0 | clean_first_read | outside_protected_intervals |
| 0dc38d8a164908535db54ed8 | CADJPY | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | c99fb1744c68 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 1deb2d2370a35ffba08289f7 | CADJPY | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | bb06d54742b1 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 9ce9f711a042156c2370ac80 | CHFJPY | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 268b96a15bc4 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 09a99eb09e31bdaedd4c4191 | CHFJPY | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | ee2fcae1c9e1 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 1f88f533575396b87ea4a2da | DOGEUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | cca14728c899 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| c91a743316e90353453124cb | DOGEUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | d26a00509041 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| bf629eae41fd47b705d8277b | DOTUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 4425ee0d5916 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 89c4613d1d72efa008bc0499 | DOTUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | dfb8999fc4d4 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 14cfdcad3095953593d732bf | ETHUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 86ffc00c6e20 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 743d69735dec5f4d37cf225b | ETHUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | b85fe0567ce7 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| e1f06101d6a34ed9996e91d9 | EURAUD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 8e14bc294a85 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 0e26317ceb22c2b314ebff9f | EURAUD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 3f86cec96a8b | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| bea38c79d0e834eb4656154e | EURGBP | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 22effe9a4ca7 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| f24de31428e722feeeecf67c | EURGBP | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 30f317e3f7f2 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 0cd2fa1723ea86507b689380 | EURJPY | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e2b7049b0732 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 9139404eb2d76b61ad71f5fc | EURJPY | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 79bc6927ea06 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| c4b169c2bf658cafad387409 | EURUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | da281bc8a288 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 65a0e0cfc68c2fbc8e10430e | EURUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 0c0711202c3e | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| b808e8015d85275495a2e27e | GBPJPY | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 056a317dae9a | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| be6b0c0157eb9ae776ad779a | GBPJPY | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 8f018aa76a87 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 700f4652d66d5583480961af | GBPUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 55f073399ad1 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| cc6ade232fa2bfdfc271f767 | GBPUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 7b64e07db063 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 3d77d6cd049d93cde8e6be62 | GER40 | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–15:30 | f975e4625c7a | missing_segments | 527 | False | 0 | clean_first_read | outside_protected_intervals |
| 15f3db9f48a3f3e9a904f7a7 | GER40 | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–15:30 | 3a2bb27841b2 | missing_segments | 527 | False | 0 | clean_first_read | outside_protected_intervals |
| 7ecde84ce8cbe6c1353333b4 | JPN225 | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–06:30 | b775f2778895 | missing_segments | 341 | False | 0 | clean_first_read | outside_protected_intervals |
| 61ed57fd3eccc7f0a9b57574 | JPN225 | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–06:30 | 9c7a121eb4e4 | missing_segments | 341 | False | 0 | clean_first_read | outside_protected_intervals |
| 085502ec03defd273479daa9 | LINKUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 5c083853bfc4 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 38d8626b7d50661152403e83 | LINKUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 07a782ee4dd4 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 3aa3c12411e861249073242b | LTCUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 26a6572023bd | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 65bd630ef242a0cf32c4cf98 | LTCUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 8ea897a26118 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 71c12956d026bd69a24d715f | NAS100 | 2025-07-07 | registered_local_cache | 13:30–17:30 | 13:30–20:00 | 1c5dae0c7898 | missing_segments | 403 | False | 0 | clean_first_read | outside_protected_intervals |
| 00b72c8130b8d25638b1b9b8 | NAS100 | 2025-07-14 | registered_local_cache | 13:30–17:30 | 13:30–20:00 | afa1bec8d1c7 | missing_segments | 403 | False | 0 | clean_first_read | outside_protected_intervals |
| 0e2c4ca5acaccd715d6b2558 | NZDUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e9d241e6d92e | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 01778b487df4e97b892ad3f5 | NZDUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e9a9544e0beb | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 5f9235d027c1422bf1b936e5 | NatGas | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e3886124c5c6 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 2cd0e59019c7ee2b2e7efd59 | NatGas | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | fbe1a4ac1384 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 39f0f8e2e1fd602374ca71dd | SOLUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 8491587ffa62 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 12b930c576e5ff65ee3ed7f9 | SOLUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | e3279aaa886d | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 835d49e718917e037ce90b18 | SpotBrent | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 93342c85bbe5 | missing_segments | 270 | True | 20 | clean_first_read | outside_protected_intervals |
| 48294fd137584a2725661351 | SpotBrent | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 560a4c7341a6 | missing_segments | 140 | True | 20 | clean_first_read | outside_protected_intervals |
| a3f3c4991fe881e8160596d7 | SpotCrude | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | cef26bc31b41 | missing_segments | 269 | True | 20 | clean_first_read | outside_protected_intervals |
| 8970de420891b9d784ed058b | SpotCrude | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 34e834b8b187 | missing_segments | 139 | True | 20 | clean_first_read | outside_protected_intervals |
| ab45e47ee50f64b8ee456c7a | UK100 | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–15:30 | 0b8f4a575d4a | missing_segments | 527 | False | 0 | clean_first_read | outside_protected_intervals |
| d454eff7237c800488e74140 | UK100 | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–15:30 | 0fd37d216976 | missing_segments | 527 | False | 0 | clean_first_read | outside_protected_intervals |
| 76eb3c77f3dabbcf4dba6276 | US30 | 2025-07-07 | registered_local_cache | 13:30–17:30 | 13:30–20:00 | 23087c38d07d | missing_segments | 403 | False | 0 | clean_first_read | outside_protected_intervals |
| 87550782f1b702dfe5d60f74 | US30 | 2025-07-14 | registered_local_cache | 13:30–17:30 | 13:30–20:00 | b8c20a0da67e | missing_segments | 403 | False | 0 | clean_first_read | outside_protected_intervals |
| 136bcb5c0c607b3895112788 | US500 | 2025-07-07 | registered_local_cache | 13:30–17:30 | 13:30–20:00 | c1b1ccd1216e | missing_segments | 147 | True | 7 | clean_first_read | outside_protected_intervals |
| 068cb592754e6491f5762257 | US500 | 2025-07-14 | registered_local_cache | 13:30–17:30 | 13:30–20:00 | 051445d6f45c | missing_segments | 82 | True | 7 | clean_first_read | outside_protected_intervals |
| b7bae7a84725303d1db2a890 | USDCAD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | aa5978286ecc | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 5c911d7be10e49425b40f084 | USDCAD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 2a77a7f977ce | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| e663d35316b51ee1e2aa46cb | USDCHF | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 5cbc2bb76fe5 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 57429c86356f6754dfc49911 | USDCHF | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | ddd335c4e97b | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 4439405cdd634f063a0783b0 | USDJPY | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 9136937afeec | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 00e43213f810417444ca96a6 | USDJPY | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e919621c54ec | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 310539d714446e4a4c51da87 | XAGUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 133691444396 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| ed2e5b33c6527114311316f0 | XAGUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 64b6807ad42a | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 7106ba337a214ad20426e39f | XAUUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 8de445ab8048 | missing_segments | 269 | True | 20 | clean_first_read | outside_protected_intervals |
| 68f8dac888fe05b1d594711a | XAUUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | ff676172ccb3 | missing_segments | 139 | True | 20 | clean_first_read | outside_protected_intervals |
| 26525ea248404210163c4b11 | XLMUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | ff614f7678e4 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 46b412edad6dfcfa2ff86d70 | XLMUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 48169e1212c4 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| f2e972076104fd4e9dea8c3c | XPDUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 41b97c6880ce | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| ec3fd68245aaf601c4fee57e | XPDUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | ea241c8c9ecb | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 9203b0224ae6c0554bc038b8 | XPTUSD | 2025-07-07 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 434e99bf4676 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 7b2e427ebc4ffbf8e1c2543c | XPTUSD | 2025-07-14 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e4bfa2360b72 | missing_segments | 806 | False | 0 | clean_first_read | outside_protected_intervals |
| 1844d0a38c5f6679ac288e0a | XRPUSD | 2025-07-07 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | cbd22c349c08 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 22f81ff70d4bf24ff650f483 | XRPUSD | 2025-07-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 12c4a80d0e08 | missing_segments | 2160 | False | 0 | clean_first_read | outside_protected_intervals |
| 353c8d0f24920df5eee2f18a | BTCUSD | 2025-09-08 | registered_local_cache | 00:00–04:00 | Unavailable/quarantined | 6924836f601a | missing_segments | — | False | 0 | unverified_receipt_history | quarantined_protected_dependency |
| 8482bc8da89d67452042680a | BTCUSD | 2025-09-09 | registered_local_cache | 00:00–04:00 | Unavailable/quarantined | 11d151c42825 | missing_segments | — | False | 0 | unverified_receipt_history | quarantined_protected_dependency |
| 824360a5e806bc48a71f861b | BTCUSD | 2025-09-10 | registered_local_cache | 00:00–04:00 | Unavailable/quarantined | 91ac8a286753 | missing_segments | — | False | 0 | unverified_receipt_history | quarantined_protected_dependency |
| 638dcbdc0b410620e837382d | BTCUSD | 2025-09-11 | registered_local_cache | 00:00–04:00 | Unavailable/quarantined | 86deb1fab2a1 | missing_segments | — | False | 0 | unverified_receipt_history | quarantined_protected_dependency |
| a69d98a9b0857fba4468653a | BTCUSD | 2025-09-12 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 955965b8f186 | missing_segments | 1872 | True | 1344 | unverified_receipt_history | outside_protected_intervals |
| c891a278d9291a11a5ed214f | BTCUSD | 2025-09-13 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 9faae412646f | missing_segments | 1824 | True | 1296 | unverified_receipt_history | outside_protected_intervals |
| c065a06364ed5a3f4d4daff8 | BTCUSD | 2025-09-14 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | d8514226607a | missing_segments | 1776 | True | 1248 | unverified_receipt_history | outside_protected_intervals |
| 13413d5eeba6ae5b95025343 | BTCUSD | 2025-09-15 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 6a1da8663851 | missing_segments | 1728 | True | 1200 | unverified_receipt_history | outside_protected_intervals |
| 3167b31ba20ab20de427348f | BTCUSD | 2025-09-16 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | a5cc85a1eedf | missing_segments | 1680 | True | 1152 | unverified_receipt_history | outside_protected_intervals |
| 022ba7185edee7254c4d8857 | BTCUSD | 2025-09-17 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 3ee88cfa7568 | missing_segments | 1632 | True | 1104 | unverified_receipt_history | outside_protected_intervals |
| 4565a5781bcced588063121c | BTCUSD | 2025-09-18 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 071016d656d8 | missing_segments | 1584 | True | 1056 | unverified_receipt_history | outside_protected_intervals |
| bad969c61c67c207ed088029 | BTCUSD | 2025-09-19 | registered_local_cache | 00:00–04:00 | 00:00–00:00 | 77680d874893 | missing_segments | 1536 | True | 1008 | unverified_receipt_history | outside_protected_intervals |
| 1dde41961c5513d2e49f48f6 | NatGas | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 1e7e9f821104 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 6992c66eff6791f4aff090ed | NatGas | 2025-09-09 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 254525d6b049 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| ccf51afdb044eabeb17a21c9 | NatGas | 2025-09-10 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 5fce2cc2bf59 | missing_segments | 832 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 2f79bd5f417eb04353c3854c | NatGas | 2025-09-11 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 606fd35649ee | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 9032a14bc79bf8cd0e653691 | NatGas | 2025-09-12 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 3823082c058f | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 9e443883551d203d2332f422 | NatGas | 2025-09-15 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 978abf581836 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 0adad7fc27bc689a1a137e02 | NatGas | 2025-09-16 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 64182c78f556 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 9aff81d1ca11c7ddef52d6f5 | NatGas | 2025-09-17 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | f93e01b3dc66 | missing_segments | 832 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| e6a8f2151786f69b9f37a433 | NatGas | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | e7e0339610fa | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 808049d904cbddafaa9e1126 | NatGas | 2025-09-19 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | a63b9de57ca9 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| cdb6706f8187637f3fe3352d | SpotBrent | 2024-11-04 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | 2c526b1cc569 | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| 3609573a2a98577036cc248d | SpotBrent | 2024-11-05 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | 407cbb2d1a50 | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| 7e1f8994fecf745690603eeb | SpotBrent | 2024-11-06 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | d2c18210726a | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| 2dc02ac0109a8eeb3762a22c | SpotBrent | 2024-11-07 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | c2ea4814ad7e | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| d7ce2442e8c5109477cfdd52 | SpotBrent | 2024-11-08 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | d1235d74c4f1 | complete_session_aware | 0 | True | 0 | clean_first_read | outside_protected_intervals |
| a92fb0ae4794ae5899abbb1d | SpotBrent | 2025-04-07 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | ee4ced7cf0e2 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| f5233c2f0a824dfd2f998e89 | SpotBrent | 2025-04-08 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 0219cd9ef4b3 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| c3a0d383d87130df509791fa | SpotBrent | 2025-04-09 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 9447f81e8018 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| 1f6ef75ca564ffa87d523f97 | SpotBrent | 2025-04-10 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 90be67edc556 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| 70e3acf0ef22a39ed2354106 | SpotBrent | 2025-04-11 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | acff5e90d2ef | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| a451c5daee55ffdac04ccd9a | SpotCrude | 2024-11-04 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | 91ea71a536a1 | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| e060f88cb7dc74c96c07b68b | SpotCrude | 2024-11-05 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | 5cd00dbdaf97 | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| 910dc7f401c979f077122690 | SpotCrude | 2024-11-06 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | 0d1237422548 | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| 0f95ad7a94031afd0dea3e9d | SpotCrude | 2024-11-07 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | 952bfaf323a8 | complete_session_aware | 0 | True | 10 | clean_first_read | outside_protected_intervals |
| be7e37171731f34c82648fa5 | SpotCrude | 2024-11-08 | 2024_Q4_Election_Vol | 07:00–17:00 | 07:00–20:00 | c4e548da1bdd | complete_session_aware | 0 | True | 0 | clean_first_read | outside_protected_intervals |
| 77b10f0f75d5383bb011c4b2 | SpotCrude | 2025-04-07 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 987cb0e0a519 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| 2b977b08318352c8c79f2cbc | SpotCrude | 2025-04-08 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 272715beb972 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| b8444e851d499207d0449237 | SpotCrude | 2025-04-09 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | d74c6f509737 | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| 9e626ae09c741e2673fd8c78 | SpotCrude | 2025-04-10 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 2ed69f12402c | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| ce5a75c11b08e52a205e5350 | SpotCrude | 2025-04-11 | 2025_Q2_Tariff_Vol | 07:00–17:00 | 07:00–20:00 | 2098bd4c155b | complete_session_aware | 0 | True | 8 | clean_first_read | outside_protected_intervals |
| 0c8d1df35ca48a63714fb422 | US500 | 2024-11-04 | 2024_Q4_Election_Vol | 14:30–18:30 | 14:30–21:00 | 7f96f1152cd4 | missing_segments | 2 | True | 7 | clean_first_read | outside_protected_intervals |
| 4d04e030932184cbabc7f97b | US500 | 2024-11-05 | 2024_Q4_Election_Vol | 14:30–18:30 | 14:30–21:00 | a70f0b0938d4 | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| 8d7090e5c928d51baf5f5d28 | US500 | 2024-11-06 | 2024_Q4_Election_Vol | 14:30–18:30 | 14:30–21:00 | e79e3f85a721 | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| f35c2955c68672466ebc4e06 | US500 | 2024-11-07 | 2024_Q4_Election_Vol | 14:30–18:30 | 14:30–21:00 | 56e422e4daaa | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| b8faf0a449b13f21ebd8db35 | US500 | 2024-11-08 | 2024_Q4_Election_Vol | 14:30–18:30 | 14:30–21:00 | 3a70df80573c | complete_session_aware | 0 | True | 0 | clean_first_read | outside_protected_intervals |
| 31ac96bb6c2db5e56d25c188 | US500 | 2025-04-07 | 2025_Q2_Tariff_Vol | 13:30–17:30 | 13:30–20:00 | dd283da9afaa | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| 5b35427f494f9de75bb0ddd1 | US500 | 2025-04-08 | 2025_Q2_Tariff_Vol | 13:30–17:30 | 13:30–20:00 | 8f14641b7f92 | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| 7be5dc89274ab1082ba3e7ba | US500 | 2025-04-09 | 2025_Q2_Tariff_Vol | 13:30–17:30 | 13:30–20:00 | ae1f6a6bce0f | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| 5ff6b8a3b787709ae2dcc9a0 | US500 | 2025-04-10 | 2025_Q2_Tariff_Vol | 13:30–17:30 | 13:30–20:00 | 08b8d4507c71 | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| acdc1e0257e10eefaf512a7f | US500 | 2025-04-11 | 2025_Q2_Tariff_Vol | 13:30–17:30 | 13:30–20:00 | e3564260fd01 | complete_session_aware | 0 | True | 7 | clean_first_read | outside_protected_intervals |
| caca0aae14aacd9f05311212 | XAUUSD | 2024-11-04 | 2024_Q4_Election_Vol | 07:00–11:00 | 07:00–20:00 | deb202c0514d | complete_session_aware | 0 | True | 22 | clean_first_read | outside_protected_intervals |
| b92b7b077dd282ac80b2c042 | XAUUSD | 2024-11-05 | 2024_Q4_Election_Vol | 07:00–11:00 | 07:00–20:00 | 1d77766c2b5d | complete_session_aware | 0 | True | 22 | clean_first_read | outside_protected_intervals |
| 877ac3424c0551b6f7772d6a | XAUUSD | 2024-11-06 | 2024_Q4_Election_Vol | 07:00–11:00 | 07:00–20:00 | 6e006e0e3b21 | complete_session_aware | 0 | True | 22 | clean_first_read | outside_protected_intervals |
| 569183452c60779cd669e2eb | XAUUSD | 2024-11-07 | 2024_Q4_Election_Vol | 07:00–11:00 | 07:00–20:00 | d05de281de7b | complete_session_aware | 0 | True | 22 | clean_first_read | outside_protected_intervals |
| f741e94dbeb081106be380a1 | XAUUSD | 2024-11-08 | 2024_Q4_Election_Vol | 07:00–11:00 | 07:00–20:00 | 47852c7462b8 | complete_session_aware | 0 | True | 0 | clean_first_read | outside_protected_intervals |
| ae86f219cd744d0fe05a351e | XAUUSD | 2025-04-07 | 2025_Q2_Tariff_Vol | 07:00–11:00 | 07:00–20:00 | 5ed344e1f833 | complete_session_aware | 0 | True | 20 | clean_first_read | outside_protected_intervals |
| 44e4ecb72fbb53cc14fc14da | XAUUSD | 2025-04-08 | 2025_Q2_Tariff_Vol | 07:00–11:00 | 07:00–20:00 | c06e6d65be20 | complete_session_aware | 0 | True | 20 | clean_first_read | outside_protected_intervals |
| 3da7750ba2a191ca753fb2e9 | XAUUSD | 2025-04-09 | 2025_Q2_Tariff_Vol | 07:00–11:00 | 07:00–20:00 | 587907b5c388 | complete_session_aware | 0 | True | 20 | clean_first_read | outside_protected_intervals |
| 46371ae1ac4c72c2b4aa725b | XAUUSD | 2025-04-10 | 2025_Q2_Tariff_Vol | 07:00–11:00 | 07:00–20:00 | 58328740525b | complete_session_aware | 0 | True | 20 | clean_first_read | outside_protected_intervals |
| d9c1023f5de634fed865d7b3 | XAUUSD | 2025-04-11 | 2025_Q2_Tariff_Vol | 07:00–11:00 | 07:00–20:00 | ec863e218a6b | complete_session_aware | 0 | True | 0 | clean_first_read | outside_protected_intervals |
| f3ec49598c955968c2cdec80 | NAS100 | 2024-01-22 | registered_local_cache | 14:30–18:30 | 14:30–21:00 | a8052e9a0149 | missing_segments | 403 | True | 0 | clean_first_read | outside_protected_intervals |
| 74608bc5524e70321ed4bed1 | NAS100 | 2024-01-23 | registered_local_cache | 14:30–18:30 | 14:30–21:00 | 4ec652a398fe | missing_segments | 395 | True | 0 | clean_first_read | outside_protected_intervals |
| 7048b77cb373429eafea7db7 | NAS100 | 2024-01-24 | registered_local_cache | 14:30–18:30 | 14:30–21:00 | 975a73ae3d2b | missing_segments | 400 | True | 0 | clean_first_read | outside_protected_intervals |
| dae3cca41d4e71510881f096 | NAS100 | 2024-01-25 | registered_local_cache | 14:30–18:30 | 14:30–21:00 | 28b16ebe3ae2 | missing_segments | 405 | True | 0 | clean_first_read | outside_protected_intervals |
| 2549b6349e4927934cea6b9a | NAS100 | 2024-01-26 | registered_local_cache | 14:30–18:30 | 14:30–21:00 | 21ab2c3a6cef | missing_segments | 397 | True | 0 | clean_first_read | outside_protected_intervals |
| fd538dee2dfee62bbdfef31a | XAGUSD | 2024-01-22 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 27422c2d2120 | missing_segments | 806 | True | 0 | clean_first_read | outside_protected_intervals |
| a7db221453c42363d5851c6b | XAGUSD | 2024-01-23 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 35f3fd493db9 | missing_segments | 798 | True | 0 | clean_first_read | outside_protected_intervals |
| 7b4c0272869b56392daed4d2 | XAGUSD | 2024-01-24 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | c2204738e504 | missing_segments | 816 | True | 0 | clean_first_read | outside_protected_intervals |
| 4688021d87094d090147d2bf | XAGUSD | 2024-01-25 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 0a8136d3cf49 | missing_segments | 834 | True | 0 | clean_first_read | outside_protected_intervals |
| 9d283ed54947635561710d02 | XAGUSD | 2024-01-26 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 72f08f3c4bea | missing_segments | 826 | True | 0 | clean_first_read | outside_protected_intervals |
| 265645a3be096326aed2aedb | AUDUSD | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 5feba92a1b3f | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 29f1fdeef321960dd072b797 | AUDUSD | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | a0cd43faa62d | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| dfb552bb618ed7a79347c413 | AUDUSD | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | f29c9077cc64 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 080470d042651b8c655dad9b | EURUSD | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 7456d174aaee | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 5b124b6cb82e8d11d58e206e | EURUSD | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | d8a3634f24a0 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| af2df2b9ecb95ec99e4ba214 | EURUSD | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | d7366f7621f9 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 10ccc5a2f8457ba6c63cade0 | GBPUSD | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | ee3bd60d26bb | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 88de0a62804d4051c67da23a | GBPUSD | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | b9933666b53d | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 203cb97bcdb4d6aa66c41465 | GBPUSD | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 1f6c416f1a7e | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 29d67181a212c5d322ae2605 | NZDUSD | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 4a4d23d8c242 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| ff8833c6c0408e7b986fa556 | NZDUSD | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 6b0ac69d6dbd | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| ee47bfaaab75edf28033ae92 | NZDUSD | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | a915a48d5290 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| b7effcb8473375ad72e30a33 | USDCAD | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 99bb61586346 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 4393face0ef1ff16fe668670 | USDCAD | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 580e7848c1c0 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 8a730a4fb8297af23d51c185 | USDCAD | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 5cca84129ac8 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 2c47602726b787377b989764 | USDCHF | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 0b403ee5ea10 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 4dd8f646786d33e103b4eaa9 | USDCHF | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | dc9e9e97f1a4 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 6c2a25e90c123f22ec1074ed | USDCHF | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | da9a75a4aeba | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| d944ef2b2fa31478531904b6 | USDJPY | 2025-09-08 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 48425f062cee | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 77ccbe44afa519c5a61a7381 | USDJPY | 2025-09-08 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 7e0dc91f536b | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| a9a4144930ba0f42db95eb6d | USDJPY | 2025-09-08 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | e7a0fff311d7 | missing_segments | 806 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| b82bbb14c003d4093cf77eab | AUDUSD | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 60827b7707b9 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| fcc1d48a348abfd833bc9b55 | AUDUSD | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | c6ed809defb2 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| f9f9893d28f7f5860313eda7 | AUDUSD | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 6f3789cf8f2d | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 55b1ac5ae47e542c569461ce | EURUSD | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 0b583c1038cf | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 31a1e202c0e8c69e112eab89 | EURUSD | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 3cdbb2152560 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 61c07a5a8818486dbf3defaf | EURUSD | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | ae53780016c9 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 22ce4c1bc5a5125f0f639e86 | GBPUSD | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 6a5156c64804 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 7a38284e1e023c49abb24d83 | GBPUSD | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | f708b6628c2b | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| b09f25a7c4841418d340291f | GBPUSD | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 14d862d38ac5 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 7fab6f81eb13a87cbd74c74c | NZDUSD | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 528b186be40a | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 78d0423f69c649afdc4c2cff | NZDUSD | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 78625cc37d1a | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 4077f769bc94792d8903123a | NZDUSD | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | bd98d0eca208 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 331ab23fa5ab236d5c9725c8 | USDCAD | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 01db48aaa315 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| ea6052d2a879cb4641909cae | USDCAD | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 42885e16a123 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 1a9e734a30e7a884b29566db | USDCAD | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 6705f8cc29ec | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 107a61488594299274e2e70f | USDCHF | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 2c1c967247b4 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 5955503cea2dcfc86e473dba | USDCHF | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | d2133d807051 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 726d63629c7dbe1067c5949d | USDCHF | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 36b30e18f1af | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 2706d186ee9c9c2bf2cb97f8 | USDJPY | 2025-09-18 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 1ada77fdd441 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| b069a6d65fff6680a2fc4ca6 | USDJPY | 2025-09-18 | registered_local_cache | 12:00–16:00 | 07:00–20:00 | 476aea8f9033 | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 428da4156a95703bac2255f6 | USDJPY | 2025-09-18 | registered_local_cache | 00:00–04:00 | 07:00–20:00 | 18fa04cf815b | missing_segments | 858 | False | 0 | unverified_receipt_history | outside_protected_intervals |
| 6381f9217d6a5d1c2c3794b4 | BTCUSD | 2024-01-22 | 2024_Q1_Winter | 00:00–04:00 | 00:00–00:00 | c5446472c233 | missing_segments | 555 | True | 192 | clean_first_read | outside_protected_intervals |
| 4c63892002b69ae4559435bd | BTCUSD | 2024-01-23 | 2024_Q1_Winter | 00:00–04:00 | 00:00–00:00 | 62e8d065660e | missing_segments | 507 | True | 144 | clean_first_read | outside_protected_intervals |
| 46b1da2b93e2935f42ee3164 | BTCUSD | 2024-01-24 | 2024_Q1_Winter | 00:00–04:00 | 00:00–00:00 | 34a3a9b492ae | missing_segments | 459 | True | 96 | clean_first_read | outside_protected_intervals |
| bf44e6b3bcb58bb2906fe359 | BTCUSD | 2024-01-25 | 2024_Q1_Winter | 00:00–04:00 | 00:00–00:00 | 13ed2c9c098d | missing_segments | 411 | True | 48 | clean_first_read | outside_protected_intervals |
| 83bd140279a8e96c93fb6105 | BTCUSD | 2024-01-26 | 2024_Q1_Winter | 00:00–04:00 | 00:00–00:00 | a39cf519e5f4 | missing_segments | 363 | True | 0 | clean_first_read | outside_protected_intervals |
| 8199d9cbcec7df48ffc053bd | BTCUSD | 2024-07-22 | 2024_Q3_Summer | 00:00–04:00 | 00:00–00:00 | e83c94b9ecab | missing_segments | 84 | True | 192 | clean_first_read | outside_protected_intervals |
| 46dbae667d09a4839bc21925 | BTCUSD | 2024-07-23 | 2024_Q3_Summer | 00:00–04:00 | 00:00–00:00 | 8716efc0e9ae | missing_segments | 82 | True | 144 | clean_first_read | outside_protected_intervals |
| 17a309f47a71bca0f571e4d8 | BTCUSD | 2024-07-24 | 2024_Q3_Summer | 00:00–04:00 | 00:00–00:00 | 5de78d3f914e | missing_segments | 60 | True | 96 | clean_first_read | outside_protected_intervals |
| 4a422a139782982730a96c44 | BTCUSD | 2024-07-25 | 2024_Q3_Summer | 00:00–04:00 | 00:00–00:00 | 4f648912a7e2 | missing_segments | 56 | True | 48 | clean_first_read | outside_protected_intervals |
| 8550e593edbfb863371e3be3 | BTCUSD | 2024-07-26 | 2024_Q3_Summer | 00:00–04:00 | 00:00–00:00 | d02b47088700 | missing_segments | 56 | True | 0 | clean_first_read | outside_protected_intervals |
| e2f8731ee1bbbd8ebec301bb | BTCUSD | 2025-05-12 | 2025-May | 00:00–04:00 | 00:00–00:00 | 44d2bc43268a | missing_segments | 41 | True | 192 | clean_first_read | outside_protected_intervals |
| 4350fd4ccc6e7fa2e964e43c | BTCUSD | 2025-05-13 | 2025-May | 00:00–04:00 | 00:00–00:00 | 1178b78fe90c | missing_segments | 39 | True | 144 | clean_first_read | outside_protected_intervals |
| ea65ff19fabbc0109ea7c967 | BTCUSD | 2025-05-14 | 2025-May | 00:00–04:00 | 00:00–00:00 | 9eafa9ddb746 | missing_segments | 39 | True | 96 | clean_first_read | outside_protected_intervals |
| 9ce2dd036808138bac5852eb | BTCUSD | 2025-05-15 | 2025-May | 00:00–04:00 | 00:00–00:00 | d542dc0dd92e | missing_segments | 35 | True | 48 | clean_first_read | outside_protected_intervals |
| 8a9060211292afa3d94ff18b | BTCUSD | 2025-05-16 | 2025-May | 00:00–04:00 | 00:00–00:00 | 9444e865da33 | missing_segments | 35 | True | 0 | clean_first_read | outside_protected_intervals |
| 39f5596f1015e6d5706ba25b | BTCUSD | 2025-10-06 | 2025_Q4_PreHoldout | 00:00–04:00 | 00:00–00:00 | e69fc5cc36eb | missing_segments | 720 | True | 192 | clean_first_read | outside_protected_intervals |
| 5faad815c6cd7484d9d45885 | BTCUSD | 2025-10-07 | 2025_Q4_PreHoldout | 00:00–04:00 | 00:00–00:00 | 5b833510e132 | missing_segments | 672 | True | 144 | clean_first_read | outside_protected_intervals |
| daaa576e731c68edb339d1a8 | BTCUSD | 2025-10-08 | 2025_Q4_PreHoldout | 00:00–04:00 | 00:00–00:00 | e8168f91c6fe | missing_segments | 624 | True | 96 | clean_first_read | outside_protected_intervals |
| 7e2b39cab272c70c3468188b | BTCUSD | 2025-10-09 | 2025_Q4_PreHoldout | 00:00–04:00 | 00:00–00:00 | b76c297db69b | missing_segments | 576 | True | 48 | clean_first_read | outside_protected_intervals |
| aae2661c0700eeb400008256 | BTCUSD | 2025-10-10 | 2025_Q4_PreHoldout | 00:00–04:00 | 00:00–00:00 | d72502fdf16b | missing_segments | 528 | False | 0 | clean_first_read | outside_protected_intervals |
| a51722a99e186dedf667cb58 | SpotBrent | 2024-01-22 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | 645a8d7f6a83 | missing_segments | 52 | True | 24 | clean_first_read | outside_protected_intervals |
| 2d22569eb65ac698f7c74c93 | SpotBrent | 2024-01-23 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | d3d36220d002 | missing_segments | 52 | True | 24 | clean_first_read | outside_protected_intervals |
| 413acab1546b60736c1ea52d | SpotBrent | 2024-01-24 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | 1a28aa663f34 | missing_segments | 52 | True | 24 | clean_first_read | outside_protected_intervals |
| f2ddfe7b0570504c76b992ab | SpotBrent | 2024-01-25 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | 1c729223ba22 | missing_segments | 52 | True | 24 | clean_first_read | outside_protected_intervals |
| 550647596835f77c4cbbcfda | SpotBrent | 2024-01-26 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | 12a7bda287bc | missing_segments | 52 | True | 12 | clean_first_read | outside_protected_intervals |
| 2f64878c52b286c335f65e4a | SpotBrent | 2024-07-22 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 82031a48d908 | missing_segments | 12 | True | 20 | clean_first_read | outside_protected_intervals |
| 6ebdfc6f362294e0e607bf79 | SpotBrent | 2024-07-23 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 86444c07efcc | missing_segments | 12 | True | 20 | clean_first_read | outside_protected_intervals |
| 1a897af29e15f58be0c69494 | SpotBrent | 2024-07-24 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 93e283b3a78d | missing_segments | 12 | True | 20 | clean_first_read | outside_protected_intervals |
| b6074168c8eb0dc0e2f8ae9a | SpotBrent | 2024-07-25 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 6a543b783908 | missing_segments | 12 | True | 20 | clean_first_read | outside_protected_intervals |
| c6668faccec1c79439413775 | SpotBrent | 2024-07-26 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | d8a664a4ed9e | missing_segments | 12 | True | 12 | clean_first_read | outside_protected_intervals |
| 3ddc5b55e115b2e2e1936c1c | SpotBrent | 2025-05-12 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | f3849f3cae51 | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| e784ecf99abd10f7c3a1ae30 | SpotBrent | 2025-05-13 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | 51940641d5ef | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| 8b39ab6c712fb7c7397fdfdb | SpotBrent | 2025-05-14 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | 4e4b7af1e2d5 | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| affe109b180bfeb08d170ec7 | SpotBrent | 2025-05-15 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | 733c776829bf | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| e77a82873dfae309f20bfbc8 | SpotBrent | 2025-05-16 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | 5d3b49189d63 | missing_segments | 26 | True | 0 | clean_first_read | outside_protected_intervals |
| 0d8adf49267297042d98d06a | SpotBrent | 2025-10-06 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | 1ac86a109457 | missing_segments | 286 | True | 20 | clean_first_read | outside_protected_intervals |
| 362375ec6c45fb8d7a2b7d3f | SpotBrent | 2025-10-07 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | dfafce1eed0f | missing_segments | 260 | True | 20 | clean_first_read | outside_protected_intervals |
| a859f91f6cb95922ddcbacb1 | SpotBrent | 2025-10-08 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | a5ea210c4d7d | missing_segments | 260 | True | 20 | clean_first_read | outside_protected_intervals |
| c905518f0136641e39bad9c5 | SpotBrent | 2025-10-09 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | 48c3497b593d | missing_segments | 260 | True | 20 | clean_first_read | outside_protected_intervals |
| 64641fa516e0860f2ba2f3ec | SpotBrent | 2025-10-10 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | e1152e733759 | missing_segments | 234 | False | 0 | clean_first_read | outside_protected_intervals |
| ab6aca1489429fe387e656d0 | SpotCrude | 2024-01-22 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | 802484c1bd97 | missing_segments | 53 | True | 22 | clean_first_read | outside_protected_intervals |
| 8507537493997a182445c464 | SpotCrude | 2024-01-23 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | a87cb06568bc | missing_segments | 53 | True | 22 | clean_first_read | outside_protected_intervals |
| 4a7617b76a56942a450d6768 | SpotCrude | 2024-01-24 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | 34235262c572 | missing_segments | 53 | True | 22 | clean_first_read | outside_protected_intervals |
| d1bc6b75ea34848580a14441 | SpotCrude | 2024-01-25 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | c254ed9e8db7 | missing_segments | 53 | True | 22 | clean_first_read | outside_protected_intervals |
| 83dde1f2c6ae2b80589433bf | SpotCrude | 2024-01-26 | 2024_Q1_Winter | 07:00–11:00 | 07:00–20:00 | fc52c2eb4616 | missing_segments | 53 | True | 12 | clean_first_read | outside_protected_intervals |
| 20abc09e536b839a6e7fe9c8 | SpotCrude | 2024-07-22 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 7a0af499b39e | missing_segments | 8 | True | 20 | clean_first_read | outside_protected_intervals |
| e48e81f1c2a54ffeea4d2c00 | SpotCrude | 2024-07-23 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 6b77c326d410 | missing_segments | 8 | True | 20 | clean_first_read | outside_protected_intervals |
| 86d05af4f69296fddf2b69d9 | SpotCrude | 2024-07-24 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | e9ddf96bf0df | missing_segments | 8 | True | 20 | clean_first_read | outside_protected_intervals |
| 13d1b94f806804837755c3fb | SpotCrude | 2024-07-25 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | c74c522125b6 | missing_segments | 8 | True | 20 | clean_first_read | outside_protected_intervals |
| 16b8f2818209bcec0d6c2761 | SpotCrude | 2024-07-26 | 2024_Q3_Summer | 07:00–11:00 | 07:00–20:00 | 29f4c99f149e | missing_segments | 8 | True | 12 | clean_first_read | outside_protected_intervals |
| b29542347402bab6234839d6 | SpotCrude | 2025-05-12 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | f8f0ed0f6faa | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| d0e4ba21f85ecdbfbfe40a8b | SpotCrude | 2025-05-13 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | 0f69baa8d3e9 | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| 9c22155b1c7571aa5a99b515 | SpotCrude | 2025-05-14 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | 60aba46f1fe9 | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| 8d44e2c3e3ecabd890265eb0 | SpotCrude | 2025-05-15 | 2025_Q2_Spring | 07:00–11:00 | 07:00–20:00 | 098f1261d1e6 | missing_segments | 26 | True | 20 | clean_first_read | outside_protected_intervals |
| 27771eec2fd461a5bb8b89fc | SpotCrude | 2025-05-16 | registered_local_cache | 07:00–11:00 | 07:00–20:00 | a0888de0cb3e | missing_segments | 26 | True | 0 | clean_first_read | outside_protected_intervals |
| 134bb1b1c5cae3963b7a4c6c | SpotCrude | 2025-10-06 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | 1380ed339cb5 | missing_segments | 286 | True | 20 | clean_first_read | outside_protected_intervals |
| 5aca717f89d879a30446d2ed | SpotCrude | 2025-10-07 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | 3ff78383c626 | missing_segments | 260 | True | 20 | clean_first_read | outside_protected_intervals |
| 7c936f35c2bb48764b26236a | SpotCrude | 2025-10-08 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | 0f7ed898aaf5 | missing_segments | 260 | True | 20 | clean_first_read | outside_protected_intervals |
| 5294e3691e86e4143000b304 | SpotCrude | 2025-10-09 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | 0a02a46d236a | missing_segments | 260 | True | 20 | clean_first_read | outside_protected_intervals |
| 0d43b0a72fed3835f3c8e4ba | SpotCrude | 2025-10-10 | 2025_Q4_PreHoldout | 07:00–11:00 | 07:00–20:00 | a7f97fe1c3e8 | missing_segments | 234 | False | 0 | clean_first_read | outside_protected_intervals |
| 4fe286efdb366d24671e8115 | US500 | 2024-01-22 | 2024_Q1_Winter | 14:30–18:30 | 14:30–21:00 | a995295de93f | missing_segments | 403 | True | 0 | clean_first_read | outside_protected_intervals |
| 845b548701e7627dc3f53be3 | US500 | 2024-01-23 | 2024_Q1_Winter | 14:30–18:30 | 14:30–21:00 | 6fa79b15c391 | missing_segments | 403 | True | 0 | clean_first_read | outside_protected_intervals |
| c544f73b7b6de1593d3c133a | US500 | 2024-01-25 | 2024_Q1_Winter | 14:30–18:30 | 14:30–21:00 | 121a54b12f1d | missing_segments | 429 | True | 0 | clean_first_read | outside_protected_intervals |
| e56cc14a79f4db75fde05223 | US500 | 2024-01-26 | 2024_Q1_Winter | 14:30–18:30 | 14:30–21:00 | 97d4fee2560f | missing_segments | 429 | True | 0 | clean_first_read | outside_protected_intervals |
| 6681fcc669deff45d1f60f88 | US500 | 2024-07-22 | 2024_Q3_Summer | 13:30–17:30 | 13:30–20:00 | bc766d85ccd6 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 04ea9a8aed2ded217967b9a0 | US500 | 2024-07-23 | 2024_Q3_Summer | 13:30–17:30 | 13:30–20:00 | 85f0d62c8e91 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 001f989aba50039f6db56506 | US500 | 2024-07-24 | 2024_Q3_Summer | 13:30–17:30 | 13:30–20:00 | ea484433f352 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| ff90f42b6db284ffa5fee37a | US500 | 2024-07-25 | 2024_Q3_Summer | 13:30–17:30 | 13:30–20:00 | 51899a0406e2 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 9856b173f9625f9053df2d6a | US500 | 2024-07-26 | 2024_Q3_Summer | 13:30–17:30 | 13:30–20:00 | 87941d5857bb | missing_segments | 17 | True | 0 | clean_first_read | outside_protected_intervals |
| 4d12cf15cd8e08baa46e7d29 | US500 | 2025-05-12 | 2025-May | 13:30–17:30 | 13:30–20:00 | b3ef3ee6e556 | missing_segments | 13 | True | 7 | clean_first_read | outside_protected_intervals |
| cf4371537a3ed53252478781 | US500 | 2025-05-13 | 2025-May | 13:30–17:30 | 13:30–20:00 | 0b300f8e655a | missing_segments | 13 | True | 7 | clean_first_read | outside_protected_intervals |
| c1bf02ca4edb813e89f28dd7 | US500 | 2025-05-14 | 2025-May | 13:30–17:30 | 13:30–20:00 | 94b7fd397a35 | missing_segments | 13 | True | 7 | clean_first_read | outside_protected_intervals |
| 7781d2ba777facd710ca1457 | US500 | 2025-05-15 | 2025-May | 13:30–17:30 | 13:30–20:00 | 473ac847610d | missing_segments | 13 | True | 7 | clean_first_read | outside_protected_intervals |
| 43eeb5e76c9e47a95bccd800 | US500 | 2025-05-16 | 2025-May | 13:30–17:30 | 13:30–20:00 | f3f8d6b63ee0 | missing_segments | 13 | True | 0 | clean_first_read | outside_protected_intervals |
| 9546b2e4fa6800370e6b787f | US500 | 2025-10-06 | 2025_Q4_PreHoldout | 13:30–17:30 | 13:30–20:00 | f4bf38085ed3 | missing_segments | 143 | True | 7 | clean_first_read | outside_protected_intervals |
| da807a1142c70b322b781e37 | US500 | 2025-10-07 | 2025_Q4_PreHoldout | 13:30–17:30 | 13:30–20:00 | 17cf570ebbe9 | missing_segments | 130 | True | 7 | clean_first_read | outside_protected_intervals |
| 67e0c555d803a502cc15d6dd | US500 | 2025-10-08 | 2025_Q4_PreHoldout | 13:30–17:30 | 13:30–20:00 | c53131c32172 | missing_segments | 130 | True | 7 | clean_first_read | outside_protected_intervals |
| 25e98df1d7dd285726528311 | US500 | 2025-10-09 | 2025_Q4_PreHoldout | 13:30–17:30 | 13:30–20:00 | 36568092cb38 | missing_segments | 130 | True | 7 | clean_first_read | outside_protected_intervals |
| c47e497770890fb64a6335ad | US500 | 2025-10-10 | 2025_Q4_PreHoldout | 13:30–17:30 | 13:30–20:00 | ec726c5ca58a | missing_segments | 117 | False | 0 | clean_first_read | outside_protected_intervals |
| 7d29c206fcaad3e409105de3 | SpotBrent | 2023-01-23 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | c957f10d5989 | missing_segments | 78 | True | 12 | clean_first_read | outside_protected_intervals |
| 6e18297f6eeed480dc9234ba | SpotBrent | 2023-01-24 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 939170be53d4 | missing_segments | 78 | True | 12 | clean_first_read | outside_protected_intervals |
| d85383ce4fc5cc9e6b2a3788 | SpotBrent | 2023-01-25 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | c033b1fc3c1f | missing_segments | 78 | True | 12 | clean_first_read | outside_protected_intervals |
| 9d1d88c1aeae55f5d7850090 | SpotBrent | 2023-01-26 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | c248912b7f1d | missing_segments | 78 | True | 12 | clean_first_read | outside_protected_intervals |
| a6bc4b04896b9807e1b6e9c6 | SpotBrent | 2023-01-27 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 0d67bbf3af0a | missing_segments | 78 | True | 0 | clean_first_read | outside_protected_intervals |
| d2dea2acc6abeda4dd84ac5c | SpotBrent | 2023-07-24 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | eb270d01a172 | missing_segments | 10 | True | 10 | clean_first_read | outside_protected_intervals |
| 9beae18a5071ebb3f315a848 | SpotBrent | 2023-07-25 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | c0c789330a32 | missing_segments | 10 | True | 10 | clean_first_read | outside_protected_intervals |
| c8420ab56a4df29d24c8d1b5 | SpotBrent | 2023-07-26 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 6365ba88cf5a | missing_segments | 10 | True | 10 | clean_first_read | outside_protected_intervals |
| 5da65ba1cd0c09e8b1e69062 | SpotBrent | 2023-07-27 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 8ad1eb5edefb | missing_segments | 10 | True | 10 | clean_first_read | outside_protected_intervals |
| 6b43c186e44f7ce4adf78a84 | SpotBrent | 2023-07-28 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 3ec85e99ae00 | missing_segments | 10 | True | 0 | clean_first_read | outside_protected_intervals |
| 9e1b23b325aba77cbb703cee | SpotBrent | 2024-01-22 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | f1c8d903515b | missing_segments | 52 | True | 12 | inherited_repeat_interval | outside_protected_intervals |
| 58c200ae909a848ff926da74 | SpotBrent | 2024-01-23 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 6e362fb02153 | missing_segments | 52 | True | 12 | inherited_repeat_interval | outside_protected_intervals |
| 01bf260784267d93ce76e221 | SpotBrent | 2024-01-24 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | c5bef04b5789 | missing_segments | 52 | True | 12 | inherited_repeat_interval | outside_protected_intervals |
| 6397444a3039e1c4867dd15e | SpotBrent | 2024-01-25 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 5194f35cad89 | missing_segments | 52 | True | 12 | inherited_repeat_interval | outside_protected_intervals |
| 666c0bbe602d43cbf8baef30 | SpotBrent | 2024-01-26 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | c5b466eef033 | missing_segments | 52 | True | 0 | inherited_repeat_interval | outside_protected_intervals |
| c002e5ef4794429f5b842f26 | SpotBrent | 2024-07-22 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 9895178fb25e | missing_segments | 12 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| 8f979dc940d9b2ae211c6a90 | SpotBrent | 2024-07-23 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | fe5323f5f5e2 | missing_segments | 12 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| a2e5c8f71ba017158d5202a6 | SpotBrent | 2024-07-24 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 8cef052a0707 | missing_segments | 12 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| e5edfa214e564147e82e29b6 | SpotBrent | 2024-07-25 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 1edd535b4ab8 | missing_segments | 12 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| d6e7cff7bd1aef63aa0d67a1 | SpotBrent | 2024-07-26 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 10e0fd2e99c7 | missing_segments | 12 | True | 0 | inherited_repeat_interval | outside_protected_intervals |
| f5b1803cf39d6edeb396f06c | SpotBrent | 2025-02-10 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 46c38a119530 | missing_segments | 26 | True | 10 | clean_first_read | outside_protected_intervals |
| d7c8c21dd5d0ad94aed42504 | SpotBrent | 2025-02-11 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 61c97c4ef51c | missing_segments | 26 | True | 10 | clean_first_read | outside_protected_intervals |
| 73f72980deab43e63b63aa4e | SpotBrent | 2025-02-12 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | f47cf7423fd9 | missing_segments | 26 | True | 10 | clean_first_read | outside_protected_intervals |
| 4fe7cb2650e4b5c284d9356d | SpotBrent | 2025-02-13 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | d55c59857515 | missing_segments | 26 | True | 10 | clean_first_read | outside_protected_intervals |
| 4d2806f05a3b115aa753655d | SpotBrent | 2025-02-14 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 235e78c34853 | missing_segments | 26 | True | 0 | clean_first_read | outside_protected_intervals |
| 0223cba302b7e83b96407428 | SpotBrent | 2025-07-21 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 237fd73dbdda | missing_segments | 10 | True | 8 | clean_first_read | outside_protected_intervals |
| c4de64f1ffa0ef8e3de8fc0a | SpotBrent | 2025-07-22 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | b8c6e66d2a68 | missing_segments | 10 | True | 8 | clean_first_read | outside_protected_intervals |
| e876f67aef231b91b3c5c5a4 | SpotBrent | 2025-07-23 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 0d7050bcb21d | missing_segments | 10 | True | 8 | clean_first_read | outside_protected_intervals |
| 8c5697c68cd1fc74ace69625 | SpotBrent | 2025-07-24 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 4e69154e34de | missing_segments | 10 | True | 8 | clean_first_read | outside_protected_intervals |
| 258b031ea15e984f9aa6c571 | SpotBrent | 2025-07-25 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 7f9b7a033a0c | missing_segments | 10 | True | 0 | clean_first_read | outside_protected_intervals |
| c3e3b322f89aa16d477fe5d7 | SpotCrude | 2023-01-23 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 03268e06c9f8 | missing_segments | 53 | True | 10 | clean_first_read | outside_protected_intervals |
| 355ca1c07adee0d1aeb493fd | SpotCrude | 2023-01-24 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 98df0ecf1dca | missing_segments | 53 | True | 10 | clean_first_read | outside_protected_intervals |
| b50a68dfbdb04a30f12e9a60 | SpotCrude | 2023-01-25 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | b3f8a2c0ef92 | missing_segments | 53 | True | 10 | clean_first_read | outside_protected_intervals |
| c2380b0b84e765bc472a0564 | SpotCrude | 2023-01-26 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 4ea21a1e1990 | missing_segments | 53 | True | 10 | clean_first_read | outside_protected_intervals |
| 736039437cfe8703c32341ae | SpotCrude | 2023-01-27 | 2023-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 98b9e12873be | missing_segments | 53 | True | 0 | clean_first_read | outside_protected_intervals |
| 8f401fb18724fa4ecb2f8a36 | SpotCrude | 2023-07-24 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 8f155120072a | missing_segments | 6 | True | 8 | clean_first_read | outside_protected_intervals |
| a7b42dc8ec6b6738c775ba17 | SpotCrude | 2023-07-25 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 45a895905c78 | missing_segments | 6 | True | 8 | clean_first_read | outside_protected_intervals |
| dd362c3ee13db2ddcbd9cf99 | SpotCrude | 2023-07-26 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 0d2dcb770c45 | missing_segments | 6 | True | 8 | clean_first_read | outside_protected_intervals |
| 82256abfa13bcc7222b6f93d | SpotCrude | 2023-07-27 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 0dee2ee6b826 | missing_segments | 6 | True | 8 | clean_first_read | outside_protected_intervals |
| 09fc7226899dff17f5bcca96 | SpotCrude | 2023-07-28 | 2023-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | f75295589013 | missing_segments | 6 | True | 0 | clean_first_read | outside_protected_intervals |
| e8eb43e304e030909039f220 | SpotCrude | 2024-01-22 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | af43b3d52c19 | missing_segments | 53 | True | 10 | inherited_repeat_interval | outside_protected_intervals |
| 1991c7fc6471376a6cf96521 | SpotCrude | 2024-01-23 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | bd0f92b93618 | missing_segments | 53 | True | 10 | inherited_repeat_interval | outside_protected_intervals |
| 8d0d254ad2d721727cf3c150 | SpotCrude | 2024-01-24 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 298271a8f864 | missing_segments | 53 | True | 10 | inherited_repeat_interval | outside_protected_intervals |
| 573e133e718b83d048cb2275 | SpotCrude | 2024-01-25 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 97bd0f3dcdaa | missing_segments | 53 | True | 10 | inherited_repeat_interval | outside_protected_intervals |
| 8b0de7c273b591bb6c13525f | SpotCrude | 2024-01-26 | 2024-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 9e8cf5c18ded | missing_segments | 53 | True | 0 | inherited_repeat_interval | outside_protected_intervals |
| 3e84577037ce7238ec755f2f | SpotCrude | 2024-07-22 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | e21b50bc0567 | missing_segments | 8 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| 072399489fb15bacf30c5918 | SpotCrude | 2024-07-23 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 7a5b074019e8 | missing_segments | 8 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| bee5886b5e233e86567cceb5 | SpotCrude | 2024-07-24 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 13cae1296668 | missing_segments | 8 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| 890b1d65a5c1e496b77e01d3 | SpotCrude | 2024-07-25 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 89bc2ddd548f | missing_segments | 8 | True | 8 | inherited_repeat_interval | outside_protected_intervals |
| 95c2666f32b2231028de74aa | SpotCrude | 2024-07-26 | 2024-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 15bf5389932f | missing_segments | 8 | True | 0 | inherited_repeat_interval | outside_protected_intervals |
| 260d8b5043d33a0c4948f9e4 | SpotCrude | 2025-02-10 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | a058525c2cb7 | missing_segments | 27 | True | 10 | clean_first_read | outside_protected_intervals |
| dfc9bcd5e87d8e9dbb62c5dc | SpotCrude | 2025-02-11 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | d28e9ffbe51a | missing_segments | 27 | True | 10 | clean_first_read | outside_protected_intervals |
| 4e8c147c7c6baa545fddbb82 | SpotCrude | 2025-02-12 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | a4dcc7f9767b | missing_segments | 27 | True | 10 | clean_first_read | outside_protected_intervals |
| 11a6a8c22334e78217761ee7 | SpotCrude | 2025-02-13 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | a085da5581c5 | missing_segments | 27 | True | 10 | clean_first_read | outside_protected_intervals |
| 2c4718db334d50d0069b28ca | SpotCrude | 2025-02-14 | 2025-Q1-WINTER | 07:00–17:00 | 07:00–20:00 | 479f85d83fbc | missing_segments | 27 | True | 0 | clean_first_read | outside_protected_intervals |
| 97d0fcd243c70f3d16342c6b | SpotCrude | 2025-07-21 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 1dc882486b2c | missing_segments | 9 | True | 8 | clean_first_read | outside_protected_intervals |
| 35ba9e2c232645825deb093d | SpotCrude | 2025-07-22 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 5ed5886e62cf | missing_segments | 9 | True | 8 | clean_first_read | outside_protected_intervals |
| 8157378e5e5a500e0590fb57 | SpotCrude | 2025-07-23 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 7ca0af44d6e1 | missing_segments | 9 | True | 8 | clean_first_read | outside_protected_intervals |
| baa63ee6bb8b431a9d217332 | SpotCrude | 2025-07-24 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | d73cc8e540f9 | missing_segments | 9 | True | 8 | clean_first_read | outside_protected_intervals |
| 5aff841b20cdc59bd1507911 | SpotCrude | 2025-07-25 | 2025-Q3-SUMMER | 07:00–17:00 | 07:00–20:00 | 54d5dff916a6 | missing_segments | 9 | True | 0 | clean_first_read | outside_protected_intervals |
| 876f6148e2ef9692e251c46d | US500 | 2023-01-23 | 2023-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 1b217a65b4a5 | missing_segments | 26 | True | 192 | clean_first_read | outside_protected_intervals |
| 714dcd4ea643c7abb6c67ecd | US500 | 2023-01-24 | 2023-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | a563d9625e25 | missing_segments | 26 | True | 144 | clean_first_read | outside_protected_intervals |
| 7f35dcccb70ef17e64457117 | US500 | 2023-01-25 | 2023-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 4302aac7c275 | missing_segments | 26 | True | 96 | clean_first_read | outside_protected_intervals |
| 40cc399289d0861550320dd8 | US500 | 2023-01-26 | 2023-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | b211aa26537b | missing_segments | 26 | True | 48 | clean_first_read | outside_protected_intervals |
| f8992619aab40fdf9fa815cf | US500 | 2023-01-27 | 2023-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 6bdecda65792 | missing_segments | 26 | True | 0 | clean_first_read | outside_protected_intervals |
| 5acc5528cd945dc8e3fc1e27 | US500 | 2023-07-24 | 2023-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 08acebbe211c | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| daa7c84bfb25caac4549cc96 | US500 | 2023-07-25 | 2023-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 5a87bceb0c70 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 7515dfa76df9ce2dd2f5c809 | US500 | 2023-07-26 | 2023-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 0b9d25637471 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 43e781706c514672a6a96738 | US500 | 2023-07-27 | 2023-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 5f38e7f77734 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| a21704982a48b223406411f6 | US500 | 2023-07-28 | 2023-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 6b3b2309c4c1 | missing_segments | 17 | True | 0 | clean_first_read | outside_protected_intervals |
| 2148bb57b5270748d1dc5857 | US500 | 2025-02-10 | 2025-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 71de29f2775e | missing_segments | 34 | True | 7 | clean_first_read | outside_protected_intervals |
| 2fb13c0d341a2a7c4d93d305 | US500 | 2025-02-11 | 2025-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 4e88f1abd2f1 | missing_segments | 34 | True | 7 | clean_first_read | outside_protected_intervals |
| 2c7fe293cc545089e134e08c | US500 | 2025-02-12 | 2025-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 23cdf061a359 | missing_segments | 34 | True | 7 | clean_first_read | outside_protected_intervals |
| d81b55b30354e68499a01af1 | US500 | 2025-02-13 | 2025-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 01862494cd97 | missing_segments | 34 | True | 7 | clean_first_read | outside_protected_intervals |
| 7b74fe5bafc42e70f1ae1521 | US500 | 2025-02-14 | 2025-Q1-WINTER | 14:30–18:30 | 14:30–21:00 | 1e5469dcde77 | missing_segments | 34 | True | 0 | clean_first_read | outside_protected_intervals |
| f3fb5435535d98dde263ca12 | US500 | 2025-07-21 | 2025-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 10feea1dc018 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 96de88c9fded8e055ca84165 | US500 | 2025-07-22 | 2025-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 159455122bb5 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 3525cfcc7489cf8cd70fb13a | US500 | 2025-07-23 | 2025-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 15fd4c4e06be | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| e37467317fb2c0e7cbde2e15 | US500 | 2025-07-24 | 2025-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 343afa2a40a4 | missing_segments | 17 | True | 7 | clean_first_read | outside_protected_intervals |
| 35aec47a684766f6889db8e4 | US500 | 2025-07-25 | 2025-Q3-SUMMER | 13:30–17:30 | 13:30–20:00 | 71a29c16bf5d | missing_segments | 17 | True | 0 | clean_first_read | outside_protected_intervals |
| a3305367b536943b295e9831 | XAUUSD | 2023-01-23 | 2023-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | c7612bf60cf1 | missing_segments | 52 | True | 22 | clean_first_read | outside_protected_intervals |
| d01543c086951e45fc2662f6 | XAUUSD | 2023-01-24 | 2023-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 6f47b329efb2 | missing_segments | 52 | True | 22 | clean_first_read | outside_protected_intervals |
| 6fcf6013c2d5308631e99a7d | XAUUSD | 2023-01-25 | 2023-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 625360c277a4 | missing_segments | 52 | True | 22 | clean_first_read | outside_protected_intervals |
| b25d14035008c3a9fb364d7b | XAUUSD | 2023-01-26 | 2023-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | eeac8dd134ad | missing_segments | 52 | True | 22 | clean_first_read | outside_protected_intervals |
| e5c3746472e635be94b42589 | XAUUSD | 2023-01-27 | 2023-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 57251d5fc2c7 | missing_segments | 52 | True | 0 | clean_first_read | outside_protected_intervals |
| 7e4329e54fa5d8930de487f8 | XAUUSD | 2023-07-24 | 2023-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 820bd86b8c69 | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| a5ac3d675d4a755b21393c6f | XAUUSD | 2023-07-25 | 2023-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 873ce2164001 | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| 4bf29f97bbe50d6359520302 | XAUUSD | 2023-07-26 | 2023-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 06344dad2967 | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| 301cc40952795f0f5982c6b9 | XAUUSD | 2023-07-27 | 2023-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 60af852b14ff | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| 00aba56cdad194a6a6e1630d | XAUUSD | 2023-07-28 | 2023-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 7a9156b9c5ac | missing_segments | 6 | True | 0 | clean_first_read | outside_protected_intervals |
| 11bdabc3c84d460a73a8eb56 | XAUUSD | 2024-01-22 | 2024-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 7b9e2199b504 | missing_segments | 53 | True | 20 | clean_first_read | outside_protected_intervals |
| 10d1ae4831eddd0539ba98b8 | XAUUSD | 2024-01-23 | 2024-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | a95118b0d51b | missing_segments | 53 | True | 20 | clean_first_read | outside_protected_intervals |
| d35cbc2e2f670f3769850370 | XAUUSD | 2024-01-24 | 2024-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | d89ca4fffb7a | missing_segments | 53 | True | 20 | clean_first_read | outside_protected_intervals |
| 5204aa8a50ac25f4b1d6d4bb | XAUUSD | 2024-01-25 | 2024-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 54c0a47949df | missing_segments | 53 | True | 20 | clean_first_read | outside_protected_intervals |
| 8cc480b98b6fa8a6e114d91d | XAUUSD | 2024-01-26 | 2024-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | cad5ad90a6b5 | missing_segments | 53 | True | 0 | clean_first_read | outside_protected_intervals |
| 4818610971895f5197695336 | XAUUSD | 2024-07-22 | 2024-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | db6c88c1b97b | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| 1d8414a63d83d92b96f7d5c8 | XAUUSD | 2024-07-23 | 2024-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 42e4b36d0a15 | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| 2fae307746859cdb6463521e | XAUUSD | 2024-07-24 | 2024-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | cf8ee6743305 | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| 78212982822d191a812f095b | XAUUSD | 2024-07-25 | 2024-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 8b936523436a | missing_segments | 6 | True | 20 | clean_first_read | outside_protected_intervals |
| f31451969f167fdb421b9195 | XAUUSD | 2024-07-26 | 2024-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 49f630faf71f | missing_segments | 6 | True | 0 | clean_first_read | outside_protected_intervals |
| 4ad806629abdf9d7d75ac14e | XAUUSD | 2025-02-10 | 2025-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 1757eb5cc7f7 | missing_segments | 27 | True | 22 | clean_first_read | outside_protected_intervals |
| 758fe8143b87051535048c5d | XAUUSD | 2025-02-11 | 2025-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 6ef32b97f85b | missing_segments | 27 | True | 22 | clean_first_read | outside_protected_intervals |
| e08a05e0094d27338df76a67 | XAUUSD | 2025-02-12 | 2025-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | d2ab8541bab2 | missing_segments | 27 | True | 22 | clean_first_read | outside_protected_intervals |
| 18c4d4b660a470dcab36d784 | XAUUSD | 2025-02-13 | 2025-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 632777da8384 | missing_segments | 27 | True | 22 | clean_first_read | outside_protected_intervals |
| d5f172ae04b389f87a33926f | XAUUSD | 2025-02-14 | 2025-Q1-WINTER | 07:00–11:00 | 07:00–20:00 | 786ec1c5a01e | missing_segments | 27 | True | 0 | clean_first_read | outside_protected_intervals |
| 70d6f75e60e33e934c60a925 | XAUUSD | 2025-07-21 | 2025-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 79de71bfd0ed | missing_segments | 9 | True | 20 | clean_first_read | outside_protected_intervals |
| bdce29dd8539285849b9e070 | XAUUSD | 2025-07-22 | 2025-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | 1b2bbccd9412 | missing_segments | 9 | True | 20 | clean_first_read | outside_protected_intervals |
| 196614c16843b2ea7ef2c9a0 | XAUUSD | 2025-07-23 | 2025-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | c60eea704985 | missing_segments | 9 | True | 20 | clean_first_read | outside_protected_intervals |
| b51bc946e3e8272f970a1779 | XAUUSD | 2025-07-24 | 2025-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | d254554e2bb2 | missing_segments | 9 | True | 20 | clean_first_read | outside_protected_intervals |
| 25a29bafe6cd3fdda3e5faa4 | XAUUSD | 2025-07-25 | 2025-Q3-SUMMER | 07:00–11:00 | 07:00–20:00 | dc86bfbb7dbc | missing_segments | 9 | False | 0 | clean_first_read | outside_protected_intervals |
| b229fad53b8ccfe2fa012013 | NatGas | 2025-10-06 | registered_local_cache | 07:00–11:00 | Unavailable/quarantined | 7aaa3a9c64b5 | missing_segments | — | False | 0 | unverified_receipt_history | quarantined_protected_dependency |

### 8.2 Verified M30 source segments

| Segment ID | Account | Symbol | From UTC | To UTC | Bars | Receipt SHA256 prefix | Seal |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ac1b5ea7a6947582da8ca77e | 49236051 | SpotBrent | 2024-09-20T07:00:00Z | 2024-09-27T07:00:00Z | 214 | f6310daa8f24 | outside_protected_intervals |
| 2fb3945e51768de5e5123c51 | 49236051 | SpotBrent | 2024-09-27T07:00:00Z | 2024-10-04T07:00:00Z | 214 | cc1975936128 | outside_protected_intervals |
| 5e145381d2af1a429253f1d5 | 49236051 | SpotBrent | 2024-10-04T07:00:00Z | 2024-10-11T07:00:00Z | 214 | 97ed5127f5f2 | outside_protected_intervals |
| f6ea3887e696f764cf891c9a | 49236051 | SpotBrent | 2024-10-11T07:00:00Z | 2024-10-18T07:00:00Z | 214 | 41c33b73e421 | outside_protected_intervals |
| 36c348f7ae37706165399d84 | 49236051 | SpotBrent | 2024-10-18T07:00:00Z | 2024-10-25T07:00:00Z | 214 | 1a49a281b6af | outside_protected_intervals |
| df804fa57a24495e7f170fdf | 49236051 | SpotBrent | 2024-10-25T07:00:00Z | 2024-11-01T07:00:00Z | 214 | 92c35b1bea33 | outside_protected_intervals |
| 013ec5ed14100ffadb7eb6c7 | 49236051 | SpotBrent | 2024-11-01T07:00:00Z | 2024-11-08T07:00:00Z | 212 | 1da3caf0c858 | outside_protected_intervals |
| 91bf8c73f9b6c4bb6bd13e79 | 49236051 | SpotBrent | 2024-11-08T07:00:00Z | 2024-11-08T17:00:00Z | 20 | 0d19d8145c2b | outside_protected_intervals |
| 71511583906e50dcf52c2e12 | 49236051 | SpotCrude | 2024-09-20T07:00:00Z | 2024-09-27T07:00:00Z | 230 | 36c87aadf60d | outside_protected_intervals |
| e44e03b9f5f7cb2d6e3a8173 | 49236051 | SpotCrude | 2024-09-27T07:00:00Z | 2024-10-04T07:00:00Z | 230 | 94da8d43ffd5 | outside_protected_intervals |
| 785c3b1d2f8586a40a335676 | 49236051 | SpotCrude | 2024-10-04T07:00:00Z | 2024-10-11T07:00:00Z | 230 | b09bce62e6b6 | outside_protected_intervals |
| 06043f402639853a55668c93 | 49236051 | SpotCrude | 2024-10-11T07:00:00Z | 2024-10-18T07:00:00Z | 230 | 21d7bd94523a | outside_protected_intervals |
| 92b1761e6c2d90ea8d680e5f | 49236051 | SpotCrude | 2024-10-18T07:00:00Z | 2024-10-25T07:00:00Z | 230 | 4d4907a7c0d3 | outside_protected_intervals |
| d4d2f5846c896f00c48d449a | 49236051 | SpotCrude | 2024-10-25T07:00:00Z | 2024-11-01T07:00:00Z | 230 | d2badf7c0dcb | outside_protected_intervals |
| dd1a99f4202ed8e10fc9674a | 49236051 | SpotCrude | 2024-11-01T07:00:00Z | 2024-11-08T07:00:00Z | 228 | 532f44286f9c | outside_protected_intervals |
| 7b3a7fd5cb2944ee5cb86405 | 49236051 | SpotCrude | 2024-11-08T07:00:00Z | 2024-11-08T17:00:00Z | 20 | 495d36f460eb | outside_protected_intervals |
| 76200d522e3f72efabf0ddb7 | 49236051 | US500 | 2024-09-20T14:30:00Z | 2024-09-27T14:30:00Z | 230 | 230a1b8d445c | outside_protected_intervals |
| 6126bb0a21a9e0110e7b765e | 49236051 | US500 | 2024-09-27T14:30:00Z | 2024-10-04T14:30:00Z | 230 | 80b09eb6e2e9 | outside_protected_intervals |
| 30f032a8d84ac335344f2965 | 49236051 | US500 | 2024-10-04T14:30:00Z | 2024-10-11T14:30:00Z | 230 | 127550634508 | outside_protected_intervals |
| 9f0ab6c401ffa1c6e4095799 | 49236051 | US500 | 2024-10-11T14:30:00Z | 2024-10-18T14:30:00Z | 230 | a77218e27cf0 | outside_protected_intervals |
| 9c4ce2bf7293bb3f98f4ad32 | 49236051 | US500 | 2024-10-18T14:30:00Z | 2024-10-25T14:30:00Z | 230 | 1daf82af128a | outside_protected_intervals |
| a582280fec07c98fc263005c | 49236051 | US500 | 2024-10-25T14:30:00Z | 2024-11-01T14:30:00Z | 230 | 1f5ee7e95cb6 | outside_protected_intervals |
| c64eb19ccb229c0d5ec6e15c | 49236051 | US500 | 2024-11-01T14:30:00Z | 2024-11-08T14:30:00Z | 228 | 1036eede9226 | outside_protected_intervals |
| cf25398ce88cffeb414f50bc | 49236051 | US500 | 2024-11-08T14:30:00Z | 2024-11-08T18:30:00Z | 8 | 49e59c43118b | outside_protected_intervals |
| 93ac98749e82b29695e30d50 | 49236051 | XAUUSD | 2024-09-20T07:00:00Z | 2024-09-27T07:00:00Z | 230 | 88be8f080849 | outside_protected_intervals |
| 74c7e1fb924d78470ed53aa5 | 49236051 | XAUUSD | 2024-09-27T07:00:00Z | 2024-10-04T07:00:00Z | 230 | 7f8f4c33ad60 | outside_protected_intervals |
| 5361ba4c30221b1ee3b7280c | 49236051 | XAUUSD | 2024-10-04T07:00:00Z | 2024-10-11T07:00:00Z | 230 | f879032ea346 | outside_protected_intervals |
| 69941212436d5b3b25f4508e | 49236051 | XAUUSD | 2024-10-11T07:00:00Z | 2024-10-18T07:00:00Z | 230 | 023d0c83b9c0 | outside_protected_intervals |
| 02d0a8b47913453bfb8ec391 | 49236051 | XAUUSD | 2024-10-18T07:00:00Z | 2024-10-25T07:00:00Z | 230 | a8cd4b09611f | outside_protected_intervals |
| 0e468a2f23ebf1d61de5dd12 | 49236051 | XAUUSD | 2024-10-25T07:00:00Z | 2024-11-01T07:00:00Z | 230 | a26b0ee20241 | outside_protected_intervals |
| 77e31983271f136bd887d28f | 49236051 | XAUUSD | 2024-11-01T07:00:00Z | 2024-11-08T07:00:00Z | 228 | b6f64c093df9 | outside_protected_intervals |
| f0cd7d9ef562379badd43af2 | 49236051 | XAUUSD | 2024-11-08T07:00:00Z | 2024-11-08T11:00:00Z | 8 | 6912058a373d | outside_protected_intervals |
| b986ea6362a4c249a83e68a0 | 49236051 | SpotBrent | 2025-02-21T07:00:00Z | 2025-02-28T07:00:00Z | 214 | 0e872abbb9cd | outside_protected_intervals |
| 03d09c5af5f7b73dda242c05 | 49236051 | SpotBrent | 2025-02-28T07:00:00Z | 2025-03-07T07:00:00Z | 214 | ef0460e4ec0a | outside_protected_intervals |
| ca09b653f2518ff87bb491c7 | 49236051 | SpotBrent | 2025-03-07T07:00:00Z | 2025-03-14T07:00:00Z | 216 | b2426743a443 | outside_protected_intervals |
| 2d297125ca7f9825c05c442e | 49236051 | SpotBrent | 2025-03-14T07:00:00Z | 2025-03-21T07:00:00Z | 214 | 92a65d271046 | outside_protected_intervals |
| f0f0d0dde67e7f626bf50311 | 49236051 | SpotBrent | 2025-03-21T07:00:00Z | 2025-03-28T07:00:00Z | 214 | 9d66f213dd98 | outside_protected_intervals |
| 7e642e67c1f1b1dd73e00300 | 49236051 | SpotCrude | 2025-02-21T07:00:00Z | 2025-02-28T07:00:00Z | 230 | a636c257047b | outside_protected_intervals |
| abaac42c961eecd6723330c4 | 49236051 | SpotCrude | 2025-02-28T07:00:00Z | 2025-03-07T07:00:00Z | 230 | b5d907bd27c4 | outside_protected_intervals |
| aef239c34ad155e9460abb3b | 49236051 | SpotCrude | 2025-03-07T07:00:00Z | 2025-03-14T07:00:00Z | 232 | 496bf544a2b8 | outside_protected_intervals |
| 6ccc29e455123e5bec62f7a1 | 49236051 | SpotCrude | 2025-03-14T07:00:00Z | 2025-03-21T07:00:00Z | 230 | 4d44c3835e02 | outside_protected_intervals |
| 85b1eb83e683c69265239c30 | 49236051 | SpotCrude | 2025-03-21T07:00:00Z | 2025-03-28T07:00:00Z | 230 | f1b7ecd069c2 | outside_protected_intervals |
| f25b660c5e2c859826f50166 | 49236051 | US500 | 2025-02-21T13:30:00Z | 2025-02-28T13:30:00Z | 230 | e00972e4545b | outside_protected_intervals |
| 205ad350c1efc8ee51abbd8a | 49236051 | US500 | 2025-02-28T13:30:00Z | 2025-03-07T13:30:00Z | 230 | 86a1e6120c53 | outside_protected_intervals |
| c29799b332ae77f59372b1f0 | 49236051 | US500 | 2025-03-07T13:30:00Z | 2025-03-14T13:30:00Z | 232 | 2de34187e127 | outside_protected_intervals |
| 1d52519493333d5c0237aa47 | 49236051 | US500 | 2025-03-14T13:30:00Z | 2025-03-21T13:30:00Z | 230 | 95bcec8f84c5 | outside_protected_intervals |
| 41e0ef97f8d942aec948c3cc | 49236051 | US500 | 2025-03-21T13:30:00Z | 2025-03-28T13:30:00Z | 230 | 2f9adf3e92e3 | outside_protected_intervals |
| 0d0d5766be0496ded91ae7b8 | 49236051 | XAUUSD | 2025-02-21T07:00:00Z | 2025-02-28T07:00:00Z | 230 | 760f2ed0a5df | outside_protected_intervals |
| 96aabf8b2eb622e1d5ca9acb | 49236051 | XAUUSD | 2025-02-28T07:00:00Z | 2025-03-07T07:00:00Z | 230 | 5a32e0a39605 | outside_protected_intervals |
| 5bd97c583faa1d2f5b025a3b | 49236051 | XAUUSD | 2025-03-07T07:00:00Z | 2025-03-14T07:00:00Z | 232 | a0906b59369a | outside_protected_intervals |
| bd698bcf12e725982d2ff3ef | 49236051 | XAUUSD | 2025-03-14T07:00:00Z | 2025-03-21T07:00:00Z | 230 | 9386a01c1245 | outside_protected_intervals |
| 33799a490fcaa2a057dcba51 | 49236051 | XAUUSD | 2025-03-21T07:00:00Z | 2025-03-28T07:00:00Z | 230 | c3f708047cb6 | outside_protected_intervals |
| 4c788e15438be88da2bf11a7 | 49236051 | XAUUSD | 2025-03-28T07:00:00Z | 2025-04-04T07:00:00Z | 230 | a4eac4cd6d80 | outside_protected_intervals |
| 90cd812a3f2e5f695796fe4c | 49236051 | XAUUSD | 2025-04-04T07:00:00Z | 2025-04-11T07:00:00Z | 230 | 09a9671f3244 | outside_protected_intervals |
| 2d21de90d25876287a143678 | 49236051 | XAUUSD | 2025-04-11T07:00:00Z | 2025-04-11T11:00:00Z | 8 | 515ee0bff9f2 | outside_protected_intervals |
| 0f3dac4f57877b3d6809bf7b | 49236051 | BTCUSD | 2024-03-29T00:00:00Z | 2024-04-05T00:00:00Z | 328 | 1d4da73d70b2 | outside_protected_intervals |
| 81b7fb27ea603a2587a5b745 | 49236051 | BTCUSD | 2024-04-05T00:00:00Z | 2024-04-12T00:00:00Z | 313 | d2aab570a2cd | outside_protected_intervals |
| 5a97a4f7177fdd5673f4e606 | 49236051 | BTCUSD | 2024-04-12T00:00:00Z | 2024-04-19T00:00:00Z | 318 | 0f2ed56fcffc | outside_protected_intervals |
| ea39a1b323158b483d7bce8f | 49236051 | BTCUSD | 2024-04-19T00:00:00Z | 2024-04-26T00:00:00Z | 319 | f0d32b348649 | outside_protected_intervals |
| 69c81c6764579cb6f55c8340 | 49236051 | BTCUSD | 2024-04-26T00:00:00Z | 2024-05-03T00:00:00Z | 330 | 7bf6023cb8f7 | outside_protected_intervals |
| 13ff978959568aeffd534fdd | 49236051 | BTCUSD | 2024-05-03T00:00:00Z | 2024-05-10T00:00:00Z | 312 | f167e9e0d2ef | outside_protected_intervals |
| 8c03199b63d772e62545c411 | 49236051 | BTCUSD | 2024-05-10T00:00:00Z | 2024-05-17T00:00:00Z | 323 | d78d4f0779b9 | outside_protected_intervals |
| bd70035eb20b11e806eaea67 | 49236051 | BTCUSD | 2024-05-17T00:00:00Z | 2024-05-17T04:00:00Z | 8 | dcd561e5024e | outside_protected_intervals |
| 1b470f31323ad962bcd7b442 | 49236051 | ETHUSD | 2024-03-29T00:00:00Z | 2024-04-05T00:00:00Z | 328 | bc58d767aa90 | outside_protected_intervals |
| 9491de84c607bcf9d4779fe4 | 49236051 | ETHUSD | 2024-04-05T00:00:00Z | 2024-04-12T00:00:00Z | 313 | feb730d052fe | outside_protected_intervals |
| 8ca4951fc196a8cfc88fdd37 | 49236051 | ETHUSD | 2024-04-12T00:00:00Z | 2024-04-19T00:00:00Z | 318 | 113010141c99 | outside_protected_intervals |
| 8f5c3c68c4519bbffd55c876 | 49236051 | ETHUSD | 2024-04-19T00:00:00Z | 2024-04-26T00:00:00Z | 319 | a56b482462fb | outside_protected_intervals |
| e78b95d7d5ec852a0eef83ae | 49236051 | ETHUSD | 2024-04-26T00:00:00Z | 2024-05-03T00:00:00Z | 330 | 21e39b8a9dc6 | outside_protected_intervals |
| 64a0d1a7d1df02b3bfc8b931 | 49236051 | ETHUSD | 2024-05-03T00:00:00Z | 2024-05-10T00:00:00Z | 312 | 78f14cafa5aa | outside_protected_intervals |
| 2cef2f22f466e48d16172d59 | 49236051 | ETHUSD | 2024-05-10T00:00:00Z | 2024-05-17T00:00:00Z | 323 | 4dfb393da277 | outside_protected_intervals |
| 1fd31747131d9a47b4799af9 | 49236051 | ETHUSD | 2024-05-17T00:00:00Z | 2024-05-17T04:00:00Z | 8 | a9633a4e5453 | outside_protected_intervals |
| 5b69f66503c4dd78aaf10201 | 49236051 | NAS100 | 2024-03-29T13:30:00Z | 2024-04-05T13:30:00Z | 215 | 33b8442789dc | outside_protected_intervals |
| 1e3cd081887a5c1f2dca2777 | 49236051 | NAS100 | 2024-04-05T13:30:00Z | 2024-04-12T13:30:00Z | 230 | 6ed1e32eaa5a | outside_protected_intervals |
| e7a4c5494bfdd8c5fc411031 | 49236051 | NAS100 | 2024-04-12T13:30:00Z | 2024-04-19T13:30:00Z | 230 | e94f75f75150 | outside_protected_intervals |
| 4f8ebaa884b4c2a3cbe35684 | 49236051 | NAS100 | 2024-04-19T13:30:00Z | 2024-04-26T13:30:00Z | 230 | cc15b12ce8ea | outside_protected_intervals |
| 1e5d141103790cf8efb31593 | 49236051 | NAS100 | 2024-04-26T13:30:00Z | 2024-05-03T13:30:00Z | 230 | f3eaa05abe26 | outside_protected_intervals |
| 55594714caa7907478fbac52 | 49236051 | NAS100 | 2024-05-03T13:30:00Z | 2024-05-10T13:30:00Z | 230 | 99734b2f61c8 | outside_protected_intervals |
| 6e146197492f1cf6c0d163a0 | 49236051 | NAS100 | 2024-05-10T13:30:00Z | 2024-05-17T13:30:00Z | 230 | 9843cfe800a0 | outside_protected_intervals |
| 53f5ac99ba622d316e79ffe4 | 49236051 | NAS100 | 2024-05-17T13:30:00Z | 2024-05-17T17:30:00Z | 8 | 20fd45ca3a07 | outside_protected_intervals |
| 581aa39c407f6fa7b2be1c0c | 49236051 | XAGUSD | 2024-03-29T07:00:00Z | 2024-04-05T07:00:00Z | 202 | 007c0994f934 | outside_protected_intervals |
| 33b053b8304436f5c7c37010 | 49236051 | XAGUSD | 2024-04-05T07:00:00Z | 2024-04-12T07:00:00Z | 230 | e4e8f0c988c0 | outside_protected_intervals |
| 55eb1ae3affb1c36a4c6e152 | 49236051 | XAGUSD | 2024-04-12T07:00:00Z | 2024-04-19T07:00:00Z | 230 | bec2c39afcb6 | outside_protected_intervals |
| 9ade04d29ccd451de828b0f0 | 49236051 | XAGUSD | 2024-04-19T07:00:00Z | 2024-04-26T07:00:00Z | 230 | d75dea7e2dd8 | outside_protected_intervals |
| 4da63798e815c60be9c67eb5 | 49236051 | XAGUSD | 2024-04-26T07:00:00Z | 2024-05-03T07:00:00Z | 230 | 98cdfa09f03f | outside_protected_intervals |
| 071876b7b806f0e1aa38b9d3 | 49236051 | XAGUSD | 2024-05-03T07:00:00Z | 2024-05-10T07:00:00Z | 230 | 33e3bfa541ee | outside_protected_intervals |
| 0d46f16115244dc0e03c0b63 | 49236051 | XAGUSD | 2024-05-10T07:00:00Z | 2024-05-17T07:00:00Z | 230 | 62c8b2ed2d98 | outside_protected_intervals |
| 576b8cac1b7cc1b54e9718c6 | 49236051 | XAGUSD | 2024-05-17T07:00:00Z | 2024-05-17T11:00:00Z | 8 | 05398026d048 | outside_protected_intervals |
| e66a3ca9c571e74ffc85b310 | 49236051 | ETHUSD | 2025-03-28T00:00:00Z | 2025-04-04T00:00:00Z | 330 | 935cb3d0d83d | outside_protected_intervals |
| 3ec107ecc15343aaf924e6d5 | 49236051 | ETHUSD | 2025-04-04T00:00:00Z | 2025-04-11T00:00:00Z | 330 | 604a6b0ce8d5 | outside_protected_intervals |
| 271da65ca3aacd98c81fe3dd | 49236051 | ETHUSD | 2025-04-11T00:00:00Z | 2025-04-18T00:00:00Z | 330 | 2a508e858146 | outside_protected_intervals |
| 674874453397e7ba3f299324 | 49236051 | ETHUSD | 2025-04-18T00:00:00Z | 2025-04-25T00:00:00Z | 330 | 9a1d9e7f5885 | outside_protected_intervals |
| 23d1b0b4f3ac8d979d3e6b5a | 49236051 | ETHUSD | 2025-04-25T00:00:00Z | 2025-05-02T00:00:00Z | 330 | 20f9e9a1d800 | outside_protected_intervals |
| 473ba3ef771ce5e84dffb2ef | 49236051 | ETHUSD | 2025-05-02T00:00:00Z | 2025-05-09T00:00:00Z | 330 | 0c1b2caece99 | outside_protected_intervals |
| a9865db405c2ffe433eb6b56 | 49236051 | ETHUSD | 2025-05-09T00:00:00Z | 2025-05-16T00:00:00Z | 330 | 2b36226f1f34 | outside_protected_intervals |
| b239f99bd70ce5350aec14ad | 49236051 | ETHUSD | 2025-05-16T00:00:00Z | 2025-05-16T04:00:00Z | 8 | e06146a0a39a | outside_protected_intervals |
| a08832ba18b4ba4044d790c9 | 49236051 | NAS100 | 2025-03-28T13:30:00Z | 2025-04-04T13:30:00Z | 230 | 816aea66009b | outside_protected_intervals |
| eaae469c12cd31abc2dc1daf | 49236051 | NAS100 | 2025-04-04T13:30:00Z | 2025-04-11T13:30:00Z | 230 | 20c044f398f6 | outside_protected_intervals |
| 3e9949a779e572fd563b058a | 49236051 | NAS100 | 2025-04-11T13:30:00Z | 2025-04-18T13:30:00Z | 199 | 2b26ccd5dfdd | outside_protected_intervals |
| 6683a202e262dd47c8b02e9e | 49236051 | NAS100 | 2025-04-18T13:30:00Z | 2025-04-25T13:30:00Z | 215 | 177be1b501fa | outside_protected_intervals |
| 9be9b3f00b00d60dd753a8ae | 49236051 | NAS100 | 2025-04-25T13:30:00Z | 2025-05-02T13:30:00Z | 230 | 5048d4a041df | outside_protected_intervals |
| d4b7cc1f1387c555d76bb1d4 | 49236051 | NAS100 | 2025-05-02T13:30:00Z | 2025-05-09T13:30:00Z | 230 | 2d053a4b0bb1 | outside_protected_intervals |
| f39bed06c4b9698bff5795e3 | 49236051 | NAS100 | 2025-05-09T13:30:00Z | 2025-05-16T13:30:00Z | 230 | 8bc587f89e27 | outside_protected_intervals |
| 8902807e9c86a2bd3abad239 | 49236051 | NAS100 | 2025-05-16T13:30:00Z | 2025-05-16T17:30:00Z | 8 | 0bf52ebbf66a | outside_protected_intervals |
| 78f99c0d2aba9d273b4ce279 | 49236051 | XAGUSD | 2025-03-28T07:00:00Z | 2025-04-04T07:00:00Z | 230 | b31010b43ed7 | outside_protected_intervals |
| 36ee6e0852a23153cc09821a | 49236051 | XAGUSD | 2025-04-04T07:00:00Z | 2025-04-11T07:00:00Z | 230 | a98cf432958b | outside_protected_intervals |
| 6fd8f88a31de30b50dfbb79c | 49236051 | XAGUSD | 2025-04-11T07:00:00Z | 2025-04-18T07:00:00Z | 212 | 11f5837ddf3c | outside_protected_intervals |
| e4f1d957666c393ac7b1a6d7 | 49236051 | XAGUSD | 2025-04-18T07:00:00Z | 2025-04-25T07:00:00Z | 202 | 9f7cc514154b | outside_protected_intervals |
| bec9bbc490538fc763118e69 | 49236051 | XAGUSD | 2025-04-25T07:00:00Z | 2025-05-02T07:00:00Z | 230 | d3614c12450d | outside_protected_intervals |
| c511eef2308a70c00ba84944 | 49236051 | XAGUSD | 2025-05-02T07:00:00Z | 2025-05-09T07:00:00Z | 230 | 0e9d6f7099ad | outside_protected_intervals |
| df18a29139d05faa6a426eeb | 49236051 | XAGUSD | 2025-05-09T07:00:00Z | 2025-05-16T07:00:00Z | 230 | 195fcee81901 | outside_protected_intervals |
| e11697db8c662b3ee2704f9b | 49236051 | XAGUSD | 2025-05-16T07:00:00Z | 2025-05-16T11:00:00Z | 8 | e312a6d5336b | outside_protected_intervals |
| 8e586b1322f77a5000785820 | 49236051 | BTCUSD | 2023-12-22T00:00:00Z | 2023-12-29T00:00:00Z | 281 | 4bb708bc17eb | outside_protected_intervals |
| 3c184478d6e84cbfaaa1d3d9 | 49236051 | BTCUSD | 2023-12-29T00:00:00Z | 2024-01-05T00:00:00Z | 281 | 0d63acfe6a7a | outside_protected_intervals |
| 977335a5fe84e7353fcb384d | 49236051 | BTCUSD | 2024-01-05T00:00:00Z | 2024-01-12T00:00:00Z | 258 | 0e90bf24efe1 | outside_protected_intervals |
| be7d4d1fce2048b09ec11e98 | 49236051 | BTCUSD | 2024-01-12T00:00:00Z | 2024-01-19T00:00:00Z | 330 | 04e8b9780112 | outside_protected_intervals |
| a455dde5d0c80db9d3eca84f | 49236051 | BTCUSD | 2024-01-19T00:00:00Z | 2024-01-26T00:00:00Z | 321 | 6b8f85a1eacb | outside_protected_intervals |
| c1862b45d71cdaaa30c9139d | 49236051 | BTCUSD | 2024-01-26T00:00:00Z | 2024-01-26T04:00:00Z | 8 | 09ac02ea258f | outside_protected_intervals |
| 1e7b23eba7a39c33f57df1aa | 49236051 | BTCUSD | 2024-06-07T00:00:00Z | 2024-06-14T00:00:00Z | 298 | 73ebf761fb6b | outside_protected_intervals |
| 7babe49e8be8f65f598f5163 | 49236051 | BTCUSD | 2024-06-14T00:00:00Z | 2024-06-21T00:00:00Z | 330 | 702e8fc501fd | outside_protected_intervals |
| a7e1da3b1dd0d17ee2630b0a | 49236051 | BTCUSD | 2024-06-21T00:00:00Z | 2024-06-28T00:00:00Z | 330 | 846dc1e7a7c0 | outside_protected_intervals |
| d161c1e05f6490d321c06469 | 49236051 | BTCUSD | 2024-06-28T00:00:00Z | 2024-07-05T00:00:00Z | 320 | 1eb21451317b | outside_protected_intervals |
| 3a662bbd9e2bd403f0a8379f | 49236051 | BTCUSD | 2024-07-05T00:00:00Z | 2024-07-12T00:00:00Z | 330 | 946408947343 | outside_protected_intervals |
| 882da34c0c955ac8bca35731 | 49236051 | BTCUSD | 2024-07-12T00:00:00Z | 2024-07-19T00:00:00Z | 330 | d0a28c80ba8b | outside_protected_intervals |
| ec75cd6e4c7863ee2343fd66 | 49236051 | BTCUSD | 2024-07-19T00:00:00Z | 2024-07-26T00:00:00Z | 330 | 3c43b166a0cf | outside_protected_intervals |
| fc26237fea66462e0cfe4fdd | 49236051 | BTCUSD | 2024-07-26T00:00:00Z | 2024-07-26T04:00:00Z | 8 | 6a7feb20354a | outside_protected_intervals |
| 81b0ac3e59446a14c2bd5232 | 49236051 | SpotBrent | 2024-06-07T07:00:00Z | 2024-06-14T07:00:00Z | 222 | fc50d1c63e4a | outside_protected_intervals |
| 649fec374b117f8631ccf7f7 | 49236051 | SpotBrent | 2024-06-14T07:00:00Z | 2024-06-21T07:00:00Z | 213 | 242710145c7e | outside_protected_intervals |
| 5975fd19c9084f60cd153514 | 49236051 | SpotBrent | 2024-06-21T07:00:00Z | 2024-06-28T07:00:00Z | 222 | cfff68d2ca98 | outside_protected_intervals |
| 93dbc2091e29f4fe9d678734 | 49236051 | SpotBrent | 2024-06-28T07:00:00Z | 2024-07-05T07:00:00Z | 213 | 12697d8e5cf1 | outside_protected_intervals |
| 6f61691124d64d29d1452354 | 49236051 | SpotBrent | 2024-07-05T07:00:00Z | 2024-07-12T07:00:00Z | 222 | e47345495e68 | outside_protected_intervals |
| 34355240e0e91d2e53744f84 | 49236051 | SpotBrent | 2024-07-12T07:00:00Z | 2024-07-19T07:00:00Z | 222 | d4708000f4a6 | outside_protected_intervals |
| 2d1a3d46348f14e4eaef529c | 49236051 | SpotBrent | 2024-07-19T07:00:00Z | 2024-07-26T07:00:00Z | 212 | 50de3d0c7678 | outside_protected_intervals |
| 76baad5f05758b9b7239c703 | 49236051 | SpotBrent | 2024-07-26T07:00:00Z | 2024-07-26T11:00:00Z | 8 | d0b11d1cde4a | outside_protected_intervals |
| 41e9e8bacb5f1369cb37eaf1 | 49236051 | SpotCrude | 2024-06-07T07:00:00Z | 2024-06-14T07:00:00Z | 230 | b04d80f9929c | outside_protected_intervals |
| fb24b7a40d7a9f5a365d6181 | 49236051 | SpotCrude | 2024-06-14T07:00:00Z | 2024-06-21T07:00:00Z | 225 | 69a713a8762f | outside_protected_intervals |
| 97b5b8af31ef377486764322 | 49236051 | SpotCrude | 2024-06-21T07:00:00Z | 2024-06-28T07:00:00Z | 230 | 31efa61f591b | outside_protected_intervals |
| 5babed7ae77fa7374ba70619 | 49236051 | SpotCrude | 2024-06-28T07:00:00Z | 2024-07-05T07:00:00Z | 225 | f880e0140a98 | outside_protected_intervals |
| faa5670f092180bd159852e6 | 49236051 | SpotCrude | 2024-07-05T07:00:00Z | 2024-07-12T07:00:00Z | 230 | 9f420b05cd1c | outside_protected_intervals |
| d8e7a5592c477c75651c08df | 49236051 | SpotCrude | 2024-07-12T07:00:00Z | 2024-07-19T07:00:00Z | 230 | 3a27a9e08a8f | outside_protected_intervals |
| b8c4145cc77e9f4bf146d4be | 49236051 | SpotCrude | 2024-07-19T07:00:00Z | 2024-07-26T07:00:00Z | 228 | c2f19985961f | outside_protected_intervals |
| 5e4e7009556b78ab979f08e9 | 49236051 | SpotCrude | 2024-07-26T07:00:00Z | 2024-07-26T11:00:00Z | 8 | 96a0a168ef19 | outside_protected_intervals |
| 58375447f6da3e9e649c0a59 | 49236051 | US500 | 2024-06-07T13:30:00Z | 2024-06-14T13:30:00Z | 230 | ae7f62b45ff8 | outside_protected_intervals |
| c737d1e6e7f7cfa8382d23ec | 49236051 | US500 | 2024-06-14T13:30:00Z | 2024-06-21T13:30:00Z | 222 | ba105e1514a2 | outside_protected_intervals |
| fde1557124b3d306ef3c4238 | 49236051 | US500 | 2024-06-21T13:30:00Z | 2024-06-28T13:30:00Z | 230 | f12232169f17 | outside_protected_intervals |
| bf69ae31381a979e3d8100ab | 49236051 | US500 | 2024-06-28T13:30:00Z | 2024-07-05T13:30:00Z | 215 | 00fb1172e2ab | outside_protected_intervals |
| 6098c28186244fddc8d407ee | 49236051 | US500 | 2024-07-05T13:30:00Z | 2024-07-12T13:30:00Z | 230 | 14e3df4c366d | outside_protected_intervals |
| f0dd232155e584a955caaf94 | 49236051 | US500 | 2024-07-12T13:30:00Z | 2024-07-19T13:30:00Z | 230 | 4eab000dc655 | outside_protected_intervals |
| 1a55532bc0bd1303d6962207 | 49236051 | US500 | 2024-07-19T13:30:00Z | 2024-07-26T13:30:00Z | 230 | a9ca59298ced | outside_protected_intervals |
| 11c6787ae9a39c0aacbca339 | 49236051 | US500 | 2024-07-26T13:30:00Z | 2024-07-26T17:30:00Z | 8 | 88148881cec0 | outside_protected_intervals |
| ee96379d07c93d6e5076c56e | 49236051 | BTCUSD | 2025-03-28T00:00:00Z | 2025-04-04T00:00:00Z | 330 | 0683068433bf | outside_protected_intervals |
| dbf39c1c7922e573e304117b | 49236051 | BTCUSD | 2025-04-04T00:00:00Z | 2025-04-11T00:00:00Z | 330 | db1b9f4df8ee | outside_protected_intervals |
| f160a0ab462b492fecdd1c1a | 49236051 | BTCUSD | 2025-04-11T00:00:00Z | 2025-04-18T00:00:00Z | 330 | 9e1ac77623b1 | outside_protected_intervals |
| bdb2fed39dc7587f22de131d | 49236051 | BTCUSD | 2025-04-18T00:00:00Z | 2025-04-25T00:00:00Z | 330 | 820415b23543 | outside_protected_intervals |
| 551d0fe06b1a749b3135e56e | 49236051 | BTCUSD | 2025-04-25T00:00:00Z | 2025-05-02T00:00:00Z | 330 | a119e13fcc16 | outside_protected_intervals |
| 07a97421fe2fdecf7943b486 | 49236051 | BTCUSD | 2025-05-02T00:00:00Z | 2025-05-09T00:00:00Z | 331 | 265db840d87f | outside_protected_intervals |
| bd111e1537731826fb8f2878 | 49236051 | BTCUSD | 2025-05-09T00:00:00Z | 2025-05-16T00:00:00Z | 330 | 891c78902884 | outside_protected_intervals |
| 2bd961d35ca24dabf02464c1 | 49236051 | BTCUSD | 2025-05-16T00:00:00Z | 2025-05-16T04:00:00Z | 8 | 668bf2004258 | outside_protected_intervals |
| 026cc2749ad4d4aacd04104a | 49236051 | SpotBrent | 2025-03-28T07:00:00Z | 2025-04-04T07:00:00Z | 214 | 9e0063dcb80f | outside_protected_intervals |
| f1a0e8660bff9f718d53450b | 49236051 | SpotBrent | 2025-04-04T07:00:00Z | 2025-04-11T07:00:00Z | 214 | ea9b54a6981c | outside_protected_intervals |
| 0b1384069474c46278885bf6 | 49236051 | SpotBrent | 2025-04-11T07:00:00Z | 2025-04-18T07:00:00Z | 200 | ca86f4f729dc | outside_protected_intervals |
| e4cce4bfa08d44dca5908a58 | 49236051 | SpotBrent | 2025-04-18T07:00:00Z | 2025-04-25T07:00:00Z | 186 | 3de82ab8b4c2 | outside_protected_intervals |
| 3bfc19484f8110e181c8ff16 | 49236051 | SpotBrent | 2025-04-25T07:00:00Z | 2025-05-02T07:00:00Z | 214 | a96a6e6ffc87 | outside_protected_intervals |
| f9151975c1c5b4b8832ab29c | 49236051 | SpotBrent | 2025-05-02T07:00:00Z | 2025-05-09T07:00:00Z | 214 | fb2faf63ce9a | outside_protected_intervals |
| de74ba8944683ede4b456075 | 49236051 | SpotBrent | 2025-05-09T07:00:00Z | 2025-05-16T07:00:00Z | 214 | ddd83caf3e58 | outside_protected_intervals |
| ee13002a2d2bc1320a15dfce | 49236051 | SpotBrent | 2025-05-16T07:00:00Z | 2025-05-16T11:00:00Z | 8 | b925f9de3da3 | outside_protected_intervals |
| 240af2886791eeefd31e966f | 49236051 | SpotCrude | 2025-03-28T07:00:00Z | 2025-04-04T07:00:00Z | 230 | c8c9b2cb1dca | outside_protected_intervals |
| f4120ba057662600f353f0fb | 49236051 | SpotCrude | 2025-04-04T07:00:00Z | 2025-04-11T07:00:00Z | 230 | 240ba22aab8a | outside_protected_intervals |
| 9be85e1b4c100d86518a355f | 49236051 | SpotCrude | 2025-04-11T07:00:00Z | 2025-04-18T07:00:00Z | 212 | 456386ccedbb | outside_protected_intervals |
| cc4d53819eb2500c73587ddf | 49236051 | SpotCrude | 2025-04-18T07:00:00Z | 2025-04-25T07:00:00Z | 202 | be737b634462 | outside_protected_intervals |
| 45f8875bf5dc829d3f34c547 | 49236051 | SpotCrude | 2025-04-25T07:00:00Z | 2025-05-02T07:00:00Z | 230 | 236531e1cec8 | outside_protected_intervals |
| 37beb3f4097cece1edf4c3c0 | 49236051 | SpotCrude | 2025-05-02T07:00:00Z | 2025-05-09T07:00:00Z | 230 | 0d8ae713c2de | outside_protected_intervals |
| b856518d82b84f8a4cb3c035 | 49236051 | SpotCrude | 2025-05-09T07:00:00Z | 2025-05-16T07:00:00Z | 230 | 955ad24e51c8 | outside_protected_intervals |
| bbeb2838e7c7b83c883d1434 | 49236051 | SpotCrude | 2025-05-16T07:00:00Z | 2025-05-16T11:00:00Z | 8 | 2bbd2d62e0c4 | outside_protected_intervals |
| a2f4967ef10082d4cfa59141 | 49236051 | US500 | 2025-03-28T13:30:00Z | 2025-04-04T13:30:00Z | 230 | 5fdfc4207f58 | outside_protected_intervals |
| 54428e8654cdaffab70d4f02 | 49236051 | US500 | 2025-04-04T13:30:00Z | 2025-04-11T13:30:00Z | 230 | 3fa9829aebd1 | outside_protected_intervals |
| 7ffc856479a6c7be67502af8 | 49236051 | US500 | 2025-04-11T13:30:00Z | 2025-04-18T13:30:00Z | 199 | a50f491b369f | outside_protected_intervals |
| 58000de505b45b09877c060d | 49236051 | US500 | 2025-04-18T13:30:00Z | 2025-04-25T13:30:00Z | 215 | 3139624dc9ec | outside_protected_intervals |
| 291c05f097439c831a0c20c5 | 49236051 | US500 | 2025-04-25T13:30:00Z | 2025-05-02T13:30:00Z | 230 | 1083d629999a | outside_protected_intervals |
| b3dd92746976d0b924a3db47 | 49236051 | US500 | 2025-05-02T13:30:00Z | 2025-05-09T13:30:00Z | 230 | f9aeaa1b9f38 | outside_protected_intervals |
| 19e375b0c6be75a16a5e33b5 | 49236051 | US500 | 2025-05-09T13:30:00Z | 2025-05-16T13:30:00Z | 230 | 0acc015e0fd9 | outside_protected_intervals |
| 40e35aeb312c98f9ad0c4f59 | 49236051 | US500 | 2025-05-16T13:30:00Z | 2025-05-16T17:30:00Z | 8 | 6d8645199df1 | outside_protected_intervals |
| 7d2db63aaa0f695064490231 | 49236051 | BTCUSD | 2025-09-06T00:00:00Z | 2025-09-13T00:00:00Z | 336 | 90cbb2d26638 | outside_protected_intervals |
| 2fcdccfb02b79cbadf40f574 | 49236051 | BTCUSD | 2025-09-13T00:00:00Z | 2025-09-20T00:00:00Z | 336 | 691bf85cddcf | outside_protected_intervals |
| 2ebfee5f8a90877dba8e0f7a | 49236051 | BTCUSD | 2025-09-20T00:00:00Z | 2025-09-27T00:00:00Z | 336 | 4f20fcde73eb | outside_protected_intervals |
| 23daf76196e1b046cab2d700 | 49236051 | BTCUSD | 2025-09-27T00:00:00Z | 2025-10-04T00:00:00Z | 336 | 58baf0225203 | outside_protected_intervals |
| ae7005960ee7d251cc57eb1e | 49236051 | BTCUSD | 2025-10-04T00:00:00Z | 2025-10-10T04:00:00Z | 296 | b1139e77cc20 | outside_protected_intervals |
| 1fb3c982b3d33d879c57932f | 49236051 | SpotBrent | 2025-09-06T00:00:00Z | 2025-09-13T00:00:00Z | 214 | 05a9b8a00395 | outside_protected_intervals |
| 0e438329759858e75ba49ea7 | 49236051 | SpotBrent | 2025-09-13T00:00:00Z | 2025-09-20T00:00:00Z | 214 | 17eab219090a | outside_protected_intervals |
| 65ff740c7aaf7701e77740cc | 49236051 | SpotBrent | 2025-09-20T00:00:00Z | 2025-09-27T00:00:00Z | 214 | 49de1752f643 | outside_protected_intervals |
| 94ebaa90a30a723bfaf016e4 | 49236051 | SpotBrent | 2025-09-27T00:00:00Z | 2025-10-04T00:00:00Z | 214 | e52f1f9ea86a | outside_protected_intervals |
| 9ac05a1dd99ad4028d1d55b1 | 49236051 | SpotBrent | 2025-10-04T00:00:00Z | 2025-10-10T11:00:00Z | 194 | 290c757d655e | outside_protected_intervals |
| 24e5055e2dd542911e15d469 | 49236051 | SpotCrude | 2025-09-06T00:00:00Z | 2025-09-13T00:00:00Z | 230 | b55309462755 | outside_protected_intervals |
| e62d41bcbbf567159e8707dc | 49236051 | SpotCrude | 2025-09-13T00:00:00Z | 2025-09-20T00:00:00Z | 230 | 0785c343a9fa | outside_protected_intervals |
| 6144c6f32c36924d27f419a2 | 49236051 | SpotCrude | 2025-09-20T00:00:00Z | 2025-09-27T00:00:00Z | 230 | b199fe95597c | outside_protected_intervals |
| d190c4158c7dc20faed5a26d | 49236051 | SpotCrude | 2025-09-27T00:00:00Z | 2025-10-04T00:00:00Z | 230 | 1503fc17a5e1 | outside_protected_intervals |
| 642a942a8fbdebda0fce61e1 | 49236051 | SpotCrude | 2025-10-04T00:00:00Z | 2025-10-10T11:00:00Z | 210 | 4bfae806e137 | outside_protected_intervals |
| 1885f13baca30e3518e8440c | 49236051 | US500 | 2025-09-06T00:00:00Z | 2025-09-13T00:00:00Z | 230 | a92f3c8bf210 | outside_protected_intervals |
| 7fd8eb4f3b531df2224ce5e5 | 49236051 | US500 | 2025-09-13T00:00:00Z | 2025-09-20T00:00:00Z | 230 | 809dc4596384 | outside_protected_intervals |
| e4fa84400350fd23fff03e68 | 49236051 | US500 | 2025-09-20T00:00:00Z | 2025-09-27T00:00:00Z | 230 | a82c9756b375 | outside_protected_intervals |
| c8aa7aab43b0bcb01a802238 | 49236051 | US500 | 2025-09-27T00:00:00Z | 2025-10-04T00:00:00Z | 230 | b886077726d6 | outside_protected_intervals |
| c7850c26170a60477ea0032c | 49236051 | US500 | 2025-10-04T00:00:00Z | 2025-10-10T17:30:00Z | 223 | f08832ceff4b | outside_protected_intervals |
| c116c21b5071593c30cbf961 | 49236051 | SpotBrent | 2022-12-09T07:00:00Z | 2022-12-16T07:00:00Z | 220 | 655fb190b326 | outside_protected_intervals |
| f1c09ef0c8a4b75893cc1edb | 49236051 | SpotBrent | 2022-12-16T07:00:00Z | 2022-12-23T07:00:00Z | 220 | d918894f1c71 | outside_protected_intervals |
| 938c2088af4e229f9e35629d | 49236051 | SpotBrent | 2022-12-23T07:00:00Z | 2022-12-30T07:00:00Z | 128 | 12025b425acc | outside_protected_intervals |
| 18de62d070af9f4273ed9613 | 49236051 | SpotBrent | 2022-12-30T07:00:00Z | 2023-01-06T07:00:00Z | 170 | b50b72290e0b | outside_protected_intervals |
| 22d1844dbbd5e6c70c53c2a8 | 49236051 | SpotBrent | 2023-01-06T07:00:00Z | 2023-01-13T07:00:00Z | 220 | d955a9d53717 | outside_protected_intervals |
| 1904fc64a0e604bfd907ea0c | 49236051 | SpotBrent | 2023-01-13T07:00:00Z | 2023-01-20T07:00:00Z | 220 | d686778bbe47 | outside_protected_intervals |
| d3a8fb5034ce4fada4b45650 | 49236051 | SpotBrent | 2023-01-20T07:00:00Z | 2023-01-27T07:00:00Z | 220 | edb1b31b4592 | outside_protected_intervals |
| 26afaf21ed4e167ee2f25583 | 49236051 | SpotBrent | 2023-01-27T07:00:00Z | 2023-01-27T17:00:00Z | 20 | b5aa6a339733 | outside_protected_intervals |
| 9941367bc01eed3ce2130227 | 49236051 | SpotCrude | 2022-12-09T07:00:00Z | 2022-12-16T07:00:00Z | 230 | 7fa05d523104 | outside_protected_intervals |
| ae7eec529d1a06853da6878c | 49236051 | SpotCrude | 2022-12-16T07:00:00Z | 2022-12-23T07:00:00Z | 230 | bbd792638fe2 | outside_protected_intervals |
| 30571629803bc3e9e55e5df0 | 49236051 | SpotCrude | 2022-12-23T07:00:00Z | 2022-12-30T07:00:00Z | 184 | b2ab9a9f089e | outside_protected_intervals |
| 219ab1bdaddee5bb28013741 | 49236051 | SpotCrude | 2022-12-30T07:00:00Z | 2023-01-06T07:00:00Z | 184 | a9e376c7c6b0 | outside_protected_intervals |
| 7751edc04afddc9f2696b9a8 | 49236051 | SpotCrude | 2023-01-06T07:00:00Z | 2023-01-13T07:00:00Z | 230 | ca0df6f3aedf | outside_protected_intervals |
| b96b4bfd18a7b3ab6e28fa55 | 49236051 | SpotCrude | 2023-01-13T07:00:00Z | 2023-01-20T07:00:00Z | 225 | 8bb4ee32a279 | outside_protected_intervals |
| 64638cff8a2da370c0925e5a | 49236051 | SpotCrude | 2023-01-20T07:00:00Z | 2023-01-27T07:00:00Z | 230 | 0723875944d3 | outside_protected_intervals |
| b14025b624d3b7633fefbcf6 | 49236051 | SpotCrude | 2023-01-27T07:00:00Z | 2023-01-27T17:00:00Z | 20 | b6610604bd82 | outside_protected_intervals |
| 8492412e8bcbc9b83908d1a5 | 49236051 | US500 | 2022-12-09T14:30:00Z | 2022-12-16T14:30:00Z | 238 | cb6ad99f12bd | outside_protected_intervals |
| e63296bab3543636e07444a9 | 49236051 | US500 | 2022-12-16T14:30:00Z | 2022-12-23T14:30:00Z | 238 | dc686d417609 | outside_protected_intervals |
| a1fff49fdd4cde2f1c110c99 | 49236051 | US500 | 2022-12-23T14:30:00Z | 2022-12-30T14:30:00Z | 190 | 9ec2547cf9cb | outside_protected_intervals |
| de80883750ebbc0d97703876 | 49236051 | US500 | 2022-12-30T14:30:00Z | 2023-01-06T14:30:00Z | 190 | e74beb715e0c | outside_protected_intervals |
| f1e2dbfd533f26e133132147 | 49236051 | US500 | 2023-01-06T14:30:00Z | 2023-01-13T14:30:00Z | 238 | d6140672988b | outside_protected_intervals |
| 546fd0354efb9fc2d124df6a | 49236051 | US500 | 2023-01-13T14:30:00Z | 2023-01-20T14:30:00Z | 236 | 02b999fe96da | outside_protected_intervals |
| 9922ef0253f7b0a6dc7fda23 | 49236051 | US500 | 2023-01-20T14:30:00Z | 2023-01-27T14:30:00Z | 238 | 65c49baefd50 | outside_protected_intervals |
| fe9a5cce14437cf2d458f888 | 49236051 | US500 | 2023-01-27T14:30:00Z | 2023-01-27T18:30:00Z | 8 | 3a2554efe8ff | outside_protected_intervals |
| c622910240885a41d52972be | 49236051 | XAUUSD | 2022-12-09T07:00:00Z | 2022-12-16T07:00:00Z | 230 | cec931ca31af | outside_protected_intervals |
| 75ba2fadf8d2d5a882d4de34 | 49236051 | XAUUSD | 2022-12-16T07:00:00Z | 2022-12-23T07:00:00Z | 230 | ef090b6edc29 | outside_protected_intervals |
| 4113def897ec340196ddb29e | 49236051 | XAUUSD | 2022-12-23T07:00:00Z | 2022-12-30T07:00:00Z | 184 | 189d53793e88 | outside_protected_intervals |
| 30b1eaf69fcd6d4dedeb1469 | 49236051 | XAUUSD | 2022-12-30T07:00:00Z | 2023-01-06T07:00:00Z | 184 | fa7cbeceeea1 | outside_protected_intervals |
| 1bb64dd320c74cf62ead71c8 | 49236051 | XAUUSD | 2023-01-06T07:00:00Z | 2023-01-13T07:00:00Z | 230 | 6c53b3943397 | outside_protected_intervals |
| 575a70c088e209b07149587e | 49236051 | XAUUSD | 2023-01-13T07:00:00Z | 2023-01-20T07:00:00Z | 226 | 7fbb9c64965c | outside_protected_intervals |
| c37351f5750c3c7888d8d342 | 49236051 | XAUUSD | 2023-01-20T07:00:00Z | 2023-01-27T07:00:00Z | 230 | c27c4255d248 | outside_protected_intervals |
| 55612fd959c50a7f25ce4edc | 49236051 | XAUUSD | 2023-01-27T07:00:00Z | 2023-01-27T11:00:00Z | 8 | 820f48535ba6 | outside_protected_intervals |
| 36b74ec790bf2a1906eedc04 | 49236051 | SpotBrent | 2023-06-09T07:00:00Z | 2023-06-16T07:00:00Z | 222 | b5aa7d107983 | outside_protected_intervals |
| e38782ce6042aacf4e512d75 | 49236051 | SpotBrent | 2023-06-16T07:00:00Z | 2023-06-23T07:00:00Z | 213 | 1185e2d2d91c | outside_protected_intervals |
| 3dc91659eeb559389c502309 | 49236051 | SpotBrent | 2023-06-23T07:00:00Z | 2023-06-30T07:00:00Z | 222 | 501ff532e744 | outside_protected_intervals |
| 01781eef703fdd5b851d7bf1 | 49236051 | SpotBrent | 2023-06-30T07:00:00Z | 2023-07-07T07:00:00Z | 213 | 5f7989050e27 | outside_protected_intervals |
| b503303cd637d7fbf4ebb810 | 49236051 | SpotBrent | 2023-07-07T07:00:00Z | 2023-07-14T07:00:00Z | 222 | a6889186987c | outside_protected_intervals |
| fe367d0213a45ccb8a155890 | 49236051 | SpotBrent | 2023-07-14T07:00:00Z | 2023-07-21T07:00:00Z | 222 | 073b14ebb56b | outside_protected_intervals |
| 76c7962be96ca79a650d4494 | 49236051 | SpotBrent | 2023-07-21T07:00:00Z | 2023-07-28T07:00:00Z | 222 | 387d5c62cc4b | outside_protected_intervals |
| ecb2d15397a638efe4c65f70 | 49236051 | SpotBrent | 2023-07-28T07:00:00Z | 2023-07-28T17:00:00Z | 20 | 141e6a7fbd12 | outside_protected_intervals |
| 17ebb7ff5e98be7438e9856f | 49236051 | SpotCrude | 2023-06-09T07:00:00Z | 2023-06-16T07:00:00Z | 230 | e88ea4357780 | outside_protected_intervals |
| e97f21a7e4666983ec2b8fd5 | 49236051 | SpotCrude | 2023-06-16T07:00:00Z | 2023-06-23T07:00:00Z | 225 | 03f1e49ffb97 | outside_protected_intervals |
| a8256f6b35c0e534908ac1d5 | 49236051 | SpotCrude | 2023-06-23T07:00:00Z | 2023-06-30T07:00:00Z | 230 | 57f185be7c2b | outside_protected_intervals |
| ec7ed81142c7ee3d40cbd8ed | 49236051 | SpotCrude | 2023-06-30T07:00:00Z | 2023-07-07T07:00:00Z | 225 | b63870424fa8 | outside_protected_intervals |
| 04c6a42bf1c0d8d732232ab6 | 49236051 | SpotCrude | 2023-07-07T07:00:00Z | 2023-07-14T07:00:00Z | 230 | 7263351b0672 | outside_protected_intervals |
| 49ea0a924d28ba71640939a8 | 49236051 | SpotCrude | 2023-07-14T07:00:00Z | 2023-07-21T07:00:00Z | 230 | ac67bee90be8 | outside_protected_intervals |
| d36eef40cbfa1185610dd8b6 | 49236051 | SpotCrude | 2023-07-21T07:00:00Z | 2023-07-28T07:00:00Z | 230 | b6fc1b9c9c08 | outside_protected_intervals |
| 6003c499b61978de034e4099 | 49236051 | SpotCrude | 2023-07-28T07:00:00Z | 2023-07-28T17:00:00Z | 20 | 4002e0f90353 | outside_protected_intervals |
| 6e32864cbaf6504717d3ac17 | 49236051 | US500 | 2023-06-09T13:30:00Z | 2023-06-16T13:30:00Z | 230 | cf88d426a74e | outside_protected_intervals |
| 76c261db869f9baedbd3968c | 49236051 | US500 | 2023-06-16T13:30:00Z | 2023-06-23T13:30:00Z | 222 | 401a700247f9 | outside_protected_intervals |
| be020b0764836b278feb0755 | 49236051 | US500 | 2023-06-23T13:30:00Z | 2023-06-30T13:30:00Z | 230 | 82fd5fc0a605 | outside_protected_intervals |
| 38d6635c3c23d6e9be2dd049 | 49236051 | US500 | 2023-06-30T13:30:00Z | 2023-07-07T13:30:00Z | 215 | 4f108a9eefbd | outside_protected_intervals |
| 75b01ed71a5f08a9d7443d8e | 49236051 | US500 | 2023-07-07T13:30:00Z | 2023-07-14T13:30:00Z | 230 | 0280310edd49 | outside_protected_intervals |
| ebb9ffddea37d25c225a89ef | 49236051 | US500 | 2023-07-14T13:30:00Z | 2023-07-21T13:30:00Z | 230 | 508c2f19fe87 | outside_protected_intervals |
| 6a6fa88500eb1d7d41941f14 | 49236051 | US500 | 2023-07-21T13:30:00Z | 2023-07-28T13:30:00Z | 230 | 11e584cd4a58 | outside_protected_intervals |
| 9ca0bcdbc7b2182dbb436088 | 49236051 | US500 | 2023-07-28T13:30:00Z | 2023-07-28T17:30:00Z | 8 | 7f491f675d8b | outside_protected_intervals |
| dbb5c9abb7d238c95fed3b15 | 49236051 | XAUUSD | 2023-06-09T07:00:00Z | 2023-06-16T07:00:00Z | 230 | dbd6e8150767 | outside_protected_intervals |
| be27afdbe8a7d3b7d32049e7 | 49236051 | XAUUSD | 2023-06-16T07:00:00Z | 2023-06-23T07:00:00Z | 225 | f0eea002eb22 | outside_protected_intervals |
| 43e1d1d7bc452ca9bb903943 | 49236051 | XAUUSD | 2023-06-23T07:00:00Z | 2023-06-30T07:00:00Z | 230 | 8f31ec6de507 | outside_protected_intervals |
| ab20ed560217428d6520b29f | 49236051 | XAUUSD | 2023-06-30T07:00:00Z | 2023-07-07T07:00:00Z | 225 | bdeca4db114e | outside_protected_intervals |
| 598839121b8f7ccdcb39c89a | 49236051 | XAUUSD | 2023-07-07T07:00:00Z | 2023-07-14T07:00:00Z | 230 | 7ad99d0b851a | outside_protected_intervals |
| c6f2c3f8bdfbb0db11289079 | 49236051 | XAUUSD | 2023-07-14T07:00:00Z | 2023-07-21T07:00:00Z | 230 | e1ac05882a4e | outside_protected_intervals |
| 96dc649869013f129740e0f0 | 49236051 | XAUUSD | 2023-07-21T07:00:00Z | 2023-07-28T07:00:00Z | 230 | 7706df46ecd9 | outside_protected_intervals |
| a07bedf7e422328845e93e49 | 49236051 | XAUUSD | 2023-07-28T07:00:00Z | 2023-07-28T11:00:00Z | 8 | b335eb19848e | outside_protected_intervals |
| aff2d0ed64980298fd97f802 | 49236051 | SpotBrent | 2023-12-08T07:00:00Z | 2023-12-15T07:00:00Z | 222 | 04472a60dc0c | outside_protected_intervals |
| 6bc59ceb0611ea43678ac12a | 49236051 | SpotBrent | 2023-12-15T07:00:00Z | 2023-12-22T07:00:00Z | 222 | 34042a7b2abb | outside_protected_intervals |
| 1ad2457f90b8f62cd95fa1b4 | 49236051 | SpotBrent | 2023-12-22T07:00:00Z | 2023-12-29T07:00:00Z | 170 | 9dfa07c356b2 | outside_protected_intervals |
| dbd65eaf45651b8e82d1abaf | 49236051 | SpotBrent | 2023-12-29T07:00:00Z | 2024-01-05T07:00:00Z | 170 | 5bd00fa9319f | outside_protected_intervals |
| 9949ee6247193eea6c2bd909 | 49236051 | SpotBrent | 2024-01-05T07:00:00Z | 2024-01-12T07:00:00Z | 222 | d203571a64e7 | outside_protected_intervals |
| 9b98b661fb3af4aecfd4e229 | 49236051 | SpotBrent | 2024-01-12T07:00:00Z | 2024-01-19T07:00:00Z | 222 | a75642dde14c | outside_protected_intervals |
| a85820663b4f8eb3e608bbe6 | 49236051 | SpotBrent | 2024-01-19T07:00:00Z | 2024-01-26T07:00:00Z | 222 | cea6609bfc0f | outside_protected_intervals |
| c5d3e87ff2cb601895865b09 | 49236051 | SpotBrent | 2024-01-26T07:00:00Z | 2024-01-26T17:00:00Z | 20 | 60708c191041 | outside_protected_intervals |
| 9ae314c4351dba92fe8232ab | 49236051 | SpotCrude | 2023-12-08T07:00:00Z | 2023-12-15T07:00:00Z | 230 | 0ae8307d7982 | outside_protected_intervals |
| 2f8e42131bb06fa65c27ac8e | 49236051 | SpotCrude | 2023-12-15T07:00:00Z | 2023-12-22T07:00:00Z | 230 | bb9171ab5d70 | outside_protected_intervals |
| cf59b58551f887a35ade77c0 | 49236051 | SpotCrude | 2023-12-22T07:00:00Z | 2023-12-29T07:00:00Z | 183 | 7be304487f12 | outside_protected_intervals |
| 01815d33c57a36acb85a4016 | 49236051 | SpotCrude | 2023-12-29T07:00:00Z | 2024-01-05T07:00:00Z | 183 | b8f9ba23368f | outside_protected_intervals |
| bd23345d256440891ff5848a | 49236051 | SpotCrude | 2024-01-05T07:00:00Z | 2024-01-12T07:00:00Z | 230 | da3f714f46df | outside_protected_intervals |
| e6acd44143a9b90a53a0c822 | 49236051 | SpotCrude | 2024-01-12T07:00:00Z | 2024-01-19T07:00:00Z | 225 | 47fa063bcd84 | outside_protected_intervals |
| 8ea8d85f1abc3c9d670b28ca | 49236051 | SpotCrude | 2024-01-19T07:00:00Z | 2024-01-26T07:00:00Z | 230 | c5081684807e | outside_protected_intervals |
| 71b7a94c61535500ceacf0d8 | 49236051 | SpotCrude | 2024-01-26T07:00:00Z | 2024-01-26T17:00:00Z | 20 | a0d0133c4593 | outside_protected_intervals |
| 3aacb63fca1c27524742de9f | 49236051 | XAUUSD | 2023-12-08T07:00:00Z | 2023-12-15T07:00:00Z | 220 | 1d5422080290 | outside_protected_intervals |
| 3b2acd45a67d192c024a4d77 | 49236051 | XAUUSD | 2023-12-15T07:00:00Z | 2023-12-22T07:00:00Z | 220 | 98bb09436f38 | outside_protected_intervals |
| 7a18c5f55045af34c4a6bc08 | 49236051 | XAUUSD | 2023-12-22T07:00:00Z | 2023-12-29T07:00:00Z | 176 | 2182e2d2d3e7 | outside_protected_intervals |
| eecd242a6705df5aa437bc5e | 49236051 | XAUUSD | 2023-12-29T07:00:00Z | 2024-01-05T07:00:00Z | 176 | 0862d465f5a1 | outside_protected_intervals |
| 76345b92e210c698649c4bed | 49236051 | XAUUSD | 2024-01-05T07:00:00Z | 2024-01-12T07:00:00Z | 220 | 281790e45f49 | outside_protected_intervals |
| be60ec8398d2149ada90d515 | 49236051 | XAUUSD | 2024-01-12T07:00:00Z | 2024-01-19T07:00:00Z | 217 | 1757f951d2cc | outside_protected_intervals |
| 8c4a8a309b23fc1931172613 | 49236051 | XAUUSD | 2024-01-19T07:00:00Z | 2024-01-26T07:00:00Z | 220 | 77c8dc1c39f2 | outside_protected_intervals |
| 4300e5fe7077adf139d679d1 | 49236051 | XAUUSD | 2024-01-26T07:00:00Z | 2024-01-26T11:00:00Z | 8 | 0a5c596b6f2a | outside_protected_intervals |
| ba199a6f73f5a7ed645660a4 | 49236051 | SpotBrent | 2024-06-07T07:00:00Z | 2024-06-14T07:00:00Z | 222 | 81820dc9f9cd | outside_protected_intervals |
| b6c7c9faf594341b759775d2 | 49236051 | SpotBrent | 2024-06-14T07:00:00Z | 2024-06-21T07:00:00Z | 213 | 6e39028d13e0 | outside_protected_intervals |
| eba18d3e35b6e3ee2dc2384d | 49236051 | SpotBrent | 2024-06-21T07:00:00Z | 2024-06-28T07:00:00Z | 222 | 382eda6f3263 | outside_protected_intervals |
| cb5c7017166c4ecd154e5716 | 49236051 | SpotBrent | 2024-06-28T07:00:00Z | 2024-07-05T07:00:00Z | 213 | 3dc184fd884f | outside_protected_intervals |
| 32a4b081e8b8acdb78c68c0c | 49236051 | SpotBrent | 2024-07-05T07:00:00Z | 2024-07-12T07:00:00Z | 222 | c1f0e08f21b0 | outside_protected_intervals |
| 8fce320b215eb849a8c1226b | 49236051 | SpotBrent | 2024-07-12T07:00:00Z | 2024-07-19T07:00:00Z | 222 | 32f656cfa03a | outside_protected_intervals |
| da39db4e465cad14dbdef689 | 49236051 | SpotBrent | 2024-07-19T07:00:00Z | 2024-07-26T07:00:00Z | 212 | e5c585cd4246 | outside_protected_intervals |
| 3614e01eb15ee18b13393594 | 49236051 | SpotBrent | 2024-07-26T07:00:00Z | 2024-07-26T17:00:00Z | 20 | 9456714b4d98 | outside_protected_intervals |
| e8c9019ec572d015b6a4aa25 | 49236051 | SpotCrude | 2024-06-07T07:00:00Z | 2024-06-14T07:00:00Z | 230 | 0b98e7bc72e0 | outside_protected_intervals |
| 1934472757c9736c2fb70a46 | 49236051 | SpotCrude | 2024-06-14T07:00:00Z | 2024-06-21T07:00:00Z | 225 | 3ca05ab72b38 | outside_protected_intervals |
| 2064403895b182b1242e6823 | 49236051 | SpotCrude | 2024-06-21T07:00:00Z | 2024-06-28T07:00:00Z | 230 | 6e536ece3e04 | outside_protected_intervals |
| 7e136eb8ff1a5345e8f284ec | 49236051 | SpotCrude | 2024-06-28T07:00:00Z | 2024-07-05T07:00:00Z | 225 | 76185b4e2ced | outside_protected_intervals |
| 539c65e1b48b5b55b3f4f8be | 49236051 | SpotCrude | 2024-07-05T07:00:00Z | 2024-07-12T07:00:00Z | 230 | 9abb4ce302d2 | outside_protected_intervals |
| f67e013c5449db5dd03e6aef | 49236051 | SpotCrude | 2024-07-12T07:00:00Z | 2024-07-19T07:00:00Z | 230 | 03d3ce1a325f | outside_protected_intervals |
| 020c0ae033514a0ded814183 | 49236051 | SpotCrude | 2024-07-19T07:00:00Z | 2024-07-26T07:00:00Z | 228 | 2385c0207f94 | outside_protected_intervals |
| 0bffae13399fd03d50208898 | 49236051 | SpotCrude | 2024-07-26T07:00:00Z | 2024-07-26T17:00:00Z | 20 | 467980cc1bd8 | outside_protected_intervals |
| ef167de6a9c1a43f9cba181a | 49236051 | XAUUSD | 2024-06-07T07:00:00Z | 2024-06-14T07:00:00Z | 230 | 787b70abbbcf | outside_protected_intervals |
| d7cac550dc91419cb8eed50b | 49236051 | XAUUSD | 2024-06-14T07:00:00Z | 2024-06-21T07:00:00Z | 225 | 379814308d1d | outside_protected_intervals |
| f6234b3e03ebaab518c60edc | 49236051 | XAUUSD | 2024-06-21T07:00:00Z | 2024-06-28T07:00:00Z | 230 | 4accf481f265 | outside_protected_intervals |
| b578294f94a202672a482f77 | 49236051 | XAUUSD | 2024-06-28T07:00:00Z | 2024-07-05T07:00:00Z | 225 | 80074bc9aa86 | outside_protected_intervals |
| 51d2683e7bf6db6f23152cc4 | 49236051 | XAUUSD | 2024-07-05T07:00:00Z | 2024-07-12T07:00:00Z | 230 | c29f396481c1 | outside_protected_intervals |
| 9f58aa5b0d802bfb54e0f33e | 49236051 | XAUUSD | 2024-07-12T07:00:00Z | 2024-07-19T07:00:00Z | 230 | 0f92d774a588 | outside_protected_intervals |
| 7c09adfbff1390fbc2d9b2d4 | 49236051 | XAUUSD | 2024-07-19T07:00:00Z | 2024-07-26T07:00:00Z | 230 | c94aca4ec871 | outside_protected_intervals |
| 638166565dbe472b7d178a9f | 49236051 | XAUUSD | 2024-07-26T07:00:00Z | 2024-07-26T11:00:00Z | 8 | 7da56149e8a6 | outside_protected_intervals |
| 24d2f801d707aa8ca1a8577e | 49236051 | SpotBrent | 2024-12-27T07:00:00Z | 2025-01-03T07:00:00Z | 168 | 75da4d95d313 | outside_protected_intervals |
| c82b870e25cc393c32c1d8dc | 49236051 | SpotBrent | 2025-01-03T07:00:00Z | 2025-01-10T07:00:00Z | 214 | 019c76cda318 | outside_protected_intervals |
| d25467c881e61f7fb2008b00 | 49236051 | SpotBrent | 2025-01-10T07:00:00Z | 2025-01-17T07:00:00Z | 214 | 9184da510c7f | outside_protected_intervals |
| 26a0d398d9a1c93445c83528 | 49236051 | SpotBrent | 2025-01-17T07:00:00Z | 2025-01-24T07:00:00Z | 214 | 1bee1ac19c88 | outside_protected_intervals |
| 64d7b1575b6576ebd7bfa950 | 49236051 | SpotBrent | 2025-01-24T07:00:00Z | 2025-01-31T07:00:00Z | 214 | 83b0c9cc8b9f | outside_protected_intervals |
| 1fc239a3218d8819a83622bd | 49236051 | SpotBrent | 2025-01-31T07:00:00Z | 2025-02-07T07:00:00Z | 214 | c5ec34a7bb26 | outside_protected_intervals |
| 953250834b62dfb6a30e0c35 | 49236051 | SpotBrent | 2025-02-07T07:00:00Z | 2025-02-14T07:00:00Z | 214 | d17e4a6da10d | outside_protected_intervals |
| 4729059ab72567719c57ce74 | 49236051 | SpotBrent | 2025-02-14T07:00:00Z | 2025-02-14T17:00:00Z | 20 | 33624ebd632d | outside_protected_intervals |
| 60a24f9ba22ded1828bc4f6d | 49236051 | SpotCrude | 2024-12-27T07:00:00Z | 2025-01-03T07:00:00Z | 180 | 49e4b4832e61 | outside_protected_intervals |
| 125157a7bffd014bcaae7286 | 49236051 | SpotCrude | 2025-01-03T07:00:00Z | 2025-01-10T07:00:00Z | 230 | 923e39301337 | outside_protected_intervals |
| e8b6e2faf511792c82021b6d | 49236051 | SpotCrude | 2025-01-10T07:00:00Z | 2025-01-17T07:00:00Z | 230 | 118fbfd31624 | outside_protected_intervals |
| b36adf7fc47eaf0b5794c3e3 | 49236051 | SpotCrude | 2025-01-17T07:00:00Z | 2025-01-24T07:00:00Z | 225 | 3264ac52ac58 | outside_protected_intervals |
| 74e195b8f822830c6f23623a | 49236051 | SpotCrude | 2025-01-24T07:00:00Z | 2025-01-31T07:00:00Z | 230 | d26e6379ecca | outside_protected_intervals |
| 83bd2cbd267609ce875f9bed | 49236051 | SpotCrude | 2025-01-31T07:00:00Z | 2025-02-07T07:00:00Z | 230 | 06f57d7f40bb | outside_protected_intervals |
| 8a607dde8e7a6ce58d97fa4f | 49236051 | SpotCrude | 2025-02-07T07:00:00Z | 2025-02-14T07:00:00Z | 230 | 46cd85541ac2 | outside_protected_intervals |
| c1f45a399c784ae8e854b783 | 49236051 | SpotCrude | 2025-02-14T07:00:00Z | 2025-02-14T17:00:00Z | 20 | 874502745192 | outside_protected_intervals |
| 1d593f2384c9e66170c1147d | 49236051 | US500 | 2024-12-27T14:30:00Z | 2025-01-03T14:30:00Z | 180 | 256226ccb2a9 | outside_protected_intervals |
| 941d39d903985a5b24df61db | 49236051 | US500 | 2025-01-03T14:30:00Z | 2025-01-10T14:30:00Z | 215 | f1d1f1e3fd9f | outside_protected_intervals |
| 9eceabb3cfcfe5669b9cfd6a | 49236051 | US500 | 2025-01-10T14:30:00Z | 2025-01-17T14:30:00Z | 230 | f7b6c1f69343 | outside_protected_intervals |
| 570640d56eb7e9f3d91cae34 | 49236051 | US500 | 2025-01-17T14:30:00Z | 2025-01-24T14:30:00Z | 222 | 9c87c69272c1 | outside_protected_intervals |
| ec04c3584bc4da5ea5e56243 | 49236051 | US500 | 2025-01-24T14:30:00Z | 2025-01-31T14:30:00Z | 230 | e7ce68799fcc | outside_protected_intervals |
| 6824b9fab68170ba24766df3 | 49236051 | US500 | 2025-01-31T14:30:00Z | 2025-02-07T14:30:00Z | 230 | e561f1a27768 | outside_protected_intervals |
| 741598214bc25ae96dae3652 | 49236051 | US500 | 2025-02-07T14:30:00Z | 2025-02-14T14:30:00Z | 230 | 8227e5f2eb03 | outside_protected_intervals |
| 558de8ea9a34b340aef8c1f0 | 49236051 | US500 | 2025-02-14T14:30:00Z | 2025-02-14T18:30:00Z | 8 | 9b91d5141d88 | outside_protected_intervals |
| a31f35d0c9dbabd28c9653fd | 49236051 | XAUUSD | 2024-12-27T07:00:00Z | 2025-01-03T07:00:00Z | 180 | 70f80439d9d3 | outside_protected_intervals |
| 7d08e8aeeb68a5f213c30d00 | 49236051 | XAUUSD | 2025-01-03T07:00:00Z | 2025-01-10T07:00:00Z | 230 | 25579a425a1b | outside_protected_intervals |
| c834f29ef7934ab1407fcbb2 | 49236051 | XAUUSD | 2025-01-10T07:00:00Z | 2025-01-17T07:00:00Z | 230 | 9d618e9c5d17 | outside_protected_intervals |
| dbe0d624876608efef4b0f38 | 49236051 | XAUUSD | 2025-01-17T07:00:00Z | 2025-01-24T07:00:00Z | 225 | df998a2ce780 | outside_protected_intervals |
| a4fb308e790f4cc3b70f76cc | 49236051 | XAUUSD | 2025-01-24T07:00:00Z | 2025-01-31T07:00:00Z | 230 | 96320b8b398c | outside_protected_intervals |
| fec37fdb9f362a3df78642ab | 49236051 | XAUUSD | 2025-01-31T07:00:00Z | 2025-02-07T07:00:00Z | 230 | 6252d546638d | outside_protected_intervals |
| f738006db6888c812aca1101 | 49236051 | XAUUSD | 2025-02-07T07:00:00Z | 2025-02-14T07:00:00Z | 230 | 12ac09f42cf9 | outside_protected_intervals |
| f20d724eab0b85cd91581473 | 49236051 | XAUUSD | 2025-02-14T07:00:00Z | 2025-02-14T11:00:00Z | 8 | deb389082f73 | outside_protected_intervals |
| ab2f4a0707dc9f2735e82283 | 49236051 | SpotBrent | 2025-06-06T07:00:00Z | 2025-06-13T07:00:00Z | 214 | e7d39968be13 | outside_protected_intervals |
| e0ae5b6bfede6e585beb7287 | 49236051 | SpotBrent | 2025-06-13T07:00:00Z | 2025-06-20T07:00:00Z | 207 | d85b93b2daec | outside_protected_intervals |
| 1ef853f6a4bea0678f59ea11 | 49236051 | SpotBrent | 2025-06-20T07:00:00Z | 2025-06-27T07:00:00Z | 214 | 1179f4225b65 | outside_protected_intervals |
| 74848f52e123ff44e91c1a80 | 49236051 | SpotBrent | 2025-06-27T07:00:00Z | 2025-07-04T07:00:00Z | 214 | f1361dc66456 | outside_protected_intervals |
| d513a63b540e85124bc64990 | 49236051 | SpotBrent | 2025-07-04T07:00:00Z | 2025-07-11T07:00:00Z | 207 | c0ca69de15e9 | outside_protected_intervals |
| 4e8052ee32049dfaa412d344 | 49236051 | SpotBrent | 2025-07-11T07:00:00Z | 2025-07-18T07:00:00Z | 214 | 780de7ec959b | outside_protected_intervals |
| 8409e7976865d64b646bb875 | 49236051 | SpotBrent | 2025-07-18T07:00:00Z | 2025-07-25T07:00:00Z | 214 | bf7bb37bec0d | outside_protected_intervals |
| f63bbe30a75983bee025455a | 49236051 | SpotBrent | 2025-07-25T07:00:00Z | 2025-07-25T17:00:00Z | 20 | 52d9a5793b7b | outside_protected_intervals |
| 9873e7e908f1516c2e16dd17 | 49236051 | SpotCrude | 2025-06-06T07:00:00Z | 2025-06-13T07:00:00Z | 230 | 420807805501 | outside_protected_intervals |
| 4cb757b50dbb69dd6e0bc35a | 49236051 | SpotCrude | 2025-06-13T07:00:00Z | 2025-06-20T07:00:00Z | 225 | 53dad7084de1 | outside_protected_intervals |
| 1104e45f59d4650d65302f3b | 49236051 | SpotCrude | 2025-06-20T07:00:00Z | 2025-06-27T07:00:00Z | 230 | cb511b0b6fa1 | outside_protected_intervals |
| e592a2ca8d2a2682895af40b | 49236051 | SpotCrude | 2025-06-27T07:00:00Z | 2025-07-04T07:00:00Z | 230 | ae6e637c7a0c | outside_protected_intervals |
| 7ff9e3af2f459faf23e88ccb | 49236051 | SpotCrude | 2025-07-04T07:00:00Z | 2025-07-11T07:00:00Z | 222 | 3e45dd0d812a | outside_protected_intervals |
| 0a10a6b87c2c6907825cae3c | 49236051 | SpotCrude | 2025-07-11T07:00:00Z | 2025-07-18T07:00:00Z | 230 | 0d7895d37fd2 | outside_protected_intervals |
| 16305212cbaa44f5d81d4910 | 49236051 | SpotCrude | 2025-07-18T07:00:00Z | 2025-07-25T07:00:00Z | 230 | d347dfff9b41 | outside_protected_intervals |
| ff7c79ce7ab92f7fefd5e8d2 | 49236051 | SpotCrude | 2025-07-25T07:00:00Z | 2025-07-25T17:00:00Z | 20 | 38043b318c9e | outside_protected_intervals |
| f9abb6f30d169784347f61c1 | 49236051 | US500 | 2025-06-06T13:30:00Z | 2025-06-13T13:30:00Z | 230 | 8416265bda80 | outside_protected_intervals |
| 49b1fa141d99f04c80603ee2 | 49236051 | US500 | 2025-06-13T13:30:00Z | 2025-06-20T13:30:00Z | 222 | 09104a0ceac8 | outside_protected_intervals |
| ce1a3ae806857a061446a48f | 49236051 | US500 | 2025-06-20T13:30:00Z | 2025-06-27T13:30:00Z | 230 | f71418460f8a | outside_protected_intervals |
| b29ab6548f07c1b42b657cce | 49236051 | US500 | 2025-06-27T13:30:00Z | 2025-07-04T13:30:00Z | 223 | eb66eb06bb09 | outside_protected_intervals |
| 6ef64b43d5d1878779f3f4e6 | 49236051 | US500 | 2025-07-04T13:30:00Z | 2025-07-11T13:30:00Z | 222 | edc99903a2c7 | outside_protected_intervals |
| c5b2fb6cf7ecad6ece11f52e | 49236051 | US500 | 2025-07-11T13:30:00Z | 2025-07-18T13:30:00Z | 230 | 229773c58082 | outside_protected_intervals |
| fc3798f4d963f1228862fbba | 49236051 | US500 | 2025-07-18T13:30:00Z | 2025-07-25T13:30:00Z | 230 | 0e749798f3d1 | outside_protected_intervals |
| f2f7edca5e352ddaf64b26ee | 49236051 | US500 | 2025-07-25T13:30:00Z | 2025-07-25T17:30:00Z | 8 | 67a47d4c12c3 | outside_protected_intervals |
| 18564c364dfd1f5a3fa6082c | 49236051 | XAUUSD | 2025-06-06T07:00:00Z | 2025-06-13T07:00:00Z | 230 | 60921f44b36d | outside_protected_intervals |
| 76662582a6bc5128e60da7e2 | 49236051 | XAUUSD | 2025-06-13T07:00:00Z | 2025-06-20T07:00:00Z | 225 | f31cff857df6 | outside_protected_intervals |
| 210ef4ab05770ad1cd5d9cd9 | 49236051 | XAUUSD | 2025-06-20T07:00:00Z | 2025-06-27T07:00:00Z | 230 | 7cab57fa17bc | outside_protected_intervals |
| c8acb9bdc9639790fd2b1d7a | 49236051 | XAUUSD | 2025-06-27T07:00:00Z | 2025-07-04T07:00:00Z | 230 | 187018dd4e7b | outside_protected_intervals |
| 124f72df872065be2198e68b | 49236051 | XAUUSD | 2025-07-04T07:00:00Z | 2025-07-11T07:00:00Z | 222 | b5f8865bd32f | outside_protected_intervals |
| 3f80a368c0234c235391d3d6 | 49236051 | XAUUSD | 2025-07-11T07:00:00Z | 2025-07-18T07:00:00Z | 230 | e4e341bc3c1f | outside_protected_intervals |
| 91e5122683d3af69b67d11aa | 49236051 | XAUUSD | 2025-07-18T07:00:00Z | 2025-07-25T07:00:00Z | 230 | a9bde6e0d370 | outside_protected_intervals |
| 41ac6478ad8235b76637391a | 49236051 | XAUUSD | 2025-07-25T07:00:00Z | 2025-07-25T11:00:00Z | 8 | d6567b5decdb | outside_protected_intervals |

### 8.3 Failed and unqualified retained sources

| Symbol | State | From UTC | To UTC | Reason | Receipt SHA256 prefix |
| --- | --- | --- | --- | --- | --- |
| BTCUSD | captured_raw | 2023-12-08T00:00:00Z | 2023-12-15T00:00:00Z | raw_response_not_qualified | 081d4edd77c7 |
| BTCUSD | captured_raw | 2023-12-15T00:00:00Z | 2023-12-22T00:00:00Z | raw_response_not_qualified | ae24d38d3d50 |
| SpotBrent | failed | 2023-12-08T07:00:00Z | 2023-12-15T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2023-12-15T07:00:00Z | 2023-12-22T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2023-12-22T07:00:00Z | 2023-12-29T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2023-12-29T07:00:00Z | 2024-01-05T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2024-01-05T07:00:00Z | 2024-01-12T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2024-01-12T07:00:00Z | 2024-01-19T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2024-01-19T07:00:00Z | 2024-01-26T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotBrent | failed | 2024-01-26T07:00:00Z | 2024-01-26T11:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2023-12-08T07:00:00Z | 2023-12-15T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2023-12-15T07:00:00Z | 2023-12-22T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2023-12-22T07:00:00Z | 2023-12-29T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2023-12-29T07:00:00Z | 2024-01-05T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2024-01-05T07:00:00Z | 2024-01-12T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2024-01-12T07:00:00Z | 2024-01-19T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2024-01-19T07:00:00Z | 2024-01-26T07:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| SpotCrude | failed | 2024-01-26T07:00:00Z | 2024-01-26T11:00:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2023-12-08T14:30:00Z | 2023-12-15T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2023-12-15T14:30:00Z | 2023-12-22T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2023-12-22T14:30:00Z | 2023-12-29T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2023-12-29T14:30:00Z | 2024-01-05T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2024-01-05T14:30:00Z | 2024-01-12T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2024-01-12T14:30:00Z | 2024-01-19T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2024-01-19T14:30:00Z | 2024-01-26T14:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2024-01-26T14:30:00Z | 2024-01-26T18:30:00Z | bar_identity_or_truncation_unverified | 8f89ac566b45 |
| US500 | failed | 2024-01-24T14:30:00Z | 2024-01-24T18:30:00Z | interrupted_reserved_job_no_retry | ce29c8066136 |
| US500 | failed | 2024-01-24T14:30:00Z | 2024-01-24T18:30:00Z | retained_prior_failed_job | fe1eff4f17b8 |

### 8.4 Duplicate tick receipts

| Receipt episode ID | Canonical ID | Source path | Receipt SHA256 prefix | Seal |
| --- | --- | --- | --- | --- |
| 826b4b0dad599e0600a0031b | 4fe286efdb366d24671e8115 | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-01-22.json | e8aef722f639 | outside_protected_intervals |
| a95f7f6fd3ef755fefdfc803 | 845b548701e7627dc3f53be3 | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-01-23.json | 5c3b322e0f45 | outside_protected_intervals |
| 2e9c4218de98ef40567c9486 | c544f73b7b6de1593d3c133a | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-01-25.json | 1601fd40f4be | outside_protected_intervals |
| d971c110da9825ff16a0e415 | e56cc14a79f4db75fde05223 | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-01-26.json | f9810350c950 | outside_protected_intervals |
| 6df19c5dbb09e38a45997e92 | 6681fcc669deff45d1f60f88 | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-07-22.json | 30998d998f41 | outside_protected_intervals |
| 9ef93a0f7b5a2976dc9e88dd | 04ea9a8aed2ded217967b9a0 | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-07-23.json | 9d4c0c555ae9 | outside_protected_intervals |
| 7681edb67b8ebcb48f554478 | 001f989aba50039f6db56506 | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-07-24.json | 192885778bde | outside_protected_intervals |
| 780ceea0a4e17a766f52d46e | ff90f42b6db284ffa5fee37a | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-07-25.json | 75078f2a0db5 | outside_protected_intervals |
| e238d78ebb5991270f634f04 | 9856b173f9625f9053df2d6a | /workspace/jiaoyi-three-year-pilot-18325/job-US500-2024-07-26.json | 3e26ffa83af9 | outside_protected_intervals |
| 3d05baebb817f53a22180886 | 39f5596f1015e6d5706ba25b | /workspace/jiaoyi-tick-replay-evidence/btc/result.json | 9ed84a9fce15 | quarantined_protected_dependency |

The matching journal entries use unique §LOG indices after 145 because the requested §LOG.132 already exists. No published journal entry was overwritten.
