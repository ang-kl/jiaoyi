# Entry dispatch policy v1

Trace: owner “execute the plan”, №18,090–18,092; 1 November 2026. By <Codex (GPT-6), effort: not exposed>.

The final-entry guard is checked synchronously after acquisition of the shared transport writer lock and immediately before starting the frame write. The connection reader maintains an independent bounded quote cache. The guard never polls a broker, database or model while holding the lock. Once writing starts, a timeout or cancellation remains an uncertain outcome and preserves the existing interrupted-stream recovery.

Strategy 1 permits dispatch for six seconds after selecting its daily trend/breakout side. Its current executable side must still trigger the same frozen level, both sides must be fresh within six seconds, and account execution state must remain unchanged. Decisions record `entry-dispatch-v1-2026-11-01`. Stop/target geometry, daily entry allowance and existing exits are unchanged. A proven-unsent daily attempt may be assessed again after 60 seconds, using a new suffixed intent key; all earlier attempts must have durable zero-byte refusal evidence. A sent, broker-rejected or unresolved attempt retains the prior daily hold.

Strategy 3 follows the separately hashed [v7 registration](preregistration-2026-11-01-strategy3-v7.md). A valid five-second quote does not grant unlimited time to finish asynchronous entry preparation.

The `REFUSED_BEFORE_SEND` terminal event is local evidence that the transport guard refused before attempting a byte. It is not a broker rejection and is never used for a failed or partially written frame. Failure to persist that event leaves the recorded intent unresolved. Session and risk accounting recognise the durable unsent event; a future permissible decision receives a new key. No frame is resent for an existing intent.

A refused attempt stores a compact validation snapshot in the existing local event's raw evidence: quote values and receipt ages, check time, account revision, connection generation, policy, remaining monotonic deadline and refusal stage. It is captured from the exact snapshot used for refusal, not reconstructed from a later quote. For a successful guard, the exact final validation snapshot is carried through the transport write and recorded once as `ENTRY_VALIDATED_BEFORE_SEND` only after the write and lock release. It is neither a fill nor a profitability verdict and is excluded from the returned broker-event list. A receipt failure after transmission is an evidence failure with unresolved outcome, never a proven-unsent refusal or permission to resend. No per-tick rows or model calls are added.

These are engineering conditions, not a promise of a profitable fill or of zero market movement after the guard. Validate normal and delayed paths before release and report refused/expired attempts honestly.
