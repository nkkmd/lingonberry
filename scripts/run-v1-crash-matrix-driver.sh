#!/usr/bin/env bash
set -Eeuo pipefail

candidate_sha="${CANDIDATE_SHA:-f9543019f2c219aea3b085ff90f2da201b268a48}"
repo_root="${CRASH_MATRIX_REPO_ROOT:-$(pwd)}"
out="${CRASH_MATRIX_OUT:-target/v1-crash-matrix-driver}"
cycles="${CRASH_MATRIX_CYCLES:-1}"
expected_tests=(
  injected_commit_transition_failure_resumes_without_second_switch
  injected_journal_failures_leave_a_retryable_empty_workspace
  injected_rollback_pointer_restore_failure_keeps_target_until_retry
  injected_rolled_back_transition_failure_retries_after_pointer_restore
)

[[ "$cycles" =~ ^[1-9][0-9]*$ ]] || { echo "CRASH_MATRIX_CYCLES must be a positive integer" >&2; exit 2; }
cd "$repo_root"
[[ "$(git rev-parse HEAD)" = "$candidate_sha" ]] || { echo "candidate mismatch" >&2; exit 3; }
[[ -z "$(git status --porcelain=v1)" ]] || { echo "candidate checkout is dirty" >&2; exit 3; }
[[ ! -e "$out" ]] || { echo "output already exists; stopped runs are not resumable" >&2; exit 3; }
mkdir -p "$out"/{logs,cycles,manifests}
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo_json="$out/manifests/cargo-build.jsonl"
cargo test -p lingonberry-core --test quarantine_replacement_crash_matrix --no-run --message-format=json >"$cargo_json"
test_bin="$(python3 - "$cargo_json" <<'PY'
import json, pathlib, sys
matches=[]
for line in pathlib.Path(sys.argv[1]).read_text().splitlines():
    try: obj=json.loads(line)
    except json.JSONDecodeError: continue
    profile=obj.get('profile') or {}
    target=obj.get('target') or {}
    exe=obj.get('executable')
    if exe and target.get('name') == 'quarantine_replacement_crash_matrix' and profile.get('test'):
        matches.append(exe)
if len(matches) != 1:
    raise SystemExit(f'expected one crash-matrix test binary, found {matches!r}')
print(matches[0])
PY
)"
[[ -x "$test_bin" ]] || { echo "test binary is not executable: $test_bin" >&2; exit 4; }

test_bin_sha="$(sha256sum "$test_bin" | awk '{print $1}')"
lock_sha="$(sha256sum Cargo.lock | awk '{print $1}')"
"$test_bin" --list --format terse >"$out/manifests/test-list.txt"
python3 - "$out/manifests/test-list.txt" "${expected_tests[@]}" <<'PY'
import pathlib, sys
path=pathlib.Path(sys.argv[1])
expected=sorted(sys.argv[2:])
actual=[]
for line in path.read_text().splitlines():
    name=line.split(':',1)[0].strip()
    if name: actual.append(name)
actual=sorted(actual)
if actual != expected:
    raise SystemExit(f'test enumeration mismatch: expected={expected!r} actual={actual!r}')
PY

cat >"$out/manifests/identity.json" <<JSON
{
  "candidateCommit": "$candidate_sha",
  "cargoLockSha256": "$lock_sha",
  "testBinary": "$test_bin",
  "testBinarySha256": "$test_bin_sha",
  "cyclesRequired": $cycles,
  "testsPerCycle": 4,
  "rustc": "$(rustc --version)",
  "cargo": "$(cargo --version)",
  "startedAt": "$started_at"
}
JSON

status="passed"
completed=0
for ((cycle=1; cycle<=cycles; cycle++)); do
  cycle_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  log="$out/logs/cycle-$cycle.log"
  set +e
  "$test_bin" --test-threads=1 --nocapture >"$log" 2>&1
  code=$?
  set -e
  cycle_finished="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  python3 - "$out/cycles/cycle-$cycle.json" "$cycle" "$code" "$cycle_started" "$cycle_finished" "$test_bin_sha" <<'PY'
import json, pathlib, sys
path, cycle, code, started, finished, digest = sys.argv[1:]
pathlib.Path(path).write_text(json.dumps({
  'cycle': int(cycle), 'status': 'passed' if int(code)==0 else 'failed',
  'exitCode': int(code), 'startedAt': started, 'finishedAt': finished,
  'testBinarySha256': digest, 'log': f'logs/cycle-{cycle}.log'
}, indent=2)+'\n')
PY
  if [[ $code -ne 0 ]]; then status="failed"; break; fi
  [[ "$(sha256sum "$test_bin" | awk '{print $1}')" = "$test_bin_sha" ]] || { status="failed"; break; }
  completed=$cycle
done

finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
python3 - "$out" "$candidate_sha" "$status" "$cycles" "$completed" "$test_bin_sha" "$started_at" "$finished_at" <<'PY'
import json, pathlib, sys
out=pathlib.Path(sys.argv[1])
candidate,status,required,completed,digest,started,finished=sys.argv[2:]
cycles=[json.loads(p.read_text()) for p in sorted((out/'cycles').glob('*.json'))]
summary={
  'schemaVersion':1, 'candidateCommit':candidate, 'status':status,
  'qualification':False, 'qualifyingPass':False,
  'cyclesRequired':int(required), 'cyclesCompleted':int(completed),
  'testsPerCycle':4, 'testBinarySha256':digest,
  'startedAt':started, 'finishedAt':finished, 'cycles':cycles
}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
PY
(cd "$out" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum >SHA256SUMS)
[[ "$status" = passed && "$completed" -eq "$cycles" ]]
