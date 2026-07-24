# Quarantine Replacement Smoke Test

**Status: normative v1.0 pre-release test and rehearsal procedure** | **Last reviewed: 2026-07-24**

This procedure validates the checked-in quarantine replacement workflow with disposable state directories. It covers backup, preview, apply, status, bounded metrics, index and segment verification, generation inspection, idempotent recovery behavior, and selected injected-failure paths.

It is not the formal 72-hour soak and is not privileged reference-host qualification.

## 1. Safety boundary

Use only disposable fixtures. Do not point the smoke test at a production state directory.

The test must not:

- delete production generations or workspaces;
- rewrite immutable evidence;
- remove archive segments;
- enable unattended cleanup;
- expose credentials in logs or evidence bundles;
- treat a successful test as release authorization.

Failure injection must be explicitly enabled and limited to a dedicated test process.

## 2. Required fixture classes

Maintain isolated fixtures for:

```text
legacy root-ledger layout
generation layout with an active committed generation
previous committed generation
rolled-back generation
stale runtime fingerprint or pointer state
altered managed-path inventory
interrupted pre-switch transaction
interrupted post-switch transaction
rollback-capable transaction
```

The fixtures may be created programmatically by tests. They do not need to preserve historical release binaries unless the specific exercise is an upgrade rehearsal.

## 3. Baseline automated smoke test

The checked-in Rust test:

```text
packages/core/tests/quarantine_replacement_operator_smoke.rs
```

covers this baseline sequence:

1. create a disposable legacy-root quarantine state;
2. export a complete archive-inclusive backup;
3. verify the backup;
4. create a replacement preview and proof;
5. verify the replacement proof;
6. apply a replacement transaction;
7. require terminal `committed` state;
8. require the transaction generation to be active;
9. read structured committed status;
10. render bounded replacement metrics and verify that the transaction ID is absent;
11. verify the derived quarantine index;
12. verify archive segments;
13. inspect generations and classify the transaction as `active-committed-generation`;
14. repeat apply against the same committed workspace;
15. resume the committed transaction;
16. require both repeated operations to remain `committed` without a second generation switch.

This automated test validates the library workflow. It does not exercise service-manager configuration, administrator authentication, TLS publication, filesystem mount semantics, or multi-host behavior.

## 4. CLI rehearsal sequence

For a disposable state directory, the operator rehearsal should execute the checked-in CLI surfaces in this order:

```text
backup export and verification
replacement-preview
verify-replacement-proof
replacement-apply
replacement-status
replacement-metrics
verify-index
verify-segments
replacement-inspect-generations
replacement-recover --resume against committed state
```

Record canonical stdout and stderr. Verify that:

- status version is `lingonberry-quarantine-replacement-status/v1`;
- state and classification are `committed`;
- `targetGenerationActive` is true;
- metrics use bounded state, layout, target, and phase labels;
- metrics contain no transaction ID, generation digest, filesystem path, or record ID;
- index and segment verification succeed;
- generation inspection reports the active generation as committed and not requiring manual review;
- repeated apply or resume does not create another generation or duplicate a terminal transition.

## 5. Failure-injection rehearsal

Use the registry:

```text
docs/operations/quarantine-replacement-crash-points.v1.json
```

Enable one registered point per process:

```text
LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION=1
LINGONBERRY_REPLACEMENT_FAILURE_POINT=<registered-point>
```

At minimum, exercise separate fixtures for:

- a pre-switch boundary such as `publication.pointer-rename`;
- a post-switch boundary such as `publication.index-rebuild`;
- a commit boundary such as `publication.commit-transition`;
- a rollback boundary such as `rollback.pointer-restore`.

For each injected failure:

1. record the registered point and expected classification;
2. require the command to fail or return `recovery-required` as defined by the registry;
3. run `replacement-status` without editing artifacts;
4. compare the observed pointer visibility and classification with the registry;
5. run only an allowed recovery action;
6. verify the final pointer, journal state, index, and archive segments;
7. verify immutable evidence remains byte-identical;
8. verify repeated recovery is idempotent.

Production service configuration must not retain either failure-injection environment variable.

## 6. Retention and cleanup validation

Generation inspection and retention validation remain non-destructive until a separate cleanup transaction is explicitly prepared.

Validate that:

- the active generation is ineligible;
- at least one previous committed generation remains protected by the retention floor;
- disabled classifications are rejected;
- missing durable age evidence is rejected;
- orphan, incomplete, corrupt, unknown, and legacy-root classifications require manual review or remain ineligible;
- exact selection rejects duplicates, path syntax, wildcards, and implicit-all behavior;
- the cleanup preview binds the complete eligible generation set;
- changing the active pointer, journal, completion evidence, generation digest, runtime fingerprint, or managed-path inventory causes stale-proof rejection.

Do not execute irreversible deletion in a general replacement smoke test. Destructive cleanup requires the dedicated cleanup runbook, isolated fixtures, explicit acknowledgement, and recovery evidence.

## 7. Cleanup transaction fixture

When testing cleanup implementation separately, use disposable generations and verify:

- reversible tomb preparation preserves a sealed inventory;
- rollback before irreversible deletion restores every subject;
- deletion does not begin without explicit destructive acknowledgement;
- path-level deletion progress is durable;
- interrupted deletion classifies as `recovery-required` or `partially-deleted` according to the journal frontier;
- missing tomb entries are reconciled against the sealed inventory;
- terminal cleanup workspaces remain retained;
- there is no scheduled or unattended cleanup entry point.

A cleanup test result must not be combined with replacement apply status as though they were one transaction.

## 8. Evidence record

Retain:

```text
repository commit under test
binary version or build identity
state-directory fixture identity
backup manifest and verification result
replacement plan and proof digests
transaction journal and digest
structured status output
bounded metrics output
relevant replacement audit lines
failure-injection registry version and selected point
current-generation pointer evidence
index verification result
segment verification result
generation inspection report
cleanup proof, journal, tomb inventory, and progress evidence when cleanup is tested
CI workflow and test identifiers
```

Do not publish bearer tokens, private keys, environment secrets, full user payloads, or unrelated record data.

## 9. Pass criteria

The smoke test passes only when:

- all invoked Rust and JavaScript tests succeed;
- backup and proof verification succeed;
- apply reaches `committed`;
- the expected generation is active;
- status, metrics, index, segment, and inspection checks succeed;
- repeated apply or resume is idempotent;
- every injected failure matches the registered state and allowed recovery action;
- recovery reaches a verified terminal state;
- stale or altered state fails closed;
- no automatic generation or workspace deletion occurs;
- retained evidence is sufficient to explain every terminal or interrupted fixture.

## 10. Release boundary

Smoke-test success is ordinary technical evidence only. It does not establish:

- formal 72-hour soak completion;
- privileged reference-host qualification or rehearsal completion;
- production backup restoration rehearsal;
- distributed deployment safety;
- version update approval;
- release PR approval;
- tag or GitHub Release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. This procedure and its results do not redefine that candidate.
