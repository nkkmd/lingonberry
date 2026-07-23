#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

BILINGUAL_REQUIRED = {
    "README.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "docs/DOCUMENTATION_POLICY.md",
    "docs/operations/README.md",
    "docs/operations/RELAY_QUICKSTART.md",
    "docs/operations/V1_0_OPERATOR_RUNBOOK.md",
    "docs/operations/V1_0_UPGRADE_AND_ROLLBACK.md",
}
BILINGUAL_SCOPED = {
    "docs/concepts/GLOSSARY.md",
    "docs/concepts/CARRIER.md",
    "docs/protocols/VERSIONING_AND_COMPATIBILITY.md",
}
HISTORICAL_OPERATION_DOCS = {
    "docs/operations/V0_8_OPERATOR_RUNBOOK.md",
    "docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md",
}
NORMALIZED_OPERATION_DOCS = {
    "docs/operations/OPERATOR_CLI_CONTRACT.md",
    "docs/operations/STORAGE_MIGRATION_AND_UPGRADE.md",
    "docs/operations/STORAGE_NODE_QUICKSTART.md",
    "docs/operations/STORAGE_NODE_RUNTIME.md",
    "docs/operations/SUPPORTED_PLATFORMS.md",
    "docs/operations/SYSTEMD_UNIT_TEMPLATES.md",
}


def tracked_markdown() -> list[str]:
    cp = subprocess.run(["git", "ls-files", "*.md"], text=True, capture_output=True, check=True)
    return sorted(line for line in cp.stdout.splitlines() if line)


def classify(path: str) -> tuple[str, str, str]:
    if path in BILINGUAL_REQUIRED:
        return "BILINGUAL_REQUIRED", "NORMALIZE_BEFORE_V1", "yes"
    if path in BILINGUAL_SCOPED:
        return "BILINGUAL_SCOPED", "REVIEW_SCOPE_BEFORE_V1", "yes"
    if path in HISTORICAL_OPERATION_DOCS:
        return "ENGLISH_ONLY", "KEEP_HISTORICAL", "no"
    if path in NORMALIZED_OPERATION_DOCS:
        return "ENGLISH_ONLY", "KEEP_ENGLISH", "no"
    if path.startswith("docs/roadmap/RELEASE_0_"):
        return "ENGLISH_ONLY", "ARCHIVE_REVIEW", "no"
    if path.startswith("docs/roadmap/"):
        return "ENGLISH_ONLY", "ENGLISH_NORMALIZATION", "yes" if "V1_0" in path or path.endswith("ROADMAP_TO_V1_0.md") else "no"
    if path.startswith(("docs/protocols/", "docs/architecture/", "docs/security/")):
        return "ENGLISH_ONLY", "ENGLISH_NORMALIZATION", "yes"
    if path.startswith("docs/operations/"):
        return "ENGLISH_ONLY", "MERGE_OR_ENGLISH_NORMALIZATION", "yes"
    if path.startswith("packages/") or path == "AGENTS.md":
        return "ENGLISH_ONLY", "KEEP_ENGLISH", "no"
    if path == "CHANGELOG.md":
        return "ENGLISH_ONLY", "KEEP_ENGLISH_ADD_BILINGUAL_V1_SUMMARY", "yes"
    return "ENGLISH_ONLY", "REVIEW", "no"


def render(paths: list[str]) -> str:
    counts: dict[str, int] = {}
    rows = []
    blockers = 0
    for path in paths:
        category, action, blocker = classify(path)
        counts[category] = counts.get(category, 0) + 1
        blockers += blocker == "yes"
        rows.append(f"| `{path}` | `{category}` | `{action}` | {blocker} |")
    summary = ", ".join(f"{key}: {counts[key]}" for key in sorted(counts))
    return f"""# Documentation Inventory\n\n> English is normative. This inventory is generated from tracked Markdown files.\n> 英語を正本とします。この一覧は追跡対象のMarkdownファイルから生成されます。\n\n## Status\n\n- Tracked Markdown files: **{len(paths)}**\n- Release-blocking review entries: **{blockers}**\n- Classification totals: {summary}\n- Governing policy: [`DOCUMENTATION_POLICY.md`](./DOCUMENTATION_POLICY.md)\n- Tracking issue: [#144](https://github.com/nkkmd/lingonberry/issues/144)\n\n## English\n\nEvery tracked Markdown file must appear below. `yes` in the final column means the listed action must be resolved before v1.0.0 publication. Classification does not claim that translation or normalization is already complete.\n\n## 日本語\n\n追跡対象のMarkdownファイルは、すべて以下に掲載されなければなりません。最終列が`yes`の項目は、v1.0.0公開前に記載された作業を完了する必要があります。分類済みであることは、翻訳や正規化が完了済みであることを意味しません。\n\n## Inventory\n\n| Current path | Classification | Required action | v1.0 blocker |\n|---|---|---|---|\n""" + "\n".join(rows) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", type=Path, default=Path("docs/DOCUMENTATION_INVENTORY.md"))
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    generated = render(tracked_markdown())
    if args.check:
        current = args.output.read_text() if args.output.exists() else ""
        if current != generated:
            print("documentation inventory is stale; regenerate it", flush=True)
            return 1
        return 0
    args.output.write_text(generated)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
