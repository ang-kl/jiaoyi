# S3 preparation before the session opens

Owner source: chat №17,616 §1 (28-10-2026 07:09 SGT); implementation details
№17,617 §1, №17,619 §1 and №17,621 §1. This is operational preparation,
with the current registration's entry, exit, sizing, ownership and pause rules.

The existing trading service starts read-only preparation **20 minutes before
open**, with a **T-10-minute deadline**, for every configured symbol/account.
The registered exchange timezone handles DST; crypto uses its UTC-day boundary,
and FX/metals/energy use their registered 07:00 UTC session. No AI call or new
service is required. Commodity coverage is the configured metals and energy
universe; unresolved symbols remain visible as missing inputs.

Preparation reads the last sessions' M30 bars, computes the 14-session noise
coefficients for eligible marks, and checks symbol/asset/volume metadata,
spread samples and commission evidence. Only closed historical bars are used.
The broker's existing historical-request pacing is retained. One read-only
worker per existing connection is polled independently of the state-owning
trader, including while an execution handler waits. No pre-open order is sent.

The **actual session opening price cannot exist before open**. Its M30 bar,
fresh executable quote, current spread/cost reserve, cost/R test, Kelly input,
equity/margin, ownership and existing-order checks remain at entry. A prepared
coefficient table avoids downloading the full history at the decision mark;
an incomplete or absent table keeps the existing entry-time fallback. The
first eligible 30-minute mark and its first-quote identity are unchanged.

The existing decision table retains one compact coefficient/check receipt per
successful session and material retry. No raw bars or new schema are added.
Reconnect reloads that receipt with its original session, hash and completion
time. `/health` → `strategy3[].preopen_preparation` shows every configured
symbol's current/upcoming session, start, deadline, completion and missing
inputs. States distinguish scheduled, awaiting, prepared on time, incomplete,
prepared late and missed deadline. A late startup never invents an earlier
completion. The preparation task retries source failures while registered
position management continues.

For **28 October 2026 SGT**, the registered schedule gives:

| Session | Start preparation | Deadline | Open |
|---|---|---|---|
| AUS200 | 06:40 | 06:50 | 07:00 |
| JPN225 and the new crypto UTC day | 07:40 | 07:50 | 08:00 |
| FX, metals and energy | 14:40 | 14:50 | 15:00 |
| GER40 and UK100 | 15:40 | 15:50 | 16:00 |
| US500, NAS100 and US30 | 21:10 | 21:20 | 21:30 |

These times follow the registered calendars, not an assertion of exchange
holiday status. The existing holiday/open-bar check still applies. Today's
AUS200 deadline passed before this request; it cannot be met retroactively.

Validation: the original loop failed a quote-free pre-open test. The repair
passed that test with a complete coefficient table, no trading frames before
open, durable reconnect recovery and a single opening-bar read at entry.
Another fixture held the pre-open history reply ten seconds while an owned
TP1 still closed once. Boundary tests cover late completion, absent marks and
exchange DST. All46 S3 integration tests, three new unit checks, formatting
and denied-warning workspace Clippy passed locally. Required CI and deployed
observations are separate receipts. The ≥73%/1:1.38 latest20 financial goal
remains unproven; preparation completion is not a financial PASS.

The separately queued US/European stock request is recorded in
[queued-us-europe-stock-preparation.md](queued-us-europe-stock-preparation.md).
