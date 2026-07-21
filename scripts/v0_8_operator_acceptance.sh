#!/usr/bin/env bash
set -euo pipefail

root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

storage_bin="${LINGONBERRY_STORAGE_BIN:-/usr/local/bin/lingonberry-storage}"
relay_bin="${LINGONBERRY_RELAY_BIN:-/usr/local/bin/lingonberry-relay}"

test -x "$storage_bin"
test -x "$relay_bin"

storage() {
  "$storage_bin" \
    --state-dir "$root/state" \
    --data-dir "$root/data" \
    --backup-dir "$root/backups" \
    --temp-dir "$root/tmp" \
    "$@"
}

storage config
storage health
storage status
storage doctor
storage metrics

LINGONBERRY_STATE_DIR="$root/data" \
  "$relay_bin" publish fixtures/http-publish-request/minimal-request.json

# Each invocation is a new process. Identical output proves that persisted state
# survives the operator-process restart boundary.
storage list >"$root/list-before-restart.json"
storage ready
storage status
storage list >"$root/list-after-restart.json"
cmp "$root/list-before-restart.json" "$root/list-after-restart.json"

storage backup create "$root/archive"
storage backup verify "$root/archive"
storage restore plan "$root/archive" "$root/restored"
restore_output="$(storage restore apply "$root/archive" "$root/restored")"
grep -q '"status":"restored"' <<<"$restore_output"
grep -q '"readVerified":true' <<<"$restore_output"
storage index verify
storage index rebuild

drill_output="$(storage drill restore "$root/archive")"
grep -q '"status":"passed"' <<<"$drill_output"
grep -q '"readVerified":true' <<<"$drill_output"
grep -q '"writeVerified":true' <<<"$drill_output"
grep -q '"cleanupVerified":true' <<<"$drill_output"

if find "$root/tmp" -mindepth 1 -print -quit 2>/dev/null | grep -q .; then
  echo 'isolated restore left temporary state behind' >&2
  exit 1
fi

# A partial archive must fail closed and must not create the requested target.
mkdir -p "$root/partial-archive"
printf '{}\n' >"$root/partial-archive/manifest.json"
if storage restore apply "$root/partial-archive" "$root/partial-target"; then
  echo 'partial archive unexpectedly restored' >&2
  exit 1
fi
if [[ -e "$root/partial-target" ]] && find "$root/partial-target" -mindepth 1 -print -quit | grep -q .; then
  echo 'failed restore left partial target state behind' >&2
  exit 1
fi

# Active and non-empty targets remain protected.
if storage restore apply "$root/archive" "$root/data"; then
  echo 'active data directory unexpectedly accepted as restore target' >&2
  exit 1
fi
mkdir -p "$root/non-empty-target"
printf 'sentinel\n' >"$root/non-empty-target/sentinel"
if storage restore apply "$root/archive" "$root/non-empty-target"; then
  echo 'non-empty restore target unexpectedly accepted' >&2
  exit 1
fi
grep -q '^sentinel$' "$root/non-empty-target/sentinel"

test -f deploy/systemd/lingonberry-storage-ready.service
test -f deploy/systemd/lingonberry-relay.service
grep -q '^Type=oneshot$' deploy/systemd/lingonberry-storage-ready.service
grep -q '^Type=simple$' deploy/systemd/lingonberry-relay.service

echo 'v0.8 operator acceptance passed'
