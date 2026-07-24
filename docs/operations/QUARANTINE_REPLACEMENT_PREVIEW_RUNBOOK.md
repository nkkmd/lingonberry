# Quarantine Replacement Preview Runbook

**Status: implemented** | **v1.0 pre-release normative operator procedure**

This runbook generates, verifies, and reviews a non-mutating policy-v2 quarantine replacement preview. A verified preview is input to later transaction preparation; it does not authorize or perform mutation.

## Preconditions

Prepare:

- the runtime quarantine state directory;
- a verified archive-inclusive backup with version `lingonberry-quarantine-backup/v2`;
- a new empty proof output directory;
- the reviewed `lingonberry-quarantine-maintenance` binary; and
- an evidence directory outside the runtime state directory.

Quiesce public ingestion, administrator mutation routes, scheduled maintenance, operator CLI writers, and any other process that can change managed ledgers, segments, indexes, generations, or pointers.

Preview does not acquire the mutation lock. The before/after runtime fingerprint check detects observed changes during the scan but does not provide a transactional snapshot.

Record before execution:

```bash
git rev-parse HEAD
lingonberry-quarantine-maintenance status
```

Also retain the binary identity, runtime-state location, backup-manifest digest, segment verification output, operator identity, and start time.

## Verify source inputs

Verify the archive segments and backup before preview. Stop on any error.

The input backup must be v2. Backup v1 is not archive-inclusive and is not accepted for replacement preview.

Do not proceed when:

- a segment or manifest is corrupt;
- the backup does not verify;
- the backup version is unsupported;
- the output directory is non-empty;
- the runtime is not quiescent; or
- the runtime path is ambiguous or resolves to an unintended state directory.

## Generate the preview

```bash
lingonberry-quarantine-maintenance replacement-preview \
  <verified-backup-v2-dir> \
  <empty-proof-dir>
```

Runtime state and backup are read-only inputs. Only the supplied output directory may be written.

Expected final artifacts:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
quarantine-replacement-proof.json
quarantine-replacement-proof.digest
```

A successful proof must report:

```text
mutationAllowed = false
rewritePerformed = false
```

File presence alone is not success. Require command success and independent artifact verification.

## Verify the proof

```bash
lingonberry-quarantine-maintenance verify-replacement-proof \
  <proof-dir>
```

Verification checks digest pairs, supported versions, exact managed-ledger membership, immutable-ledger restrictions, logical ordinals, replacement-key uniqueness, one-to-one provenance, parsed-value equivalence, aggregate counts, ordering, and required semantic-equivalence fields.

Artifact verification does not re-read the runtime state or original backup. It does not prove the runtime still matches the recorded fingerprint and does not authorize apply.

Stop when verification fails. Preserve the runtime state, verified backup, complete proof directory, stdout, stderr, exit status, and stable error code. Do not edit the artifacts manually.

## Review the plan

For immutable evidence ledgers, all decisions must preserve bytes:

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

For terminal single-event ledgers, only these decisions are permitted:

```text
retain-byte-for-byte
canonical-json-representation
```

Canonical representation may alter insignificant whitespace, object-key order, and line termination only. It must preserve the parsed JSON value, logical order, terminal state, state-derived metrics, promotion eligibility, reader behavior, and idempotent/conflicting action results.

Reject the preview if it proposes or implies:

- deletion or retention cleanup;
- deduplication;
- event merging or splitting;
- conflict resolution by choosing a winner;
- schema migration or default insertion;
- unknown-field removal;
- immutable-ledger replacement;
- archive-boundary movement; or
- a replacement outside the exact managed-ledger set.

Review source locations and line numbers for a sample that includes active-ledger and archived-segment entries. Confirm that each terminal replacement maps to one exact source line.

## Reproducibility check

With writers still quiesced, run preview again against the same state and verified backup using another empty output directory.

These files must be byte-identical:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
```

The proof may have a different generation timestamp. The timestamp is outside the deterministic plan digest.

A reproducibility mismatch is a stop condition. Preserve both artifact directories and do not select either result for apply preparation.

## Error handling

Use the stable `LB_QUARANTINE_REPLACEMENT_*` error family for classification. Human-readable messages are diagnostic and must not be the sole automation contract.

Common stop conditions include:

- `BACKUP`: backup verification or version failure;
- `CHANGED`: runtime fingerprint changed during preview;
- `CONFLICT` or `CORRUPT`: duplicate or conflicting terminal state;
- `POLICY`: unsupported or forbidden transformation;
- `PROOF`: malformed, incomplete, or digest-mismatched artifact; and
- `SEMANTICS`: equivalence check failed.

Treat every failure as fail closed. Do not retry with an old backup after runtime state changes. Produce and verify a new backup, then repeat the procedure from the beginning.

Partial or conflicting final artifact sets require manual review or removal before retry. Do not infer success from temporary files or a subset of final files.

## Evidence bundle

Retain:

- application commit and binary identity;
- source state identity and path;
- verified backup v2 and manifest;
- segment verification output;
- both command invocations and exit statuses;
- plan, proof, and digest files;
- reproducibility comparison result;
- pre/post status output;
- stdout and stderr;
- stable error codes, if any;
- operator identity and timestamps; and
- review decision and selected proof-directory identity.

Do not place credentials or secret-bearing environment dumps in the evidence bundle.

## Handoff to apply preparation

A preview is eligible for handoff only when:

1. both preview and verification commands succeeded;
2. the artifact set is complete;
3. `mutationAllowed` and `rewritePerformed` are false;
4. manual policy review found no forbidden transformation;
5. the reproducibility check passed; and
6. the evidence bundle is complete.

Apply preparation must independently revalidate the proof, backup, segments, runtime fingerprint, and transaction preconditions. The preview procedure does not keep writers locked between preview and apply.

## Safety boundary

This procedure does not:

- rewrite active or archived ledgers;
- stage replacement ledgers;
- create or advance a replacement transaction;
- seal or publish a generation;
- switch the current-generation pointer;
- perform rollback or recovery;
- authorize retention cleanup; or
- establish formal soak or privileged reference-host qualification.

## Related documents

- [`QUARANTINE_REPLACEMENT_PREVIEW.md`](./QUARANTINE_REPLACEMENT_PREVIEW.md)
- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)
- [`QUARANTINE_REPLACEMENT_TRANSACTION.md`](./QUARANTINE_REPLACEMENT_TRANSACTION.md)
- [`QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)
