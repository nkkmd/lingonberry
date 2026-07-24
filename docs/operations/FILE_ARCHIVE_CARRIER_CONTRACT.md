# File / Archive Carrier Contract

**Status: v1.0 pre-release normative**

## Purpose

This document defines the implemented file/archive carrier used to export accepted Lingonberry records into a portable directory and to import those records through the current validation, acceptance, finalization, duplicate, conflict, storage, and quarantine contracts.

The archive carrier is a transfer and replay format. It is not the reference-node storage backup format, a byte-for-byte storage image, a migration backup, or proof that an archive has been preserved durably.

## Implemented command surface

The reference relay binary exposes:

```bash
lingonberry-relay export-archive <archive-dir>
lingonberry-relay import-archive <archive-dir>
```

The commands operate on the runtime storage backend selected by the relay process. They are CLI operations; no public HTTP archive export or import route is part of the v1 contract.

A successful export prints canonical JSON containing:

- `ok`
- `archiveDir`
- `manifestPath`
- `wireLogPath`
- `catalogPath`
- `recordCount`

A successful import prints canonical JSON containing:

- `ok`
- `archiveDir`
- `recordCount`
- `duplicateCount`

`recordCount` on import counts newly stored records. Exact duplicates are counted separately in `duplicateCount`.

## Archive directory layout

The implemented logical and physical layout is exactly:

```text
<archive-dir>/
├── manifest.json
├── wire-log.jsonl
└── canonical-catalog.jsonl
```

The implementation creates the destination directory when necessary. The v1 command accepts a directory path; tar, zip, object storage packaging, encryption, compression, signing, and remote transfer are outside this command contract.

### `manifest.json`

The exporter writes one canonical JSON object followed by a newline. Its implemented fields are:

- `archiveVersion`
- `capabilityVersion`
- `protocolVersion`
- `carrierKind`
- `createdAt`
- `itemCount`
- `schemaVersions.knowledgeObject`
- `schemaVersions.httpPublishRequest`
- `policy.defaultAccess`
- `policy.defaultRetention`
- `policy.privateEnabled`
- `policy.scrubMode`
- `paths.manifest`
- `paths.wireLog`
- `paths.catalog`

The current manifest declares:

- archive carrier kind
- the repository-defined current archive and protocol versions
- the current knowledge-object and HTTP publish-request schema versions
- default access `public`
- default retention `long-lived`
- private-object support disabled
- scrub mode as operator-controlled

The importer currently validates only the required compatibility gates:

- `archiveVersion` equals the running implementation's archive version
- `protocolVersion` equals the running implementation's protocol version
- `carrierKind` equals the archive carrier identifier

Missing or mismatched values fail import with `LB_ARCHIVE_IMPORT`. Operators must not infer that every other manifest field is cryptographically authenticated or exhaustively validated.

### `wire-log.jsonl`

This is the replay input used by import. Each line is canonical JSON with:

- `storedAt`
- `canonicalId`
- `carrierIdentity`
- `requestJson`

`requestJson` contains the original publish-request JSON string retained by storage. Import reparses that string and requires an object containing `object`.

The importer does not trust the archived `canonicalId`, `carrierIdentity`, or `storedAt` as authority for a new write. It validates, evaluates acceptance, finalizes, derives identity, and appends again through the current runtime contracts.

### `canonical-catalog.jsonl`

The exporter writes a convenience projection with one canonical JSON object per stored record:

- `storedAt`
- `canonicalId`
- `carrierIdentity`
- `object`

The current importer does not read this file. It is an exported inspection and interoperability projection, not the authoritative replay source. Import authority comes from `manifest.json`, `wire-log.jsonl`, and the running implementation's validation and storage contracts.

## Export semantics

Export performs these steps:

1. enumerate canonical IDs from the active backend;
2. require both a retained raw publish request and a catalog record for every ID;
3. write `wire-log.jsonl` from retained raw-request records;
4. write `canonical-catalog.jsonl` from catalog records;
5. write `manifest.json` with the exported item count.

Export fails rather than silently omitting an object when the raw request or catalog record is missing.

The exporter writes the full currently listed dataset. The v1 implementation does not provide:

- incremental or differential export;
- a base checkpoint or cursor;
- selection by object type, time, or ID;
- an archive scrub transformation;
- archive encryption or signature generation;
- automatic destination cleanup;
- an atomic directory replacement guarantee.

Operators should export into a new or controlled destination and inspect the resulting paths and counts. Reusing a directory containing unrelated or stale files is not a portable contract.

## Import semantics

Import performs these steps in wire-log order:

1. read and parse `manifest.json`;
2. enforce archive-version, protocol-version, and carrier-kind compatibility;
3. read `wire-log.jsonl` line by line;
4. parse each wire-log record and its `requestJson`;
5. require a publish-request object containing `object`;
6. run full knowledge-object validation;
7. load the active acceptance policy from the environment;
8. accept, reject, or defer according to that policy;
9. finalize accepted objects;
10. append them through the active storage backend;
11. count exact duplicates separately from newly stored records.

### Accepted records

Accepted records are finalized and appended. The active duplicate-and-conflict contract applies; archived identifiers do not bypass it.

### Exact duplicates

An exact duplicate is idempotent. It is not stored again and increments `duplicateCount`.

### Conflicts

A canonical-ID or carrier-identity conflict fails import through the storage contract. It is not converted into a duplicate or policy rejection.

### Rejected records

A hard validation failure or active policy rejection stops import and returns the classified error. Import is not an all-or-nothing transaction: records appended before the failing line remain stored.

### Deferred records

When the active acceptance policy returns Defer, the request is written to the runtime quarantine store. Import then stops and reports the acceptance code together with the generated quarantine ID.

A deferred record is not appended to canonical storage. Quarantine persistence failure is an operational import failure.

### Operational failures

Malformed JSONL, missing `requestJson`, missing `object`, finalization failure, conflict, storage error, quarantine error, or I/O failure stops import. Operators must preserve the archive and record the failing line or command evidence before retrying.

## Partial-import and retry boundary

The importer is sequential and non-transactional across the complete archive. A failed import may have already stored an earlier prefix.

Before retrying:

1. record the target commit, archive manifest, command, error code, and active acceptance policy;
2. inspect the destination storage and quarantine state;
3. determine whether earlier records were newly stored, duplicated, deferred, or failed;
4. correct the cause without editing evidence in place;
5. rerun only when exact-duplicate handling makes the expected retry safe.

A successful retry can legitimately report duplicates for the prefix imported during the earlier attempt.

## Archive versus backup, restore, and migration

These artifacts are not interchangeable.

| Artifact | Primary purpose | Authoritative mechanism |
|---|---|---|
| Archive carrier | Portable semantic replay through current contracts | `export-archive` / `import-archive` |
| Storage backup | Recover the reference storage layout and verified backup contents | `lingonberry-storage backup` / `restore` |
| Migration backup | Protect a storage-format migration operation | `lingonberry-storage-migrate` workflow |
| Qualification evidence | Demonstrate checks performed against a named candidate and environment | qualification/soak tooling and evidence bundle |

An archive does not preserve all node state. In particular, operators must not assume it contains:

- SQLite database bytes or storage-format manifest;
- quarantine records, annotations, resolutions, or permanent rejections;
- administrator authentication audit records;
- migration journal or migration backup;
- service configuration, credentials, systemd state, or reverse-proxy configuration;
- formal-soak or incident evidence.

Use the storage recovery contract for node recovery. Use archive import when semantic replay through the current software and policy is intended.

## Access, retention, secrets, and redaction

The exporter includes retained publish-request JSON. Archives may therefore contain publisher metadata, object content, provenance, timestamps, and other operationally sensitive material even when the protocol default access scope is `public`.

The v1 exporter does not scrub or redact content. `policy.scrubMode = operator-controlled` is descriptive policy metadata, not evidence that scrubbing occurred.

Operators must:

- protect archive directories according to their actual contents;
- avoid adding environment files, administrator tokens, private keys, or unrelated logs;
- perform any approved redaction as an explicit derived-artifact workflow;
- preserve the original archive when redaction or transformation is required for publication;
- document the transformation, tool version, operator, time, and resulting digest;
- apply retention and deletion decisions under the access-and-retention policy rather than archive semantics.

The repository does not currently provide private-object archive encryption or a secure-erasure guarantee.

## Verification checklist

After export:

1. confirm the command exited successfully;
2. confirm all three implemented files exist;
3. compare `recordCount` with manifest `itemCount`;
4. ensure the wire log and catalog have the expected number of non-empty lines;
5. inspect file permissions and destination ownership;
6. copy or package the directory only through an operator-approved process;
7. record a digest when the archive is retained as evidence or transferred.

Before import:

1. preserve the received archive unchanged;
2. inspect `manifest.json` and record the versions and item count;
3. confirm the destination commit and active acceptance policy;
4. ensure writer coordination for the destination backend;
5. use an isolated destination for destructive or compatibility testing;
6. plan for partial import and quarantine evidence.

After import:

1. record `recordCount` and `duplicateCount`;
2. inspect destination IDs and selected objects;
3. inspect quarantine state when import deferred;
4. run the applicable storage readiness or verification checks;
5. do not treat archive import success as backup verification, migration completion, privileged-host qualification, or formal-soak completion.

## Compatibility and release boundary

Archive compatibility is versioned and currently strict for archive version, protocol version, and carrier kind. A future implementation that supports conversion, multiple archive versions, signatures, compression, or incremental archives must introduce an explicit versioned contract and tests.

Documentation changes, local archive tests, and successful CI do not redefine the v1 candidate and do not constitute formal soak or reference-host qualification evidence.

## Related documents

- [Acceptance Policy](./ACCEPTANCE_POLICY.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
