# Compiled human-trade protection — №18,403

№18,403 §§1–6 · 09-11-2026 15:34:47 SGT · by <Codex (GPT-6), effort: not exposed>.

## TLDR

The three owner laws are enforced in S3 and the shared broker writer, with an exact pre-change source archive. S3 yields account-by-account and symbol-by-symbol; current label and historical provenance must both authorise management. This is a safety repair, not evidence of improved trading profitability.

## §1. Before-change archive

[119 source files](../archive/strategies_rust_backup_2026_11_09_18403/manifest.json), 2,421,318 bytes, copied before edits from `1d279e8980d1c31b6ebebd496a644bea7f755510`. SHA256 manifest `90d9a35686f169ffb2af4a85f898f9385353b45ba8f7b4db675263c079c66867`. All archive bytes match both their manifest and the original Git blobs. The `.rs.bak` files are outside Cargo's active source tree.

## §2. The three laws and enforcement points

| Law | Enforcement | Result |
|---|---|---|
| Exact `jiaoyi-s3` label | S3 reconciliation, S3-specific ledger provenance, fresh broker-position validation in shared amend/close writer | A missing, blank, changed or different label prevents S3 amendments and closes, even if an old S3 fill exists. A label alone is insufficient without opening evidence. |
| Yield to external position or pending order | Account-local reconcile book, entry/prepare/arm and both dispatch paths, fresh broker admission and existing final revision guard | S1 and every other non-S3 label block S3 on that symbol. Cached/durable arms are cancelled, including after restart. When the external book clears, normal new-signal eligibility returns without cancelling the human order. |
| NatGas/JPN225 reserved | Compiled `HUMAN_RESERVED_SYMBOLS`, entry/prepare gates and owned-position exit paths | S3 never arms, opens, amends or closes these symbols. Case-insensitive symbol-name matching. |

An existing S3-owned position retains its registered exits if a human opens another position on the same symbol; the new-entry lock never grants authority over the human position. Permanent NatGas/JPN225 reservation remains the explicit exception to S3 management.

A changed account revision invalidates admission immediately. The actor's existing one-second timer refreshes the ownership book using a lightweight broker reconcile, without repeating full history recovery. Unknown or stale ownership blocks new entries. Broker/network latency can delay completion; one-second scheduling is not an atomic broker execution guarantee. Existing writer revision checks still refuse stale prepared admission at the socket boundary.

Historical market-data observation and read-only human-position assessment remain permitted. Background order admission is blocked for locked symbols. The connected-account allowlist, owner pauses, margin floors, sizing, quote freshness and live one-program-position rule remain required. These laws have no environment-variable opt-out. They govern Strategy 3; they do not rewrite Strategy 1's trading algorithm.

## §3. Focused verification

- Archive regression verifies all file paths, byte counts and SHA256 values; separate comparison verifies all base Git blobs.
- S3 suite: 80 passed, one existing manual benchmark ignored before the final durable-arm restart hardening; the strengthened restart/cancellation regression passes separately. Full exact-head CI repeats the full suite.
- cTrader suite passes, including 49 writer-gate tests. New tests cover external pending orders, automatic release without cancellation, S1 exclusion and relabelled-position amendment/close refusal.
- CI also exposed an old quantity-authority fixture with no current label. It now explicitly requires rejection without a label, then checks the original quantity/side cases with the exact S3 label.
- Existing quantity, risk, margin, uncertain transmission, late revision, owned exit and partial-fill tests remain enabled. No test or lint gate was disabled.
- Reconnect fixtures now obtain a broker ownership snapshot before entry, as the production actor does. The pre-open test uses its simulated first-slot reconnect time instead of the real wall clock.

## §4. Before-release production observations

The existing trading deployment remained `806ccc1d-4e9f-41d6-8b20-d36e2c7d98de`, commit `6aa5100a5055ec786e16fc05f8069113b00d7eff`, during local implementation. At 15:28 SGT all four accounts were connected and unpaused. The 15:31 owner inventory showed live human JPN225 and NatGas positions/pending entries with no S3 attribution; no S3-managed reserved-symbol position was observed. This observation is time-limited, not a future book guarantee.

## §5. Release verification

Merge only after `check`, `docker` and `engineering` pass on the exact PR head. Verify the merged SHA on one active trading deployment, then take two fresh health reads: account connection, unpaused actors, advancing quote/exit processing, core opening references and the 07:00 UTC / 15:00 SGT London session. The final deployment receipt and actual opening-reference counts will be recorded on the PR and in the owner JSON reply. This pre-release report does not claim deployment success. No manual test trade is required.

## §6. Goals and limits

The invariant remains 73% net wins (at least 15/20 fully closed positions) and 1:1.38 risk:reward, separately per strategy/account. No financial pass or win-rate improvement is claimed. Shared revision checks protect against observed broker changes; a client cannot make a simultaneous external broker action and its own write globally atomic. An already transmitted order cannot be recalled by a later label check. Read-only monitoring and Telegram policy remain unchanged.
