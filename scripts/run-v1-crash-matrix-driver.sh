#!/usr/bin/env bash
set -euo pipefail

active_candidate='8c6b48082205a3af555130eec1f3e7d2ac8811fe'
candidate_sha="${CANDIDATE_SHA:-$active_candidate}"
[[ "$candidate_sha" = "$active_candidate" ]] || {
  echo "unsupported crash-matrix candidate: $candidate_sha" >&2
  exit 3
}
export CANDIDATE_SHA="$candidate_sha"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec bash "$script_dir/run-v1-crash-matrix-driver-legacy.sh" "$@"
