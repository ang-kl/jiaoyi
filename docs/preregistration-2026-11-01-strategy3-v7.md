# Pre-registration: Strategy 3 v7 — final entry dispatch validation

Owner instruction: “execute the plan”, 1 November 2026; source plan №18,090–18,092, Phase C. By <Codex (GPT-6), effort: not exposed>.

This registration inherits the v6 band, first eligible quote at each 30-minute mark, original midpoint side, Q=1.38, cost formula, quantity/risk limits, one entry per symbol/session, durable session cost veto, exits and owner-only pause policy. It adds an entry dispatch condition; the earlier five-second quote-eligibility window did not itself impose this send deadline.

The inherited cost skip stands for the whole session; a later tighter spread does not reopen it. Do not refit the band, Q, stop or size to the observed closes.

## New entry condition

The MARKET frame must begin transmission before the original decision mark + five seconds. The elapsed budget is measured with a monotonic clock from selection of the eligible quote. After the ledger write and transport writer-lock wait, the connection reader's latest bid and ask must both be positive, finite, non-crossed and no older than six seconds. The original frozen band, using the existing `mark_decision` midpoint method, must still select the original side. An account execution received during admission invalidates that admission. A disconnect cannot certify cached quotes on a new session.

The final cost check retains the prepared median spread, measured commission and registered slippage rate; it evaluates the registered reserve at the current executable entry side. It does not substitute instantaneous spread or change the recorded TP/SL payload. If C >= original R, the existing durable session cost veto is recorded. Expiry, missing quote, changed side and changed account state are separate refusals, never cost vetoes.

A final refusal sends no frame, changes no direction and does not reuse the consumed decision mark. When the intent was already persisted, a durable `REFUSED_BEFORE_SEND` event identifies it as proven unsent. That attempt does not use the one-entry allowance, but its serial/key is never recycled. Uncertain or partial transmission retains unresolved status and cannot be retried automatically.

## Direct-predecessor session veto compatibility

Because the cost rule is unchanged, a durable v6 cost veto for the same account, symbol and session remains binding after this upgrade. Only the exact embedded v6 predecessor hash is accepted in addition to v7; a different account, symbol, date or experimental registration is not inherited. This does not relabel the original event or import v6 outcomes into Kelly.

## Attribution and unchanged exits

New decisions carry this file's hash and the `entry-dispatch-v1-2026-11-01` policy identifier. Existing positions retain their actual opening registration, original risk and exit plan. Kelly remains scoped to the current registration and symbol; prior v6 closes are not silently imported, so a new v7 cohort initially has insufficient closes for Kelly. This is not a financial PASS.

No stop, close, partial-profit or session-exit command is subject to the new entry-only guard. No automatic reversal, manual-order management or change to risk-limit settings is authorised by this registration.
