# Access and Retention Audit Checklist

**Status: v1.0 pre-release normative** | **Last reviewed: 2026-07-24**

This checklist is the executable review companion to [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md). Use it when changing access defaults, retention behavior, quarantine handling, audit handling, backup or restore procedures, evidence retention, or node-retirement procedures.

The checklist verifies the behavior that exists in the v1.0 pre-release implementation. It must not be used to claim support for private objects, automatic expiry, general record deletion, tombstones, secure scrub, or cross-carrier erasure.

## 1. Record the audit scope

- [ ] Record the repository commit being audited.
- [ ] Record the operator, date, environment, and affected carrier or storage surface.
- [ ] State whether the audit is a documentation review, local rehearsal, qualification run, formal soak review, incident review, migration review, or retirement review.
- [ ] Identify the evidence directory or artifact bundle used for the review.
- [ ] Confirm that rehearsal evidence is not being represented as formal soak evidence.

## 2. Verify implemented protocol and carrier defaults

- [ ] Confirm that the implementation default is `accessScope=public`.
- [ ] Confirm that the implementation default is `retentionHint=long-lived`.
- [ ] Confirm that `accessScope` does not itself provide authentication, authorization, confidentiality, encryption, or a private-object transport.
- [ ] Confirm that `retentionHint` is advisory metadata and does not create an automatic expiry or deletion deadline.
- [ ] Confirm that any changed vocabulary is implemented consistently in protocol serialization, carrier capability output, fixtures, tests, and normative documentation.
- [ ] Reject the audit if documentation claims support for `curated`, `private`, encrypted, or other access modes that are not implemented by the audited release surface.

## 3. Verify acceptance and access-control boundaries

- [ ] Confirm that object validation and acceptance policy are distinct from administrator authentication and authorization.
- [ ] Confirm that public relay publication does not imply that all administrator endpoints are public.
- [ ] Confirm that administrator credentials are injected through the deployment boundary described in [Secret Management](./SECRET_MANAGEMENT.md).
- [ ] Confirm that role permissions match the implemented observer, reviewer, and operator boundaries.
- [ ] Confirm that credentials, bearer tokens, and environment-file contents are absent from logs, screenshots, evidence bundles, issue comments, and support archives.

## 4. Verify active canonical storage retention

- [ ] Identify the resolved `stateDir`, `dataDir`, `backupDir`, and `tempDir` for the audited node.
- [ ] Confirm that the raw relay log and canonical catalog are present in the resolved active data directory when expected.
- [ ] Confirm that the storage-format manifest and migration journal are handled according to [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md).
- [ ] Confirm that replay and canonical reconstruction requirements are based on files and manifests that actually exist in the audited implementation.
- [ ] Reject any checklist or runbook requirement that invents fixed files such as a generic `replay-metadata.json` or `resolved-config.json` unless the audited implementation actually creates them.
- [ ] Confirm that no general canonical-record deletion command is being assumed.
- [ ] Confirm that `retentionHint=long-lived` is not being treated as proof of permanent preservation or as authorization to delete shorter-lived records.

## 5. Verify quarantine retention

- [ ] Confirm that quarantine records are operational state separate from accepted canonical storage.
- [ ] Confirm that promotion changes the record's operational disposition but does not automatically authorize immediate destruction of incident or audit evidence.
- [ ] Confirm that permanent rejection is not described as deletion of an accepted canonical object.
- [ ] Confirm that annotations, resolution records, and permanent-rejection records needed for an incident or qualification review are retained together.
- [ ] Confirm that no automatic quarantine expiry or purge scheduler is being assumed unless separately implemented and tested.
- [ ] Confirm that any manual disposal procedure records the operator, scope, reason, timestamp, and verification result before removing data.

## 6. Verify administrator authentication audit retention

- [ ] Confirm the resolved path of `admin-auth-audit.jsonl` under the active state directory.
- [ ] Confirm that audit records contain only the implemented fields: attempted time, remote address, method, path, resolved role when available, and outcome code.
- [ ] Confirm that bearer tokens and credential values are not recorded.
- [ ] Confirm that audit records relevant to an active incident, investigation, qualification, or release decision are protected from routine disposal.
- [ ] Confirm that remote addresses and other operational identifiers are reviewed before publishing evidence externally.
- [ ] Confirm that the deployment's retention period is documented locally; the repository does not impose one universal duration.

## 7. Verify backup, migration, archive, and restore boundaries

- [ ] Distinguish a verified storage backup from a migration backup, archive export, qualification bundle, and soak evidence bundle.
- [ ] Confirm that each artifact is validated using its own manifest or verification procedure.
- [ ] Confirm that a migration backup is not represented as a complete release rollback package unless the operator runbook explicitly verifies that scope.
- [ ] Confirm that restore verification checks the restored node rather than modifying the backup in place.
- [ ] Confirm that backup retention preserves at least one known-good recovery point before destructive upgrade, migration, or retirement work.
- [ ] Confirm that backup disposal occurs only after replacement recovery points have been verified and documented.
- [ ] Confirm that secret-bearing environment files and credentials are not copied into general backup or evidence artifacts without an explicit protected-secret procedure.

## 8. Verify temporary and derived state

- [ ] Confirm that `tempDir` content is not treated as the authoritative recovery source.
- [ ] Confirm that temporary files are not removed while a storage, migration, backup, restore, or qualification operation still owns them.
- [ ] Confirm that caches or derived artifacts are deleted only when their authoritative source and regeneration path are known.
- [ ] Confirm that cleanup commands resolve and print the exact target path before removal.
- [ ] Confirm that cleanup instructions cannot expand to `/var/lib/lingonberry`, `/var/backups/lingonberry`, `/`, or an empty path through an unset variable.

## 9. Verify qualification, soak, and incident evidence

- [ ] Record the exact candidate commit associated with the evidence.
- [ ] Confirm that later documentation or tooling commits do not silently redefine the qualified candidate.
- [ ] Confirm that formal-soak evidence includes the complete required duration and continuity records before being called complete.
- [ ] Confirm that local rehearsal, CI walkthrough, or independent inspection evidence is labeled accurately.
- [ ] Confirm that evidence includes checksums or an equivalent integrity mechanism where required by the governing runbook.
- [ ] Confirm that secret values, private environment files, bearer headers, and unnecessary personal or network identifiers are redacted before publication.
- [ ] Preserve evidence connected to an unresolved release blocker, security incident, data-integrity concern, or rollback decision.

## 10. Verify node retirement and disposal

- [ ] Stop new writes and record the retirement boundary.
- [ ] Create and verify the required final backup or archive before removing active data.
- [ ] Record the release version, storage format, manifest identity, and verification result needed to restore or inspect the retired node.
- [ ] Preserve unresolved quarantine, administrator audit, incident, migration, and release evidence according to their separate retention decisions.
- [ ] Confirm that deleting the active node does not also delete the only verified backup.
- [ ] Confirm that physical disposal or secure media erasure is performed by an explicit deployment procedure; Lingonberry v1.0 does not provide a general secure-scrub command.
- [ ] Record final deletion or handoff evidence, including operator, timestamp, scope, and verification.

## 11. Audit result

Complete the following result block in the issue, pull request, or retained evidence record:

```text
Audited commit:
Environment:
Audit type:
Access default verified: yes/no
Retention default verified: yes/no
Canonical storage boundary verified: yes/no
Quarantine boundary verified: yes/no
Admin audit boundary verified: yes/no
Backup/restore boundary verified: yes/no
Evidence classification verified: yes/no
Secrets/redaction verified: yes/no
Open findings:
Decision: pass/fail/conditional
Operator:
Completed at:
```

A conditional result must identify the owner, remediation, deadline, and whether the finding blocks release publication or node operation.

## Reference order

1. [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
2. [Secret Management](./SECRET_MANAGEMENT.md)
3. [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
4. [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
5. [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
6. [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
7. Carrier-specific contracts only where their statements are verified against the audited implementation
