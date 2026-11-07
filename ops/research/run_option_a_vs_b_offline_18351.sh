#!/usr/bin/env bash
# №18,351: cached-only evaluation. Freeze/commit before running; publication separate.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec python3 ops/research/option_a_vs_b_offline.py "${1:-run}"
