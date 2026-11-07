#!/usr/bin/env bash
# №18,342: ten real peer jobs; existing frozen models; no invented outcomes.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
exec python3 ops/research/jan2024_peer.py all "$@"
