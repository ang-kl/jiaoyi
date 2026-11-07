#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
exec python3 ops/research/high_vol_cohorts.py all "$@"
