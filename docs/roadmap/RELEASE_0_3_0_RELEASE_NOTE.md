# Lingonberry v0.3.0 Release Notes

**Status: released** | **Release: v0.3.0** | **Released: 2026-07-15**

Lingonberry v0.3.0 adds a verified, recoverable replacement transaction for quarantine ledgers and hardens its operator-facing status, metrics, audit, crash recovery, and generation inspection contracts.

## Highlights

### Verified replacement transaction

Quarantine replacement now follows a proof-bound transaction rather than modifying active ledger files in place.

The transaction requires:

- a verified complete quarantine backup v2;
- a verified QL-5C3B replacement proof;
- journal-bound backup, plan, proof, and runtime fingerprint digests;
- a complete staged ledger set;
- an independently verified sealed generation;
- a single atomic current-generation pointer switch;
- post-switch index and archive-segment verification.

The active ledger namespace is resolved through a generation directory after first publication. Existing root ledgers remain intact and are used only while no current-generation pointer exists.

### Generation-directory publication

Published generations are stored under:

```text
<state-dir>/quarantine-generations/<transaction-id>/
```

The active generation is selected by:

```text
<state-dir>/quarantine-current-generation.json
```

The pointer is replaced by one atomic rename. Readers never accept a pointer-present state by falling back to legacy root ledgers. Missing, contradictory, mixed, or corrupt generation state fails closed.

### Recovery and rollback

The replacement transaction supports deterministic status classification and idempotent operator recovery.

```bash
lingonberry-quarantine-maintenance replacement-status <transaction-dir>
lingonberry-quarantine-maintenance replacement-recover <transaction-dir> --resume
lingonberry-quarantine-maintenance replacement-recover <transaction-dir> --rollback
```

Resume completes an interrupted transaction from its durable journal, publication intent, staged generation, and active pointer state.

Rollback is allowed only before commit. A committed transaction is terminal. Rollback restores the previous generation pointer and rebuilds the derived index, but it does not delete the materialized target generation or transaction workspace.

### Versioned status, metrics, and audit

Structured status uses:

```text
lingonberry-quarantine-replacement-status/v1
```

Prometheus output uses bounded labels only and does not expose transaction IDs, generation digests, filesystem paths, record IDs, secrets, or free-form errors.

Replacement operations emit secret-free append-only audit events. Audit failure prevents a mutating CLI operation from starting. An operation failure that leaves a durable `recovery-required` state is recorded as a recovery failure rather than a generic preflight rejection.

### Deterministic failure injection

Failure injection is disabled by default and requires explicit double opt-in:

```text
LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION=1
LINGONBERRY_REPLACEMENT_FAILURE_POINT=<stable-point-id>
```

The release validates all 18 registered journal, staging, generation, publication, verification, commit, and rollback failure points through direct seams or explicit post-boundary aliases. Failure points are one-shot within a process.

### Read-only generation retention inspection

Operators can classify generation directories without deleting or repairing anything:

```bash
lingonberry-quarantine-maintenance \
  replacement-inspect-generations \
  [transaction-dir ...]
```

The report distinguishes:

- active committed generations;
- previous committed generations;
- rolled-back generations;
- incomplete transaction generations;
- orphan unreferenced generations;
- legacy root layout;
- unknown or corrupt generations.

Orphan and corrupt classifications require manual review. There is no automatic deletion path.

## Upgrade notes

1. Update all Lingonberry workspace packages together to v0.3.0.
2. Export and verify a complete quarantine backup v2 before replacement work.
3. Generate and verify the QL-5C3B replacement preview and proof.
4. Apply replacement with a dedicated, initially absent transaction directory.
5. Record the transaction directory as durable operational evidence.
6. Check structured status and bounded metrics after apply.
7. Verify the quarantine ledger index and archive segments.
8. Inspect retained generation directories before considering any future manual cleanup policy.

A deployment with no current-generation pointer continues to use the legacy root-ledger layout. The first successful replacement publication activates generation-aware resolution without deleting root ledgers.

Existing backup v1 verification and restore compatibility remains as documented. New replacement apply requires complete backup v2.

## Operator workflow

```bash
lingonberry-quarantine-maintenance replacement-preview \
  <verified-backup-v2-dir> \
  <empty-proof-dir>

lingonberry-quarantine-maintenance verify-replacement-proof \
  <proof-dir>

lingonberry-quarantine-maintenance replacement-apply \
  <verified-backup-v2-dir> \
  <verified-proof-dir> \
  <transaction-dir>

lingonberry-quarantine-maintenance replacement-status \
  <transaction-dir>

lingonberry-quarantine-maintenance replacement-metrics \
  <transaction-dir>

lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance verify-segments
```

If status reports `recovery-required`, inspect the classification and choose exactly one recovery direction:

```bash
lingonberry-quarantine-maintenance replacement-recover \
  <transaction-dir> \
  --resume
```

or, only before commit:

```bash
lingonberry-quarantine-maintenance replacement-recover \
  <transaction-dir> \
  --rollback
```

## Release validation

The release passed:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

The JavaScript canonicalization, identity, validation, and crash-point contract tests also passed.

The operator smoke test covers backup export and verification, replacement preview and proof verification, committed apply, versioned status, bounded metrics, index and segment verification, active-generation retention classification, and repeated apply/resume idempotency. Main-branch CI passed after the merge.

## Non-goals and deferred work

v0.3.0 does not add:

- automatic generation deletion;
- automatic transaction-workspace deletion;
- archive-segment rewriting or deletion;
- retention deletion;
- deduplication or event collapse;
- conflict resolution;
- schema migration;
- distributed locking or multi-node consensus;
- remote backup upload;
- backup encryption or cryptographic signing.

Generation and workspace retention remain evidence-preserving and operator-reviewed.

## Tag and release

```text
release commit: efb77415f76b4ba4340536b5b29f5754a1173d59
tag: v0.3.0
GitHub Release: Lingonberry v0.3.0
release state: published as the latest stable release
```

The published tag remains immutable. Post-release documentation updates are committed to `main` and do not alter the v0.3.0 release artifact.
