# Access and Retention Policy

**Status: v1.0.0 pre-release**  
**Normative language: English**

This document defines the access and retention boundaries implemented or relied on by the v1.0.0 pre-release line. It separates protocol metadata from operator policy and does not claim that future private-object, deletion, or lifecycle features already exist.

Lingonberry v1.0.0 has not been published. The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Evidence and documentation commits after that candidate do not redefine it.

## 1. Governing principles

- Access scope and retention hints are descriptive policy metadata, not authorization credentials.
- The protocol defaults are `accessScope=public` and `retentionHint=long-lived`.
- `long-lived` is not a legally or operationally fixed expiration period.
- The current core does not provide a general private or encrypted object mode.
- The current storage command surface does not provide a general delete or tombstone command for canonical records.
- Operators must not physically remove active canonical storage merely because a retention hint is shorter than local policy.
- Active storage, quarantine, audit records, backups, temporary workspaces, and release evidence are distinct retention domains.
- Authentication secrets are governed by [Secret Management](./SECRET_MANAGEMENT.md), not by protocol metadata.

## 2. Protocol and carrier metadata

The implemented protocol defaults are:

```text
accessScope = public
retentionHint = long-lived
```

These values communicate the expected handling of an accepted object. They do not:

- grant access to an administrator interface;
- encrypt content;
- create an automatic deletion timer;
- authorize removal from canonical storage;
- override local legal, incident-response, or evidence-preservation obligations;
- guarantee that every carrier implements independent retention scheduling.

Carrier capability output may expose these defaults. Automation should treat them as declared capability metadata and not as proof that storage has already expired or been deleted.

## 3. Access boundaries

### 3.1 Public data surface

The v1 public relay accepts and serves the implemented public object and publish surfaces. Validation and acceptance determine whether input is stored, deferred, or rejected. Acceptance does not certify factual truth.

The public listener must not expose administrator-only routes. Administrator routes belong on the separately configured authenticated admin listener.

### 3.2 Administrator surface

Administrator access is controlled by bearer credentials and role-based authorization:

- `observer` may use observe operations;
- `reviewer` may observe and annotate;
- `operator` may observe, annotate, and operate.

This authorization model applies to the implemented administrator and quarantine surfaces. It does not make canonical protocol objects private.

### 3.3 Private and encrypted objects

Private membership distribution, encrypted object payloads, per-object ACLs, and confidential canonical storage are outside the current v1 contract. An external deployment may add a restricted network boundary, but it must not describe that deployment choice as an implemented core private-object feature.

## 4. Active canonical storage

Active canonical storage includes the raw publish log, canonical catalog, format state, migration journal when present, and derived index state required by the implementation.

The operator must preserve enough source state to:

- read canonical records;
- replay the append history;
- verify or rebuild derived index state;
- inspect storage format and migration state;
- create verified backups and isolated restores.

The current command contract does not define general deletion of individual canonical records. Therefore:

- do not document `delete` as an implemented tombstone operation;
- do not edit the raw log or canonical catalog manually;
- do not use `index rebuild` as a deletion or corruption-repair mechanism;
- do not replace active storage with an unverified archive or restore target.

Any future deletion, tombstone, compaction, or scrub mechanism requires a separately versioned contract and migration path.

## 5. Quarantine retention

Deferred input may be retained in the quarantine subsystem for review. Quarantine records, annotations, promotion results, resolutions, and permanent-rejection records are operational state distinct from accepted canonical storage.

Operators must preserve quarantine state when it is needed for:

- review or promotion decisions;
- permanent-rejection evidence;
- incident investigation;
- qualification or soak evidence;
- audit of administrator actions.

Promotion may create or identify a canonical record, but it does not imply that all quarantine evidence may immediately be deleted. Permanent rejection prevents the rejected quarantine record from being promoted through the implemented administrator flow; it is not deletion of an accepted canonical object.

No default automatic quarantine-expiry scheduler is part of the v1 contract. Local cleanup requires an explicit policy that preserves required audit and incident evidence.

## 6. Administrator authentication audit

Authentication and authorization failures are appended to:

```text
<state-dir>/admin-auth-audit.jsonl
```

The audit record contains operational metadata such as attempt time, remote address, method, path, resolved role when available, and outcome code. It does not contain the bearer token value.

Retention requirements:

- protect the file with the same care as other security-relevant state;
- preserve relevant records during an active incident or credential investigation;
- do not publish raw remote-address data without reviewing privacy and disclosure requirements;
- do not rewrite records merely to reduce their size;
- never add token values to derived reports or evidence bundles.

The implementation does not define a universal audit-log expiration period. The deployment owner must define one consistent with incident-response and applicable legal requirements.

## 7. Backup and restore retention

A verified storage backup is a recovery artifact, not an alternate active node and not an automatic retention scheduler.

Operators must:

- create backups in the configured backup root or another explicitly approved destination;
- verify a backup before relying on it;
- preserve the manifest and files as one bound artifact;
- avoid modifying backup contents in place;
- restore only to an isolated missing or empty target;
- verify the restored target before considering any active-path switch;
- retain at least one known-good recovery point before migration, upgrade, or destructive operator action.

Backup creation, migration backup, archive export, and qualification evidence are related but distinct artifacts. Their manifests and verification rules must not be treated as interchangeable.

Post-commit migration recovery follows the v1 upgrade and rollback runbook; the migration primitive does not provide arbitrary record-level retention or deletion.

## 8. Temporary and derived state

Temporary workspaces and reproducible derived state may be removed only when the owning operation has completed or been safely abandoned.

Examples include:

- isolated restore drill targets;
- temporary verification directories;
- generated index state that can be rebuilt from intact canonical storage;
- transient qualification workspaces.

Before cleanup, confirm that the path is not:

- active state or data storage;
- the configured backup root;
- a migration-controlled path;
- evidence required by an open incident, qualification, or soak run;
- a target still needed to diagnose a failed operation.

A path under a temporary directory is not automatically safe to delete while an operation is active.

## 9. Release, qualification, and soak evidence

Qualification and soak artifacts must be retained long enough to support the release decision and subsequent audit. Evidence retention is candidate-bound.

Operators must preserve:

- the exact candidate commit identity;
- harness and workflow identity;
- timestamps and host context required by the evidence contract;
- command outputs and classified results;
- hashes or manifests used to bind the bundle;
- failure evidence until disposition is documented.

Evidence must not contain administrator tokens, secret environment files, or other live credentials. Redaction must not alter the result-bearing fields required to verify the evidence.

Formal 72-hour soak evidence does not exist until the formal run has actually started and completed under its governing contract. Rehearsal evidence must not be relabeled as formal-soak evidence.

## 10. Node retirement

Before retiring a node:

1. stop new writes according to the operator runbook;
2. record the exact running version and configured paths;
3. create and verify a final backup or export appropriate to the recovery contract;
4. perform an isolated restore or recovery verification when required;
5. preserve security, quarantine, migration, and release evidence still under retention;
6. document the disposition of active storage and every retained copy;
7. remove secrets separately from data disposal;
8. only then remove local active and temporary paths according to the approved disposal policy.

Retirement is an operator lifecycle action. It is not evidence that individual protocol objects were semantically deleted from every copy or carrier.

## 11. Data disposal and scrub boundary

The v1 implementation does not define a general secure-erase or canonical-record scrub command. Filesystem deletion alone may not guarantee physical media sanitization, snapshot removal, remote backup removal, or deletion from another carrier.

When disposal is required, the deployment owner must account for:

- active storage;
- verified backups and migration backups;
- filesystem snapshots;
- exported archives;
- quarantine and administrator audit state;
- qualification and incident evidence;
- external log or backup systems;
- the underlying media-sanitization requirement.

Do not claim global deletion unless all controlled copies and applicable external systems have been addressed.

## 12. Operator verification checklist

Before changing retention or disposing of state, verify:

- the exact node and candidate or release version;
- resolved state, data, backup, and temporary directories;
- storage doctor and strict verification results;
- migration journal stage;
- backup verification status;
- open incidents or security investigations;
- quarantine records requiring disposition;
- formal qualification or soak evidence requirements;
- secret rotation and audit-preservation requirements;
- legal or organizational retention constraints outside this repository.

## 13. Non-goals for v1

The current contract does not promise:

- private or encrypted canonical objects;
- per-object ACL enforcement in the protocol core;
- automatic object expiration;
- a general delete or tombstone command;
- online compaction or secure erase;
- globally coordinated deletion across carriers;
- a repository-defined universal number of retention days;
- automatic quarantine or audit-log purging.

## 14. Related documents

- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability Contract](./OBSERVABILITY.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Quarantine Admin HTTP](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
