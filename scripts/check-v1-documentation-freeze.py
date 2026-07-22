#!/usr/bin/env python3
"""Validate the v1.0 documentation-freeze set and local Markdown links."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_PATHS = (
    "README.md",
    "docs/operations/README.md",
    "docs/operations/SUPPORTED_PLATFORMS.md",
    "docs/operations/V0_8_OPERATOR_RUNBOOK.md",
    "docs/operations/OPERATOR_CLI_CONTRACT.md",
    "docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md",
    "docs/operations/STORAGE_MIGRATION_AND_UPGRADE.md",
    "docs/architecture/V1_COMPATIBILITY_POLICY.md",
    "docs/architecture/V1_0_RUST_API_AUDIT.md",
    "docs/security/V1_0_SECURITY_DIFF_REVIEW.md",
    "docs/roadmap/V1_0_QUALIFICATION_PLAN.md",
    "docs/roadmap/V1_0_QUALIFICATION_STATUS.md",
    "docs/roadmap/V1_0_SOAK_PLAN.md",
    "docs/roadmap/V1_0_DOCUMENTATION_FREEZE_PLAN.md",
    "docs/roadmap/V1_0_RELEASE_EVIDENCE.md",
    "deploy/systemd/lingonberry-storage-ready.service",
    "deploy/systemd/lingonberry-relay.service",
)

LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")


def local_target(source: Path, raw: str) -> Path | None:
    target = raw.strip().split("#", 1)[0]
    if not target or target.startswith(("http://", "https://", "mailto:", "#")):
        return None
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1]
    return (source.parent / target).resolve()


def main() -> int:
    errors: list[str] = []
    required = [ROOT / path for path in REQUIRED_PATHS]

    for path in required:
        if not path.exists():
            errors.append(f"missing required freeze document: {path.relative_to(ROOT)}")

    markdown_files = [path for path in required if path.suffix.lower() == ".md" and path.exists()]
    checked_links = 0
    for source in markdown_files:
        text = source.read_text(encoding="utf-8")
        for match in LINK_RE.finditer(text):
            target = local_target(source, match.group(1))
            if target is None:
                continue
            checked_links += 1
            try:
                target.relative_to(ROOT.resolve())
            except ValueError:
                errors.append(
                    f"local link escapes repository: {source.relative_to(ROOT)} -> {match.group(1)}"
                )
                continue
            if not target.exists():
                errors.append(
                    f"broken local link: {source.relative_to(ROOT)} -> {match.group(1)}"
                )

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        print(
            f"v1 documentation freeze check failed: {len(errors)} error(s), "
            f"{len(required)} required paths, {checked_links} local links checked",
            file=sys.stderr,
        )
        return 1

    print(
        f"v1 documentation freeze check passed: {len(required)} required paths, "
        f"{checked_links} local links checked"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
