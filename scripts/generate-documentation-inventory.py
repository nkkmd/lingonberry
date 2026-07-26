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
REVIEWED_BILINGUAL_REQUIRED = {
    "README.md",
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
REVIEWED_BILINGUAL_SCOPED = {
    "docs/concepts/CARRIER.md",
    "docs/concepts/GLOSSARY.md",
    "docs/protocols/VERSIONING_AND_COMPATIBILITY.md",
}
HISTORICAL_OPERATION_DOCS = {
    "docs/operations/V0_8_OPERATOR_RUNBOOK.md",
    "docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md",
}
NORMALIZED_ARCHITECTURE_DOCS = {
    "docs/architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md",
    "docs/architecture/DUPLICATE_AND_CONFLICT_CONTRACT.md",
    "docs/architecture/LINGONBERRY_PROTOCOL_EVOLUTION_PROPOSAL.md",
    "docs/architecture/README.md",
    "docs/architecture/TOITOI_REFERENCE_CHECKLIST.md",
    "docs/architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md",
    "docs/architecture/V0_9_RUST_API_INVENTORY.md",
    "docs/architecture/V1_0_RUST_API_AUDIT.md",
    "docs/architecture/V1_COMPATIBILITY_POLICY.md",
}
NORMALIZED_OPERATION_DOCS = {
    "docs/operations/ACCEPTANCE_POLICY.md",
    "docs/operations/ACCESS_RETENTION_AUDIT_CHECKLIST.md",
    "docs/operations/ACCESS_RETENTION_POLICY.md",
    "docs/operations/CADDY_RELAY_PUBLICATION.md",
    "docs/operations/CARRIER_CAPABILITY_NEGOTIATION.md",
    "docs/operations/CARRIER_DECISION_MEMO.md",
    "docs/operations/CONTAINER_EXECUTION_TEMPLATES.md",
    "docs/operations/FILE_ARCHIVE_CARRIER_CONTRACT.md",
    "docs/operations/HTTP_CARRIER_CONTRACT.md",
    "docs/operations/KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md",
    "docs/operations/MIGRATION_AND_SCHEMA_VERSIONING.md",
    "docs/operations/MULTI_NODE_CAPACITY_AND_PLACEMENT_POLICY.md",
    "docs/operations/MULTI_NODE_CONFLICT_POLICY.md",
    "docs/operations/MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md",
    "docs/operations/MULTI_NODE_SYNC_CONTRACT.md",
    "docs/operations/NODE_LIFECYCLE_RUNBOOK.md",
    "docs/operations/OBSERVABILITY.md",
    "docs/operations/OPERATIONAL_PREMISES_MEMO.md",
    "docs/operations/OPERATOR_CLI_CONTRACT.md",
    "docs/operations/QUARANTINE_ADMIN_HTTP.md",
    "docs/operations/QUARANTINE_ANNOTATIONS.md",
    "docs/operations/QUARANTINE_BACKUP_RESTORE.md",
    "docs/operations/QUARANTINE_COMPACTION_PROOF.md",
    "docs/operations/QUARANTINE_CONCURRENCY.md",
    "docs/operations/QUARANTINE_DISMISSALS.md",
    "docs/operations/QUARANTINE_JSONL_MAINTENANCE.md",
    "docs/operations/QUARANTINE_OBSERVABILITY_METRICS.md",
    "docs/operations/QUARANTINE_PERMANENT_REJECTIONS.md",
    "docs/operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md",
    "docs/operations/QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md",
    "docs/operations/QUARANTINE_REPLACEMENT_GENERATION.md",
    "docs/operations/QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md",
    "docs/operations/QUARANTINE_REPLACEMENT_POLICY.md",
    "docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md",
    "docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md",
    "docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md",
    "docs/operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md",
    "docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md",
    "docs/operations/QUARANTINE_REPLACEMENT_V0_4_0_SMOKE_TEST.md",
    "docs/operations/QUARANTINE_SCHEDULER.md",
    "docs/operations/RELAY_STORAGE_SEPARATION.md",
    "docs/operations/SECRET_MANAGEMENT.md",
    "docs/operations/STORAGE_MIGRATION_AND_UPGRADE.md",
    "docs/operations/STORAGE_NODE_QUICKSTART.md",
    "docs/operations/STORAGE_NODE_RUNTIME.md",
    "docs/operations/SUPPORTED_PLATFORMS.md",
    "docs/operations/SYSTEMD_UNIT_TEMPLATES.md",
    "docs/operations/TECH_DECISION_ADR.md",
}
NORMALIZED_PROTOCOL_DOCS = {
    "docs/protocols/CANONICALIZATION.md",
    "docs/protocols/EFFECTIVE_VIEW_DIAGNOSTICS.md",
    "docs/protocols/EFFECTIVE_VIEW_DIAGNOSTIC_CURSOR_LEASE.md",
    "docs/protocols/EFFECTIVE_VIEW_DIAGNOSTIC_PAGINATION.md",
    "docs/protocols/EFFECTIVE_VIEW_DIAGNOSTIC_READ_GUARD.md",
    "docs/protocols/EFFECTIVE_VIEW_DIAGNOSTIC_READ_GUARD_HEARTBEAT.md",
    "docs/protocols/EFFECTIVE_VIEW_DIAGNOSTIC_RETENTION.md",
    "docs/protocols/EFFECTIVE_VIEW_READ_API.md",
    "docs/protocols/HTTP_PUBLISH_SIGNATURE.md",
    "docs/protocols/HTTP_TRANSITION_API.md",
    "docs/protocols/IDENTITY_AND_PROVENANCE.md",
    "docs/protocols/INDEX_GENERATION_DIGEST.md",
    "docs/protocols/LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md",
    "docs/protocols/ORPHAN_TRANSITIONS.md",
    "docs/protocols/PROTOCOL_CONTRACT.md",
    "docs/protocols/PROTOCOL_IDENTIFIERS.md",
    "docs/protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md",
    "docs/protocols/README.md",
    "docs/protocols/TIMESTAMP_SEMANTICS.md",
    "docs/protocols/TRANSITION_AUTHORITY.md",
    "docs/protocols/TRANSITION_EVIDENCE_GENERATION.md",
    "docs/protocols/TRANSITION_OBJECT.md",
    "docs/protocols/TRANSITION_REEVALUATION_COALESCING.md",
    "docs/protocols/TRANSITION_REEVALUATION_QUEUE.md",
    "docs/protocols/TRANSITION_SUPERSESSION.md",
}
NORMALIZED_ROADMAP_DOCS = {
    "docs/roadmap/ROADMAP_TO_V1_0.md",
    "docs/roadmap/V1_0_CANDIDATE.md",
    "docs/roadmap/V1_0_CRASH_MATRIX_DRIVER.md",
    "docs/roadmap/V1_0_DISK_PRESSURE_DRIVER.md",
    "docs/roadmap/V1_0_DOCUMENTATION_FREEZE_PLAN.md",
    "docs/roadmap/V1_0_DOCUMENTATION_WALKTHROUGH.md",
    "docs/roadmap/V1_0_FORMAL_SOAK_COMMAND_MAP.md",
    "docs/roadmap/V1_0_FORMAL_SOAK_SCHEDULER.md",
    "docs/roadmap/V1_0_QUALIFICATION_PLAN.md",
    "docs/roadmap/V1_0_QUALIFICATION_STATUS.md",
    "docs/roadmap/V1_0_REFERENCE_HOST_REHEARSAL.md",
    "docs/roadmap/V1_0_RELEASE_EVIDENCE.md",
    "docs/roadmap/V1_0_SOAK_PLAN.md",
    "docs/roadmap/V1_0_SOAK_REHEARSAL.md",
}
NORMALIZED_SECURITY_DOCS = {
    "docs/security/V0_9_SECURITY_FINDINGS.md",
    "docs/security/V0_9_SECURITY_REVIEW.md",
    "docs/security/V0_9_SIGNATURE_WORKSPACE_REMEDIATION.md",
    "docs/security/V1_0_SECURITY_DIFF_REVIEW.md",
}


def tracked_markdown() -> list[str]:
    cp = subprocess.run(["git", "ls-files", "*.md"], text=True, capture_output=True, check=True)
    return sorted(line for line in cp.stdout.splitlines() if line)


def classify(path: str) -> tuple[str, str, str]:
    if path in REVIEWED_BILINGUAL_REQUIRED:
        return "BILINGUAL_REQUIRED", "KEEP_BILINGUAL", "no"
    if path in BILINGUAL_REQUIRED:
        return "BILINGUAL_REQUIRED", "NORMALIZE_BEFORE_V1", "yes"
    if path in REVIEWED_BILINGUAL_SCOPED:
        return "BILINGUAL_SCOPED", "KEEP_BILINGUAL", "no"
    if path in BILINGUAL_SCOPED:
        return "BILINGUAL_SCOPED", "REVIEW_SCOPE_BEFORE_V1", "yes"
    if path in HISTORICAL_OPERATION_DOCS:
        return "ENGLISH_ONLY", "KEEP_HISTORICAL", "no"
    if path in NORMALIZED_ARCHITECTURE_DOCS or path in NORMALIZED_OPERATION_DOCS or path in NORMALIZED_PROTOCOL_DOCS or path in NORMALIZED_ROADMAP_DOCS or path in NORMALIZED_SECURITY_DOCS:
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
        return "ENGLISH_ONLY", "KEEP_ENGLISH_ADD_BILINGUAL_V1_SUMMARY", "no"
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
    return f"""# Documentation Inventory

> English is normative. This inventory is generated from tracked Markdown files.
> 英語を正本とします。この一覧は追跡対象のMarkdownファイルから生成されます。

## Status

- Tracked Markdown files: **{len(paths)}**
- Release-blocking review entries: **{blockers}**
- Classification totals: {summary}
- Governing policy: [`DOCUMENTATION_POLICY.md`](./DOCUMENTATION_POLICY.md)
- Tracking issue: [#144](https://github.com/nkkmd/lingonberry/issues/144)

## English

Every tracked Markdown file must appear below. `yes` in the final column means the listed action must be resolved before v1.0.0 publication. Classification does not claim that translation or normalization is already complete.

## 日本語

追跡対象のMarkdownファイルは、すべて以下に掲載されなければなりません。最終列が`yes`の項目は、v1.0.0公開前に記載された作業を完了する必要があります。分類済みであることは、翻訳や正規化が完了済みであることを意味しません。

## Inventory

| Current path | Classification | Required action | v1.0 blocker |
|---|---|---|---|
""" + "\n".join(rows) + "\n"


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
