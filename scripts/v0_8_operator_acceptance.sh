#!/usr/bin/env bash
set -euo pipefail

root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

storage=(cargo run -q -p lingonberry-storage --bin lingonberry-storage --
  --state-dir "$root/state"
  --data-dir "$root/data"
  --backup-dir "$root/backups"
  --temp-dir "$root/tmp")

"${storage[@]}" config
"${storage[@]}" health
"${storage[@]}" status
"${storage[@]}" doctor
"${storage[@]}" metrics

LINGONBERRY_STATE_DIR="$root/data" \
  cargo run -q -p lingonberry-relay --bin lingonberry-relay -- \
  publish fixtures/http-publish-request/minimal-request.json

"${storage[@]}" list
"${storage[@]}" backup create "$root/archive"
"${storage[@]}" backup verify "$root/archive"
"${storage[@]}" restore plan "$root/archive" "$root/restored"
restore_output="$("${storage[@]}" restore apply "$root/archive" "$root/restored")"
grep -q '"status":"restored"' <<<"$restore_output"
grep -q '"readVerified":true' <<<"$restore_output"
"${storage[@]}" index verify
"${storage[@]}" index rebuild

drill_output="$("${storage[@]}" drill restore "$root/archive")"
grep -q '"status":"passed"' <<<"$drill_output"
grep -q '"readVerified":true' <<<"$drill_output"
grep -q '"writeVerified":true' <<<"$drill_output"
grep -q '"cleanupVerified":true' <<<"$drill_output"

if find "$root/tmp" -mindepth 1 -print -quit 2>/dev/null | grep -q .; then
  echo 'isolated restore left temporary state behind' >&2
  exit 1
fi

test -f deploy/systemd/lingonberry-storage-ready.service
test -f deploy/systemd/lingonberry-relay.service
grep -q '^Type=oneshot$' deploy/systemd/lingonberry-storage-ready.service
grep -q '^Type=simple$' deploy/systemd/lingonberry-relay.service

echo 'v0.8 operator acceptance passed'
