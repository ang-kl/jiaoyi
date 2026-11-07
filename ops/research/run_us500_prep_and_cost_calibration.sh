#!/usr/bin/env bash
# №18,346: offline evidence audit and same-fill cost sensitivity; no broker reads.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
exec python3 ops/research/us500_prep_cost.py all "$@"
