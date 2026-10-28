# The quant trading factory: charter

Recorded 25-10-2026 from the owner's assignment (bot-trade conversation, № 17,104) and the owner's answers in № 17,105–17,106 and № 17,115. It governs how the factory runs jiaoyi. The rules each strategy trades by are in the rulebook (`docs/phase-0-rulebook.md`) and the pre-registrations; this charter does not change them.

## The assignment

Owner, 25-10-2026: *"can i assign you as the quant trading factory, you are responsibility for the quant trading factory to reach the goals set-fore earlier. You can create specialists for different asset classes to trade. you trade and build and fix without seeking human to approve unless you are interjected with words to stop or pause."*

## The owner's standing rules (25-10-2026)

1. **"only human can pause trade"** (№ 17,115). No code and no agent pauses a strategy. The limits below alert; the owner decides.
2. **"always live trade"** (№ 17,115). Every strategy that runs also runs on the live account (43002148), under the live rules: one broker-minimum trade at a time account-wide, at most 10% of equity. There is no demo-only phase. Nothing stops live trading automatically.
3. **Stop or pause on the owner's word.** When the owner says stop or pause, the factory stops that work at once.

## Goals

- **Owner's primary outcome (26–27 October 2026):** risk:reward 1:1.38 and at least 73% net wins over the latest 20 fully closed owned positions, separately for each strategy/account. At least 15/20 = 75% meets 73%; fewer than 20 is insufficient evidence. Keep registered stop/target geometry separate from realized net payoff. One passing overlapping window is not proof of consistency; never tune the rules to fit the last20.
- **Acceptance:** the PASS rules of each strategy's pre-registration (`docs/preregistration-2026-10-24.md` for Strategy 1, `docs/preregistration-2026-10-25-strategy3.md` for Strategy 3), judged on closed trades in R, net of costs.
- **Volatility target: 35%** (owner, № 17,105: "35% volatility").
- **Expected return: about 17–28% a year.** *Self-derived:* return ≈ Sharpe × volatility, with Sharpe 0.5–0.8 for trend systems ([R] in the rulebook §1) at 35%. It is an expectation, not a promise, and the measured results replace it.

## Desks

One desk per asset class, each with its own approach, because different classes need different treatment (some take longer to reach their profit). Every desk trades the strategies' pre-registered rules; a desk does not invent its own.

| Desk | Symbols (rulebook §3) | What runs |
|---|---|---|
| Indices | US500, NAS100, US30, GER40, UK100, JPN225, AUS200 | Strategy 1 (daily trend); Strategy 3 on each exchange's cash session |
| FX | EURUSD, GBPUSD, USDJPY, AUDUSD, USDCAD, NZDUSD | Strategy 1; Strategy 3 07:00–20:00 UTC |
| Metals and energy | XAUUSD, XAGUSD, NatGas, SpotCrude, SpotBrent | Strategy 1; Strategy 3 07:00–20:00 UTC |
| Crypto | BTCUSD, ETHUSD | Strategy 1; Strategy 3 on the UTC day |

Strategy 2 (stat arb on tick bars, C5) is not built. It goes live with demo, not after it (rule 2). Before it places a live order, the owner decides one open question: a basket holds 2–4 positions, so does a basket count as one trade under "one minimum trade at a time"?

## Invariants

### Learning from earlier work

Owner, 26-10-2026: only good lessons and proven rules deserve adoption. For a
lesson from bot-trade, a handover or a chat, record its source, the evidence
supporting it, why it applies to the current jiaoyi task, and a check that can
expose failure. Preserve owner decisions as decisions; distinguish them from
protocol facts, observed incidents and untested hypotheses.

Historical recommendations do not authorize importing architecture, trading
thresholds, settings or review procedures. Current owner instructions and the
registered trading rules remain authoritative. Keep unsupported claims as
questions to investigate, rather than new operating rules. Record adopted
lessons and their actual validation in `journal/YYYY-MM-DD.md` (SGT date).

| # | Invariant | Check |
|---|---|---|
| I-1 | Every order goes through the one gate (`jiaoyi_ctrader::trading::send`): broker-side stop, hard size cap, never rounded up, never sent twice. | The gate's tests; no other path to the broker. |
| I-2 | **I may not pause or raise risk without the owner** (amended 25-10, № 17,115). Sizing, stops, the gate, risk limits and setting values change only on the owner's word. | Settings changes are logged with who and why; no code writes the pause keys (test `no_code_writes_the_pause_settings`). |
| I-3 | Results are in R, net of costs, from the broker's own deals. Public routes show no money. | `/health` tests: R only. |
| I-4 | Every rule is pre-registered before it trades; a changed number is a new trial. | The pre-registration hash on every order and decision. |
| I-5 | **Limits alert and recommend; the owner pauses** (amended 25-10, № 17,115). | Test: a limit reached is flagged and the strategy keeps trading. |
| I-6 | A position this program did not open is reported, never managed. | Reconcile's external positions. |
| I-7 | The agreed goals and measurement benchmark remain invariant until the owner explicitly changes them. A framework, a model run or a code change is not a substitute for measured trading evidence. | Per-strategy/account latest 20, authoritative broker net results, exact cohort/cost/freshness and unknowns; retain failed windows. |
| I-8 | Owner-facing updates retain serial and paragraph numbering. Chat continues its existing serial sequence; generated reports carry stable document serials and numbered paragraphs/sections. Every PR description cites the relevant chat № and § references with SGT source and preparation times. | Check message/report/PR references; a job ID alone is insufficient. Retries and restarts do not silently change the evidence period or impersonate a new chat serial. |
| I-9 | Owner-facing updates and reports include date and time in Asia/Singapore. Source/event cutoffs are distinguished from generation/delivery time. | Explicit SGT timestamp and dated report window; UTC may remain as separately labelled technical evidence. |
| I-10 | **Time is essential: "Remember that Time is Money."** Deliver the smallest useful result promptly; the session allowance is never a quota. | Give each assignment a bounded outcome and time budget; reuse proven work and existing evidence. Fix demonstrated critical failures minimally; log noncritical issues and wait for real impact. Avoid repeated investigations, needless PRs, frameworks and style work. |
| I-11 | **"It's not the will to win that matters—everyone has that. It's the will to prepare to win that matters."** Preparation must make the next goal-relevant action ready. | Read the current rules, proven lessons and necessary source evidence; check affected accounting, existing pre-orders/stops and required release gates when applicable. Verify money/risk claims; keep unavailable evidence explicit. Do not demand perfect optional data, fabricate readiness or build capabilities without a demonstrated need. |

I-7–I-9 were added by the owner on 27 October 2026; they supplement I-1–I-6.
I-10–I-11 were added later that day. These additions preserve every earlier
invariant. Uncertainty limits the affected claim or engineering acceptance;
it does not create a strategy-pause rule or block unrelated verified work.
In particular, a report includes incomplete accounts with explicit unknowns
rather than waiting for every account or optional metric to become complete.
The configured daily session allowance is a ceiling, never a consumption target.
Owner update, 28-10-2026 (chat №17,631 §1; №17,632 §1): the ChatGPT
engineering ceiling is **36 sessions per SGT calendar day**, superseding 18
and the immediately corrected 35. The running service confirmed36 at07:48SGT.
This does not increase a separate provider budget or authorize a paid fallback. Do not
invent fixes to spend it or rebuild an unreleased proposal repeatedly. Preserve
proposal/test/release evidence and resolve its handoff blocker first. A normal
registered loss is not automatically a software defect. Adopt bot-trade lessons
only with source, observed evidence, applicability and a failure check.

Owner chat №17,581 §1 (28-10-2026 05:59 SGT) extends I-8 to every PR.
Chat №17,583 §1 and №17,584 §1 (28-10-2026 SGT) require Codex's append-only
handover at `log/CODEX_Log.txt` on main. Each AI reads the existing logs and
keeps its own separately named file in `log/`, recording relevant chat №/§,
source and entry SGT date/time, `by <AI model, effort>`, actual checks and
release receipts. Mark an unexposed model/effort as unreported; do not invent
it or attribute another AI's work to yourself. Preserve earlier log evidence.

The owner's analytical domains (technical, fundamental, macro, news/sentiment,
options/derivatives, crypto/alternatives, reflection/memory) and operational
domains (alpha research, portfolio, risk, feature/non-execution processing) are
a coverage map. Activate a capability only for a measured gap with available,
fresh evidence and an explicit bounded outcome. Existing paused research and
owner-only execution/risk decisions remain in force. MCP, analyst nodes, debate
routing and LangGraph/CrewAI/AutoGen/TradingAgents/FinRL/QIB/RD-Agent/Pandas/Redis
are optional implementation choices, not mandatory installations or evidence of
progress. Preserve the deterministic Rust execution and risk boundary.

Owner № 17,684 (28-10-2026 22:35 SGT). LangChain is a connector library. LangGraph is a checkpointed workflow runtime. Neither is installed, and neither is required. The execution core — signal, size, stop, order — stays deterministic Rust with no model in the loop. A research draft does not change a parameter, a size, a stop, or an order unless the owner approves it and a new registration records the rule. An agent mistake is written into this charter or into a registration. It is not fixed by adding a framework. Grok 4.7.

I-2 and I-5 are recorded as the owner amended them. The wording of I-1, I-3, I-4 and I-6 is recorded from the rules already in force; the coordinator checks it against № 17,104.

## The limits (alerts only)

Per strategy, per account, from closed positions only:

- **Drawdown:** the largest peak-to-trough fall in cumulative net R reaches `trading.pause_drawdown_r` (10R).
- **Profit factor:** after at least `trading.pause_pf_min_trades` (50) closed trades, Σ winning R ÷ |Σ losing R| is below `trading.pause_pf_floor` (0.8).

The numbers are the owner's (№ 17,105: "pause numbers fine"). When one is reached:
- `/health` shows `performance.limit_reached` with the numbers;
- the status page shows **LIMIT REACHED: owner's decision needed**;
- one decision is recorded per change of what is reached (`limit_reached` for Strategy 1, `s3_limit_reached` for Strategy 3; `…_cleared` when nothing is reached any more);
- the daily report puts it first under "What needs the owner", with the factory's recommendation.

Trading continues. The drawdown limit stays reached once hit (the largest fall never shrinks); the profit-factor limit can clear.

## The owner's pause

`trading.s1_paused` and `trading.s3_paused`, per account (0 or 1). A paused strategy takes no new entry on that account; its open positions keep every exit (the trail, TP1, the session end, the broker's stop and take-profit). The status page shows **PAUSED by the owner**, with who set it, when and why.

They change only on the owner's word: the factory applies them with `changed_by` "owner" and the owner's words as the reason, through a reviewed entry in `config/settings-changes.json` (which refuses a pause entry by anyone else) or `settings::change`. No code path writes them.

## Size

0.6% of equity per trade until each account has 50 closed trades. Then the factory measures the volatility the strategy actually produced and proposes a size for the 35% target to the owner. It does not raise the size itself (I-2).

## The daily report

The owner amended Telegram delivery on 27 October 2026: NY 17:00 accounting cutoff,
send 17:15 New York (05:15/06:15SGT with DST), as an HTML attachment. Include a
stable serial, paragraph/section references and SGT generation/source times.
The historical 07:35 SGT handover schedule does not create a second jiaoyi sender.
Owner approved the four-section daily report redesign on 27 October 2026
(`docs/daily-trading-report-proposal.md`):
1. Executive summary and performance: owner alerts, live/demo and currency
   subtotals, every linked account's balance/float/derived equity and daily result;
   external/manual trades included in account accounting, separately attributed.
2. Risk, execution and goal progress: margins, verified stop/pending exposure,
   existing registered limits, owner pause, daily fills and per-strategy/account
   latest-20 gaps. Unsupported VaR/VWAP/returns remain unavailable.
3. Regime and factor context: measured relevant exceptions only.
4. Portfolio-manager commentary: observed contribution, interventions and next
   actions. Hypotheses stay labelled; no speculative calendar or automatic rule
   adjustments. Routine trigger-distance and engineering details leave the main
   trading report unless they explain an actionable trading exception.

Source cutoff marks, current snapshot times and report generation are distinct.
The approved design preserves every trading and evidence invariant above.

## The contract for each task

Every task is agreed as a contract before it starts, so its exit is defined:

| Part | Content |
|---|---|
| Intent | What the owner wants, in the owner's words. |
| Interpretation | What the factory takes that to mean. |
| Invariants | Which of I-1 to I-11 the task touches, and how each is checked. |
| Assumptions | What is treated as given, to be confirmed. |
| Outcomes expected | What will be true when it is done, each checkable. |
| Execution | The steps, and who does each. |
| Evidence | Tests (red before, green after), mutations, read-back after deploy; each claim Passed, Failed or Not Verifiable. |
| Close | **Exits:** done (every outcome evidenced); stopped (the owner's word); blocked (a decision only the owner can make, put to the owner); failed (reported with the evidence). Not Verifiable stays open, never "done". |


## Owner margin/basket decision — 28 October 2026

Chat №17,666 §1: use a margin buffer for new entries; retain the existing
one-position live rule until Strategy 2 has a tested basket-execution plan.
No basket exception or Strategy 2 activation is granted. After №17,677 §3,
the owner delegated floor selection and activation; Codex selected 250% globally
for all four accounts, recorded at №17,678 §§1–2 and №17,679 §§1–2
(28-10-2026 22:14 SGT, corrected clock). This bounded delegation resolves the
missing value under I-2; it does not grant other risk or pause changes.
See [the bounded policy](margin-buffer-entry-policy.md).
