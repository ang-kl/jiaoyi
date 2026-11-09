#!/usr/bin/env python3
"""Replay owner18407 on original-arm checkpoints and real local recorded quotes.

No broker/network calls. Source archives are deliberately not committed: provide
private captured CSV.gz streams plus their SHA256 receipt manifests. The native
example supplies production state/geometry; this orchestrator reconciles evidence
and makes every counterfactual execution/account assumption explicit.
"""
import argparse
import csv
import gzip
import hashlib
import json
import subprocess
from pathlib import Path

SYMBOLS = ("ETHUSD", "BTCUSD", "SpotCrude", "SpotBrent", "US500", "NAS100")
ACCOUNTS = (46130058, 47790949, 49236051)


def read_json(path):
    return json.loads(path.read_text())


def stream(root, account, symbol, reverse_ties=False):
    rows = []
    receipts = []
    for folder in (root, root / "crypto-close-boundary"):
        receipt_path = folder / f"{account}-{symbol}.receipt.json"
        if not receipt_path.exists():
            continue
        receipt = read_json(receipt_path)
        content = gzip.decompress((folder / f"{account}-{symbol}.csv.gz").read_bytes())
        if hashlib.sha256(content).hexdigest() != receipt["sha256"]:
            raise ValueError("source_hash_mismatch")
        source_rows = list(csv.reader(content.decode().splitlines()))
        if len(source_rows) != receipt["rows"]:
            raise ValueError("source_row_count_mismatch")
        rows.extend((int(r[0]), receipt["symbol_id"],
                     int(r[1]) if r[1] else None, int(r[2]) if r[2] else None)
                    for r in source_rows)
        receipts.append({k: v for k, v in receipt.items() if k != "sql"})
    unique = sorted(set(rows), key=lambda q: (q[0], q[1],
        -(q[2] or 0) if reverse_ties else (q[2] or 0),
        -(q[3] or 0) if reverse_ties else (q[3] or 0)))
    groups = {}
    for q in unique:
        groups[q[:2]] = groups.get(q[:2], 0) + 1
    if max(groups.values(), default=0) > 2:
        raise ValueError("more_than_two_distinct_same_timestamp_quotes_require_all_orderings")
    return unique, receipts


def request(root, evidence, ledger, account, pair, reverse_ties=False):
    rows, receipts = [], []
    for symbol in SYMBOLS:
        r, rs = stream(root, account, symbol, reverse_ties)
        rows.extend(r)
        receipts.extend(rs)
    rows.sort(key=lambda q: (q[0], q[1]))
    seeds = []
    for order in evidence["orders"]:
        if order["kind"] != "open" or order["account_id"] != account:
            continue
        detail = order["detail"]
        saved = next(r for r in ledger if r["entry_key"] == order["entry_key"])
        peer = saved["dispatch_guards"][-1]["peer"]
        limits = detail["volume_limits"]
        seeds.append({"symbol": order["symbol"], "symbol_id": order["symbol_id"],
                      "peer_id": peer["symbol_id"], "peer_origin": peer["origin"],
                      "arm": detail["armed"], "digits": limits["digits"],
                      "min_volume": limits["min"], "volume_step": limits["step"],
                      "max_volume": limits["max"],
                      "equity_proxy": detail["prepared_admission"]["margin"]["fx_guard"]["equity"],
                      "commission_round_trip_per_unit": detail["cost"]["commission_round_trip"],
                      "old_net_pnl": saved["net_pnl"]})
    return {"account_id": account, "through": "2026-11-09T09:30:00Z",
            "enforce_pair": pair, "seeds": seeds, "quotes": rows}, receipts


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--cache", type=Path, required=True)
    p.add_argument("--audit", type=Path, required=True)
    p.add_argument("--binary", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    evidence = read_json(args.audit / "evidence-refresh.json")
    ledger = read_json(args.audit / "trade-ledger.json")
    results, isolated, sources = [], [], []
    request_hashes = []
    for account in ACCOUNTS:
        for pair in (True, False):
            req, receipts = request(args.cache, evidence, ledger, account, pair)
            data = json.dumps(req, separators=(",", ":"), allow_nan=False)
            request_hashes.append({"account": account, "pair_enforced": pair,
                                   "sha256": hashlib.sha256(data.encode()).hexdigest(),
                                   "unique_quotes": len(req["quotes"])})
            run = subprocess.run([str(args.binary.resolve())], input=data, text=True,
                                 capture_output=True, check=True, timeout=90)
            result = json.loads(run.stdout)
            alt, _ = request(args.cache, evidence, ledger, account, pair, reverse_ties=True)
            alt_data = json.dumps(alt, separators=(",", ":"), allow_nan=False)
            if alt_data != data:
                alternative = subprocess.run([str(args.binary.resolve())], input=alt_data,
                    text=True, capture_output=True, check=True, timeout=90)
                if json.loads(alternative.stdout) != result:
                    raise ValueError("ambiguous_timestamp_order_changes_counterfactual")
            request_hashes[-1]["reverse_same_timestamp_order_checked_identical"] = True
            (results if pair else isolated).append(result)
            if pair:
                sources.extend(receipts)
    root = Path(__file__).resolve().parents[2]
    source_paths = (
        "ops/research/forensic18407_counterfactual.py",
        "crates/jiaoyi-core/examples/forensic18407_replay.rs",
        "crates/jiaoyi-core/src/strategy3/macro_pullback.rs",
        "crates/jiaoyi-core/src/strategy3/production_sessions.rs",
        "crates/jiaoyi-core/src/strategy3/targets.rs",
        "crates/jiaoyi-core/src/strategy3/mod.rs",
        "Cargo.lock",
    )
    source_hashes = {name: hashlib.sha256((root / name).read_bytes()).hexdigest()
                     for name in source_paths}
    evidence_hashes = {name: hashlib.sha256((args.audit / name).read_bytes()).hexdigest()
                       for name in ("evidence-refresh.json", "trade-ledger.json")}
    observed = []
    for order in evidence["orders"]:
        if order["kind"] != "open" or order["account_id"] not in ACCOUNTS:
            continue
        saved = next(r for r in ledger if r["entry_key"] == order["entry_key"])
        observed.append({"account_id": order["account_id"], "symbol": order["symbol"],
                         "position_id": saved["position_id"],
                         "observed_volume_cents": order["volume"],
                         "actual_entry_fill": saved["entry_fill"],
                         "actual_exit_fill": saved["exit_fill"],
                         "actual_net_pnl": saved["net_pnl"]})
    output = {
        "trace": "№18,407", "policy": "s3_macro_pullback_owner_18407_v1",
        "source_kind": "production recorded quotes; zero new broker collection requests",
        "scope": "Conditional replay of the original 15 recorded arms; no new arms after disarm, no full-day or independent backtest claim.",
        "cutoff_utc": "2026-11-09T09:30:00Z", "new_broker_requests": 0,
        "assumptions": [
            "Original arm, causal macro context and peer opening reference are taken from actual audit receipts.",
            "Running-extreme/touch-turn, deep invalidation, session eligibility and brackets use shared native production helpers.",
            "All symbols of each account share one chronological quote timeline; cross-symbol same-time ties order by numeric symbol ID, never by outcome. One distinct-price BTC peer timestamp on 47790949 is replayed in both possible within-symbol orders; all full outputs must agree or the run refuses.",
            "Pair occupancy disarms the sibling original arm; closure releases occupancy but this original-arm study does not generate another arm.",
            "Prepared geometry is available from the immediately preceding fresh quote; extensions invalidate previous geometry; five-second preparation age is enforced. Actual background I/O readiness is not reconstructed.",
            "Hypothetical entries/stops fill at observed executable bid/ask; targets fill at their limits; BE becomes effective on the next quote. No actual counterfactual broker fills or transport latency are claimed.",
            "Net sensitivity subtracts recorded round-trip commission and 3.74 bp per-side slippage reserve from executable-quote cash P&L; spread is already in bid/ask and is not charged twice.",
            "Sizing is recomputed at 0.6% using each original actual-world admission equity as a proxy, not a counterfactual equity trajectory. Broker volume limits are the captured per-account original limits.",
            "S3 3% open-risk cap and pair exclusion are simulated for the supplied cohort; other historical account/manual books, margin admission and pending broker acknowledgements are not reconstructed.",
            "No post-capture exit is invented. Open hypothetical trades are censored and total counterfactual net P&L is null until resolved.",
            "This day's data motivated the repairs; improved retrospective results cannot qualify an independent trading edge."
        ],
        "request_sha256": request_hashes, "source_receipts": sources,
        "implementation_sha256": source_hashes, "audit_source_sha256": evidence_hashes,
        "observed_actual_positions": observed,
        "native_binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(),
        "accounts": results, "diagnostic_without_pair_guard": isolated,
        "financial_pass": False,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2, ensure_ascii=False, allow_nan=False) + "\n")
    for r in results:
        print(r["account_id"], "old", r["old_net_pnl"], "new modeled", r["counterfactual_net_pnl"],
              [(t["symbol"], t["status"], t["net_cash_3_74bp"]) for t in r["trials"]])


if __name__ == "__main__":
    main()
