# Quarantine Annotation Contract

**Status: implemented v1.0 pre-release contract** | **Last updated: 2026-07-24**

This document defines the normative v1.0 pre-release contract for operator annotations attached to quarantine records.

Annotations are append-only operational notes. They do not mutate the quarantined publish request, do not change lifecycle state, and do not alter acceptance or promotion decisions.

## 1. Scope

The contract covers:

- annotation event shape;
- active and archived annotation ledgers;
- CLI and administrative HTTP surfaces;
- validation and authorization behavior;
- append-only correction practice;
- interactions with promotion, permanent rejection, rotation, backup, and audit evidence.

It does not define a general-purpose case-management system.

## 2. Persistence contract

The active annotation ledger is:

```text
<runtime-state-dir>/quarantine-annotations.jsonl
```

The runtime state directory is resolved by the same storage configuration used by the relay and quarantine store. Operators must not assume the obsolete generic `LINGONBERRY_STATE_DIR` name when a deployment uses the normalized storage configuration variables.

Each line is one canonical JSON annotation event:

```json
{
  "id": "lb:qa:...",
  "quarantineId": "lb:q:...",
  "annotatedAt": "...Z",
  "operator": "operator-name",
  "note": "reviewed source material"
}
```

Required fields:

| Field | Contract |
|---|---|
| `id` | Implementation-generated annotation identifier with the `lb:qa:` prefix. |
| `quarantineId` | Existing quarantine record identifier. |
| `annotatedAt` | Implementation-generated UTC timestamp string. |
| `operator` | Non-empty operational actor identifier after trimming. |
| `note` | Non-empty free-text note after trimming. |

The implementation appends canonical JSON and does not rewrite earlier events.

## 3. Active and archived ledgers

Annotation reads use the managed-ledger reader and therefore include both archived and active annotation segments when ledger rotation has occurred.

Consequences:

- rotating `quarantine-annotations.jsonl` does not remove earlier annotations from normal reads;
- backup and restore procedures must preserve active files, archived segments, and the associated managed-ledger index;
- an active file alone is not a complete annotation history after rotation;
- compaction, retention, and archive deletion must follow the dedicated quarantine ledger procedures rather than deleting files ad hoc.

## 4. CLI surface

Append an annotation:

```bash
lingonberry-relay quarantine-annotate \
  <quarantine-id> \
  <operator> \
  <note>
```

Quote notes containing shell metacharacters or whitespace:

```bash
lingonberry-relay quarantine-annotate \
  lb:q:123 \
  operator-42 \
  "source identity requires follow-up"
```

List all annotations:

```bash
lingonberry-relay quarantine-annotations
```

List annotations for one quarantine record:

```bash
lingonberry-relay quarantine-annotations lb:q:123
```

CLI access is local process access. The CLI does not add a separate authentication layer; host access controls, service-user permissions, and filesystem permissions remain deployment responsibilities.

## 5. Administrative HTTP surface

Annotations are available only on the authenticated administrative listener.

Append:

```text
POST /v1/quarantine/<quarantine-id>/annotations
Authorization: Bearer <reviewer-or-operator-token>
Content-Type: application/json
```

```json
{
  "operator": "operator-42",
  "note": "source identity requires follow-up"
}
```

List:

```text
GET /v1/quarantine/<quarantine-id>/annotations
Authorization: Bearer <observer-reviewer-or-operator-token>
```

The public relay listener must return `404` for these paths. The administrative listener applies the authorization order defined in [Quarantine Admin HTTP](./QUARANTINE_ADMIN_HTTP.md): route classification, authentication, permission check, then body read and execution.

Permission mapping:

- observer: read annotations;
- reviewer: read and append annotations;
- operator: read and append annotations.

Unauthorized mutation bodies are not interpreted before denial.

## 6. Validation and failure behavior

Append operations reject:

- a quarantine identifier that does not resolve to an existing record;
- an empty operator after trimming;
- an empty note after trimming;
- a non-object administrative HTTP request body;
- missing required HTTP request fields;
- malformed JSON;
- I/O or lock acquisition failures.

Annotation reads fail closed when a managed ledger entry is corrupt, not an object, or missing a required string field. The implementation does not silently skip corrupt annotation events.

Representative storage error codes include:

```text
LB_QUARANTINE_NOT_FOUND
LB_QUARANTINE_ANNOTATION
LB_QUARANTINE_CORRUPT
LB_QUARANTINE_IO
```

Operators must preserve the original evidence before attempting repair. Directly editing JSONL ledgers is not a supported recovery procedure.

## 7. Append-only semantics

The following rules are normative:

- do not modify the original quarantine record;
- do not modify the resolution ledger through annotation operations;
- do not update or delete an existing annotation;
- allow multiple annotations for the same quarantine record;
- append corrections as new annotation events;
- do not interpret a note string as a machine-readable lifecycle transition.

A correction should identify the earlier annotation in human-readable form and append the corrected information. The implementation does not provide supersession links or automatic redaction.

## 8. Lifecycle interactions

An annotation is not a lifecycle state.

Its presence or absence does not:

- promote a record;
- permanently reject a record;
- suppress scheduler or manual promotion attempts;
- override acceptance policy;
- change duplicate or conflict classification;
- make a quarantined request valid.

Promotion re-runs the implemented validation and acceptance path. Permanent rejection is recorded through its separate append-only contract. Annotation routes do not bypass either mechanism.

Annotations may remain readable after promotion or permanent rejection because they are historical operational evidence associated with the quarantine identifier.

## 9. Audit and sensitive data

Annotations are operational records and may be included in backups, incident evidence, and qualification bundles. They are distinct from the administrative authentication-failure ledger.

Operators must not place the following in `operator` or `note`:

- bearer tokens or other credentials;
- private keys;
- authentication headers;
- unnecessary personal data;
- full quarantined payloads when a stable identifier is sufficient;
- unbounded or attacker-controlled text copied without review.

Free-text notes must not be used as metric labels. Access to annotation files must be limited to the service account and authorized operators.

The `operator` field is caller-supplied metadata. It is not cryptographically bound to the bearer credential or operating-system identity. Deployments requiring stronger attribution must add external controls and must not claim that the current field alone proves actor identity.

## 10. Concurrency and locking

Append operations acquire the quarantine lock before validating the target record and writing the event. This provides the implementation's local-filesystem serialization boundary.

This contract does not guarantee:

- distributed locking across hosts;
- concurrent writers on a shared network filesystem;
- multi-node annotation replication;
- transactionality across annotation, promotion, and permanent-rejection ledgers.

Multi-node deployments require a separately qualified coordination design.

## 11. Verification

After appending an annotation, verify:

1. the command or HTTP request succeeded;
2. the returned event contains the expected quarantine identifier, operator, and note;
3. a filtered list operation returns the new event;
4. earlier events remain present and unchanged;
5. the public listener still returns `404` for annotation administration paths;
6. observer credentials cannot append;
7. reviewer and operator credentials can append;
8. backup evidence includes active and archived annotation ledgers where applicable.

A successful unit test or documentation walkthrough is not privileged reference-host qualification and is not the formal 72-hour soak.

## 12. Non-goals

The v1.0 pre-release annotation contract does not provide:

- annotation update or deletion;
- automatic redaction;
- structured labels, workflow states, or assignment queues;
- cryptographic actor attribution;
- per-annotation ACLs;
- browser sessions, OAuth, or OIDC;
- automatic retention or compaction policy;
- distributed consensus or replication;
- lifecycle transitions derived from note content.

## References

- [Quarantine Admin HTTP](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Permanent Rejections](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [Quarantine Backup and Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Quarantine Compaction Proof](./QUARANTINE_COMPACTION_PROOF.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)

## Release boundary

This document does not redefine the fixed v1.0.0 candidate commit:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

The formal 72-hour soak has not been performed. Privileged reference-host qualification and rehearsal remain incomplete. Version update, release PR, tag creation, and GitHub Release publication remain incomplete.