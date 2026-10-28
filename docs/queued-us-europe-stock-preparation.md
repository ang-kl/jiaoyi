# Queued: US and European stock preparation

Owner request: chat №17,618 §1, 28-10-2026 07:15 SGT.

After the active S3 pre-open preparation release, build pre-market preparation
and extend the stock watchlist with broker-available US and European shares.
Carry forward the deadline of at least ten minutes before the relevant open.
Exclude Chinese stocks and Chinese indices; check issuer domicile as well as
listing venue, including Chinese issuers listed through ADRs. Preserve existing
risk, ownership and registration requirements. Record unavailable symbols or
missing evidence explicitly. This queued note does not activate new trades.

## Bounded cache-only watchlist/status delivery

Source remains chat №17,618 §1 (28-10-2026 07:15 SGT), parent S3 request
№17,616 §1 (07:09 SGT). This delivery adds
`GET /operations/stock-preparation`; **actual stock market-input preparation is
unavailable**, not completed by publishing candidates.

The embedded Rust basket contains Apple/AAPL and Microsoft/MSFT (US, NASDAQ),
SAP/SAP and Siemens/SIE (Germany, Xetra). Issuer country comes from inspected
issuer incorporation/registered-office evidence, separately from exchange
evidence, with source links and 28-10-2026 07:49 SGT verification time in the
definitions/output. This is a small large-cap watchlist, not measured current
broker liquidity or a general screening platform. No prior stock candidates
were found in the existing source, configuration or task documents. Only these
positively identified issuers are included; Chinese issuers/ADRs, indices and
unknown issuer countries cannot enter through a US listing or ticker guess.

Each allowlisted account receives one compact **on-demand current view** per
exchange/session, with source run/time, broker-name resolution, true stock cash
clock, T-20 start and T-10 deadline (UTC and SGT). The existing exchange helpers
handle DST and weekends; verified holidays remain unknown. On 28 October these
give NASDAQ 21:10/21:20/21:30 SGT and Xetra 15:40/15:50/16:00 SGT. The view
does not persist snapshots, run autonomous preparation, or imply a receipt
existed before it was requested. During an open session its missed deadline
stays visible; after close the view advances to the next weekday session.

The loader already retains the entire broker catalogue. The view uses
`latest_listed_symbols` and `assets_of_run` for that exact account/run, rather
than combining it with an older successful `latest_assets` result. Matching is
case-sensitive and exact, without suffix/alias inference. Missing/duplicate IDs,
duplicate names, disabled/unknown flags, mismatched or missing base/quote assets
and catalogue timestamps outside the preceding 24 hours are explicit refusals.
That conservative cache-age check is an engineering freshness condition, not a
trading or risk threshold. Raw catalogue fields are not exposed.

Even a unique coherent ticker/base/currency match remains
`matched_name_identity_unverified`: the cached catalogue omits authenticated
issuer/exchange identifiers. SAP's US ADR also uses SAP; a name alone cannot
choose the German ordinary-share clock. The existing recorder/history loader
collects only the active universe, not these stock candidates. Consequently
every stock remains ineligible, `preparation_state=unavailable`,
`prepared_at=null`, `deadline_met=false`, with missing broker identity,
quote/history collection and verified cash-calendar evidence. No fabricated
prices, history, preparation completion, stock registration or financial PASS.

The module has no broker client or order capability. GET writes no decisions,
orders or settings, and POST is refused. No stock is added to live settings or
S3 attribution; no new polling, per-symbol API/model calls, provider, framework,
schema or quote archive. Completing actual stock preparation needs verified
broker instrument identity and an authorized source of stock inputs; this
cache-only result does not bypass the task's acquisition constraints.

Required Rust and real Docker-image CI still run on the coordinator's PR.
Dockerfile builds the binary with `cargo build --release --locked -p jiaoyi
--bin jiaoyi`; embedded definitions live under `crates/`, which is included in
the image context. Most docs are excluded by `.dockerignore`, so the module
does not embed this document. No merge/deployment or deadline read-back is
claimed by this worker. Financial movement is not measurable; the ≥73% net-win
latest20 goal per strategy/account and 1:1.38 realized payoff objective remain
unchanged and separate from planned geometry.
