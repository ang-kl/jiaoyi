#!/usr/bin/env bash
# №18,353: offline native experiment. Plan/code must be frozen before run.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec python3 ops/research/native_target_sizing.py "${1:-run}"
