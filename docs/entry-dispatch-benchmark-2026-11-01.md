# S3 v7 dispatch-policy benchmark and activation evidence

Prepared 01 November 2026, 19:25 SGT. Trace: approved plan №18,090–18,092 §PLAN.5/9. By <Codex (GPT-6), effort: not exposed>.

**Result:** all nine warmed normal controls at mark +0, +4.5 and +4.9 seconds sent successfully; measured final-validation latency was 15.6–30.6 ms. All three controls at +4.99 seconds expired because only 10 ms remained. A 250 ms account or ledger delay also expired all trials starting at +4.9 seconds. Cold preparation took 1.53–1.55 seconds. This completes the bounded measurement exercise; production rejection rates remain unmeasured, and the historical evidence below shows that zero additional missed entries cannot be assumed.

## Scope and measurement

The proposed deadline is exactly the original decision mark plus five seconds. It is stricter than the earlier five-second window for selecting a quote. These measurements assess the resulting admission restriction; they do not test profitability, claim a trading edge or alter the deadline.

The controlled benchmark exercises the complete S3 opening path with its real local PostgreSQL ledger, broker-request protobuf framing, independent socket-reader quote cache and final validation receipt. It uses synthetic prices and a well-funded fake account. Each trial starts with an empty opening ledger and a fresh trader/broker. Warm trials prepare the session before timing; the cold trial includes preparation. Injected account delay affects the first trader response; ledger delay affects the actual PostgreSQL intent insert.

Elapsed latency is measured from submitting the fresh quote through the wire to the exact final guard snapshot. Logical quote offsets after the decision mark are deliberately controlled. All measurements use an unoptimised local Rust test build, an in-memory broker and local PostgreSQL. Remote cTrader/Railway latency and concurrent production load are not represented. Three repetitions per ordinary cell are an engineering sample, not a reliable tail-latency estimate.

## Controlled results

| Scenario | Quote offset after mark | Samples | Expired / attempts | Dispatch latency min / median / max |
|---|---:|---:|---:|---:|
| normal early | 0 ms | 3 | 0/3 (0.0%) | 15.591 / 19.857 / 25.848 ms |
| normal late 4500ms | 4500 ms | 3 | 0/3 (0.0%) | 15.710 / 25.812 / 30.560 ms |
| normal late 4900ms | 4900 ms | 3 | 0/3 (0.0%) | 23.135 / 23.898 / 26.596 ms |
| normal late 4990ms | 4990 ms | 3 | 3/3 (100.0%) | 17.139 / 26.143 / 29.571 ms |
| account 250ms early | 0 ms | 3 | 0/3 (0.0%) | 267.477 / 268.090 / 268.269 ms |
| account 250ms late 4500ms | 4500 ms | 3 | 0/3 (0.0%) | 280.466 / 281.185 / 284.424 ms |
| account 250ms late 4900ms | 4900 ms | 3 | 3/3 (100.0%) | 267.407 / 269.990 / 280.409 ms |
| ledger 250ms early | 0 ms | 3 | 0/3 (0.0%) | 266.163 / 279.060 / 282.613 ms |
| ledger 250ms late 4500ms | 4500 ms | 3 | 0/3 (0.0%) | 266.222 / 276.720 / 276.834 ms |
| ledger 250ms late 4900ms | 4900 ms | 3 | 3/3 (100.0%) | 264.511 / 265.191 / 284.757 ms |
| ledger 5100ms early | 0 ms | 1 | 1/1 (100.0%) | 5139.823 / 5139.823 / 5139.823 ms |
| cold preparation early | 0 ms | 3 | 0/3 (0.0%) | 1526.872 / 1531.694 / 1547.694 ms |

An expiry emitted zero broker order frames and retained a durable `REFUSED_BEFORE_SEND` receipt. Every successful dispatch emitted exactly one frame and retained `ENTRY_VALIDATED_BEFORE_SEND`. The final receipt remains validation evidence rather than proof of a fill or positive expectancy. The overall scenario mix is intentionally artificial; combining it into a claimed production missed-entry rate would be misleading.

## S1 six-second normal-path control

The existing `a_demo_account_sends_one_entry_at_once_and_trails_it` PostgreSQL/wire-broker integration control was run three times. All three runs sent exactly one opening frame, preserved the expected trailing amendment and retained the original no-duplicate-on-restart assertions. The exact final validation receipts show selected-signal-to-guard elapsed intervals of **9–10 ms, 8–9 ms and 7–8 ms**, respectively. The one-millisecond intervals reflect the receipt's integer remaining-budget precision; they are not broker response or fill latency. Each run retained at least 5,990 ms of the six-second budget and passed in 5.36 seconds overall, including its separate exit/restart checks.

This is a full S1 normal-path control using local PostgreSQL and a synthetic in-memory broker, not a measured production rejection rate or remote latency distribution. No new ignored S1 suite was added. Reproduce each run with:

```sh
cargo test -p jiaoyi --test trading a_demo_account_sends_one_entry_at_once_and_trails_it -- --exact --nocapture
```

Private logs are `entry/s1-dispatch-benchmark-{1,2,3}.log` and `entry/s1-dispatch-benchmark-summary.json` under the same evidence directory. The summary preserves each log's SHA-256 and exact validation receipt measurements.

## Retrospective production bounds

Three independently hash-verified v6 opening examples persisted their intents at mark +3.644, +4.405 and +4.488 seconds, leaving only 1.356, 0.595 and 0.512 seconds before the proposed deadline. Broker-created timestamps were +3.774, +4.534 and +4.614 seconds. These clocks differ, and the exact transport-write timestamp was not saved. A post-result note or fill after five seconds is not proof of a late dispatch.

A broader saved-opening subset from 29 October 00:00 UTC contains six S3 openings, one with intent persistence at or after five seconds (1/6, 16.7%). Extending the selected window to 28 October 15:00 UTC gives eight openings, two already beyond five seconds (2/8, 25%). These broader rows do not independently establish the opening registration or first eligible quote time. They are conditional retrospective bounds for those selected unchanged paths, not an observed v7 miss rate, an unbiased opportunity sample or a prediction of financial impact. The reference marks are derived UTC half-hour grid points; the original recorded decision marks were unavailable.

## Activation interpretation and follow-through

The controlled results and historical bounds must be reviewed before enabling the versioned policy. A late eligible quote has only the unused portion of the original five seconds: a quote at +4.9 seconds does not receive a fresh five-second allowance. Slow account/ledger work can therefore intentionally cause additional refusals. The benchmark does not justify widening the deadline, removing the guard or claiming zero production impact.

After activation, report actual new guarded attempts separately by strategy/account: distinct successfully validated intents, expiry refusals and other refusals, with latency/budget snapshots. Count each intent once and retain the denominator; broker acknowledgements and fills are separate. This yields an observed final-guard expiry rate only for attempts reaching that guard, not for every possible market opportunity. A missed checkpoint or abnormal refusal rate needs evidence review; it must not trigger automatic policy relaxation, reversal or forced test trades.

Main-agent pre-activation review, №18,111 §§1–2, 1 November 2026, 19:26 SGT: retain the explicitly versioned five-second condition under the owner’s instruction to execute the plan. The bounded normal/delayed measurements and historical restrictions are disclosed; production expiry frequency and financial effects remain unmeasured. Required CI and exact release verification still precede closure.

## Reproduction and evidence

Run the explicit measurement fixture against the isolated local test database:

```sh
cargo test -p jiaoyi --test strategy3 dispatch_policy_benchmark -- --exact --ignored --nocapture
```

The new measurement-only test is ignored during routine CI because its machine timing is an observation, not a stable performance threshold. Existing behavioural tests for valid sends, slow ledger/writer-lock refusal, durable uncertainty and exits remain required and unchanged. No production broker order is created by this test.

- Raw benchmark log SHA-256: `18bca3c05319729a50167539b496b0dcb447df63b56c7e1d830d0c4bb7e970e6`.
- Historical timing audit SHA-256: `72217d8121c571db3cd0c1ac00e2caf75709457828f04ebaf8c3e0eabad30450`.
- Private reproducible receipts: `/workspace/jiaoyi-assessment-execution-20261101/entry/dispatch-policy-benchmark.log`, `dispatch-policy-benchmark-summary.json`, and `economics/saved-dispatch-timing-audit.json`; these contain the bounded detailed evidence and remain outside Git.

No Q, size, SL/TP, live-account restriction, ownership rule or worker permission was changed for these measurements.
