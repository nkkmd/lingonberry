#!/usr/bin/env bash
set -euo pipefail

active_candidate='8c6b48082205a3af555130eec1f3e7d2ac8811fe'
active_storage_sha='737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507'
active_relay_sha='23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c'

: "${CANDIDATE_SHA:?CANDIDATE_SHA required}"
: "${STORAGE_SHA256:?STORAGE_SHA256 required}"
: "${RELAY_SHA256:?RELAY_SHA256 required}"

[[ "$CANDIDATE_SHA" = "$active_candidate" ]] || {
  echo "walkthrough candidate mismatch: $CANDIDATE_SHA" >&2
  exit 2
}
[[ "$STORAGE_SHA256" = "$active_storage_sha" ]] || {
  echo "walkthrough storage digest mismatch" >&2
  exit 2
}
[[ "$RELAY_SHA256" = "$active_relay_sha" ]] || {
  echo "walkthrough relay digest mismatch" >&2
  exit 2
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec bash "$script_dir/run-v1-documentation-walkthrough-legacy.sh" "$@"
