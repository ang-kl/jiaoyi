#!/usr/bin/env bash
# №18,338: real bounded collection/replay; never substitutes invented results.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
exec python3 ops/research/peer_collection_eval.py all "$@"
