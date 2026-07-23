#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAP = ROOT / "deploy/soak/v1-formal-command-map.json"
REQUIRED = {
    "publish", "retrieve", "query", "graceful_restart", "abrupt_termination",
    "verify", "index_rebuild", "backup", "isolated_restore", "crash_matrix",
    "malformed", "oversized", "nested", "disk_pressure"
}
ALLOWED_ADAPTERS = {
    "exec", "systemd", "generated-path", "test-suite", "stdin-fixture",
    "host-scenario", "exec-sequence", "http-rbac"
}
PLACEHOLDER = re.compile(r"^\{[A-Za-z][A-Za-z0-9]*\}$")
SHELL_META = re.compile(r"[;&|`$<>\n\r]")


def fail(message: str) -> None:
    print(f"formal command map invalid: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_argv(argv: object, variables: dict[str, object], location: str) -> None:
    if not isinstance(argv, list) or not argv:
        fail(f"{location} must be a non-empty argv array")
    for i, token in enumerate(argv):
        if not isinstance(token, str) or not token:
            fail(f"{location}[{i}] must be a non-empty string")
        if SHELL_META.search(token):
            fail(f"{location}[{i}] contains shell metacharacters")
        for match in re.findall(r"\{[^{}]+\}", token):
            if not PLACEHOLDER.match(match):
                fail(f"{location}[{i}] has malformed placeholder {match}")
            key = match[1:-1]
            if key not in variables and key not in {
                "generatedArchiveDir", "latestVerifiedArchiveDir"
            }:
                fail(f"{location}[{i}] references unknown variable {key}")


def main() -> None:
    data = json.loads(MAP.read_text())
    if data.get("schemaVersion") != 1:
        fail("schemaVersion must be 1")
    candidate = data.get("candidateCommit")
    if not isinstance(candidate, str) or not re.fullmatch(r"[0-9a-f]{40}", candidate):
        fail("candidateCommit must be a full lowercase SHA-1")
    variables = data.get("variables")
    operations = data.get("operations")
    if not isinstance(variables, dict) or not isinstance(operations, dict):
        fail("variables and operations must be objects")
    missing = REQUIRED - operations.keys()
    if missing:
        fail(f"missing required operations: {sorted(missing)}")

    for name, spec in operations.items():
        if not isinstance(spec, dict):
            fail(f"operation {name} must be an object")
        adapter = spec.get("adapter")
        enabled = spec.get("enabled")
        if adapter not in ALLOWED_ADAPTERS:
            fail(f"operation {name} has unsupported adapter {adapter!r}")
        if not isinstance(enabled, bool):
            fail(f"operation {name} enabled must be boolean")
        if enabled:
            if adapter == "exec-sequence":
                sequence = spec.get("argvSequence")
                if not isinstance(sequence, list) or not sequence:
                    fail(f"operation {name} requires argvSequence")
                for i, argv in enumerate(sequence):
                    validate_argv(argv, variables, f"operations.{name}.argvSequence[{i}]")
            else:
                validate_argv(spec.get("argv"), variables, f"operations.{name}.argv")
        else:
            reason = spec.get("reason")
            if not isinstance(reason, str) or not reason.strip():
                fail(f"disabled operation {name} requires a reason")

    qualification_blockers = [
        name for name in REQUIRED
        if operations[name].get("enabled") is not True
    ]
    report = {
        "schemaVersion": 1,
        "candidateCommit": candidate,
        "requiredOperations": len(REQUIRED),
        "enabledRequiredOperations": len(REQUIRED) - len(qualification_blockers),
        "qualificationReady": not qualification_blockers,
        "qualificationBlockers": sorted(qualification_blockers),
    }
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
