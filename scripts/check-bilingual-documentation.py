#!/usr/bin/env python3
"""Validate the structural contract for bilingual-required documentation."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "docs" / "DOCUMENTATION_INVENTORY.md"
REQUIRED_RE = re.compile(
    r"^\| `(?P<path>[^`]+)` \| `BILINGUAL_REQUIRED` \|",
    re.MULTILINE,
)

ENGLISH_MARKER = "## English"
JAPANESE_MARKER = "## 日本語"
NORMATIVE_ENGLISH = "English is the normative"
NORMATIVE_JAPANESE = "英語"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--allow-pending",
        action="append",
        default=[],
        help="Bilingual-required path that remains an explicit v1.0 blocker.",
    )
    return parser.parse_args()


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)


def main() -> int:
    args = parse_args()
    pending = set(args.allow_pending)
    inventory_text = INVENTORY.read_text(encoding="utf-8")
    required = set(REQUIRED_RE.findall(inventory_text))

    unknown_pending = pending - required
    if unknown_pending:
        fail(f"pending allowlist contains non-required paths: {sorted(unknown_pending)}")
        return 1

    errors: list[str] = []
    checked = 0

    for relative in sorted(required):
        if relative in pending:
            print(f"pending release blocker: {relative}")
            continue

        path = ROOT / relative
        if not path.is_file():
            errors.append(f"missing bilingual-required file: {relative}")
            continue

        text = path.read_text(encoding="utf-8")
        english_index = text.find(ENGLISH_MARKER)
        japanese_index = text.find(JAPANESE_MARKER)

        if english_index < 0:
            errors.append(f"missing {ENGLISH_MARKER!r}: {relative}")
        if japanese_index < 0:
            errors.append(f"missing {JAPANESE_MARKER!r}: {relative}")
        if english_index >= 0 and japanese_index >= 0 and english_index > japanese_index:
            errors.append(f"Japanese section precedes English section: {relative}")
        if NORMATIVE_ENGLISH not in text:
            errors.append(f"missing English normative notice: {relative}")
        if NORMATIVE_JAPANESE not in text:
            errors.append(f"missing Japanese normative notice: {relative}")

        checked += 1

    if errors:
        for error in errors:
            fail(error)
        return 1

    print(
        f"bilingual documentation structure valid: checked={checked}, "
        f"pending={len(pending)}, required={len(required)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
