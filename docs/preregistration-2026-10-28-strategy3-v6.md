# Pre-registration: Strategy 3 v6 — one entry, one cost skip (28-10-2026)

This file supersedes the entry count of `docs/preregistration-2026-10-28-strategy3-v5.md` for new Strategy 3 decisions. The service embeds this file and hashes it (SHA-256). Every Strategy 3 decision, intent and trade recorded after this registration carries this hash and the label `jiaoyi-s3`. A further changed rule is a further pre-registration with a further hash.

**Repeat note for other models.** Owner, 28-10-2026, 22:23 SGT: “Proceed to fix.” The fix is the repeat, not the band. Live Strategy 3 was 2 wins and 17 losses in 19 closes, and one symbol was opened eight times in one session. Do not refit the band, Q, the stop, or the size to those 19 closes. Do not spend a Grok permit on this.

## What changes from the 28 Oct v5 registration

- **One entry per symbol per session.** If this account already has a Strategy 3 open order for the symbol on this session date, a later signal is not sent. The first open still uses the mark, the band, the cost gate, and Kelly. A refused broker send that recorded no open order does not count.
- **One cost skip per symbol per session.** The first time the cost reserve C is at least R, the skip is recorded and remembered for that session date. Later marks in the same session do not decide it again. The next session computes the cost again. A tighter spread later in the same session does not reopen the skip.

## What does not change

- The band, the 30-minute mark, the side, Q = 1.38, the original stop, TP1, TP2, the Chandelier, MAE, MFE, sizing, the open-risk budget, the single gate, and the owner pause.
- The 28 Oct margin-buffer admission is unchanged. This registration does not set the floor and does not remove it.
- Kelly still reads only closes under this hash. v5 closes are not an input. Under 20 closes of this hash on this symbol, Kelly is silent. A non-positive fraction still skips. It does not raise size.

## Approval

- Owner, 28-10-2026, 22:23 SGT: “Proceed to fix.”

## Entry, as traded from this registration

- Unchanged until the gates above. The mark decision, the band, the side, the cost gate (C ≥ R skips), and the Kelly skip are the same on the first attempt.
- After one recorded open in the session, no second open on that symbol.
- After one cost skip in the session, no second cost decision on that symbol.

## Exit, as traded from this registration

Unchanged from v5. The Chandelier, TP1, TP2, and the session-end close are the same.

## What is not changed by this registration

- Sizing, the open-risk budget, the single order gate, ownership and intent-before-send are unchanged.
- Live still runs by the owner’s earlier order, not by a PASS.
- This registration does not pause an account. Only the owner pauses.
