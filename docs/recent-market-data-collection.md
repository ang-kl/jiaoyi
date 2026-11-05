# Recent broker market-data collection

№18,263 §§1–6 · 06-11-2026, 07:00:14 SGT · by <Codex (GPT-6), effort: not exposed>.
Owner instruction: “collect fresh market data”.

The existing `/owner/backtests` runner only accepts development data before
24 October 2025. Its seal remains unchanged. Recent acquisition is a separate,
owner-authenticated `/owner/market-data` POST/GET route in the existing **jiaoyi**
service. It collects quotes without running a strategy, changing live settings,
placing orders, refreshing tokens or writing a research archive to PostgreSQL.

Request fields: `account`, exact original-20 `symbol`, UTC `from`, UTC `to`.
The window starts at the symbol's registered local session open and lasts four
hours. It must start on/after **24 October 2026**, beyond the entire original
holdout, and end at least five seconds before the request. Both boundaries are
exact UTC half-hour marks. Unknown symbols, held-out dates, future or invalid
windows and accounts outside the allowlist are refused before broker access.

The internal acquisition adapter reuses registered session validation and the
existing read-only broker connection/paginator. Its private placeholder replay
fields never enter a study and are not presented as a trading plan. No original
development validator is relaxed. The collection and backtest endpoints share
one job slot, timeout and memory bound; neither can start while the other runs.
Both GET routes expose the same latest owner job, so clients must verify its
ID, kind and request. Download before replacing a completed job or deploying.

Limits remain **200,000 bid+ask ticks, 120 historical requests, 240 seconds per
job**, with 750ms pacing after each history response. Truncated responses are
discarded and bisected into disjoint closed intervals. Unresolvable truncation
or a cap produces a failed job, never a partial success. Returned ticks must
have positive on-grid prices, ordered timestamps and stay within the requested
window plus its five-second tail. Separate bid and ask streams preserve their
timestamps without inventing cross-side order or continuous executable quotes.

Results contain both tick arrays, current contract precision/minimum/step/lot
size, native source SHA256, collector source SHA256, collection time, request
bounds and separate side counts. `study` is null and qualification is
`RECENT_COLLECTION_ONLY_NOT_FINANCIAL_PASS`. A successful empty broker response
has `both_sides_present:false`; that is missing market coverage, not a flat price.
Contract metadata is current, not certified historical metadata. Collection
alone adds no qualifying trades or evidence that a win-rate goal has been met.

## One-off collector

```sh
python3 ops/research/market_collection.py --date 2026-11-05 \
  --account 49236051 --output /tmp/jiaoyi-market-2026-11-05
```

Use the existing `JIAOYI_UI_PASSWORD` privately. HTTPS redirects are refused;
no credential is stored in output. The client freezes all 20 requests before
submission, checks that every five-second tail has ended, starts only one job,
verifies identity/digest/no-study, and saves each response before moving on.
It resumes existing matching completed/failed receipts without resubmission;
replacements, restarts and unexpected running jobs require inspection.
Failures and empty coverage remain visible in the summary. Source files stay
outside Git and the database. No scheduler, new service or model API is added.

For 5 November 2026, cash-index windows follow local time/DST: the US window is
14:30–18:30 UTC, Sydney is 23:00 UTC on 4 November through03:00 on 5 November,
Tokyo/crypto start00:00 UTC; London/Frankfurt start08:00 UTC; FX/metals/energy
start07:00 UTC. Actual
request timestamps in the manifest govern; a local session date is not a shared
UTC start. This is a bounded fresh snapshot, not a full-day or live quote feed.

Three focused Rust checks cover the post-holdout/closed-window boundary, invalid
source data and authentication/account scope. The existing backtest seal check
still passes. One Python check covers canonical source hashes, altered ticks,
identity and accidental study output; all14research client tests pass. Local
denied-warning Clippy passed. Exact-head CI, deployment and actual20symbol
collection receipts must be recorded on the PR before claiming delivery.

The invariant goal remains73% net wins and1:1.38 risk:reward per latest20 fully
closed owned positions per strategy/account;50% per class is an interim research
target. Neither is implied by a successful download. Keep proven fixes and
respect “Time is Money”: collect bounded evidence once, then evaluate under an
explicit protocol instead of repeatedly downloading or tuning outcomes.
