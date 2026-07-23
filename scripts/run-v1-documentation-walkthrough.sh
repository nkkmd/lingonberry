#!/usr/bin/env bash
set -euo pipefail

candidate_root="${1:?candidate checkout path required}"
out="${2:-target/v1-documentation-walkthrough}"
expected_sha="${CANDIDATE_SHA:-f9543019f2c219aea3b085ff90f2da201b268a48}"
expected_storage_sha="${STORAGE_SHA256:-22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0}"
expected_relay_sha="${RELAY_SHA256:-9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d}"

candidate_root="$(cd "$candidate_root" && pwd)"
out="$(mkdir -p "$out" && cd "$out" && pwd)"
rm -rf "$out"/*
mkdir -p "$out"/{logs,results,manifests}

cleanup() {
  sudo systemctl stop lingonberry-relay.service lingonberry-storage-ready.service >/dev/null 2>&1 || true
  sudo systemctl disable lingonberry-relay.service lingonberry-storage-ready.service >/dev/null 2>&1 || true
  sudo rm -f /etc/systemd/system/lingonberry-relay.service /etc/systemd/system/lingonberry-storage-ready.service
  sudo systemctl daemon-reload >/dev/null 2>&1 || true
}
trap cleanup EXIT

record() {
  local id="$1" classification="$2"; shift 2
  local log="$out/logs/$id.log" result="$out/results/$id.json"
  local started status=passed exit_code=0
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  set +e
  "$@" > >(tee "$log") 2>&1
  exit_code=$?
  set -e
  [[ $exit_code -eq 0 ]] || status=failed
  python3 - "$result" "$id" "$classification" "$status" "$exit_code" "$started" <<'PY'
import json, pathlib, sys, datetime
path, ident, classification, status, code, started = sys.argv[1:]
pathlib.Path(path).write_text(json.dumps({
  "id": ident,
  "classification": classification,
  "status": status,
  "exitCode": int(code),
  "startedAt": started,
  "finishedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
  "log": f"logs/{ident}.log",
}, indent=2) + "\n")
PY
  [[ $exit_code -eq 0 ]]
}

candidate_sha="$(git -C "$candidate_root" rev-parse HEAD)"
test "$candidate_sha" = "$expected_sha"
test -z "$(git -C "$candidate_root" status --porcelain=v1)"

cat >"$out/manifests/candidate.json" <<JSON
{"candidateCommit":"$candidate_sha","expectedStorageSha256":"$expected_storage_sha","expectedRelaySha256":"$expected_relay_sha","workflowRunId":"${GITHUB_RUN_ID:-local}","startedAt":"$(date -u +%Y-%m-%dT%H:%M:%SZ)"}
JSON

record DOC-01 EXECUTED bash -c 'uname -s; uname -m; cat /etc/os-release; systemctl --version; test "$(uname -s)" = Linux; test "$(uname -m)" = x86_64; . /etc/os-release; test "$ID" = ubuntu; test "$VERSION_ID" = 24.04'

record DOC-02 EXECUTED bash -c '
  cd "$1"
  cargo build --release -p lingonberry-storage --bin lingonberry-storage -p lingonberry-relay --bin lingonberry-relay
  storage_actual="$(sha256sum target/release/lingonberry-storage | cut -d" " -f1)"
  relay_actual="$(sha256sum target/release/lingonberry-relay | cut -d" " -f1)"
  test "$storage_actual" = "$2"
  test "$relay_actual" = "$3"
  sudo install -m 0755 target/release/lingonberry-storage /usr/local/bin/lingonberry-storage
  sudo install -m 0755 target/release/lingonberry-relay /usr/local/bin/lingonberry-relay
  stat -c "%n %a %U:%G" /usr/local/bin/lingonberry-storage /usr/local/bin/lingonberry-relay
  sha256sum /usr/local/bin/lingonberry-storage /usr/local/bin/lingonberry-relay
' _ "$candidate_root" "$expected_storage_sha" "$expected_relay_sha"

record DOC-03 EXECUTED bash -c 'cd "$1"; systemd-analyze verify deploy/systemd/lingonberry-storage-ready.service deploy/systemd/lingonberry-relay.service' _ "$candidate_root"

record DOC-04 EXECUTED bash -c '
  root="$(mktemp -d)"; trap "rm -rf \"$root\"" EXIT
  /usr/local/bin/lingonberry-storage --state-dir "$root/state" --data-dir "$root/data" --backup-dir "$root/backups" --temp-dir "$root/tmp" config
  LINGONBERRY_STORAGE_DATA_DIR="$root/env-data" /usr/local/bin/lingonberry-storage --data-dir "$root/cli-data" config | tee "$root/config.json"
  grep -q "$root/cli-data" "$root/config.json"
'

record DOC-05 EXECUTED bash -c '
  root="$(mktemp -d)"; trap "rm -rf \"$root\"" EXIT
  s=(/usr/local/bin/lingonberry-storage --state-dir "$root/state" --data-dir "$root/data" --backup-dir "$root/backups" --temp-dir "$root/tmp")
  "${s[@]}" health; "${s[@]}" ready; "${s[@]}" status; "${s[@]}" doctor; "${s[@]}" verify; "${s[@]}" metrics
'

record DOC-06 EXECUTED bash -c '
  cd "$1"
  sudo useradd --system --home /var/lib/lingonberry --shell /usr/sbin/nologin lingonberry 2>/dev/null || true
  sudo install -d -o lingonberry -g lingonberry /var/lib/lingonberry/storage/data /var/lib/lingonberry/storage/tmp /var/backups/lingonberry /etc/lingonberry
  sudo install -m 0640 deploy/systemd/storage.env.example /etc/lingonberry/storage.env
  sudo install -m 0640 deploy/systemd/relay.env.example /etc/lingonberry/relay.env
  sudo chown root:lingonberry /etc/lingonberry/*.env
  sudo install -m 0644 deploy/systemd/lingonberry-storage-ready.service /etc/systemd/system/
  sudo install -m 0644 deploy/systemd/lingonberry-relay.service /etc/systemd/system/
  sudo systemctl daemon-reload
  sudo systemctl enable --now lingonberry-storage-ready.service
  sudo systemctl enable --now lingonberry-relay.service
  systemctl is-active lingonberry-storage-ready.service
  systemctl is-active lingonberry-relay.service
  curl --retry 10 --retry-delay 1 --fail --silent --show-error http://127.0.0.1:8787/v1/ready
  sudo systemctl restart lingonberry-relay.service
  curl --retry 10 --retry-delay 1 --fail --silent --show-error http://127.0.0.1:8787/v1/ready
  journalctl -u lingonberry-storage-ready.service -u lingonberry-relay.service --no-pager --since "5 minutes ago"
' _ "$candidate_root"

record DOC-07 EXECUTED bash -c '
  cd "$1"
  LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage/data /usr/local/bin/lingonberry-relay publish fixtures/http-publish-request/minimal-request.json
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage list > /tmp/lingonberry-list-before.json
  sudo systemctl restart lingonberry-relay.service
  curl --retry 10 --retry-delay 1 --fail --silent http://127.0.0.1:8787/v1/ready
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage list > /tmp/lingonberry-list-after.json
  cmp /tmp/lingonberry-list-before.json /tmp/lingonberry-list-after.json
' _ "$candidate_root"

record DOC-08 CROSS_REFERENCED bash -c 'cd "$1"; cargo test -p lingonberry-protocol; cargo test -p lingonberry-validation; node conformance/run.mjs' _ "$candidate_root"

record DOC-09 EXECUTED bash -c '
  sudo rm -rf /var/backups/lingonberry/manual-backup
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage backup create /var/backups/lingonberry/manual-backup
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage backup verify /var/backups/lingonberry/manual-backup
  find /var/backups/lingonberry/manual-backup -type f -print0 | sort -z | xargs -0 sha256sum
'

record DOC-10 EXECUTED bash -c '
  sudo rm -rf /var/lib/lingonberry/restore-candidate
  sudo install -d -o lingonberry -g lingonberry /var/lib/lingonberry/restore-candidate
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage restore plan /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage restore apply /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage drill restore /var/backups/lingonberry/manual-backup
'

record DOC-11 EXECUTED bash -c '
  root="$(mktemp -d)"; trap "rm -rf \"$root\"" EXIT
  s=(/usr/local/bin/lingonberry-storage --state-dir "$root/state" --data-dir "$root/data" --backup-dir "$root/backups" --temp-dir "$root/tmp")
  mkdir -p "$root/non-empty"; echo sentinel > "$root/non-empty/sentinel"
  if "${s[@]}" restore apply /var/backups/lingonberry/manual-backup "$root/non-empty"; then exit 1; fi
  grep -q sentinel "$root/non-empty/sentinel"
  mkdir -p "$root/real"; ln -s "$root/real" "$root/link"
  if "${s[@]}" restore apply /var/backups/lingonberry/manual-backup "$root/link"; then exit 1; fi
  mkdir -p "$root/partial"; echo "{}" > "$root/partial/manifest.json"
  if "${s[@]}" restore apply "$root/partial" "$root/partial-target"; then exit 1; fi
  test ! -e "$root/partial-target" || test -z "$(find "$root/partial-target" -mindepth 1 -print -quit)"
'

record DOC-12 EXECUTED bash -c '
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index verify
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index rebuild
  sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index verify
'

record DOC-13 CROSS_REFERENCED bash -c 'cd "$1"; cargo test -p lingonberry-storage' _ "$candidate_root"
record DOC-14 CROSS_REFERENCED bash -c 'cd "$1"; cargo test -p lingonberry-core quarantine' _ "$candidate_root"
record DOC-15 CROSS_REFERENCED bash -c 'cd "$1"; cargo test -p lingonberry-core --test quarantine_replacement_crash_matrix' _ "$candidate_root"
record DOC-16 CROSS_REFERENCED bash -c 'cd "$1"; cargo test --workspace' _ "$candidate_root"

python3 - "$out" "$candidate_sha" <<'PY'
import json, pathlib, sys, datetime
out = pathlib.Path(sys.argv[1])
results = [json.loads(p.read_text()) for p in sorted((out / "results").glob("*.json"))]
summary = {
  "schemaVersion": 1,
  "candidateCommit": sys.argv[2],
  "finishedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
  "status": "passed" if len(results) == 16 and all(r["status"] == "passed" for r in results) else "failed",
  "procedures": results,
}
(out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
PY
(
  cd "$out"
  find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
  sha256sum -c SHA256SUMS
)
echo "v1.0 documentation walkthrough passed for $candidate_sha"
