# Separate entry timing from execution cost

Trace: №18,249 §§1–5 and №18,250 §§1–5 · 05-11-2026, 17:24 SGT.
By <Codex (GPT-6), effort: not exposed>.

The current `slippage` metric compares the broker fill with the decision quote.
It cannot establish the execution cost after dispatch: price can move while
admission and writer-lock waits run. The inherited 24bp policy also does not
certify a measured round-trip fee. Bot-trade's source used two FX observations,
proposal-relative adverse movement and a TP1 reserve, without measured exits.
Neither that evidence nor a cheaper placeholder justifies lowering the reserve.

## Minimal correction

`GET /trading/orders` adds nullable `entry_cost` on opening orders. It reads the
existing `ENTRY_VALIDATED_BEFORE_SEND` receipt and actual broker deal VWAP.
The receipt must match the intent's account, request, symbol and side, and show
a passed final check after the writer lock with valid two-sided quotes. No new
recording, table, scheduled job, model call or live trading rule is introduced.

All components are signed, positive adverse, and use the same dispatch-price
basis-point denominator:

- Decision to dispatch: movement before the final local check.
- Dispatch to fill: movement and execution after that check, not pure broker latency.
- Decision to fill: the sum of those components.

The executable side is ask for a buy and bid for a sell. Quote age is local
receipt age; it does not certify source timestamp freshness. Commission is not
included. Missing receipts or actual deals return unknown, not zero. Reconciled
position prices alone are not execution samples. Older `slippage` fields retain
their existing definition. This endpoint remains a bounded newest-orders view,
not a complete or unbiased calibration cohort. Entry evidence cannot certify
exit costs or a round-trip reserve.

## Verification and next economic decision

Three focused unit tests check signed decomposition, missing evidence and
invalid receipts. One PostgreSQL integration test checks actual joins, request
identity and exclusion of reconcile-only prices. Required repository CI remains
mandatory. Once proven, preserve this correction; change it only for a newly
evidenced defect, not to improve a backtest score.

After release, inspect actual BTC/NatGas/FX receipt coverage and report sample
counts separately by account, strategy and symbol. Do not fit costs from tiny
samples or favourable medians. Keep 24bp as the frozen policy/stress case until
independent entry and exit evidence supports an explicitly reviewed alternative.
Then preregister at most one mechanism-based entry amendment per instrument,
compare against the unchanged baseline and validate on separate dates. Failed
candidates remain failed. The interim 50% research target and original 73% /
1:1.38 per-strategy/account goal are not met by this measurement correction.
