#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import re
import stat
import subprocess
import sys
from pathlib import Path

EXPECTED_CANDIDATE = "f9543019f2c219aea3b085ff90f2da201b268a48"
FORBIDDEN_MOUNTS = {"/", "/boot", "/boot/efi", "/home", "/var", "/var/lib"}


def fail(message: str) -> None:
    print(f"disk-pressure contract invalid: {message}", file=sys.stderr)
    raise SystemExit(1)


def real(path: str) -> Path:
    p = Path(path)
    if p.is_symlink():
        fail(f"symlink path refused: {path}")
    return p.resolve(strict=False)


def overlaps(a: Path, b: Path) -> bool:
    return a == b or a in b.parents or b in a.parents


def command(*argv: str) -> str:
    return subprocess.check_output(argv, text=True).strip()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("contract", type=Path)
    parser.add_argument("--live", action="store_true")
    args = parser.parse_args()
    data = json.loads(args.contract.read_text())

    if data.get("schemaVersion") != 1:
        fail("schemaVersion must be 1")
    if data.get("candidateCommit") != EXPECTED_CANDIDATE:
        fail("candidateCommit mismatch")
    if data.get("filesystemType") != "ext4":
        fail("filesystemType must be ext4")
    if data.get("mode") != "reference-host":
        fail("mode must be reference-host")

    mount = real(data["mountPoint"])
    workspace = real(data["workspace"])
    pressure = real(data["pressureFile"])
    evidence = real(data["evidenceDir"])
    journal = real(data["journalDir"])
    backing = real(data["backingFile"])
    active = [real(p) for p in data["activePaths"]]

    if str(mount) in FORBIDDEN_MOUNTS or len(mount.parts) < 3:
        fail("mountPoint is a host-critical or insufficiently specific path")
    if mount not in workspace.parents or mount not in pressure.parents:
        fail("workspace and pressureFile must be below mountPoint")
    if evidence == journal or overlaps(evidence, mount) or overlaps(journal, mount):
        fail("evidence and journal must be outside the pressure filesystem")
    if overlaps(backing, mount):
        fail("backingFile must be outside the pressure filesystem")
    for p in active:
        if overlaps(p, mount) or overlaps(p, workspace) or overlaps(p, pressure):
            fail(f"active path overlaps pressure filesystem: {p}")

    capacity = data.get("capacityBytes")
    target = data.get("pressureTargetFreeBytes")
    if not isinstance(capacity, int) or capacity < 268435456:
        fail("capacityBytes must be at least 256 MiB")
    if not isinstance(target, int) or target < 16777216 or target >= capacity // 2:
        fail("pressureTargetFreeBytes must be >=16 MiB and below half capacity")
    if data.get("cleanupMarker") != ".lingonberry-disk-pressure-owned":
        fail("cleanupMarker must use the reserved marker")
    uuid = data.get("filesystemUuid")
    if not isinstance(uuid, str) or not uuid:
        fail("filesystemUuid is required")

    live_checks = {
        "root": os.geteuid() == 0,
        "deviceBlock": False,
        "mounted": False,
        "sourceMatches": False,
        "uuidMatches": False,
        "capacityMatches": False,
        "ownershipMatches": False,
        "separateEvidenceDevice": False,
        "qualificationEnabled": data.get("qualificationEnabled") is True,
    }

    if args.live:
        if not live_checks["root"]:
            fail("live validation requires root")
        device = Path(data["device"])
        try:
            live_checks["deviceBlock"] = stat.S_ISBLK(device.stat().st_mode)
        except FileNotFoundError:
            pass
        if not live_checks["deviceBlock"]:
            fail("device is not a block device")
        source = command("findmnt", "-n", "-o", "SOURCE", "--target", str(mount))
        live_checks["mounted"] = True
        live_checks["sourceMatches"] = Path(source).resolve() == device.resolve()
        if not live_checks["sourceMatches"]:
            fail("mount source does not match frozen device")
        actual_uuid = command("blkid", "-s", "UUID", "-o", "value", str(device))
        live_checks["uuidMatches"] = actual_uuid == uuid
        if not live_checks["uuidMatches"]:
            fail("filesystem UUID mismatch")
        total = int(command("findmnt", "-b", "-n", "-o", "SIZE", "--target", str(mount)))
        live_checks["capacityMatches"] = abs(total - capacity) <= max(1048576, capacity // 100)
        if not live_checks["capacityMatches"]:
            fail("filesystem capacity mismatch")
        st = mount.stat()
        live_checks["ownershipMatches"] = st.st_uid == data["ownerUid"] and st.st_gid == data["ownerGid"]
        if not live_checks["ownershipMatches"]:
            fail("mount ownership mismatch")
        mount_dev = mount.stat().st_dev
        live_checks["separateEvidenceDevice"] = evidence.parent.stat().st_dev != mount_dev and journal.parent.stat().st_dev != mount_dev
        if not live_checks["separateEvidenceDevice"]:
            fail("evidence or journal shares pressure filesystem")
        if not live_checks["qualificationEnabled"]:
            fail("qualificationEnabled is false")

    report = {
        "schemaVersion": 1,
        "candidateCommit": EXPECTED_CANDIDATE,
        "staticContractValid": True,
        "liveValidationRequested": args.live,
        "qualificationReady": args.live and all(live_checks.values()),
        "liveChecks": live_checks,
        "qualificationBlocker": None if data.get("qualificationEnabled") else data.get("qualificationBlocker"),
    }
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
