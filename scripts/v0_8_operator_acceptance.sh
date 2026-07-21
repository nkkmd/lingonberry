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
"${storage[@]}" restore apply "$root/archive" "$root/restored"
"${storage[@]}" index verify
"${storage[@]}" index rebuild
"${storage[@]}" drill restore "$root/archive"

test -f deploy/systemd/lingonberry-storage-ready.service
test -f deploy/systemd/lingonberry-relay.service
grep -q '^Type=oneshot$' deploy/systemd/lingonberry-storage-ready.service
grep -q '^Type=simple$' deploy/systemd/lingonberry-relay.service

echo 'v0.8 operator acceptance passed'
