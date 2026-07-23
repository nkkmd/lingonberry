#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import uuid
from pathlib import Path

PHASES = [
    "preflight", "pre_pressure", "allocate_pressure", "threshold_crossing",
    "expected_failure", "release_pressure", "restart", "readiness",
    "verify", "index_verify", "workspace_cleanliness", "complete",
]


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, text=True, capture_output=True, check=check)


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def snapshot(contract: dict, phase: str, mode: str) -> dict:
    mount = Path(contract["mountPoint"])
    if mode == "dry-run":
        return {
            "phase": phase,
            "mode": mode,
            "mountPoint": str(mount),
            "filesystemUuid": contract["filesystemUuid"],
            "freeBytes": 512 * 1024 * 1024 if phase not in {"threshold_crossing", "expected_failure"} else contract["pressureTargetFreeBytes"],
            "freeInodes": 100000,
            "serviceActive": phase not in {"expected_failure"},
        }
    st = os.statvfs(mount)
    return {
        "phase": phase,
        "mode": mode,
        "mountPoint": str(mount),
        "filesystemUuid": run(["findmnt", "-n", "-o", "UUID", "--target", str(mount)]).stdout.strip(),
        "freeBytes": st.f_bavail * st.f_frsize,
        "freeInodes": st.f_favail,
        "serviceActive": run(["systemctl", "is-active", "--quiet", contract["serviceUnit"]], check=False).returncode == 0,
    }


def validate_live(contract_path: Path, contract: dict) -> None:
    if os.geteuid() != 0:
        raise RuntimeError("live mode requires root")
    if os.environ.get("LINGONBERRY_DISK_PRESSURE_ACK") != contract["candidateCommit"]:
        raise RuntimeError("candidate acknowledgement mismatch")
    if contract.get("qualificationEnabled") is not True:
        raise RuntimeError("qualification is disabled in the frozen contract")
    validator = Path(__file__).with_name("check-v1-disk-pressure-contract.py")
    run([sys.executable, str(validator), str(contract_path), "--live"])


def checksum_bundle(out: Path) -> None:
    files = sorted(p for p in out.rglob("*") if p.is_file() and p.name != "SHA256SUMS")
    lines = [f"{sha256(p)}  {p.relative_to(out)}" for p in files]
    (out / "SHA256SUMS").write_text("\n".join(lines) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--contract", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--mode", choices=["dry-run", "live"], default="dry-run")
    ap.add_argument("--inject-failure-at", choices=PHASES)
    args = ap.parse_args()

    if args.out.exists():
        raise SystemExit("output directory already exists; stopped runs cannot resume")
    args.out.mkdir(parents=True)
    contract = json.loads(args.contract.read_text())
    run_id = str(uuid.uuid4())
    started = int(time.time())
    timeline = args.out / "timeline.jsonl"
    snapshots = args.out / "snapshots"
    pressure_file = Path(contract["pressureFile"])
    workspace = Path(contract["workspace"])
    completed: list[str] = []
    status = "failed"
    stop_reason = None

    try:
        if args.mode == "live":
            validate_live(args.contract, contract)
        for phase in PHASES:
            if args.inject_failure_at == phase:
                raise RuntimeError(f"forced failure at {phase}")
            event = {"runId": run_id, "phase": phase, "timestamp": int(time.time())}
            with timeline.open("a") as f:
                f.write(json.dumps(event, sort_keys=True) + "\n")
            write_json(snapshots / f"{len(completed)+1:02d}-{phase}.json", snapshot(contract, phase, args.mode))

            if args.mode == "live":
                if phase == "allocate_pressure":
                    workspace.mkdir(parents=True, exist_ok=True)
                    if pressure_file.exists():
                        raise RuntimeError("pressure file already exists")
                    free = os.statvfs(contract["mountPoint"]).f_bavail * os.statvfs(contract["mountPoint"]).f_frsize
                    allocate = max(0, free - int(contract["pressureTargetFreeBytes"]))
                    run(["fallocate", "-l", str(allocate), str(pressure_file)])
                elif phase == "expected_failure":
                    probe = workspace / "pressure-probe"
                    cp = run(["dd", "if=/dev/zero", f"of={probe}", "bs=1M", "count=64", "conv=fsync"], check=False)
                    if cp.returncode == 0:
                        raise RuntimeError("pressure probe unexpectedly succeeded")
                elif phase == "release_pressure":
                    if pressure_file.exists():
                        pressure_file.unlink()
                    run(["sync"])
                elif phase == "restart":
                    run(["systemctl", "restart", contract["serviceUnit"]])
                elif phase == "readiness":
                    run(["curl", "--fail", "--silent", contract["readyUrl"]])
                elif phase == "verify":
                    run([contract["storageBinary"], "--state-dir", contract["stateDir"], "--data-dir", contract["dataDir"], "--backup-dir", contract["backupDir"], "--temp-dir", contract["tempDir"], "verify"])
                elif phase == "index_verify":
                    run([contract["storageBinary"], "--state-dir", contract["stateDir"], "--data-dir", contract["dataDir"], "--backup-dir", contract["backupDir"], "--temp-dir", contract["tempDir"], "index", "verify"])
                elif phase == "workspace_cleanliness":
                    unexpected = [p.name for p in workspace.iterdir()] if workspace.exists() else []
                    if unexpected:
                        raise RuntimeError(f"workspace not clean: {unexpected}")
            completed.append(phase)
        status = "passed"
    except Exception as exc:  # evidence must survive failures
        stop_reason = str(exc)
    finally:
        if args.mode == "live" and pressure_file.exists():
            pressure_file.unlink()
            run(["sync"], check=False)
        summary = {
            "schemaVersion": 1,
            "runId": run_id,
            "candidateCommit": contract.get("candidateCommit"),
            "mode": args.mode,
            "qualification": False,
            "qualifyingPass": False,
            "status": status,
            "stopReason": stop_reason,
            "phasesRequired": PHASES,
            "phasesCompleted": completed,
            "startedAtEpoch": started,
            "endedAtEpoch": int(time.time()),
            "contractSha256": sha256(args.contract),
            "cleanupAttempted": True,
            "pressureFilePresentAfterCleanup": pressure_file.exists(),
            "referenceHostRehearsalComplete": False,
        }
        write_json(args.out / "summary.json", summary)
        checksum_bundle(args.out)
    return 0 if status == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
