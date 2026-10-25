# The quant trading factory: charter

Recorded 25-10-2026 from the owner's assignment (bot-trade conversation, № 17,104) and the owner's answers in № 17,105–17,106 and № 17,115. It governs how the factory runs jiaoyi. The rules each strategy trades by are in the rulebook (`docs/phase-0-rulebook.md`) and the pre-registrations; this charter does not change them.

## The assignment

Owner, 25-10-2026: *"can i assign you as the quant trading factory, you are responsibility for the quant trading factory to reach the goals set-fore earlier. You can create specialists for different asset classes to trade. you trade and build and fix without seeking human to approve unless you are interjected with words to stop or pause."*

## The owner's standing rules (25-10-2026)

1. **"only human can pause trade"** (№ 17,115). No code and no agent pauses a strategy. The limits below alert; the owner decides.
2. **"always live trade"** (№ 17,115). Every strategy that runs also runs on the live account (43002148), under the live rules: one broker-minimum trade at a time account-wide, at most 10% of equity. There is no demo-only phase. Nothing stops live trading automatically.
3. **Stop or pause on the owner's word.** When the owner says stop or pause, the factory stops that work at once.

## Goals

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

| # | Invariant | Check |
|---|---|---|
| I-1 | Every order goes through the one gate (`jiaoyi_ctrader::trading::send`): broker-side stop, hard size cap, never rounded up, never sent twice. | The gate's tests; no other path to the broker. |
| I-2 | **I may not pause or raise risk without the owner** (amended 25-10, № 17,115). Sizing, stops, the gate, risk limits and setting values change only on the owner's word. | Settings changes are logged with who and why; no code writes the pause keys (test `no_code_writes_the_pause_settings`). |
| I-3 | Results are in R, net of costs, from the broker's own deals. Public routes show no money. | `/health` tests: R only. |
| I-4 | Every rule is pre-registered before it trades; a changed number is a new trial. | The pre-registration hash on every order and decision. |
| I-5 | **Limits alert and recommend; the owner pauses** (amended 25-10, № 17,115). | Test: a limit reached is flagged and the strategy keeps trading. |
| I-6 | A position this program did not open is reported, never managed. | Reconcile's external positions. |

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

Every day at 07:35 SGT, the same shape each day:
1. What needs the owner (limit alerts first, each with a recommendation).
2. Each strategy and account: closed trades, net R, profit factor, drawdown, pause.
3. Fills: slippage, spread and order-to-fill time against the plan.
4. How near each symbol came to its trigger.
5. What was built or fixed, and what is open.

## The contract for each task

Every task is agreed as a contract before it starts, so its exit is defined:

| Part | Content |
|---|---|
| Intent | What the owner wants, in the owner's words. |
| Interpretation | What the factory takes that to mean. |
| Invariants | Which of I-1 to I-6 the task touches, and how each is checked. |
| Assumptions | What is treated as given, to be confirmed. |
| Outcomes expected | What will be true when it is done, each checkable. |
| Execution | The steps, and who does each. |
| Evidence | Tests (red before, green after), mutations, read-back after deploy; each claim Passed, Failed or Not Verifiable. |
| Close | **Exits:** done (every outcome evidenced); stopped (the owner's word); blocked (a decision only the owner can make, put to the owner); failed (reported with the evidence). Not Verifiable stays open, never "done". |
