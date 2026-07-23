#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path

CANDIDATE = "f9543019f2c219aea3b085ff90f2da201b268a48"
REQUIRED_COMMANDS = {
    "blkid", "curl", "df", "findmnt", "losetup", "mountpoint", "sha256sum",
    "systemctl", "uname",
}


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, text=True, capture_output=True, check=False)


def fail(message: str) -> None:
    print(f"reference host invalid: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_os_release() -> dict[str, str]:
    values: dict[str, str] = {}
    for line in Path("/etc/os-release").read_text().splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            values[key] = value.strip().strip('"')
    return values


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--contract", required=True, type=Path)
    ap.add_argument("--live", action="store_true")
    args = ap.parse_args()

    data = json.loads(args.contract.read_text())
    if data.get("schemaVersion") != 1:
        fail("schemaVersion must be 1")
    if data.get("candidateCommit") != CANDIDATE:
        fail("candidate commit mismatch")

    required = [
        "device", "backingFile", "filesystemType", "filesystemUuid", "mountPoint",
        "workspace", "pressureFile", "capacityBytes", "evidenceDir", "journalDir",
        "serviceUnit", "storageBinary",
    ]
    for key in required:
        if key not in data:
            fail(f"missing {key}")

    paths = [Path(data[k]) for k in [
        "backingFile", "mountPoint", "workspace", "pressureFile", "evidenceDir", "journalDir"
    ]]
    if any(p.is_symlink() for p in paths if p.exists()):
        fail("contract paths must not be symlinks")
    if not str(data["mountPoint"]).startswith("/mnt/lingonberry-"):
        fail("mountPoint must use dedicated /mnt/lingonberry-* namespace")
    if Path(data["workspace"]).parent != Path(data["mountPoint"]):
        fail("workspace must be a direct child of mountPoint")
    if Path(data["pressureFile"]).parent != Path(data["mountPoint"]):
        fail("pressureFile must be a direct child of mountPoint")
    if Path(data["backingFile"]).is_relative() or Path(data["mountPoint"]).is_relative():
        fail("host paths must be absolute")
    if int(data["capacityBytes"]) < 512 * 1024 * 1024:
        fail("capacityBytes must be at least 512 MiB")
    if data["filesystemType"] != "ext4":
        fail("filesystemType must be ext4")
    if not re.fullmatch(r"[0-9a-fA-F-]{36}", str(data["filesystemUuid"])):
        fail("filesystemUuid must be frozen before host rehearsal")

    static_report = {
        "schemaVersion": 1,
        "candidateCommit": CANDIDATE,
        "staticContractValid": True,
        "liveValidationRequested": args.live,
        "qualificationReady": False,
        "referenceHostRehearsalComplete": False,
    }
    if not args.live:
        print(json.dumps(static_report, indent=2, sort_keys=True))
        return

    if os.geteuid() != 0:
        fail("live validation requires root")
    if os.environ.get("LINGONBERRY_REFERENCE_HOST_ACK") != CANDIDATE:
        fail("reference host acknowledgement mismatch")
    if data.get("qualificationEnabled") is not True:
        fail("qualification is disabled")

    release = read_os_release()
    if release.get("ID") != "ubuntu" or release.get("VERSION_ID") != "24.04":
        fail("host must be Ubuntu 24.04")
    if platform.machine() != "x86_64":
        fail("host architecture must be x86_64")
    missing = sorted(cmd for cmd in REQUIRED_COMMANDS if shutil.which(cmd) is None)
    if missing:
        fail(f"missing commands: {missing}")

    mount = str(data["mountPoint"])
    device = str(data["device"])
    if run(["mountpoint", "-q", mount]).returncode != 0:
        fail("pressure mount is not mounted")
    source = run(["findmnt", "-n", "-o", "SOURCE", "--target", mount]).stdout.strip()
    fstype = run(["findmnt", "-n", "-o", "FSTYPE", "--target", mount]).stdout.strip()
    uuid = run(["findmnt", "-n", "-o", "UUID", "--target", mount]).stdout.strip()
    if source != device:
        fail(f"mount source mismatch: {source}")
    if fstype != data["filesystemType"]:
        fail(f"filesystem type mismatch: {fstype}")
    if uuid.lower() != str(data["filesystemUuid"]).lower():
        fail(f"filesystem UUID mismatch: {uuid}")
    if run(["systemctl", "is-active", "--quiet", data["serviceUnit"]]).returncode != 0:
        fail("Lingonberry service is not active")
    if not Path(data["storageBinary"]).is_file():
        fail("storage binary is missing")

    mount_dev = run(["findmnt", "-n", "-o", "MAJ:MIN", "--target", mount]).stdout.strip()
    evidence_dev = run(["findmnt", "-n", "-o", "MAJ:MIN", "--target", data["evidenceDir"]]).stdout.strip()
    journal_dev = run(["findmnt", "-n", "-o", "MAJ:MIN", "--target", data["journalDir"]]).stdout.strip()
    if not mount_dev or mount_dev in {evidence_dev, journal_dev}:
        fail("evidence and journal must be on a different filesystem")

    report = static_report | {
        "hostOs": "ubuntu-24.04",
        "architecture": "x86_64",
        "mountSource": source,
        "filesystemType": fstype,
        "filesystemUuid": uuid,
        "pressureFilesystemDeviceId": mount_dev,
        "evidenceFilesystemDeviceId": evidence_dev,
        "journalFilesystemDeviceId": journal_dev,
        "liveContractValid": True,
    }
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
