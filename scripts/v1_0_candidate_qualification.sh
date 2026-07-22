#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
out="${1:-target/v1-qualification}"
rm -rf "$out"
mkdir -p "$out"/{logs,results,binaries,manifests}

candidate_sha="$(git rev-parse HEAD)"
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

run_gate() {
  local name="$1"; shift
  local log="$out/logs/$name.log"
  local result="$out/results/$name.json"
  local gate_started status=passed exit_code=0
  gate_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  set +e
  "$@" > >(tee "$log") 2>&1
  exit_code=$?
  set -e
  [[ $exit_code -eq 0 ]] || status=failed
  printf '{"name":"%s","status":"%s","exitCode":%d,"startedAt":"%s","finishedAt":"%s","log":"logs/%s.log"}\n' \
    "$name" "$status" "$exit_code" "$gate_started" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$name" >"$result"
  [[ $exit_code -eq 0 ]]
}

cat >"$out/manifests/candidate.json" <<JSON
{"candidateCommit":"$candidate_sha","repository":"${GITHUB_REPOSITORY:-local}","workflowRunId":"${GITHUB_RUN_ID:-local}","workflowRunAttempt":"${GITHUB_RUN_ATTEMPT:-local}","startedAt":"$started_at"}
JSON

git status --porcelain=v1 >"$out/manifests/git-status.txt"
test ! -s "$out/manifests/git-status.txt" || { echo 'qualification requires a clean checkout' >&2; exit 2; }
git show --no-patch --format=fuller HEAD >"$out/manifests/commit.txt"
rustc -Vv >"$out/manifests/rust-toolchain.txt"
cargo -V >>"$out/manifests/rust-toolchain.txt"
node --version >"$out/manifests/node-version.txt"
uname -a >"$out/manifests/platform.txt"
[[ ! -f /etc/os-release ]] || cat /etc/os-release >>"$out/manifests/platform.txt"

run_gate rust-format cargo fmt --all -- --check
run_gate rust-clippy-libs cargo clippy --workspace --lib -- -D warnings
run_gate rust-clippy-bins cargo clippy --workspace --bins -- -D warnings -A dead-code
run_gate rust-tests cargo test --workspace
run_gate javascript-tests node --test \
  conformance/manifest-integrity.test.mjs conformance/minimal-producer.test.mjs \
  conformance/transition-evidence-generation.test.mjs conformance/diagnostic-retention-hybrid.test.mjs \
  conformance/diagnostic-cursor-lease.test.mjs conformance/diagnostic-read-guard.test.mjs \
  conformance/diagnostic-read-guard-heartbeat.test.mjs packages/codecs/test/canonicalization.test.mjs \
  packages/identity/test/identity-claim-validator.test.mjs packages/validation/test/validation.test.mjs \
  tests/quarantine-replacement-crash-points.test.mjs
run_gate external-conformance node conformance/run.mjs
run_gate replacement-cleanup-crash-matrix cargo test -p lingonberry-core --test quarantine_replacement_crash_matrix
run_gate release-build cargo build --release -p lingonberry-storage --bin lingonberry-storage -p lingonberry-relay --bin lingonberry-relay
install -m 0755 target/release/lingonberry-storage "$out/binaries/lingonberry-storage"
install -m 0755 target/release/lingonberry-relay "$out/binaries/lingonberry-relay"
sha256sum "$out"/binaries/* >"$out/manifests/binary-sha256.txt"
run_gate operator-acceptance env \
  LINGONBERRY_STORAGE_BIN="$repo_root/$out/binaries/lingonberry-storage" \
  LINGONBERRY_RELAY_BIN="$repo_root/$out/binaries/lingonberry-relay" \
  bash scripts/v0_8_operator_acceptance.sh

python3 - "$out" "$candidate_sha" "$started_at" <<'PY'
import json, pathlib, sys, datetime
out = pathlib.Path(sys.argv[1])
gates = [json.loads(p.read_text()) for p in sorted((out/'results').glob('*.json'))]
summary = {'schemaVersion': 1, 'candidateCommit': sys.argv[2], 'startedAt': sys.argv[3],
           'finishedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
           'status': 'passed' if gates and all(g['status']=='passed' for g in gates) else 'failed',
           'gates': gates}
(out/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
PY
(
  cd "$out"
  find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum >SHA256SUMS
)
echo "v1.0 candidate qualification passed for $candidate_sha"
