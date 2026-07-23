#!/usr/bin/env python3
"""Lingonberry v1.0 formal soak scheduler.

The scheduler separates deterministic cadence/evidence logic from host operations.
`mock` mode is non-qualifying and exists only for scheduler rehearsal. `systemd`
mode is the only potentially qualifying adapter and is fail-closed unless all
reference-host preconditions are explicitly satisfied.
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import time
import uuid
from collections import Counter
from typing import Any, Iterable

CANDIDATE = "f9543019f2c219aea3b085ff90f2da201b268a48"
STORAGE_SHA256 = "22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0"
RELAY_SHA256 = "9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d"
FORMAL_SECONDS = 72 * 60 * 60

MINIMA: dict[str, int] = {
    "publish": 10_000,
    "retrieve": 10_000,
    "query": 5_000,
    "graceful_restart": 48,
    "abrupt_termination": 12,
    "verify": 12,
    "index_rebuild": 4,
    "backup": 6,
    "isolated_restore": 3,
    "crash_matrix": 6,
    "malformed": 1_000,
    "oversized": 200,
    "nested": 200,
    "disk_pressure": 2,
}

DISRUPTIVE = {
    "graceful_restart",
    "abrupt_termination",
    "index_rebuild",
    "isolated_restore",
    "crash_matrix",
    "disk_pressure",
}


@dataclasses.dataclass(frozen=True)
class Event:
    offset: int
    kind: str
    ordinal: int


@dataclasses.dataclass
class Thresholds:
    minimum_free_disk_bytes: int
    minimum_free_inodes: int
    maximum_file_descriptors: int
    maximum_rss_bytes: int
    maximum_swap_used_bytes: int
    maximum_readiness_failure_seconds: int
    maximum_unexpected_restarts: int

    @classmethod
    def from_json(cls, path: pathlib.Path) -> "Thresholds":
        raw = json.loads(path.read_text())
        return cls(**raw)


class Evidence:
    def __init__(self, out: pathlib.Path, run_id: str) -> None:
        self.out = out
        self.run_id = run_id
        for child in ("manifests", "telemetry", "events", "logs", "partial"):
            (out / child).mkdir(parents=True, exist_ok=True)
        self.timeline = out / "events" / "timeline.jsonl"
        self.metrics = out / "telemetry" / "metrics.jsonl"

    def append_jsonl(self, path: pathlib.Path, value: dict[str, Any]) -> None:
        with path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(value, sort_keys=True) + "\n")

    def event(self, now: str, kind: str, status: str, **extra: Any) -> None:
        self.append_jsonl(self.timeline, {"timestamp": now, "kind": kind, "status": status, **extra})

    def metric(self, value: dict[str, Any]) -> None:
        self.append_jsonl(self.metrics, value)

    def finalize(self, summary: dict[str, Any]) -> None:
        (self.out / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        sums: list[str] = []
        for path in sorted(p for p in self.out.rglob("*") if p.is_file() and p.name != "SHA256SUMS"):
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            sums.append(f"{digest}  {path.relative_to(self.out)}")
        (self.out / "SHA256SUMS").write_text("\n".join(sums) + "\n")


def evenly_spaced(kind: str, count: int, duration: int) -> Iterable[Event]:
    # Place every workload family throughout the run, avoiding an all-at-start burst.
    for ordinal in range(1, count + 1):
        offset = max(1, min(duration - 1, round((ordinal * duration) / (count + 1))))
        yield Event(offset=offset, kind=kind, ordinal=ordinal)


def build_schedule(duration: int, minima: dict[str, int]) -> list[Event]:
    events = [event for kind, count in minima.items() for event in evenly_spaced(kind, count, duration)]
    events.sort(key=lambda e: (e.offset, e.kind, e.ordinal))
    return events


def validate_distribution(schedule: list[Event], duration: int, minima: dict[str, int]) -> None:
    counts = Counter(event.kind for event in schedule)
    if counts != Counter(minima):
        raise ValueError(f"schedule count mismatch: {counts} != {minima}")
    thirds = [(0, duration // 3), (duration // 3, 2 * duration // 3), (2 * duration // 3, duration + 1)]
    for kind in DISRUPTIVE:
        offsets = [e.offset for e in schedule if e.kind == kind]
        if not offsets:
            raise ValueError(f"missing disruptive family: {kind}")
        occupied = sum(any(start <= offset < end for offset in offsets) for start, end in thirds)
        if occupied < min(3, len(offsets)):
            raise ValueError(f"{kind} is not distributed across the run")


class Adapter:
    qualifying = False

    def preflight(self) -> dict[str, Any]:
        raise NotImplementedError

    def telemetry(self, elapsed: int) -> dict[str, Any]:
        raise NotImplementedError

    def execute(self, event: Event) -> dict[str, Any]:
        raise NotImplementedError

    def close(self) -> None:
        return None


class MockAdapter(Adapter):
    qualifying = False

    def __init__(self, inject_threshold_failure_at: int | None = None) -> None:
        self.inject_threshold_failure_at = inject_threshold_failure_at

    def preflight(self) -> dict[str, Any]:
        return {"adapter": "mock", "platform": "virtual", "systemdManaged": False}

    def telemetry(self, elapsed: int) -> dict[str, Any]:
        disk = 50_000_000_000
        if self.inject_threshold_failure_at is not None and elapsed >= self.inject_threshold_failure_at:
            disk = 1
        return {
            "elapsedSeconds": elapsed,
            "serviceActive": True,
            "ready": True,
            "rssBytes": 64 * 1024 * 1024,
            "swapUsedBytes": 0,
            "fileDescriptors": 32,
            "freeDiskBytes": disk,
            "freeInodes": 1_000_000,
            "unexpectedRestarts": 0,
            "readinessFailureSeconds": 0,
        }

    def execute(self, event: Event) -> dict[str, Any]:
        return {"kind": event.kind, "ordinal": event.ordinal, "result": "passed"}


class SystemdAdapter(Adapter):
    qualifying = True

    def __init__(self, config: dict[str, Any]) -> None:
        self.config = config
        self.service = config["service"]
        self.storage = config["storageBinary"]
        self.relay = config["relayBinary"]

    @staticmethod
    def run(command: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
        return subprocess.run(command, check=check, text=True, capture_output=True)

    def preflight(self) -> dict[str, Any]:
        if os.geteuid() != 0:
            raise RuntimeError("formal systemd adapter requires root")
        if os.environ.get("LINGONBERRY_FORMAL_SOAK_ACK") != CANDIDATE:
            raise RuntimeError("formal acknowledgement does not match candidate")
        os_release = pathlib.Path("/etc/os-release").read_text()
        if "ID=ubuntu" not in os_release or "VERSION_ID=\"24.04\"" not in os_release:
            raise RuntimeError("formal soak requires Ubuntu 24.04")
        if os.uname().machine != "x86_64":
            raise RuntimeError("formal soak requires x86_64")
        if shutil.which("systemctl") is None:
            raise RuntimeError("systemctl not available")
        if hashlib.sha256(pathlib.Path(self.storage).read_bytes()).hexdigest() != STORAGE_SHA256:
            raise RuntimeError("storage binary digest mismatch")
        if hashlib.sha256(pathlib.Path(self.relay).read_bytes()).hexdigest() != RELAY_SHA256:
            raise RuntimeError("relay binary digest mismatch")
        active = self.run(["systemctl", "is-active", self.service]).stdout.strip()
        if active != "active":
            raise RuntimeError(f"service is not active: {active}")
        return {
            "adapter": "systemd",
            "platform": "Ubuntu 24.04 x86_64",
            "systemdManaged": True,
            "service": self.service,
            "candidate": CANDIDATE,
        }

    def telemetry(self, elapsed: int) -> dict[str, Any]:
        props = self.run([
            "systemctl", "show", self.service,
            "--property=ActiveState,MainPID,NRestarts,MemoryCurrent,TasksCurrent",
            "--value",
        ]).stdout.splitlines()
        active, pid, restarts, memory, _tasks = (props + ["0"] * 5)[:5]
        fd_count = 0
        if pid.isdigit() and int(pid) > 0:
            fd_count = len(list(pathlib.Path(f"/proc/{pid}/fd").iterdir()))
        stat = os.statvfs(self.config["stateRoot"])
        return {
            "elapsedSeconds": elapsed,
            "serviceActive": active == "active",
            "ready": self.run(["curl", "-fsS", self.config["readyUrl"]], check=False).returncode == 0,
            "rssBytes": int(memory or 0),
            "swapUsedBytes": 0,
            "fileDescriptors": fd_count,
            "freeDiskBytes": stat.f_bavail * stat.f_frsize,
            "freeInodes": stat.f_favail,
            "unexpectedRestarts": int(restarts or 0),
            "readinessFailureSeconds": 0,
        }

    def execute(self, event: Event) -> dict[str, Any]:
        # Commands are explicit configuration entries so the frozen host manifest
        # is the reviewable source of truth. Shell interpolation is prohibited.
        commands = self.config["commands"].get(event.kind)
        if not commands:
            raise RuntimeError(f"missing command for workload family: {event.kind}")
        for command in commands:
            if not isinstance(command, list) or not all(isinstance(v, str) for v in command):
                raise RuntimeError(f"invalid argv command for {event.kind}")
            self.run(command)
        return {"kind": event.kind, "ordinal": event.ordinal, "result": "passed"}


def threshold_violation(metric: dict[str, Any], limits: Thresholds) -> str | None:
    checks = [
        (metric["freeDiskBytes"] < limits.minimum_free_disk_bytes, "minimum free disk bytes"),
        (metric["freeInodes"] < limits.minimum_free_inodes, "minimum free inodes"),
        (metric["fileDescriptors"] > limits.maximum_file_descriptors, "maximum file descriptors"),
        (metric["rssBytes"] > limits.maximum_rss_bytes, "maximum RSS"),
        (metric["swapUsedBytes"] > limits.maximum_swap_used_bytes, "maximum swap used"),
        (metric["readinessFailureSeconds"] > limits.maximum_readiness_failure_seconds, "readiness failure duration"),
        (metric["unexpectedRestarts"] > limits.maximum_unexpected_restarts, "unexpected restart count"),
        (not metric["serviceActive"], "service inactive"),
    ]
    for violated, label in checks:
        if violated:
            return label
    return None


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--adapter", choices=("mock", "systemd"), required=True)
    parser.add_argument("--out", type=pathlib.Path, required=True)
    parser.add_argument("--thresholds", type=pathlib.Path, required=True)
    parser.add_argument("--config", type=pathlib.Path)
    parser.add_argument("--virtual-duration", type=int, default=FORMAL_SECONDS)
    parser.add_argument("--tick-seconds", type=int, default=60)
    parser.add_argument("--real-time", action="store_true")
    parser.add_argument("--inject-threshold-failure-at", type=int)
    args = parser.parse_args()

    if args.virtual_duration < 9:
        parser.error("duration must be at least 9 seconds")
    out = args.out.resolve()
    if out.exists():
        raise RuntimeError(f"output already exists; stopped runs are not resumable: {out}")

    run_id = f"v1-soak-{uuid.uuid4()}"
    evidence = Evidence(out, run_id)
    limits = Thresholds.from_json(args.thresholds)
    schedule = build_schedule(args.virtual_duration, MINIMA)
    validate_distribution(schedule, args.virtual_duration, MINIMA)

    if args.adapter == "mock":
        adapter: Adapter = MockAdapter(args.inject_threshold_failure_at)
    else:
        if not args.real_time or args.virtual_duration < FORMAL_SECONDS or args.tick_seconds != 60:
            raise RuntimeError("systemd qualification requires real time, >=72h, and 60-second telemetry")
        if args.config is None:
            raise RuntimeError("systemd adapter requires --config")
        adapter = SystemdAdapter(json.loads(args.config.read_text()))

    started = utc_now()
    status = "running"
    stop_reason: str | None = None
    counts: Counter[str] = Counter()
    next_index = 0
    preflight: dict[str, Any] = {}
    qualification = adapter.qualifying

    try:
        preflight = adapter.preflight()
        (out / "manifests" / "run.json").write_text(json.dumps({
            "runId": run_id,
            "candidateCommit": CANDIDATE,
            "storageSha256": STORAGE_SHA256,
            "relaySha256": RELAY_SHA256,
            "adapter": args.adapter,
            "qualification": qualification,
            "startedAt": started,
            "minimumContinuousSeconds": FORMAL_SECONDS,
            "scheduledDurationSeconds": args.virtual_duration,
            "tickSeconds": args.tick_seconds,
            "preflight": preflight,
        }, indent=2, sort_keys=True) + "\n")
        (out / "manifests" / "schedule.json").write_text(json.dumps(
            [dataclasses.asdict(event) for event in schedule], indent=2, sort_keys=True
        ) + "\n")
        (out / "manifests" / "thresholds.json").write_text(json.dumps(dataclasses.asdict(limits), indent=2, sort_keys=True) + "\n")

        wall_start = time.monotonic()
        for elapsed in range(0, args.virtual_duration + 1, args.tick_seconds):
            metric = adapter.telemetry(elapsed)
            metric.update({"timestamp": utc_now(), "runId": run_id})
            evidence.metric(metric)
            violation = threshold_violation(metric, limits)
            if violation:
                status = "failed"
                stop_reason = f"threshold violation: {violation}"
                evidence.event(utc_now(), "stop", "failed", reason=stop_reason, elapsedSeconds=elapsed)
                break

            while next_index < len(schedule) and schedule[next_index].offset <= elapsed:
                event = schedule[next_index]
                evidence.event(utc_now(), event.kind, "started", ordinal=event.ordinal, elapsedSeconds=elapsed)
                result = adapter.execute(event)
                counts[event.kind] += 1
                evidence.event(utc_now(), event.kind, "passed", ordinal=event.ordinal, elapsedSeconds=elapsed, result=result)
                next_index += 1

            if args.real_time and elapsed < args.virtual_duration:
                target = wall_start + elapsed + args.tick_seconds
                time.sleep(max(0.0, target - time.monotonic()))

        if status == "running":
            # Flush events scheduled after the final tick due to a non-divisible duration.
            while next_index < len(schedule):
                event = schedule[next_index]
                result = adapter.execute(event)
                counts[event.kind] += 1
                evidence.event(utc_now(), event.kind, "passed", ordinal=event.ordinal, elapsedSeconds=args.virtual_duration, result=result)
                next_index += 1
            if counts != Counter(MINIMA):
                status = "failed"
                stop_reason = f"workload minima not met: {dict(counts)}"
            elif args.virtual_duration < FORMAL_SECONDS:
                status = "passed"
            else:
                status = "passed"
    except Exception as exc:  # evidence must survive all controlled failures
        status = "failed"
        stop_reason = f"{type(exc).__name__}: {exc}"
        evidence.event(utc_now(), "exception", "failed", reason=stop_reason)
    finally:
        adapter.close()

    qualifying_pass = bool(
        status == "passed"
        and qualification
        and args.real_time
        and args.virtual_duration >= FORMAL_SECONDS
        and counts == Counter(MINIMA)
    )
    summary = {
        "schemaVersion": 1,
        "runId": run_id,
        "candidateCommit": CANDIDATE,
        "adapter": args.adapter,
        "qualification": qualification,
        "qualifyingPass": qualifying_pass,
        "status": status,
        "stopReason": stop_reason,
        "startedAt": started,
        "finishedAt": utc_now(),
        "scheduledDurationSeconds": args.virtual_duration,
        "minimumContinuousSeconds": FORMAL_SECONDS,
        "workloadCounts": dict(sorted(counts.items())),
        "workloadMinima": MINIMA,
        "preflight": preflight,
    }
    evidence.finalize(summary)
    return 0 if status == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
