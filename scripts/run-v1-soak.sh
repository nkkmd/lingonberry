#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

profile="${SOAK_PROFILE:-rehearsal}"
out="${SOAK_OUT:-target/v1-soak-$profile}"
candidate_sha="${CANDIDATE_SHA:-f9543019f2c219aea3b085ff90f2da201b268a48}"
expected_storage_sha="${STORAGE_SHA256:-22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0}"
expected_relay_sha="${RELAY_SHA256:-9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d}"
force_failure="${SOAK_FORCE_FAILURE:-0}"

case "$profile" in
  rehearsal)
    min_seconds=5; publishes=5; retrieves=5; queries=3; graceful=1; abrupt=1
    verifies=1; rebuilds=1; backups=1; restores=1; crash_cycles=1
    malformed=3; oversized=1; nested=1; disk_pressure=1; telemetry_interval=1
    qualification=false
    ;;
  formal)
    echo 'formal soak execution is intentionally disabled in this rehearsal harness' >&2
    echo 'enable only after the dedicated-host systemd scheduler, distributed cadence, and live threshold enforcement are implemented and reviewed' >&2
    exit 2
    ;;
  *) echo "unknown SOAK_PROFILE: $profile" >&2; exit 2 ;;
esac

rm -rf "$out"
mkdir -p "$out"/{logs,results,manifests,telemetry,evidence}
root="$(mktemp -d)"
relay_pid=""
telemetry_pid=""
started_epoch="$(date +%s)"
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
status=passed
stop_reason=""

cleanup() {
  [[ -z "$telemetry_pid" ]] || kill "$telemetry_pid" 2>/dev/null || true
  [[ -z "$relay_pid" ]] || kill "$relay_pid" 2>/dev/null || true
  rm -rf "$root"
}
trap cleanup EXIT
trap 'status=failed; stop_reason="unexpected command failure at line $LINENO"; finalize; exit 1' ERR

storage_bin="$repo_root/target/release/lingonberry-storage"
relay_bin="$repo_root/target/release/lingonberry-relay"
storage() {
  "$storage_bin" --state-dir "$root/state" --data-dir "$root/data" \
    --backup-dir "$root/backups" --temp-dir "$root/tmp" "$@"
}

record_event() {
  printf '{"timestamp":"%s","phase":"%s","event":"%s","result":"%s"}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$1" "$2" "$3" >>"$out/timeline.jsonl"
}

run_scenario() {
  local name="$1"; shift
  local log="$out/logs/$name.log" result="$out/results/$name.json"
  local began code=0 scenario_status=passed
  began="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  record_event scenario "$name" started
  set +e
  "$@" > >(tee "$log") 2>&1
  code=$?
  set -e
  [[ $code -eq 0 ]] || scenario_status=failed
  python3 - "$result" "$name" "$scenario_status" "$code" "$began" <<'PY'
import datetime, json, pathlib, sys
path, name, status, code, began = sys.argv[1:]
pathlib.Path(path).write_text(json.dumps({
  "name": name, "status": status, "exitCode": int(code), "startedAt": began,
  "finishedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
  "log": f"logs/{name}.log"
}, indent=2) + "\n")
PY
  record_event scenario "$name" "$scenario_status"
  [[ $code -eq 0 ]]
}

telemetry_loop() {
  while true; do
    local pid rss=0 vsz=0 fds=0 cpu="0.0" disk_avail inode_avail mem_avail swap_used
    pid="$relay_pid"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      read -r rss vsz cpu < <(ps -o rss=,vsz=,%cpu= -p "$pid" | awk '{print $1,$2,$3}')
      fds="$(find "/proc/$pid/fd" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l)"
    fi
    disk_avail="$(df -B1 --output=avail "$root" | tail -1 | tr -d ' ')"
    inode_avail="$(df -Pi --output=iavail "$root" | tail -1 | tr -d ' ')"
    mem_avail="$(awk '/MemAvailable/ {print $2*1024}' /proc/meminfo)"
    swap_used="$(awk '/SwapTotal/ {t=$2} /SwapFree/ {f=$2} END {print (t-f)*1024}' /proc/meminfo)"
    printf '{"timestamp":"%s","phase":"running","pid":%s,"rssKiB":%s,"vszKiB":%s,"cpuPercent":"%s","fileDescriptors":%s,"memoryAvailableBytes":%.0f,"swapUsedBytes":%.0f,"diskAvailableBytes":%s,"inodesAvailable":%s,"dataBytes":%s,"backupBytes":%s,"tempBytes":%s}\n' \
      "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${pid:-0}" "${rss:-0}" "${vsz:-0}" "${cpu:-0.0}" "${fds:-0}" \
      "$mem_avail" "$swap_used" "$disk_avail" "$inode_avail" \
      "$(du -sb "$root/data" 2>/dev/null | awk '{print $1+0}')" \
      "$(du -sb "$root/backups" 2>/dev/null | awk '{print $1+0}')" \
      "$(du -sb "$root/tmp" 2>/dev/null | awk '{print $1+0}')" >>"$out/telemetry/metrics.jsonl"
    sleep "$telemetry_interval"
  done
}

finalize() {
  trap - ERR
  local ended_epoch elapsed
  ended_epoch="$(date +%s)"; elapsed=$((ended_epoch-started_epoch))
  python3 - "$out" "$profile" "$candidate_sha" "$qualification" "$status" "$stop_reason" "$started_at" "$elapsed" <<'PY'
import datetime, json, pathlib, sys
out = pathlib.Path(sys.argv[1])
profile, candidate, qualification, status, reason, started, elapsed = sys.argv[2:]
results = [json.loads(p.read_text()) for p in sorted((out/'results').glob('*.json'))]
if any(r['status'] != 'passed' for r in results): status = 'failed'
summary = {
  'schemaVersion': 1, 'profile': profile, 'qualification': qualification == 'true',
  'candidateCommit': candidate, 'startedAt': started,
  'finishedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
  'continuousSeconds': int(elapsed), 'status': status, 'stopReason': reason or None,
  'scenarios': results,
}
(out/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
PY
  (cd "$out" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum >SHA256SUMS)
}

actual_sha="$(git rev-parse HEAD)"
[[ "$actual_sha" = "$candidate_sha" ]] || { echo "candidate mismatch: $actual_sha" >&2; exit 3; }
[[ -z "$(git status --porcelain=v1)" ]] || { echo "dirty checkout" >&2; exit 3; }

cat >"$out/manifests/run.json" <<JSON
{"profile":"$profile","qualification":$qualification,"candidateCommit":"$candidate_sha","startedAt":"$started_at","minimumContinuousSeconds":$min_seconds,"telemetryIntervalSeconds":$telemetry_interval,"forceFailure":$force_failure}
JSON
cat >"$out/manifests/workload-minima.json" <<JSON
{"publishes":$publishes,"retrieves":$retrieves,"queries":$queries,"gracefulRestarts":$graceful,"abruptTerminations":$abrupt,"verifications":$verifies,"rebuilds":$rebuilds,"backups":$backups,"restores":$restores,"crashMatrixCycles":$crash_cycles,"malformed":$malformed,"oversized":$oversized,"nested":$nested,"diskPressure":$disk_pressure}
JSON
cat >"$out/manifests/thresholds.json" <<JSON
{"minimumFreeDiskBytes":1073741824,"minimumFreeInodes":10000,"maximumFileDescriptors":4096,"maximumReadinessFailureSeconds":60,"maximumUnexpectedRestarts":0}
JSON
{
  uname -a; cat /etc/os-release; systemctl --version || true; lscpu; free -b; swapon --show; ulimit -a
  df -Th; df -Ti; git show --no-patch --format=fuller HEAD; sha256sum Cargo.lock
} >"$out/manifests/environment.txt"

run_scenario build bash -c 'cargo build --release -p lingonberry-storage --bin lingonberry-storage -p lingonberry-relay --bin lingonberry-relay'
[[ "$(sha256sum "$storage_bin" | awk '{print $1}')" = "$expected_storage_sha" ]]
[[ "$(sha256sum "$relay_bin" | awk '{print $1}')" = "$expected_relay_sha" ]]
printf '%s  %s\n%s  %s\n' "$expected_storage_sha" "$storage_bin" "$expected_relay_sha" "$relay_bin" >"$out/manifests/binary-sha256.txt"

mkdir -p "$root"/{state,data,backups,tmp}
LINGONBERRY_STATE_DIR="$root/data" "$relay_bin" serve-http 127.0.0.1:18787 >"$out/logs/relay.log" 2>&1 & relay_pid=$!
telemetry_loop & telemetry_pid=$!
sleep 1

run_scenario baseline bash -c '"$1" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" health; "$1" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" ready; curl -fsS http://127.0.0.1:18787/v1/ready' _ "$storage_bin" "$root"
run_scenario publish-retrieve-query bash -c 'for ((i=0;i<$1;i++)); do LINGONBERRY_STATE_DIR="$2/data" "$3" publish fixtures/http-publish-request/minimal-request.json >/dev/null; done; for ((i=0;i<$4;i++)); do "$5" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" list >/dev/null; done; for ((i=0;i<$6;i++)); do "$5" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" status >/dev/null; done' _ "$publishes" "$root" "$relay_bin" "$retrieves" "$storage_bin" "$queries"
run_scenario graceful-restarts bash -c 'for ((i=0;i<$1;i++)); do kill -TERM "$2"; wait "$2" || true; LINGONBERRY_STATE_DIR="$3/data" "$4" serve-http 127.0.0.1:18787 >/dev/null 2>&1 & p=$!; sleep 1; curl -fsS http://127.0.0.1:18787/v1/ready >/dev/null; kill -TERM "$p"; wait "$p" || true; done' _ "$graceful" "$relay_pid" "$root" "$relay_bin"
LINGONBERRY_STATE_DIR="$root/data" "$relay_bin" serve-http 127.0.0.1:18787 >>"$out/logs/relay.log" 2>&1 & relay_pid=$!; sleep 1
run_scenario abrupt-termination bash -c 'for ((i=0;i<$1;i++)); do LINGONBERRY_STATE_DIR="$2/data" "$3" serve-http 127.0.0.1:18788 >/dev/null 2>&1 & p=$!; sleep 1; kill -KILL "$p"; wait "$p" || true; "$4" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" doctor >/dev/null; done' _ "$abrupt" "$root" "$relay_bin" "$storage_bin"
run_scenario verify-rebuild bash -c 'for ((i=0;i<$1;i++)); do "$2" --state-dir "$3/state" --data-dir "$3/data" --backup-dir "$3/backups" --temp-dir "$3/tmp" verify; "$2" --state-dir "$3/state" --data-dir "$3/data" --backup-dir "$3/backups" --temp-dir "$3/tmp" index verify; done; for ((i=0;i<$4;i++)); do "$2" --state-dir "$3/state" --data-dir "$3/data" --backup-dir "$3/backups" --temp-dir "$3/tmp" index rebuild; done' _ "$verifies" "$storage_bin" "$root" "$rebuilds"
run_scenario backup-restore bash -c 'for ((i=0;i<$1;i++)); do a="$2/backups/archive-$i"; "$3" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" backup create "$a"; "$3" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" backup verify "$a"; done; for ((i=0;i<$4;i++)); do t="$2/restore-$i"; "$3" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" restore plan "$2/backups/archive-0" "$t"; "$3" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" restore apply "$2/backups/archive-0" "$t"; done' _ "$backups" "$root" "$storage_bin" "$restores"
run_scenario crash-matrix bash -c 'for ((i=0;i<$1;i++)); do cargo test -p lingonberry-core --test quarantine_replacement_crash_matrix; done' _ "$crash_cycles"
run_scenario invalid-boundaries bash -c 'for ((i=0;i<$1;i++)); do printf "{" | "$2" publish /dev/stdin >/dev/null 2>&1 && exit 1 || true; done; for ((i=0;i<$3;i++)); do python3 -c "print(\"x\"*(2*1024*1024))" | "$2" publish /dev/stdin >/dev/null 2>&1 && exit 1 || true; done; for ((i=0;i<$4;i++)); do python3 -c "print(\"[\"*300+\"0\"+\"]\"*300)" | "$2" publish /dev/stdin >/dev/null 2>&1 && exit 1 || true; done' _ "$malformed" "$relay_bin" "$oversized" "$nested"
run_scenario disk-pressure bash -c 'for ((i=0;i<$1;i++)); do f="$2/pressure-$i"; truncate -s 16M "$f"; "$3" --state-dir "$2/state" --data-dir "$2/data" --backup-dir "$2/backups" --temp-dir "$2/tmp" health >/dev/null; rm -f "$f"; done' _ "$disk_pressure" "$root" "$storage_bin"

if [[ "$force_failure" = 1 ]]; then
  status=failed; stop_reason="forced rehearsal failure"; record_event stop forced-failure failed
fi

while (( $(date +%s) - started_epoch < min_seconds )); do sleep 1; done
kill "$telemetry_pid" 2>/dev/null || true; wait "$telemetry_pid" 2>/dev/null || true; telemetry_pid=""

if grep -Eiq 'panic|fatal runtime error|out of memory|oom-kill' "$out"/logs/*.log; then
  status=failed; stop_reason="release-blocking log signature"
fi

finalize
[[ "$status" = passed ]]
echo "v1 soak $profile completed; qualification=$qualification candidate=$candidate_sha"
